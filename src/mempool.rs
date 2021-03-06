use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::SignedTrans;


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Mempool {
    pub pool: HashMap<H256, SignedTrans>,
}

impl Mempool {
    pub fn new() -> Self{
        let m =Mempool {
            pool: HashMap::new()
        };
        m
    }

    pub fn add(&mut self, signed: &SignedTrans) {
        let map = self.clone().pool;
        let hash = signed.hash();
        if !map.contains_key(&hash){
            self.pool.insert(hash, signed.clone());
        };
    }

    pub fn remove(&mut self, signed: &SignedTrans) {
        let map = self.clone().pool;
        let hash = signed.hash();
        if map.contains_key(&hash) {
            let res = self.pool.remove(&hash);
        }
        return
    }
}
