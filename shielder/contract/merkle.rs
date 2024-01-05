use ink::storage::Mapping;

use crate::Scalar;


#[ink::storage_item]
#[derive(Debug)]
pub struct MerkleTree {
    notes: Mapping<u32, Scalar>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            notes: Mapping::default(),
        }
    }

    pub fn add_leaf(&mut self, leaf: Scalar) {

    }
}