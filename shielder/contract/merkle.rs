use ink::storage::Mapping;

use crate::Scalar;


#[ink::storage_item]
#[derive(Debug)]
pub struct MerkleTree {
    nodes: Mapping<u32, Scalar>,
    next_leaf_id: u32,
    sz: u32,
}

fn compute_hash(first: Scalar, second: Scalar) -> Scalar {
    first
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: Mapping::default(),
            next_leaf_id: 0,
            sz: (1<<4), //TODO: make const or configurable
        }
    }

    pub fn add_leaf(&mut self, leaf_value: Scalar) {
        if self.next_leaf_id == self.sz {
            //TODO: throw specific error
            return;
        }
        let mut id = self.next_leaf_id + self.sz;
        self.nodes.insert(id, &leaf_value);
        
        id /= 2;
        while id > 0 {
            let left_n = self.node_value(id*2);
            let right_n = self.node_value(id*2+1);
            let hash = compute_hash(left_n, right_n);
            self.nodes.insert(id, &hash);
            id /= 2;
        }
        self.next_leaf_id += 1;
    }

    pub fn node_value(&self, id: u32) -> Scalar {
        self.nodes.get(id).unwrap_or_default()
    }

    pub fn root(&self) -> Scalar {
        self.node_value(1)
    }

}