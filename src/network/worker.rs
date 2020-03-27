use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};

use std::sync::{Arc, Mutex, MutexGuard};
use crate::blockchain::{Blockchain};
use crate::crypto::hash::{H256, Hashable, H160};
use crate::block::{Block};
use log::info;


use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::Serialize;
use crate::transaction;
use crate::transaction::{Transaction, verify, SignedTrans};
use crate::state::State;
use crate::mempool::Mempool;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blkchain: Arc<Mutex<Blockchain>>,
    mem_pool: Arc<Mutex<Mempool>>,
    address_list:Arc<Mutex<Vec<H160>>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blkchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
    address_list:&Arc<Mutex<Vec<H160>>>,
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blkchain: Arc::clone(blkchain),
        mem_pool: Arc::clone(mempool),
        address_list:Arc::clone(address_list),
    }
}


impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(block_hashes) => {
                    println!("Received a NewBlockHash message");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());

                    let mut new_block_hashes: Vec<H256> = Vec::new();
                    let mut blockchain = self.blkchain.lock().unwrap();
                    //let hash = block_hashes.get(0).unwrap();
                    for hash in block_hashes{
                        if !blockchain.key_val.contains_key(&hash){
                            new_block_hashes.push(hash);
                        }
                    }
                    if new_block_hashes.len() > 0 {
                        peer.write(Message::GetBlocks(new_block_hashes));
                    }
                }
                Message::NewTransactionHashes(tx_hash) => {
                    println!("NewTransactionHashes");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());

                    let mut new_tx_hashes:Vec<H256> = Vec::new();
                    let mut mem_pool = self.mem_pool.lock().unwrap();
                    for hash in tx_hash{
                        if !mem_pool.pool.contains_key(&hash){
                            new_tx_hashes.push(hash);
                        }
                    }
                    if !new_tx_hashes.is_empty(){
                        peer.write(Message::GetTransactions(new_tx_hashes));
                    }
                }
                Message::GetBlocks(block_hashes) => {
                    println!("Received a GetBlocks message");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());

                    let mut new_blocks: Vec<Block> = Vec::new();
                    let mut blockchain = self.blkchain.lock().unwrap();
                    for hash in block_hashes{
                        if blockchain.key_val.contains_key(&hash){
                            new_blocks.push(blockchain.key_val.get(&hash).unwrap().clone());
                        }
                    }
                    if new_blocks.len() > 0 {
                        peer.write(Message::Blocks(new_blocks));
                    }
                }
                Message::GetTransactions(tx_hash) => {
                    println!("Received a GetTransactions message");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());

                    let mut new_tx:Vec<SignedTrans> = Vec::new();
                    let mem_pool = self.mem_pool.lock().unwrap();
                    // let pool = mem_pool.get_pool().clone();
                    for hash in tx_hash{
                        if mem_pool.pool.contains_key(&hash){
                            let signed_tx = mem_pool.pool.get(&hash).unwrap().clone();
                            new_tx.push(signed_tx);
                        }
                    }
                    if ! new_tx.is_empty(){
                        peer.write(Message::Transactions(new_tx));
                    }
                }
                Message::Blocks(blocks) => {
                    let mut blockchain = self.blkchain.lock().unwrap();
                    println!("Received blocks");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());

                    let mut new_block_hashes: Vec<H256> = Vec::new();
                    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                    for block in blocks {
                        if !blockchain.key_val.contains_key(&block.hash()){
                            let mut parent = block.head.block_parent;
                            if blockchain.key_val.contains_key(&parent) {
                                let encoded: Vec<u8> = bincode::serialize(&block).unwrap();
                                let size = encoded.len();
                                let mut pool = self.mem_pool.lock().unwrap();
                                if !blockchain.verify_blk(&block){
                                    continue;
                                }
                                let signed_tx = block.body.data.clone();
                                for tx in signed_tx{
                                    pool.remove(&tx);
                                }
                                blockchain.insert(&block);
                                blockchain.prop_time+= start - block.head.time_stamp;
                                // println!("block size = {:?}", size);
                                println!("Time elapsed in receiving one block is: {:?}ms",start - block.head.time_stamp);
                                new_block_hashes.push(block.hash());
                                let mut flag = 1;
                                while flag!=0{
                                    flag = 0;
                                    let buf = blockchain.orphan_buf.clone();
                                    let mut new_buf = Vec::new();
                                    for orp in buf{
                                        parent = orp.head.block_parent;
                                        if blockchain.key_val.contains_key(&parent){
                                            flag = 1;
                                            // blockchain.update_state(&orp);
                                            if !blockchain.verify_blk(&block){
                                                continue;
                                            }
                                            let signed_tx = orp.body.data.clone();
                                            for tx in signed_tx{
                                                pool.remove(&tx);
                                            }
                                            blockchain.insert(&orp);
                                            // drop(signed_tx);
                                            blockchain.prop_time+=start-orp.head.time_stamp;
                                            println!("Time elapsed in receiving one block is: {:?}ms", start - orp.head.time_stamp);
                                            new_block_hashes.push(orp.hash());
                                        }
                                        else{
                                            new_buf.push(orp);
                                        }
                                    }
                                    blockchain.orphan_buf = new_buf;
                                }
                            }
                            else{
                                blockchain.orphan_buf.push(block);
                                //peer.write(Message::GetBlocks(vec![parent]));
                            }
                        }
                    }
                    println!("total block in chain {}",blockchain.get_num());
                    drop(blockchain);
                    if new_block_hashes.len() > 0 {
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
                    }
                }
                Message::Transactions(txes) => {
                    println!("Transaction");
                    // println!("total block in chain {}",self.blkchain.lock().unwrap().get_num());
                    let mut mem_pool = self.mem_pool.lock().unwrap().clone();
                    let mut new_tx_hashes = Vec::new();
                    let mut chain = self.blkchain.lock().unwrap();
                    // let mut pool = mem_pool.get_pool();
                    let pool = mem_pool.pool.clone();
                    for tx in txes{
                        if !pool.contains_key(&tx.hash()){
                            let pub_key = tx.get_public_key();
                            let trans = tx.get_tx();
                            let sig = tx.get_sig();
                            let is_verified = verify(&trans, &pub_key, &sig);
                            let is_over_spend = trans.output_val() > trans.input_val();
                            if is_verified && !(is_over_spend) {
                                let buf = tx.clone();
                                self.mem_pool.lock().unwrap().pool.insert(tx.hash(), buf);
                                new_tx_hashes.push(tx.hash());
                                chain.update_state(&tx.tx);
                            }
                        }
                    }
                    drop(chain);
                    if !new_tx_hashes.is_empty() {
                        self.server.broadcast(Message::NewTransactionHashes(new_tx_hashes));
                    }


                }
                Message::Address(add)=>{
                    println!("new address");
                    let mut blockchain = self.blkchain.lock().unwrap();
                    let mut newadd = vec![];
                    for address in add{
                        if !blockchain.address_list.contains(&address){
                            newadd.push(address);
                            blockchain.address_list.push(address);
                        }
                    }
                    // println!("{:?}", blockchain.address_list);
                    if newadd.len()>0{
                        for address in blockchain.address_list.clone(){
                            newadd.push(address);
                        }
                        self.server.broadcast(Message::Address(newadd));
                    }
                }
            }
        }
    }
}
