use serde::{Serialize, Deserialize};
use super::*;
use crate::block::{Block, generate_rand_block};
use crate::crypto::hash::{H256, Hashable};
use std::collections::HashMap;
// use crate::block::test::generate_random_block;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    pub length: usize,
    pub genesis: H256,
    pub tip: H256,
    pub key_val: HashMap<H256,Block>,
    pub orphan_buf: Vec<Block>,
    pub prop_time: u64,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // unimplemented!()
        let a = [0;32];
        let para: H256 = crypto::hash::H256::from(a);
        let mut buf: Block = generate_rand_block(&para);
        buf.head.time_stamp = 0;
//        let buf = Block{
//            head: None,
//            body: None,
//            index:0,
//        };

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
        }
    }

    pub fn get_num(&self) -> usize {
        return self.key_val.len();
    }

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
