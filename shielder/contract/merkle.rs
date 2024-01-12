use ink::{
    env::hash::{CryptoHash, Sha2x256},
    storage::Mapping,
};

use crate::{
    errors::ShielderError,
    types::{Scalar, Set},
};

/// depth of the tree
pub const DEPTH: usize = 10;

#[ink::storage_item]
#[derive(Default, Debug)]
pub struct MerkleTree {
    /// mapping of tree indexes to values held in nodes
    nodes: Mapping<u32, Scalar>,
    /// set of historical roots (nodes[1]) of tree
    roots_log: Set<Scalar>,
    /// index of next available leaf
    next_leaf_idx: u32,
    /// number of leaves in the tree, should be equal to 2^DEPTH
    size: u32,
}

pub fn compute_hash(first: Scalar, second: Scalar) -> Scalar {
    let mut res = [0x0; 32];
    Sha2x256::hash([first.bytes, second.bytes].concat().as_slice(), &mut res);
    Scalar { bytes: res }
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: Mapping::new(),
            roots_log: Mapping::new(),
            next_leaf_idx: 0,
            size: (1 << DEPTH),
        }
    }

    pub fn add_leaf(&mut self, leaf_value: Scalar) -> Result<(), ShielderError> {
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeLimitExceeded);
        }
        let mut id = self.next_leaf_idx + self.size;
        self.nodes.insert(id, &leaf_value);

        id /= 2;
        while id > 0 {
            let left_n = self.node_value(id * 2);
            let right_n = self.node_value(id * 2 + 1);
            let hash = compute_hash(left_n, right_n);
            self.nodes.insert(id, &hash);
            id /= 2;
        }
        self.next_leaf_idx += 1;
        Ok(())
    }

    pub fn is_historical_root(&self, merkle_root_possible: Scalar) -> Result<(), ShielderError> {
        self.roots_log
            .contains(merkle_root_possible)
            .then_some(())
            .ok_or(ShielderError::MerkleTreeVerificationFail)
    }

    fn node_value(&self, id: u32) -> Scalar {
        self.nodes.get(id).unwrap_or_default()
    }
}
