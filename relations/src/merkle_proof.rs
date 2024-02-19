use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::PoseidonHasher,
    utils::BigPrimeField,
    AssignedValue, Context,
};

use crate::poseidon_consts::{RATE, T_WIDTH};

#[derive(Clone, Debug)]
pub struct MerkleProof<F: BigPrimeField, const TREE_HEIGHT: usize> {
    pub path_shape: [bool; TREE_HEIGHT],
    pub path: [F; TREE_HEIGHT],
}

#[derive(Clone, Debug)]
pub struct CircuitMerkleProof<F: BigPrimeField, const TREE_HEIGHT: usize> {
    pub path_shape: [AssignedValue<F>; TREE_HEIGHT],
    pub path: [AssignedValue<F>; TREE_HEIGHT],
}

impl<F: BigPrimeField, const TREE_HEIGHT: usize> MerkleProof<F, TREE_HEIGHT> {
    pub fn new(path_shape: [bool; TREE_HEIGHT], path: [F; TREE_HEIGHT]) -> Self {
        Self { path_shape, path }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitMerkleProof<F, TREE_HEIGHT> {
        let path_shape = self
            .path_shape
            .map(|x| ctx.load_witness(F::from_u128(x as u128)));
        let path = self.path.map(|x| ctx.load_witness(x));

        CircuitMerkleProof { path_shape, path }
    }
}

impl<F: BigPrimeField, const TREE_HEIGHT: usize> CircuitMerkleProof<F, TREE_HEIGHT> {
    pub fn verify(
        &self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        poseidon: &mut PoseidonHasher<F, T_WIDTH, RATE>,
        root: AssignedValue<F>,
        leaf: AssignedValue<F>,
    ) {
        let mut current_node = leaf;

        // TREE_HIGHT is definied in a way that path[TREE_HIGHT] would be the root
        for i in 0..TREE_HEIGHT {
            let sibling = self.path[i];
            let shape = self.path_shape[i];

            let selector = gate.is_zero(ctx, shape);
            let left = gate.select(ctx, sibling, current_node, selector);
            let right = gate.select(ctx, current_node, sibling, selector);
            current_node = poseidon.hash_fix_len_array(ctx, gate, &[left, right]);
        }

        let eq = gate.is_equal(ctx, current_node, root);
        gate.assert_is_const(ctx, &eq, &F::ONE);
    }
}
