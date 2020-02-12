use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::Transaction;
use ring::digest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header{
    pub block_parent:H256,
    pub nonce:u32,
    pub difficulty:H256,
    pub mkl_root:H256,
    pub time_stamp:u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    data: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub head: Option<Header>,
    pub body: Option<Content>,
    pub index: usize,
}


impl Hashable for Block {
    fn hash(&self) -> H256 {
        //unimplemented!()
        let encoded: Vec<u8> = bincode::serialize(&self.head).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
//        let val = ring::digest::digest(&ring::digest::SHA256, self).into();
        val
    }
}

#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;
    use crate::crypto::merkle;

    pub fn generate_random_block(parent: &H256) -> Block {
        //unimplemented!()

        extern crate rand;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen();
        let encoded: Vec<u8> = bincode::serialize(&n1).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let dif = <H256>::from(fin);
        let ts: u64 = rng.gen();
        let data = Vec::new();

        let merkle_tree = merkle::MerkleTree::new(&data);
        let root = merkle_tree.root();
        let single = Block{
            head: Some(Header {block_parent:*parent,nonce:n1,difficulty:dif,mkl_root:root,time_stamp:ts}),
            body: Some(Content {data}),
            index: 0, };
        single
    }
}
