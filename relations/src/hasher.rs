use halo2_base::{
    gates::GateChip, poseidon::hasher::PoseidonHasher, utils::BigPrimeField, AssignedValue, Context,
};

use crate::{account::CircuitAccount, note::CircuitNote};

pub trait InnerHasher<F: BigPrimeField> {
    fn hash_account(
        &mut self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        account: &impl CircuitAccount<F>,
    ) -> AssignedValue<F>;

    fn hash_note(
        &mut self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        note: &CircuitNote<F>,
    ) -> AssignedValue<F>;
}

impl<F: BigPrimeField, const T: usize, const RATE: usize> InnerHasher<F>
    for PoseidonHasher<F, T, RATE>
{
    fn hash_account(
        &mut self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        account: &impl CircuitAccount<F>,
    ) -> AssignedValue<F> {
        let account_array = account.to_array();
        self.hash_fix_len_array(ctx, gate, &account_array)
    }

    fn hash_note(
        &mut self,
        ctx: &mut Context<F>,
        gate: &GateChip<F>,
        note: &CircuitNote<F>,
    ) -> AssignedValue<F> {
        let note_array = note.to_array();
        self.hash_fix_len_array(ctx, gate, &note_array)
    }
}

// #[cfg(test)]
pub mod tests {
    use poseidon::Poseidon;

    use super::*;
    use crate::{
        account::Account,
        note::Note,
        poseidon_consts::{R_F, R_P},
    };

    pub trait OuterHasher<F: BigPrimeField> {
        fn hash_account(account: &impl Account<F>) -> F;
        fn hash_note(note: &Note<F>) -> F;
        fn hash_two_to_one(elements: &[F]) -> F;
    }

    impl<F: BigPrimeField, const T: usize, const RATE: usize> OuterHasher<F> for Poseidon<F, T, RATE> {
        fn hash_account(account: &impl Account<F>) -> F {
            let mut poseidon = Self::new(R_F, R_P);
            poseidon.update(&account.to_array());
            poseidon.squeeze()
        }

        fn hash_note(note: &Note<F>) -> F {
            let mut poseidon = Self::new(R_F, R_P);
            poseidon.update(&note.to_array());
            poseidon.squeeze()
        }

        fn hash_two_to_one(elements: &[F]) -> F {
            let mut poseidon = Self::new(R_F, R_P);
            poseidon.update(elements);
            poseidon.squeeze()
        }
    }
}
