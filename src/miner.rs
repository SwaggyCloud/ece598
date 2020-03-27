use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;
use crate::transaction::{Transaction, generate_rand_transaction, SignedTrans, sign, Input, Output, gen_rand_signtx};
use crate::mempool::Mempool;
use rand::Rng;
use crate::crypto::merkle::MerkleTree;
use crate::block::{Block, Header, Content};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use crate::crypto::hash::{Hashable, generate_rand_hash256, H160, H256};
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use hex_literal::hex;
use crate::block;
use crate::network::message::Message::NewBlockHashes;
use crate::network::message::Message;
use crate::crypto::key_pair::random;
use crate::crypto::key_pair;
use ring::signature::{KeyPair, Ed25519KeyPair};
use crate::network::peer::Direction::Outgoing;
use std::collections::HashMap;
use crate::state::State;
use url::quirks::hash;
use std::thread::{sleep, current};


enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blkchain: Arc<Mutex<Blockchain>>,
    mem_pool: Arc<Mutex<Mempool>>,
    key: Ed25519KeyPair,
    self_address:H160,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blkchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
    key_pair: Ed25519KeyPair,
    self_address: &H160
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blkchain: Arc::clone(blkchain),
        mem_pool: Arc::clone(mempool),
        key: key_pair,
        self_address: self_address.clone(),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        use hex_literal::hex;

        let mut num_blocks = 0;
        let key = key_pair::random();
        let public_key = key.public_key();
        let byte_pbkey = public_key.as_ref();
        let address = H160::hash(&byte_pbkey);
        println!("self address: {:?}",address);
        let mut address_vec = vec![address];
        self.blkchain.lock().unwrap().address_list.push(address);
        self.server.broadcast(Message::Address(address_vec));
        println!("self address: {:?}", self.blkchain.lock().unwrap().address_list);
        self.key = key;
        self.self_address = address;

        let mut blkchain = self.blkchain.lock().unwrap();
        let mut st = HashMap::new();
        let hash_val = blkchain.tip;
        for addr in blkchain.address_list.clone(){
            let mut opt = Vec::new();
            let mut ipt = Vec::new();
            opt.push(Output{
                val:10,
                address:addr,
            });
            let mut tx = Transaction{
                id:0,
                tx_in:ipt,
                tx_out:opt,
            };
            info!("ico -> {:?}", tx);
            st.insert(tx.hash(), Output{
                val:10,
                address:addr,
            });
        }
        let mut blkstate = State{
            map: st
        };

        let start = Instant::now();
        blkchain.block_state.insert(hash_val.clone(), blkstate.clone() );
        blkchain.current_state = blkstate.clone();
        drop(blkchain);
        loop {
            // println!("key: {:?}, self address: {:?}", self.key, self.self_address);
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO: actual mining

            if let OperatingState::Run(i) = self.operating_state {
                if i== 1 {
                    let mut chain = self.blkchain.lock().unwrap();
                    info!("{:?}", chain.tip);
                    drop(chain);
                    println!("address list: {:?}", self.blkchain.lock().unwrap().address_list);
                    let mut chain = self.blkchain.lock().unwrap();
                    let hash_val = chain.tip;
                    let dif = chain.key_val.get(&hash_val).unwrap().head.clone().difficulty;
                    // println!("{:?}", chain.current_state);
                    drop(chain);
                    // info!("{:?},{:?}",hash_val,self.blkchain.lock().unwrap().block_state );
                    // let mut pool = self.mem_pool.lock().unwrap().pool;
                    let mut rng = rand::thread_rng();
                    let num_transactions = rng.gen_range(2, 5);
                    info!("num of transacation: {:?}", num_transactions);
                    for i in 0..num_transactions {
                        let random_transaction = self.gen_rand_signed(&hash_val);
                        // println!("{:?}", random_transaction);
                        self.blkchain.lock().unwrap().update_state(&random_transaction.clone().tx);
                        // println!("{:?}", cur_state);
                        // self.blkchain.lock().unwrap().current_state = cur_state;
                        self.mem_pool.lock().unwrap().pool.insert(random_transaction.hash(), random_transaction.clone());
                        self.server.broadcast(Message::NewTransactionHashes(vec![random_transaction.clone().hash()]));
                    }
                    // drop(pool);
                    let mut chain = self.blkchain.lock().unwrap();
                    let mut new= block::generate_rand_block(&hash_val);
                    // new.body.data = data;
                    new.index = chain.key_val.get(&hash_val).unwrap().index+1;
                    // drop(data);
                    drop(chain);
                    loop
                    {
                        // info!("mempool size: {:?}", self.mem_pool.lock().unwrap().pool.capacity());
                        let mut rng = rand::thread_rng();
                        new.head.nonce = rng.gen::<u32>();
                        let mut cnt = 0;
                        let mut data = Vec::<SignedTrans>::new();
                        let mut pool = self.mem_pool.lock().unwrap().clone().pool;
                        for (hash,val) in pool.clone(){
                            if cnt==3{
                                break;
                            }
                            data.push(val);
                            self.mem_pool.lock().unwrap().pool.remove(&hash);
                            cnt += 1;
                        }
                        drop(pool);
                        new.body.data = data;

                        if new.hash() <= dif {
                            let mut chain = self.blkchain.lock().unwrap();
                            chain.insert(&new);
                            let current = chain.clone().current_state;
                            chain.block_state.insert(new.hash(), current);
                            drop(chain);
                            let mut block_vec = Vec::new();
                            block_vec.push(new.hash());
                            let msg = Message::NewBlockHashes(block_vec);
                            self.server.broadcast(msg);
                            num_blocks += 1;
                            info!("num of blocks {}", num_blocks);
                            break;
                        }
                    }
                    let time = start.elapsed().as_secs();
                    if time >= 30{
                        let chain = self.blkchain.lock().unwrap();
                        let chain_blocks = chain.get_num();
                        println!("length of the blockchain {}", chain_blocks);
                        println!("Time elapsed in mining {} blocks is: {:?}s", num_blocks, time);
                        break;
                        info!("time out");
                        drop(chain);
                    }
                }
            }
        }
    }

    fn gen_rand_signed(&self, pre_hash:&H256) -> SignedTrans{
        let id = 1;
        let mut in_val:u8 = 1;
        let out_val:u8 = 1;
        // info!("1");
        let (tx_in,in_val) = self.get_input(&out_val, pre_hash);
        // info!("2");
        let address = self.get_address();
        // info!("3");
        let tx_out = self.get_output(&in_val,&out_val,&address);
        // info!("4");
        let tx = Transaction{id, tx_in, tx_out};
        let signature = sign(&tx, &self.key);
        let public_key = self.key.public_key().as_ref().to_vec();
        SignedTrans{
            tx,
            signature,
            public_key,
        }
    }

    fn get_input(&self, out_val:&u8, hash: &H256) -> (Vec<Input>, u8){
        // info!("11");
        let mut chain = self.blkchain.lock().unwrap().clone();
        // info!("11");

        let pre_st = chain.current_state;
        // info!("11");

        let map = pre_st.map;
        //TODO
        let mut input_vec = Vec::new();
        // info!("11");

        let mut current_val:u8 = 0;
        for (tx_hash, output) in map{
            if current_val < out_val.clone() {
                let t = Input{
                    val: output.get_val(),
                    previous_hash: tx_hash,
                };
                current_val += output.get_val();
                input_vec.push(t);
            }else{
                // info!("112");

                break;
            }
        }
        return (input_vec, current_val);
    }

    fn get_output(&self, in_val:&u8, out_val: &u8, address:&H160) -> Vec<Output>{
        let val = in_val.clone();
        let out = out_val.clone();
        return if val == out {
            let o = Output {
                val,
                address: address.clone(),
            };
            vec![o]
        } else if val > out {
            let mut tx_out = Vec::new();
            let o = Output {
                val: out,
                address: address.clone()
            };
            tx_out.push(o);
            let change = Output {
                val: val - out,
                address: self.self_address,
            };
            tx_out.push(change);
            tx_out
        } else {
            // println!(" overspend");
            Vec::new()
        }
    }

    fn get_address(&self) -> H160{
        let address_list = self.blkchain.lock().unwrap().clone().address_list;
        if address_list.len() == 1 {
            info!("Only one address in the list");
            return address_list[0];
        }
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(1, address_list.len());
        return address_list[index];
    }
}
