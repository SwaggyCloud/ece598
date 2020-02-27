use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};

use std::sync::{Arc, Mutex};
use crate::blockchain::{Blockchain};
use crate::crypto::hash::{H256, Hashable};
use crate::block::{Block};


use std::thread;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blkchain: Arc<Mutex<Blockchain>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blkchain: &Arc<Mutex<Blockchain>>,
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blkchain: Arc::clone(blkchain),
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
                Message::GetBlocks(block_hashes) => {
                    println!("Received a GetBlocks message");
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
                Message::Blocks(blocks) => {
                    let mut blockchain = self.blkchain.lock().unwrap();
                    println!("Received blocks");
                    let mut new_block_hashes: Vec<H256> = Vec::new();
                    for block in blocks {
                        if !blockchain.key_val.contains_key(&block.hash()){
                            let parent = block.head.block_parent;
                            if blockchain.key_val.contains_key(&parent) {
                                blockchain.insert(&block);
                                new_block_hashes.push(block.hash());
                            }else{
                                peer.write(Message::GetBlocks(vec![parent]));
                            }
                        }
                    }
                    drop(blockchain);
                    if new_block_hashes.len() > 0 {
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
                    }
                }

            }
        }
    }
}
