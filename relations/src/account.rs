use halo2_base::{gates::GateChip, utils::BigPrimeField, AssignedValue, Context};

use crate::operation::{CircuitOperation, Operation};

pub trait Account<F: BigPrimeField> {
    type CircuitAccount: CircuitAccount<F>;
    type Op: Operation<F>;

    fn update(&self, op: Self::Op) -> Self;

    fn to_array(&self) -> Vec<F>;

    fn load(&self, ctx: &mut Context<F>) -> Self::CircuitAccount;
}

pub trait CircuitAccount<F: BigPrimeField> {
    type Op: CircuitOperation<F>;

    fn update(&self, op: Self::Op, ctx: &mut Context<F>, gate: &GateChip<F>) -> Self;

    fn to_array(&self) -> Vec<AssignedValue<F>>;
}
