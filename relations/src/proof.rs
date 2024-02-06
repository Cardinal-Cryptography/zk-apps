use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::PoseidonHasher,
    utils::BigPrimeField,
    AssignedValue, Context,
};

use crate::poseidon_consts::{RATE, T};

#[derive(Clone, Debug)]
pub struct MerkleProof<F: BigPrimeField> {
    pub path_shape: Vec<bool>,
    pub path: Vec<F>,
    pub max_path_len: usize,
}

#[derive(Clone, Debug)]
pub struct CircuitMerkleProof<F: BigPrimeField> {
    pub path_shape: Vec<AssignedValue<F>>,
    pub path: Vec<AssignedValue<F>>,
    pub max_path_len: AssignedValue<F>,
}

impl<F: BigPrimeField> MerkleProof<F> {
    pub fn new(path_shape: Vec<bool>, path: Vec<F>, max_path_len: usize) -> Self {
        Self {
            path_shape,
            path,
            max_path_len,
        }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitMerkleProof<F> {
        let mut path_shape = vec![];
        let mut path = vec![];

        for i in 0..self.max_path_len {
            if i < path.len() {
                path_shape.push(ctx.load_witness(F::from_u128(self.path_shape[i] as u128)));
                path.push(ctx.load_witness(self.path[i]));
            } else {
                path_shape.push(ctx.load_zero());
                path.push(ctx.load_zero());
            }
        }

        CircuitMerkleProof {
            path_shape,
            path,
            max_path_len: ctx.load_witness(F::from_u128(self.max_path_len as u128)),
        }
    }
}

impl<F: BigPrimeField> CircuitMerkleProof<F> {
    pub fn verify(
        &self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        poseidon: &mut PoseidonHasher<F, T, RATE>,
        root: AssignedValue<F>,
        leaf: AssignedValue<F>,
    ) {
        let mut current_note = leaf;

        for i in 0..self.max_path_len.value().to_bytes_le()[0] {
            let sibling = self.path[i as usize];
            let shape = self.path_shape[i as usize];

            let selector = gate.is_zero(ctx, shape);
            let left = gate.select(ctx, current_note, sibling, selector);
            let right = gate.select(ctx, sibling, current_note, selector);
            current_note = poseidon.hash_fix_len_array(ctx, gate, &[left, right])
        }

        let eq = gate.is_equal(ctx, current_note, root);
        gate.assert_is_const(ctx, &eq, &F::ONE);
    }
}

#[cfg(test)]
mod tests {
    use poseidon::Poseidon;

    use super::*;
    use crate::poseidon_consts::{RATE, R_F, R_P, T};

    impl<F: BigPrimeField> MerkleProof<F> {
        pub fn verify(&self, root: F, leaf: F) -> bool {
            let mut current_note = leaf;

            for i in 0..self.max_path_len {
                let sibling = if i < self.path.len() {
                    self.path[i]
                } else {
                    F::ZERO
                };

                let mut poseidon: Poseidon<F, T, RATE> = Poseidon::new(R_F, R_P);

                //Question: SHould id be if-elif-else or if-else?
                if self.path_shape[i] {
                    poseidon.update(&[current_note, sibling]);
                } else {
                    poseidon.update(&[sibling, current_note]);
                }
                current_note = poseidon.squeeze();
            }

            current_note.eq(&root)
        }
    }
}
