use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::PoseidonHasher,
    utils::BigPrimeField,
    AssignedValue, Context,
};

use crate::poseidon_consts::{RATE, T};

#[derive(Clone, Debug)]
pub struct MerkleProof<F: BigPrimeField, const MAX_PATH_LEN: usize> {
    pub path_shape: Vec<bool>,
    pub path: Vec<F>,
}

#[derive(Clone, Debug)]
pub struct CircuitMerkleProof<F: BigPrimeField, const MAX_PATH_LEN: usize> {
    pub path_shape: Vec<AssignedValue<F>>,
    pub path: Vec<AssignedValue<F>>,
}

impl<F: BigPrimeField, const MAX_PATH_LEN: usize> MerkleProof<F, MAX_PATH_LEN> {
    pub fn new(path_shape: Vec<bool>, path: Vec<F>) -> Self {
        Self { path_shape, path }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitMerkleProof<F, MAX_PATH_LEN> {
        let mut path_shape = vec![];
        let mut path = vec![];

        for i in 0..MAX_PATH_LEN {
            if i < self.path.len() {
                path_shape.push(ctx.load_witness(F::from_u128(self.path_shape[i] as u128)));
                path.push(ctx.load_witness(self.path[i]));
            } else {
                path_shape.push(ctx.load_constant(F::ONE));
                path.push(ctx.load_zero());
            }
        }

        CircuitMerkleProof { path_shape, path }
    }
}

impl<F: BigPrimeField, const MAX_PATH_LEN: usize> CircuitMerkleProof<F, MAX_PATH_LEN> {
    pub fn verify(
        &self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        poseidon: &mut PoseidonHasher<F, T, RATE>,
        root: AssignedValue<F>,
        leaf: AssignedValue<F>,
    ) {
        let mut current_note = leaf;

        for i in 0..MAX_PATH_LEN {
            let sibling = self.path[i as usize];
            let shape = self.path_shape[i as usize];

            let selector = gate.is_zero(ctx, shape);
            let left = gate.select(ctx, sibling, current_note, selector);
            let right = gate.select(ctx, current_note, sibling, selector);
            current_note = poseidon.hash_fix_len_array(ctx, gate, &[left, right]);
        }

        let eq = gate.is_equal(ctx, current_note, root);
        gate.assert_is_const(ctx, &eq, &F::ONE);
    }
}

#[cfg(test)]
pub mod tests {
    use poseidon::Poseidon;

    use super::*;
    use crate::poseidon_consts::{R_F, R_P};

    impl<F: BigPrimeField, const MAX_PATH_LEN: usize> MerkleProof<F, MAX_PATH_LEN> {
        pub fn verify(&self, root: F, leaf: F) -> bool {
            let mut current_note = leaf;

            for i in 0..MAX_PATH_LEN {
                let sibling = if i < self.path.len() {
                    self.path[i]
                } else {
                    F::ZERO
                };

                let mut poseidon: Poseidon<F, T, RATE> = Poseidon::new(R_F, R_P);

                if i < self.path.len() && !self.path_shape[i] {
                    poseidon.update(&[sibling, current_note]);
                } else {
                    poseidon.update(&[current_note, sibling]);
                }
                current_note = poseidon.squeeze();
            }

            current_note.eq(&root)
        }
    }
}
