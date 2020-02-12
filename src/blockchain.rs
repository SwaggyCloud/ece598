use serde::{Serialize, Deserialize};
use super::*;
use crate::block::Block;
use crate::crypto::hash::{H256, Hashable};
use std::collections::HashMap;
use crate::block::test::generate_random_block;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    length: usize,
    genesis: H256,
    tip: H256,
    key_val: HashMap<H256,Block>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // unimplemented!()
//        let a = [0;32];
//        let para: H256 = crypto::hash::H256::from(a);
//        let buf: Block = generate_random_block(&para);
        let buf = Block{
            head: None,
            body: None,
            index:0,
        };
        let tip:H256 = buf.hash();
        let genesis = buf.hash();
        let mut map = HashMap::new();
        map.insert(tip, buf);

        Blockchain {
            length: 0,
            genesis,
            tip,
            key_val: map,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let mut b = (*block).clone();
        let buf = b.clone();
        let parent = b.head.unwrap().clone().block_parent;
        let parent_block = (self.key_val.get(&parent).unwrap());
        let p = (parent_block.clone());

        if p.index == self.length {
            b.index = p.index + 1;
            self.length = buf.index;
            self.tip = buf.hash();
        }
        self.key_val.insert(buf.hash(), buf);
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
        assert_eq!(blockchain.tip(), block.hash());
    }
}
