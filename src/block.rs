use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::{Transaction, generate_rand_transaction};
use ring::digest;
use crate::crypto::merkle;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub data: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub head: Header,
    pub body: Content,
    pub index: usize,
}

pub fn generate_rand_block(parent: &H256) -> Block{
    use hex_literal::hex;
    extern crate rand;
    use crate::crypto::merkle::MerkleTree;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let n1: u32 = rng.gen();
    let dif = (hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into();

    let mut data = Vec::new();
    // let num_transactions = rng.gen_range(1, 10);
    // for i in 0..num_transactions {
    //     let random_transaction = generate_rand_transaction();
    //     data.push(random_transaction);
    // }
    let merkle_tree = merkle::MerkleTree::new(&data);
    let root = merkle_tree.root();
    let single = Block{
        head: Header {block_parent:*parent,nonce:0,difficulty:dif,mkl_root:root,time_stamp:SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64},
        body: Content {data: data.clone()},
        index: 0, };
    single
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
            head: Header {block_parent:*parent,nonce:0,difficulty:dif,mkl_root:root,time_stamp:ts},
            body: Content {data},
            index: 0, };
        single
    }
}
