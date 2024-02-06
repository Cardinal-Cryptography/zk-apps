use halo2_base::{
    halo2_proofs::halo2curves::{bn256::Fr, ff::PrimeField},
    utils::testing::base_test,
};
use poseidon::Poseidon;

use crate::{
    account::Account,
    hasher::OuterHasher,
    poseidon_consts::{RATE, R_F, R_P, T},
    relations::update_account::{update_account_circuit, UpdateAccountInput},
    tests::{account::DummyAccount, operation::DummyOperation},
    Token,
};

#[test]
fn test() {
    let old_account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    let operation = DummyOperation::Deposit(Fr::from_u128(100u128), Token::AZERO, [0u8; 32]);
    let new_account = old_account.update(operation);

    let old_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&old_account);
    let new_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&new_account);

    let mut make_public = Vec::new();

    base_test().k(9).expect_satisfied(true).run(|ctx, _| {
        let old_account_hash = ctx.load_witness(old_account_hash);
        let new_account_hash = ctx.load_witness(new_account_hash);
        let operation = operation.load(ctx);
        let old_account = old_account.load(ctx);
        let input =
            UpdateAccountInput::new(old_account_hash, new_account_hash, operation, old_account);
        update_account_circuit(ctx, input, &mut make_public);
    });
}

#[test]
fn test2() {
    let old_account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    let operation = DummyOperation::Deposit(Fr::from_u128(100u128), Token::AZERO, [0u8; 32]);
    let new_account = old_account.update(operation);

    let old_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&old_account);
    let new_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&new_account);

    let mut make_public = Vec::new();

    base_test().k(9).expect_satisfied(false).run(|ctx, _| {
        let old_account_hash = ctx.load_witness(new_account_hash);
        let new_account_hash = ctx.load_witness(new_account_hash);
        let operation = operation.load(ctx);
        let old_account = old_account.load(ctx);
        let input =
            UpdateAccountInput::new(old_account_hash, new_account_hash, operation, old_account);
        update_account_circuit(ctx, input, &mut make_public);
    });
}
