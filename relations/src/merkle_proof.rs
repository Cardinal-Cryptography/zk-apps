use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::PoseidonHasher,
    utils::BigPrimeField,
    AssignedValue, Context,
};

use crate::poseidon_consts::{RATE, T_WIDTH};

#[derive(Clone, Debug)]
pub struct MerkleProof<F: BigPrimeField, const MAX_PATH_LEN: usize> {
    pub path_shape: [bool; MAX_PATH_LEN],
    pub path: [F; MAX_PATH_LEN],
}

#[derive(Clone, Debug)]
pub struct CircuitMerkleProof<F: BigPrimeField, const MAX_PATH_LEN: usize> {
    pub path_shape: [AssignedValue<F>; MAX_PATH_LEN],
    pub path: [AssignedValue<F>; MAX_PATH_LEN],
}

impl<F: BigPrimeField, const MAX_PATH_LEN: usize> MerkleProof<F, MAX_PATH_LEN> {
    pub fn new(path_shape: [bool; MAX_PATH_LEN], path: [F; MAX_PATH_LEN]) -> Self {
        Self { path_shape, path }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitMerkleProof<F, MAX_PATH_LEN> {
        let path_shape = self
            .path_shape
            .map(|x| ctx.load_witness(F::from_u128(x as u128)));
        let path = self.path.map(|x| ctx.load_witness(x));

        CircuitMerkleProof { path_shape, path }
    }
}

impl<F: BigPrimeField, const MAX_PATH_LEN: usize> CircuitMerkleProof<F, MAX_PATH_LEN> {
    pub fn verify(
        &self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        poseidon: &mut PoseidonHasher<F, T_WIDTH, RATE>,
        root: AssignedValue<F>,
        leaf: AssignedValue<F>,
    ) {
        let mut current_note = leaf;

        for i in 0..MAX_PATH_LEN {
            let sibling = self.path[i];
            let shape = self.path_shape[i];

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
                let sibling = self.path[i];

                let mut poseidon: Poseidon<F, T_WIDTH, RATE> = Poseidon::new(R_F, R_P);

                if !self.path_shape[i] {
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
