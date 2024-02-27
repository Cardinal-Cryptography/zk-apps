use ink::{
    env::hash::{CryptoHash, Sha2x256},
    storage::Mapping,
};

use crate::{errors::ShielderError, types::Set};
use mocked_zk::Scalar;

/// depth of the tree

#[ink::storage_item]
#[derive(Debug)]
pub struct MerkleTree<const DEPTH: usize> {
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

impl<const DEPTH: usize> Default for MerkleTree<DEPTH> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            roots_log: Default::default(),
            next_leaf_idx: 0,
            size: (1 << DEPTH),
        }
    }
}

impl<const DEPTH: usize> MerkleTree<DEPTH> {
    fn node_value(&self, id: u32) -> Result<Scalar, ShielderError> {
        self.nodes
            .get(id)
            .ok_or(ShielderError::MerkleTreeNonExistingNode)
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
            let left_n = self.node_value(id_mul_2).unwrap_or(0_u128.into());
            let right_n = self
                .node_value(
                    id_mul_2
                        .checked_add(1)
                        .ok_or(ShielderError::ArithmeticError)?,
                )
                .unwrap_or(0_u128.into());
            let hash = compute_hash(left_n, right_n);
            self.nodes.insert(id, &hash);
            id /= 2;
        }
        self.next_leaf_idx = self
            .next_leaf_idx
            .checked_add(1)
            .ok_or(ShielderError::ArithmeticError)?;
        self.roots_log.insert(self.root()?, &());
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
            *node = self.node_value(id ^ 1).unwrap_or(0_u128.into());
            id /= 2;
        }
        Ok(res)
    }

    pub fn root(&self) -> Result<Scalar, ShielderError> {
        self.node_value(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ink::primitives::AccountId;

    #[test]
    fn add_two_leaves_and_root() {
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));
        let mut merkle_tree = MerkleTree::<10>::default();
        let leaf0_id = merkle_tree.add_leaf(1_u128.into()).unwrap();
        assert_eq!(leaf0_id, 0);
        let leaf1_id = merkle_tree.add_leaf(2_u128.into()).unwrap();
        assert_eq!(leaf1_id, 1);

        let mut hash_left = compute_hash(1_u128.into(), 2_u128.into());
        let mut hash_right = compute_hash(0_u128.into(), 0_u128.into());
        for _i in 1..10 {
            hash_left = compute_hash(hash_left, 0_u128.into());
            hash_right = compute_hash(hash_right, hash_right);
        }

        assert_eq!(hash_left, merkle_tree.root().unwrap());
    }

    #[test]
    fn size_limit() {
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));
        let mut merkle_tree = MerkleTree::<10>::default();
        for i in 0..(1 << 10) {
            merkle_tree.add_leaf((i as u128).into()).unwrap();
        }
        assert!(merkle_tree.add_leaf(0_u128.into()).is_err());
    }

    #[test]
    fn historical_root() {
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));
        let mut merkle_tree = MerkleTree::<10>::default();
        let mut roots = vec![];
        let leaves_num = 10;
        for i in 0..leaves_num {
            merkle_tree.add_leaf((i as u128).into()).unwrap();
            roots.push(merkle_tree.root().unwrap());
        }
        // redeploy
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x2; 32]));
        let mut merkle_tree = MerkleTree::<10>::default();
        for i in 0..leaves_num {
            for j in 0..i {
                assert!(merkle_tree.is_historical_root(roots[j]).is_ok());
            }
            for j in i..leaves_num {
                assert!(merkle_tree.is_historical_root(roots[j]).is_err());
            }
            merkle_tree.add_leaf((i as u128).into()).unwrap();
        }
    }
}
