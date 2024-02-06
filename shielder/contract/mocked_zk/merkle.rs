use crate::{errors::ShielderError, types::Scalar};
use ink::env::hash::{CryptoHash, Sha2x256};
use std::num::Wrapping;

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
        let mut id = (Wrapping(self.next_leaf_idx) + Wrapping(self.size)).0;
        self.nodes[id] = leaf_value;

        id /= 2;
        while id > 0 {
            let left_n = self.nodes[(Wrapping(id) * Wrapping(2)).0];
            let right_n = self.nodes[(Wrapping(id) * Wrapping(2) + Wrapping(1)).0];
            let hash = compute_hash(left_n, right_n);
            self.nodes[id] = hash;
            id /= 2;
        }
        self.next_leaf_idx = (Wrapping(self.next_leaf_idx) + Wrapping(1)).0;
        Ok(self.nodes[1])
    }

    pub fn root(&self) -> Scalar {
        self.nodes[1]
    }

    pub fn gen_proof(&self, leaf_id: usize) -> Result<[Scalar; DEPTH], ShielderError> {
        let mut res = [Scalar { bytes: [0x0; 32] }; DEPTH];
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeProofGenFail);
        }
        let mut id = (Wrapping(leaf_id) + Wrapping(self.size)).0;
        for node in res.iter_mut().take(DEPTH) {
            *node = self.nodes[id ^ 1];
            id /= 2;
        }
        Ok(res)
    }
}
