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
#[derive(Debug)]
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
    Scalar::from_bytes(res)
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            roots_log: Default::default(),
            next_leaf_idx: 0,
            size: (1 << DEPTH),
        }
    }
}

impl MerkleTree {
    fn node_value(&self, id: u32) -> Scalar {
        self.nodes.get(id).unwrap_or_default()
    }

    pub fn add_leaf(&mut self, leaf_value: Scalar) -> Result<u32, ShielderError> {
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeLimitExceeded);
        }
        let mut id = self
            .next_leaf_idx
            .checked_add(self.size)
            .ok_or(ShielderError::ArithmeticError)?;
        let cur_leaf_id = self.next_leaf_idx;
        self.nodes.insert(id, &leaf_value);

        id /= 2;
        while id > 0 {
            let id_mul_2 = id.checked_mul(2).ok_or(ShielderError::ArithmeticError)?;
            let left_n = self.node_value(id_mul_2);
            let right_n = self.node_value(
                id_mul_2
                    .checked_add(1)
                    .ok_or(ShielderError::ArithmeticError)?,
            );
            let hash = compute_hash(left_n, right_n);
            self.nodes.insert(id, &hash);
            id /= 2;
        }
        self.next_leaf_idx = self
            .next_leaf_idx
            .checked_add(1)
            .ok_or(ShielderError::ArithmeticError)?;
        self.roots_log.insert(self.node_value(1), &());
        Ok(cur_leaf_id)
    }

    pub fn is_historical_root(&self, merkle_root_possible: Scalar) -> Result<(), ShielderError> {
        self.roots_log
            .contains(merkle_root_possible)
            .then_some(())
            .ok_or(ShielderError::MerkleTreeVerificationFail)
    }

    pub fn gen_proof(&self, leaf_id: u32) -> Result<[Scalar; DEPTH], ShielderError> {
        let mut res = [Scalar::from_bytes([0x0; 32]); DEPTH];
        if self.next_leaf_idx == self.size {
            return Err(ShielderError::MerkleTreeProofGenFail);
        }
        let mut id = leaf_id
            .checked_add(self.size)
            .ok_or(ShielderError::ArithmeticError)?;
        for node in res.iter_mut().take(DEPTH) {
            *node = self.node_value(id ^ 1);
            id /= 2;
        }
        Ok(res)
    }

    pub fn root(&self) -> Scalar {
        self.node_value(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::primitives::AccountId;

    #[test]
    fn merkle_tree_test() {
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));
        let mut merkle_tree = MerkleTree::default();
        merkle_tree.add_leaf(0_u128.into()).unwrap();
    }
}
