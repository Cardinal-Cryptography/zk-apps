use halo2_base::{
    gates::{GateChip, GateInstructions},
    utils::{BigPrimeField, ScalarField},
    AssignedValue, Context,
};

use super::operation::{DummyCircuitOperation, DummyOperation};
use crate::{
    account::{Account, CircuitAccount},
    Token,
};

#[derive(Clone, Copy, Debug)]
pub struct DummyAccount<F: ScalarField> {
    pub balance_azero: F,
    pub balance_usdt: F,
}

#[derive(Clone, Copy, Debug)]
pub struct DummyCircuitAccount<F: ScalarField> {
    pub balance_azero: AssignedValue<F>,
    pub balance_usdt: AssignedValue<F>,
}

impl<F: ScalarField> DummyAccount<F> {
    pub fn new(balance_azero: F, balance_usdt: F) -> Self {
        Self {
            balance_azero,
            balance_usdt,
        }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> DummyCircuitAccount<F> {
        DummyCircuitAccount {
            balance_azero: ctx.load_witness(self.balance_azero),
            balance_usdt: ctx.load_witness(self.balance_usdt),
        }
    }
}

impl<F: BigPrimeField> Account<F> for DummyAccount<F> {
    type Op = DummyOperation<F, [u8; 32]>;

    fn update(&self, op: Self::Op) -> Self {
        let mut result = *self;
        match op {
            DummyOperation::Deposit(amount, token, _account) => match token {
                Token::AZERO => result.balance_azero += amount,
                Token::USDT => result.balance_usdt += amount,
            },
            DummyOperation::Withdraw(amount, token, _account) => match token {
                Token::AZERO => result.balance_azero -= amount,
                Token::USDT => result.balance_usdt -= amount,
            },
        };
        result
    }

    fn to_array(&self) -> Vec<F> {
        vec![self.balance_azero, self.balance_usdt]
    }
}

#[allow(unused_variables, unreachable_patterns)]
impl<F: BigPrimeField> CircuitAccount<F> for DummyCircuitAccount<F> {
    type Op = DummyCircuitOperation<F>;

    fn update(&self, op: Self::Op, ctx: &mut Context<F>, gate: &GateChip<F>) -> Self {
        let mut result = *self;

        let zero = ctx.load_zero();
        let one = ctx.load_constant(F::ONE);

        match op {
            DummyCircuitOperation::Deposit(amount, token, _account) => match token {
                zero => result.balance_azero = gate.add(ctx, result.balance_azero, amount),
                one => result.balance_usdt = gate.add(ctx, result.balance_usdt, amount),
                _ => {}
            },
            DummyCircuitOperation::Withdraw(amount, token, _account) => match token {
                zero => result.balance_azero = gate.sub(ctx, result.balance_azero, amount),
                one => result.balance_usdt = gate.sub(ctx, result.balance_usdt, amount),
                _ => {}
            },
        };
        result
    }

    fn to_array(&self) -> Vec<AssignedValue<F>> {
        vec![self.balance_azero, self.balance_usdt]
    }
}

use halo2_base::halo2_proofs::halo2curves::{bn256::Fr, ff::PrimeField};

#[test]
fn test_create_account() {
    let account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    assert_eq!(account.balance_azero, Fr::from_u128(0u128));
    assert_eq!(account.balance_usdt, Fr::from_u128(0u128));
}

#[test]
fn test_update_account() {
    let account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    let first_operation = DummyOperation::Deposit(Fr::from_u128(10u128), Token::AZERO, [0_u8; 32]);
    let second_operation = DummyOperation::Deposit(Fr::from_u128(20u128), Token::USDT, [0_u8; 32]);

    let account = account.update(first_operation);
    assert_eq!(account.balance_azero, Fr::from_u128(10u128));
    assert_eq!(account.balance_usdt, Fr::from_u128(0u128));

    let account = account.update(second_operation);
    assert_eq!(account.balance_azero, Fr::from_u128(10u128));
    assert_eq!(account.balance_usdt, Fr::from_u128(20u128));
}

#[test]
fn test_to_array() {
    let account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    let first_operation = DummyOperation::Deposit(Fr::from_u128(10u128), Token::AZERO, [0_u8; 32]);
    let second_operation = DummyOperation::Deposit(Fr::from_u128(20u128), Token::USDT, [0_u8; 32]);

    let account = account.update(first_operation).update(second_operation);

    let account_array = account.to_array();
    assert_eq!(account_array.len(), 2);
    assert_eq!(account_array[0], Fr::from_u128(10u128));
    assert_eq!(account_array[1], Fr::from_u128(20u128));
}
