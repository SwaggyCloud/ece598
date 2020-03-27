use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters, UnparsedPublicKey};
use crate::crypto::hash::{H256, Hashable, H160, generate_rand_hash256, generate_rand_hash160};
// use crate::crypto::H160;
use ring::digest;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use bincode::serialize;
use crate::crypto::key_pair;
use ring::agreement::PublicKey;


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub id: u8,
    pub tx_in: Vec<Input>,
    pub tx_out: Vec<Output>
}

impl Transaction{
    pub fn get_id(&self) -> u8{self.clone().id}
    pub fn get_input(&self) -> Vec<Input>{self.clone().tx_in}
    pub fn get_output(&self) -> Vec<Output>{self.clone().tx_out}

    // pub fn get_output_set(&self) -> HashSet<Output> {
    //     let mut set = HashSet::new();
    //     let opt = self.clone().tx_out;
    //     for out in opt{
    //          set.insert(out);
    //     }
    //     set
    // }

    pub fn input_hash(&self) -> HashSet<H256>{
        self.tx_in
            .iter()
            .map(|input|input.previous_hash)
            .collect::<HashSet<H256>>()
    }

    pub fn output_address(&self) -> HashSet<H160>{
        self.tx_out
            .iter()
            .map(|output|output.address)
            .collect::<HashSet<H160>>()
    }
    pub fn input_val(&self) -> u8 {
        self.tx_in
            .iter()
            .map(|input| input.val)
            .sum()
    }

    pub fn output_val(&self) -> u8 {
        self.tx_out
            .iter()
            .map(|output| output.val)
            .sum()
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTrans {
    pub tx: Transaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}
impl SignedTrans{
    pub fn get_tx(&self) -> Transaction{self.clone().tx}
    pub fn get_sig(&self) -> Vec<u8>{self.clone().signature}
    pub fn get_public_key(&self) -> Vec<u8>{self.clone().public_key}
}

impl Hashable for SignedTrans {
    fn hash(&self) -> H256 {
        //unimplemented!()
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
        val
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Input {
    pub val: u8,
    pub previous_hash: H256,
}

impl Input{
    pub fn get_val(&self) -> u8 {self.clone().val}
    pub fn get_hash(&self) -> H256 {self.clone().previous_hash}
}

impl Hashable for Input {
    fn hash(&self) -> H256 {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
        val
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Output {
    pub val: u8,
    pub address:H160,
}

impl Output{
    pub fn get_val(&self) -> u8 {self.clone().val}
    pub fn get_address(&self) -> H160 {self.clone().address}
}

impl Hashable for Output{
    fn hash(&self) -> H256 {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
        val
    }
}



impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        //unimplemented!()
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
        val
    }
}
pub fn gen_rand_signtx() -> SignedTrans{
    let key = key_pair::random();
    let t = generate_rand_transaction();
    let s = sign(&t, &key);
    let p = key.public_key().as_ref().to_vec();
    SignedTrans{
        tx: t,
        signature: s,
        public_key: p,
    }
}


pub fn generate_rand_transaction() -> Transaction {
    extern crate rand;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let hash:H256 = generate_rand_hash256();
    let val = 1;
    let tx_in = Input{val, previous_hash:hash};
    let address = generate_rand_hash160();
    let tx_out = Output{val, address};
    let n1: u8 = rng.gen();
    let t = Transaction{id: n1, tx_in:vec![tx_in], tx_out:vec![tx_out]};
    return t;
}

/// Create digital signature of a transaction
// pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
//     //unimplemented!()
//     let encoded: Vec<u8> = bincode::serialize(&t).unwrap();
//     let sig = key.sign(&encoded);
//     return sig;
// }

// pub fn generate_tx_in_mempool(address: &H160, pub_key: &PublicKey,) -> SignedTrans{
//     let tx = Transaction {
//         tx_in
//     }
// }
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Vec<u8> {
    //unimplemented!()
    let encoded: Vec<u8> = bincode::serialize(&t).unwrap();
    let sig = key.sign(&encoded).as_ref().to_vec();
    return sig;
}

/// Verify digital signature of a transaction, using public key instead of secret key
// pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
//     //unimplemented!()
//     let msg: Vec<u8> = bincode::serialize(&t).unwrap();
//     let peer_pub_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519,public_key);
//     let result = peer_pub_key.verify(&msg,signature.as_ref());
//     //return public_key.verify(encoded, &signature).is_ok();
//     return result.is_ok();
// }

pub fn verify(tx: &Transaction, public_key: &[u8], signature: &[u8]) -> bool{
    let pub_key = UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
    let result = pub_key.verify(&(serialize(tx).unwrap()), signature );
    return result.is_ok();
}

pub fn coin_base(address: &H160) -> Transaction{
    use hex_literal::hex;

    let hash = (hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into();
    let input = Input{
        val: 0,
        previous_hash: hash,
    };

    let output = Output{
        val: 10,
        address: address.clone(),
    };
    let t = Transaction{
        id:1,
        tx_in: vec![input],
        tx_out: vec![output],
    };
    t
}


#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair;

    pub fn generate_random_transaction() -> Transaction {
        extern crate rand;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let hash:H256 = generate_rand_hash256();
        let val = 1;
        let tx_in = Input{val, previous_hash:hash};
        let address = generate_rand_hash160();
        let tx_out = Output{val, address};
        let n1: u8 = rng.gen();

        let t = Transaction{id: n1, tx_in:vec![tx_in], tx_out:vec![tx_out]};
        return t;
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        // assert!(verify(&t, &(key.public_key()), &signature));
    }
}
