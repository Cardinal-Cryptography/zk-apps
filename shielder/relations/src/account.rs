use halo2_base::{gates::GateChip, utils::BigPrimeField, AssignedValue, Context};

use crate::{
    operation::{CircuitOperation, Operation},
    CloneToVec,
};

pub trait Account<F: BigPrimeField>: CloneToVec<F> {
    type CircuitAccount: CircuitAccount<F>;
    type Op: Operation<F>;

    fn update(&self, op: &Self::Op) -> Self;

    fn load(&self, ctx: &mut Context<F>) -> Self::CircuitAccount;
}

pub trait CircuitAccount<F: BigPrimeField>: CloneToVec<AssignedValue<F>> {
    type Op: CircuitOperation<F>;

    fn update(&self, op: Self::Op, ctx: &mut Context<F>, gate: &GateChip<F>) -> Self;
}
