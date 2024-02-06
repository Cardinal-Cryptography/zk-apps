use halo2_base::{
    gates::GateChip, poseidon::hasher::PoseidonHasher, utils::BigPrimeField, AssignedValue, Context,
};
use poseidon::Poseidon;

use crate::{
    account::{Account, CircuitAccount},
    note::{CircuitNote, Note},
};

pub trait OuterHasher<F: BigPrimeField> {
    fn hash_account(&mut self, account: &impl Account<F>) -> F;
    fn hash_note(&mut self, note: &Note<F>) -> F;
}

impl<F: BigPrimeField, const T: usize, const RATE: usize> OuterHasher<F> for Poseidon<F, T, RATE> {
    fn hash_account(&mut self, account: &impl Account<F>) -> F {
        self.update(&account.to_array());
        self.squeeze()
    }

    fn hash_note(&mut self, note: &Note<F>) -> F {
        self.update(&note.to_array());
        self.squeeze()
    }
}

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
