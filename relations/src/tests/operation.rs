use halo2_base::{utils::BigPrimeField, AssignedValue, Context};

use crate::{
    operation::{CircuitOperation, Operation},
    Token,
};

#[derive(Clone, Copy, Debug)]
pub enum DummyOperation<Amount, AccountId>
where
    Amount: BigPrimeField,
    AccountId: From<[u8; 32]>,
{
    Deposit(Amount, Token, AccountId),
    Withdraw(Amount, Token, AccountId),
}

#[derive(Clone, Copy, Debug)]
pub enum DummyCircuitOperation<F>
where
    F: BigPrimeField,
{
    Deposit(AssignedValue<F>, AssignedValue<F>, AssignedValue<F>),
    Withdraw(AssignedValue<F>, AssignedValue<F>, AssignedValue<F>),
}

impl<Amount, AccountId> DummyOperation<Amount, AccountId>
where
    Amount: BigPrimeField,
    AccountId: From<[u8; 32]>,
{
    pub fn load(&self, ctx: &mut Context<Amount>) -> DummyCircuitOperation<Amount> {
        match self {
            DummyOperation::Deposit(amount, token, _account) => {
                let token = match token {
                    Token::AZERO => Amount::from_u128(0u128),
                    Token::USDT => Amount::from_u128(1u128),
                };
                DummyCircuitOperation::Deposit(
                    ctx.load_witness(*amount),
                    ctx.load_witness(token),
                    ctx.load_zero(),
                )
            }
            DummyOperation::Withdraw(amount, token, _account) => {
                let token = match token {
                    Token::AZERO => Amount::from_u128(0u128),
                    Token::USDT => Amount::from_u128(1u128),
                };
                DummyCircuitOperation::Withdraw(
                    ctx.load_witness(*amount),
                    ctx.load_witness(token),
                    ctx.load_zero(),
                )
            }
        }
    }
}

impl<Amount, AccountId> Operation<Amount> for DummyOperation<Amount, AccountId>
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

impl<Amount> From<DummyCircuitOperation<Amount>> for Vec<AssignedValue<Amount>>
where
    Amount: BigPrimeField,
{
    fn from(op: DummyCircuitOperation<Amount>) -> Vec<AssignedValue<Amount>> {
        match op {
            DummyCircuitOperation::Deposit(amount, token, _account) => vec![amount, token],
            DummyCircuitOperation::Withdraw(amount, token, _account) => vec![amount, token],
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
