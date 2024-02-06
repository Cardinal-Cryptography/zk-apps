use halo2_base::{utils::BigPrimeField, AssignedValue, Context};

use crate::operation::{CircuitOperation, Operation};

#[derive(Clone, Copy, Debug)]
pub enum DummyOperation<Amount, TokenId, AccountId>
where
    Amount: BigPrimeField,
    AccountId: From<[u8; 32]>,
{
    Deposit(Amount, TokenId, AccountId),
}

#[derive(Clone, Copy, Debug)]
pub enum DummyCircuitOperation<F>
where
    F: BigPrimeField,
{
    Deposit(AssignedValue<F>, AssignedValue<F>, AssignedValue<F>),
}

impl<Amount, TokenId, AccountId> DummyOperation<Amount, TokenId, AccountId>
where
    Amount: BigPrimeField,
    AccountId: From<[u8; 32]>,
{
    pub fn load(&self, ctx: &mut Context<Amount>) -> DummyCircuitOperation<Amount> {
        match self {
            DummyOperation::Deposit(amount, _token, _account) => DummyCircuitOperation::Deposit(
                ctx.load_witness(*amount),
                ctx.load_zero(),
                ctx.load_zero(),
            ),
        }
    }
}

impl<Amount, TokenId, AccountId> Operation<Amount> for DummyOperation<Amount, TokenId, AccountId>
where
    Amount: BigPrimeField,
    AccountId: From<[u8; 32]>,
{
    type OpPriv = ();
    type OpPub = Self;

    fn combine(_op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self> {
        Some(op_pub)
    }
}

impl<Amount> Into<Vec<AssignedValue<Amount>>> for DummyCircuitOperation<Amount>
where
    Amount: BigPrimeField,
{
    fn into(self) -> Vec<AssignedValue<Amount>> {
        match self {
            DummyCircuitOperation::Deposit(amount, _token, _account) => vec![amount],
        }
    }
}

impl<Amount> CircuitOperation<Amount> for DummyCircuitOperation<Amount>
where
    Amount: BigPrimeField,
{
    type OpPriv = ();

    type OpPub = Self;

    fn combine(_op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self> {
        Some(op_pub)
    }
}
