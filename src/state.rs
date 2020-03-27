use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use crate::crypto::hash::{H256, H160};
use crate::transaction::{Input, Output};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State{
    pub map: HashMap<H256, Output>
}

impl State{
    pub fn new() -> Self{
        State{
            map: HashMap::new(),
        }
    }

    pub fn is_double_spend(&self, data:Input) -> bool{
        let hash = data.get_hash();
        let mut flag = false;
        if self.map.contains_key(&hash){
            let tx_out = self.map.get(&hash).unwrap().clone();
            let val = tx_out.get_val();
            let in_val = data.get_val();
            if val == in_val{
                flag = false;
            }else{
                flag = true;
            }
        }else{
            flag = true;
        }
        flag
    }
}