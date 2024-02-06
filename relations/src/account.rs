use halo2_base::{gates::GateChip, utils::BigPrimeField, AssignedValue, Context};

use crate::operation::{CircuitOperation, Operation};

/// Trait representing an account in a financial system.
pub trait Account<F: BigPrimeField> {
    /// The type of operation associated with the account.
    type Op: Operation<F>;

    /// Updates the account with the given operation and returns the updated account.
    fn update(&self, op: Self::Op) -> Self;

    /// Converts the account to an array of field elements.
    fn to_array(&self) -> Vec<F>;
}

pub trait CircuitAccount<F: BigPrimeField> {
    /// The type of operation associated with the account.
    type Op: CircuitOperation<F>;

    /// Updates the account with the given operation and returns the updated account.
    fn update(&self, op: Self::Op, ctx: &mut Context<F>, gate: &GateChip<F>) -> Self;

    /// Converts the account to an array of field elements.
    fn to_array(&self) -> Vec<AssignedValue<F>>;
}
