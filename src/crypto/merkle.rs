use super::hash::{Hashable, H256};
use ring::digest;

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    Troot: Node,
    /// The number of leaf nodes in the tree
    depth: u32,
    count: usize,
}
#[derive(Debug, Default)]
pub struct Node{
    val: H256,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}


impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        //unimplemented!()
        let mut dep = 0;
        let mut cnt = 0;
        let mut cur = Vec::new();
        if data.len() == 0 {
            let input = [0;32];
            let val = super::hash::H256::from(input);
            let n = Node {
                val,
                left: None,
                right:None,
            };
            return MerkleTree {
                Troot: n,
                depth: 0,
                count: 0,
            };
        }
        for d in data.iter(){
            cur.push(Node{left:None,right:None,val:d.hash()});
            dep += 1;
        }
        while cur.len()>1{
            let mut nxt = Vec::new();
            if cur.len()%2==1{
                let mut tmp = cur[cur.len()-1].val;
                cur.push(Node{left:None,right:None,val:tmp});
            }
            while cur.len()!=0{
                let l = cur.remove(0);
                let r = cur.remove(0);
                let mut cat = digest::Context::new(&digest::SHA256);
                cat.update(&<[u8;32]>::from(l.val));
                cat.update(&<[u8;32]>::from(r.val));
                let fin = cat.finish();
                nxt.push(Node{
                    val: <H256>::from(fin),
                    left: Some(Box::new(l)),
                    right: Some(Box::new(r)),
                });
            }
            cur = nxt;
            dep += 1;
        }
        let root = cur.remove(0);
        let size = data.len();
        MerkleTree{Troot:root,depth:dep,count:size}

    }

    pub fn root(&self) -> H256 {
        self.Troot.val
    }

    
    pub fn get_proof(tree: &Node,index: usize,count:usize,res:&mut Vec<H256>){
        match *tree{
            Node{val,left:Some(ref left),right:Some(ref right)}=>{
                let left_count = count.next_power_of_two() / 2;
                let (sub_lem_val);
                if index < left_count {
                    res.push(right.val);
                    sub_lem_val = MerkleTree::get_proof(left, index, left_count,res);
                } else {
                    res.push(left.val);
                    sub_lem_val = MerkleTree::get_proof(right, index - left_count, count - left_count,res);
                }
            }
            _=>{}
        }
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut mt_proof = Vec::<H256>::new();
        MerkleTree::get_proof(&self.Troot,index,self.count,&mut mt_proof);
        mt_proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    //unimplemented!()
    let mut cat = digest::Context::new(&digest::SHA256);
    let mut cur = *datum;
    let mut fin = cat.finish();
    for i in 0..proof.len(){
        cat = digest::Context::new(&digest::SHA256);
        cat.update(&<[u8;32]>::from(cur));
        cat.update(&<[u8;32]>::from(proof[proof.len()-1-i]));
        fin = cat.finish();
        cur = <H256>::from(fin);
    }
    if std::cmp::Ordering::Equal != cur.cmp(root){
        return false
    }
    return true

}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}
