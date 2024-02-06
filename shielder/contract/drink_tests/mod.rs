mod utils;

use drink::{session::Session, AccountId32};

use crate::{
    drink_tests::utils::{chain::*, ops::*, psp22::*, shielder::*},
    mocked_zk::merkle::MerkleTree,
};

#[drink::contract_bundle_provider]
pub enum BundleProvider {}

#[drink::test]
fn deploy_single_deposit_single_withdraw(
    mut session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice: AccountId32 = init_alice(&mut session)?;
    let bob: AccountId32 = init_bob(&mut session)?;

    session = session.with_actor(alice.clone());

    let shielder_address = deploy_shielder(&mut session)?;
    let psp22_address = deploy_test_token(&mut session, 100)?;

    let mut merkle_tree = MerkleTree::new();

    // CREATE ACCOUNT
    let user_shielded_data = create_shielder_account(
        &mut session,
        &shielder_address,
        &psp22_address,
        &mut merkle_tree,
    )?;

    // APPROVE TRANSFER
    psp22_approve(&mut session, &psp22_address, &shielder_address, 10)?;

    // DEPOSIT
    let user_shielded_data = shielder_update(
        &mut session,
        &shielder_address,
        deposit_op(&psp22_address, &alice, 10),
        user_shielded_data,
        &mut merkle_tree,
    )?;

    let alice_psp22_balance: u128 = get_psp22_balance(&mut session, &psp22_address, &alice)?;
    assert_eq!(alice_psp22_balance, 90);
    let shielder_psp22_balance: u128 =
        get_psp22_balance(&mut session, &psp22_address, &shielder_address)?;
    assert_eq!(shielder_psp22_balance, 10);

    // SWITCH TO bob
    session = session.with_actor(bob.clone());

    // WITHDRAW
    let _ = shielder_update(
        &mut session,
        &shielder_address,
        withdraw_op(&psp22_address, &bob, 1),
        user_shielded_data,
        &mut merkle_tree,
    )?;

    let bob_psp22_balance: u128 = get_psp22_balance(&mut session, &psp22_address, &bob)?;
    assert_eq!(bob_psp22_balance, 1);
    let shielder_psp22_balance: u128 =
        get_psp22_balance(&mut session, &psp22_address, &shielder_address)?;
    assert_eq!(shielder_psp22_balance, 9);

    Ok(())
}

#[drink::test]
fn deploy_single_deposit_multiple_withdraw(
    mut session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice: AccountId32 = init_alice(&mut session)?;
    session = session.with_actor(alice.clone());

    let mut withdrawers: Vec<AccountId32> = vec![];
    for i in 3..11 {
        let acc = AccountId32::new([i as u8; 32]);
        init_acc_with_balance(&mut session, &acc)?;
        withdrawers.push(acc.clone());
    }

    let shielder_address = deploy_shielder(&mut session)?;
    let psp22_address = deploy_test_token(&mut session, 100)?;

    let mut merkle_tree = MerkleTree::new();

    // CREATE ACCOUNT
    let mut user_shielded_data = create_shielder_account(
        &mut session,
        &shielder_address,
        &psp22_address,
        &mut merkle_tree,
    )?;

    // APPROVE TRANSFER
    psp22_approve(&mut session, &psp22_address, &shielder_address, 50)?;

    let alice_shielder_allowance: u128 =
        get_psp22_allowance(&mut session, &psp22_address, &alice, &shielder_address)?;
    assert_eq!(alice_shielder_allowance, 50);

    // DEPOSIT
    user_shielded_data = shielder_update(
        &mut session,
        &shielder_address,
        deposit_op(&psp22_address, &alice, 50),
        user_shielded_data,
        &mut merkle_tree,
    )?;

    // SWITCH TO bob
    for withdrawer_addr in withdrawers {
        session = session.with_actor(withdrawer_addr.clone());

        // WITHDRAW
        user_shielded_data = shielder_update(
            &mut session,
            &shielder_address,
            withdraw_op(&psp22_address, &withdrawer_addr, 1),
            user_shielded_data,
            &mut merkle_tree,
        )?;
        let psp22_balance: u128 =
            get_psp22_balance(&mut session, &psp22_address, &withdrawer_addr)?;
        assert_eq!(psp22_balance, 1);
    }
    let shielder_psp22_balance: u128 =
        get_psp22_balance(&mut session, &psp22_address, &shielder_address)?;
    assert_eq!(shielder_psp22_balance, 42);

    Ok(())
}
