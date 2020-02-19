use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::crypto::hash::{H256, Hashable};
use ring::digest;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    msg: u8,
}
impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        //unimplemented!()
        let encoded: Vec<u8> = bincode::serialize(&self.msg).unwrap();
        let mut cat = digest::Context::new(&digest::SHA256);
        cat.update(&encoded);
        let fin = cat.finish();
        let val = <H256>::from(fin);
        val
    }
}

pub fn generate_rand_transaction() -> Transaction {
    extern crate rand;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen();
    let t = Transaction{msg: n1};
    return t;
    //unimplemented!()
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //unimplemented!()
    let encoded: Vec<u8> = bincode::serialize(&t).unwrap();
    let sig = key.sign(&encoded);
    return sig;
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
    //unimplemented!()
    let msg: Vec<u8> = bincode::serialize(&t).unwrap();
    let peer_pub_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519,public_key);
    let result = peer_pub_key.verify(&msg,signature.as_ref());
    //return public_key.verify(encoded, &signature).is_ok();
    return result.is_ok();
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair;

    pub fn generate_random_transaction() -> Transaction {
        extern crate rand;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n1: u8 = rng.gen();
        let t = Transaction{msg: n1};
        return t;
        //unimplemented!()
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
}
