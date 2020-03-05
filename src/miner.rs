use crate::network::server::Handle as ServerHandle;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;
use crate::transaction::{Transaction, generate_rand_transaction};
use rand::Rng;
use crate::crypto::merkle::MerkleTree;
use crate::block::{Block, Header, Content};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use crate::crypto::hash::Hashable;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use hex_literal::hex;
use crate::block;
use crate::network::message::Message::NewBlockHashes;
use crate::network::message::Message;


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
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blkchain: &Arc<Mutex<Blockchain>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blkchain: Arc::clone(blkchain),
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
        // let mut cnt = 1000;
        let start = Instant::now();
        // main mining loop
        loop {
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
                let chain = self.blkchain.lock().unwrap();
                let hash_val = chain.tip();
                let dif = chain.key_val.get(&hash_val).unwrap().head.clone().difficulty;
                drop(chain);
                let mut data = Vec::<Transaction>::new();
                let mut rng = rand::thread_rng();
                let num_transactions = rng.gen_range(1, 10);
                for i in 0..num_transactions {
                    let random_transaction = generate_rand_transaction();
                    data.push(random_transaction);
                }
                let mut chain = self.blkchain.lock().unwrap();
                let mut new= block::generate_rand_block(&hash_val);
                new.body.data = data;
                new.index = chain.key_val.get(&hash_val).unwrap().index+1;
                drop(chain);
                loop
                {
                    // let mut time = start.elapsed().as_micros();
                    let mut rng = rand::thread_rng();
                    new.head.nonce = rng.gen::<u32>();
                    if new.hash() <= dif {
                        let mut chain = self.blkchain.lock().unwrap();
                        chain.insert(&new);
                        drop(chain);
                        let mut block_vec = Vec::new();
                        block_vec.push(new.hash());
                        let msg = Message::NewBlockHashes(block_vec);
                        self.server.broadcast(msg);
                        // info!("got one block");
                        num_blocks += 1;
                        info!("{}", num_blocks);

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
        // let duration = start.elapsed();
    }
}
