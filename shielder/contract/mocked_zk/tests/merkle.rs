use ink::env::hash::{CryptoHash, Sha2x256};

use crate::{errors::ShielderError, types::Scalar};

/// depth of the tree
pub const DEPTH: usize = 10;

#[derive(Default)]
pub struct MerkleTree {
    nodes: Vec<Scalar>,
    next_leaf_idx: usize,
    size: usize,
}

pub fn compute_hash(first: Scalar, second: Scalar) -> Scalar {
    let mut res = [0x0; 32];
    Sha2x256::hash([first.bytes, second.bytes].concat().as_slice(), &mut res);
    Scalar { bytes: res }
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![Scalar { bytes: [0x0; 32] }; 1 << (DEPTH + 1)],
            next_leaf_idx: 0,
            size: (1 << DEPTH),
        }
    }

    pub fn add_leaf(&mut self, leaf_value: Scalar) -> Result<Scalar, ShielderError> {
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeLimitExceeded);
        }
        let mut id = self.next_leaf_idx + self.size;
        self.nodes[id] = leaf_value;

        id /= 2;
        while id > 0 {
            let left_n = self.nodes[id * 2];
            let right_n = self.nodes[id * 2 + 1];
            let hash = compute_hash(left_n, right_n);
            self.nodes[id] = hash;
            id /= 2;
        }
        self.next_leaf_idx += 1;
        Ok(self.nodes[1])
    }

    pub fn gen_proof(&self, leaf_id: usize) -> Result<[Scalar; DEPTH], ShielderError> {
        let mut res = [Scalar { bytes: [0x0; 32] }; DEPTH];
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeProofGenFail);
        }
        let mut id = leaf_id + self.size;
        for i in 0..DEPTH {
            res[i] = self.nodes[id ^ 1];
            id /= 2;
        }
        Ok(res)
    }
}
