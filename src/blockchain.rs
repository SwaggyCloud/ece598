use serde::{Serialize, Deserialize};
use super::*;
use crate::block::{Block, generate_rand_block};
use crate::crypto::hash::{H256, Hashable, H160};
use std::collections::{HashMap, HashSet};
use crate::transaction::{Transaction, SignedTrans, Output, verify};
use crate::state::State;
use std::hash::Hash;
// use crate::block::test::generate_random_block;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    pub length: usize,
    pub genesis: H256,
    pub tip: H256,
    pub key_val: HashMap<H256,Block>,
    pub orphan_buf: Vec<Block>,
    pub prop_time: u64,
    pub address_list: Vec<H160>,
    // pub address_pbkey: HashMap<H160, [u8]>,
    pub block_state: HashMap<H256, State>,
    pub current_state: State,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // unimplemented!()
        let a = [0;32];
        let para: H256 = crypto::hash::H256::from(a);
        let mut buf: Block = generate_rand_block(&para);
        buf.head.time_stamp = 0;
        let tip:H256 = buf.hash();
        let genesis = buf.hash();
        let mut map = HashMap::new();
        let mut o_buf = Vec::new();
        map.insert(tip, buf);

        Blockchain {
            length: 0,
            genesis,
            tip,
            key_val: map,
            orphan_buf:o_buf,
            prop_time: 0,
            address_list: Vec::new(),
            block_state: HashMap::new(),
            current_state: State::new(),
        }
    }

    pub fn get_num(&self) -> usize {
        return self.key_val.len();
    }

    pub fn verify_blk(&self, block:&Block) -> bool {
        let mut res = true;
        let mut tag = true;
        let b = block.clone();
        let txes = b.body.data;
        for signed in txes{
            let sig = signed.get_sig();
            let pubkey = signed.get_public_key();
            for add in self.address_list.clone(){
                if H160::hash(&pubkey) == add{
                    tag = false;
                }
            }
            if tag{
                return false;
            }
            let tx = signed.tx;
            if !verify(&tx, &pubkey, &sig){
                res = false;
                break;
            }
        }
        res
    }
    pub fn update_state(&mut self, transaction:&Transaction) {
        // let hash = block.hash();
        // self.block_state.insert(hash, State::new());
        // return
        let mut st = self.current_state.clone().map;
        let mut collection = HashMap::new();
        let mut vec_in = transaction.tx_in.clone();
        for tx_in in vec_in{
            collection.insert(tx_in.previous_hash,tx_in);
        }
        for (hash,out) in st.clone() {
            if collection.contains_key(&hash){
                st.remove(&hash);
            }
        }
        self.current_state = State{map:st};
    }
    // pub fn update_state(&mut self, transaction:&Transaction, parent:&H256 ) -> State {
    //     let mut pre_state = self.clone().current_state;
    //     let vec_in = transaction.clone().tx_in;
    //     let mut collection = HashMap::new();
    //     for tx_in in vec_in {
    //         collection.insert(tx_in.previous_hash, tx_in);
    //     }
    //     let mut map = pre_state.map;
    //     for (hash, out) in map.clone(){
    //         if collection.contains_key(&hash) {
    //             let tx_in = collection.get(&hash).unwrap().clone();
    //             if tx_in.val == out.val {
    //                 map.remove(&hash);
    //             }
    //         }
    //     }
    //     State{
    //         map
    //     }
        // let hash = block.hash();
        // self.block_state.insert(hash, State::new());
        // return
        // let parent_hash = block.clone().head.block_parent;
        // let mut st = self.block_state.get(&parent_hash).clone().unwrap().clone().map;
        // let txes = block.body.data.clone();
        // let mut output_hash_vec = Vec::new();
        // for tx in txes {
        //     let trans = tx.tx.clone();
        //     let tx_out = trans.get_output();
        //     for out in tx_out {
        //         output_hash_vec.push(out.hash());
        //     }
        // }
        // for (key, value) in st.clone().iter(){
        //     if output_hash_vec.contains(&value.hash()){
        //         st.remove(key);
        //     }
        // }
        // self.block_state.insert(block.hash(), State{map:st});
    // }
    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let mut b = (*block).clone();
        let buf = b.clone();
        let parent = b.head.clone().block_parent;
        let parent_block = self.key_val.get(&parent).unwrap();
        let p = (parent_block.clone());
        let new_idx = p.index + 1;
        let new_head = buf.head;
        let new_body = buf.body;
        let new_blk = Block{head:new_head,body:new_body,index:new_idx};

        if new_idx > self.length {
            self.length = new_idx;
            self.tip = new_blk.hash();
        }
        self.key_val.insert(new_blk.hash(), new_blk);
        return;
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        unimplemented!()
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        let block1 = generate_random_block(&genesis_hash);
        blockchain.insert(&block1);
//        let block2 = generate_random_block(&block.hash());
//        blockchain.insert(&block2);
        assert_eq!(blockchain.tip(), block.hash());
//        assert_eq!(blockchain.length,0);
    }
}
