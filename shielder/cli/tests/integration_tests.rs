// Note: All integration test will be run against shared state and accounts. Therefore, you have to
// ensure that your case won't prevent other from succeeding. In particular keep in mind that
// Shielder allowance is shared. Remember also that the execution order is not deterministic.

pub mod psp22;
pub mod shielder_wrapper;
pub mod utils;

use aleph_client::AccountId;
use anyhow::Result;
use liminal_ark_relations::shielder::types::FrontendTokenAmount;
use serial_test::serial;
use shielder::{deposit, merge};
use tracing::info;

use crate::utils::{TestContext, TOKEN_A_ID};

const NO_FEE: FrontendTokenAmount = 0;
const WITHDRAW_ALL: Option<FrontendTokenAmount> = None;
const WITHDRAW_TO_ISSUER: Option<AccountId> = None;

#[tokio::test]
#[ignore]
#[serial]
async fn basic_interaction() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        ..
    } = TestContext::local().await?;

    let damian_balance_before_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    let shield_amount = 10;

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_before_shield,
            "Balance before shielding");

    let deposit_id = damian
        .shield(TOKEN_A_ID, shield_amount, &shielder)
        .await
        .unwrap();

    let damian_balance_after_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_shield,
            "Balance after shielding");

    assert_eq!(
        damian_balance_after_shield + shield_amount,
        damian_balance_before_shield,
        "Shielding should decrease balance"
    );

    let prev_deposit = damian
        .get_deposit(deposit_id)
        .expect("deposit to exist since we just created it");

    damian
        .unshield(
            &shielder,
            prev_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await
        .unwrap();

    info!("Tokens unshielded");

    let damian_balance_after_unshield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_unshield,
            "Balance after unshielding");

    assert_eq!(damian_balance_after_unshield, damian_balance_before_shield);
    Ok(())
}

#[tokio::test]
#[ignore]
#[serial]
async fn deposit_and_merge() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        ..
    } = TestContext::local().await?;

    let damian_balance_at_start = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_at_start,
                "Balance before shielding");

    let first_shield_amount = 10;

    let first_deposit_id = damian
        .shield(TOKEN_A_ID, first_shield_amount, &shielder)
        .await
        .unwrap();

    let damian_balance_after_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_shield,
            "Balance after shielding");

    let first_deposit = damian.get_deposit(first_deposit_id).unwrap();
    let second_shield_amount = 5;

    let merged_deposit_id = deposit::deposit_and_merge(
        first_deposit.clone(),
        second_shield_amount,
        &shielder.deposit_and_merge_pk_file,
        &damian.conn,
        &shielder.instance,
        &mut damian.app_state,
    )
    .await
    .unwrap();

    let damian_balance_after_merging = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_merging,
            "Balance after merging");

    // We should not be able to withdraw with nullifier and trapdoor of the first deposit.
    let res = damian
        .unshield(
            &shielder,
            first_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await;
    assert!(res.is_err());

    // Damian's token balance should be unchanged.
    let damian_balance_after_failed_withdrawal = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_after_failed_withdrawal, damian_balance_after_merging,
        "Failed unshielding shouldn't change account's balance"
    );

    let merged_deposit = damian.get_deposit(merged_deposit_id).unwrap();

    let _ = damian
        .unshield(
            &shielder,
            merged_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await
        .expect("Withdrawing merged note should succeed");

    let damian_balance_after_unshielding = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_unshielding,
            "Balance after unshielding merged notes");

    assert_eq!(damian_balance_after_unshielding, damian_balance_at_start);

    Ok(())
}

#[tokio::test]
#[ignore]
#[serial]
async fn merge() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        ..
    } = TestContext::local().await?;

    let damian_balance_at_start = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_at_start,
                "Balance before shielding");

    let first_shield_amount = 10;
    let first_deposit_id = damian
        .shield(TOKEN_A_ID, first_shield_amount, &shielder)
        .await
        .unwrap();

    let damian_balance_after_first_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_first_shield,
            "Balance after first shielding event");

    let second_shield_amount = 5;
    let second_deposit_id = damian
        .shield(TOKEN_A_ID, second_shield_amount, &shielder)
        .await
        .unwrap();

    let damian_balance_after_second_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_second_shield,
            "Balance after second shielding event");

    let first_deposit = damian.get_deposit(first_deposit_id).unwrap();
    let second_deposit = damian.get_deposit(second_deposit_id).unwrap();

    let merged_deposit_id = merge::merge(
        first_deposit.clone(),
        second_deposit.clone(),
        &shielder.merge_pk_file,
        &damian.conn,
        &shielder.instance,
        &mut damian.app_state,
    )
    .await
    .unwrap();

    let damian_balance_after_merging = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_merging,
            "Balance after merging");

    // We should not be able to withdraw with the nullifier and trapdoor of the second deposit.
    let res = damian
        .unshield(
            &shielder,
            second_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await;
    assert!(res.is_err());

    // Damian's token balance should be unchanged.
    let damian_balance_after_failed_withdrawal = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_after_failed_withdrawal, damian_balance_after_merging,
        "Failed unshielding shouldn't change account's balance"
    );

    let merged_deposit = damian.get_deposit(merged_deposit_id).unwrap();

    let _ = damian
        .unshield(
            &shielder,
            merged_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await
        .expect("Withdrawing merged note should succeed");

    let damian_balance_after_unshielding = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after_unshielding,
            "Balance after unshielding merged notes");

    assert_eq!(
        damian_balance_after_unshielding, damian_balance_at_start,
        "Balance should not change once shielding, merging and unshielding is completed"
    );

    Ok(())
}

#[tokio::test]
#[ignore]
#[serial]
async fn withdraw_partial() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        ..
    } = TestContext::local().await?;

    let damian_balance_at_start = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    let shield_amount = 10;

    let deposit_id = damian
        .shield(TOKEN_A_ID, shield_amount, &shielder)
        .await
        .unwrap();

    let damian_balance_after_shield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_after_shield + shield_amount as u128,
        damian_balance_at_start,
        "Shielding should decrease balance"
    );

    let diff_partial = 4;

    let prev_deposit = damian
        .get_deposit(deposit_id)
        .expect("deposit to exist since we just created it");
    let unshield_amount = prev_deposit.token_amount - diff_partial;

    damian
        .unshield(
            &shielder,
            prev_deposit,
            Some(unshield_amount),
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await
        .unwrap();

    let damian_balance_after_partial_unshield = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_at_start - diff_partial as u128,
        damian_balance_after_partial_unshield
    );

    // partial unshield replaces the deposit under previous id.
    let partial_deposit = damian.get_deposit(deposit_id).unwrap();
    assert_eq!(partial_deposit.token_amount, diff_partial);

    damian
        .unshield(
            &shielder,
            partial_deposit,
            WITHDRAW_ALL,
            NO_FEE,
            WITHDRAW_TO_ISSUER,
        )
        .await
        .unwrap();

    let damian_balance_after_unshielding_all = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_after_unshielding_all,
        damian_balance_at_start
    );

    Ok(())
}

#[tokio::test]
#[ignore]
#[serial]
async fn withdraw_via_relayer() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        mut hans,
        ..
    } = TestContext::local().await?;

    let damian_balance_at_start = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    // Hansu will be our trusted relayer.
    let hansu_balance_before_relaying = token_a
        .balance_of(&connection, &hans.account_id)
        .await
        .unwrap();

    let shield_amount = 10;

    let fee_amount = 1;

    let deposit_id = damian
        .shield(TOKEN_A_ID, shield_amount, &shielder)
        .await
        .unwrap();

    let deposit = damian.get_deposit(deposit_id).unwrap();

    // Hansu here acts as a relayer: gets the fee but also withdraws
    // to a different recipient - Damian.
    hans.unshield(
        &shielder,
        deposit,
        WITHDRAW_ALL,
        fee_amount,
        Some(damian.account_id.clone()),
    )
    .await
    .unwrap();

    let hansu_balance_after_relaying = token_a
        .balance_of(&connection, &hans.account_id)
        .await
        .unwrap();

    assert_eq!(
        hansu_balance_before_relaying + fee_amount,
        hansu_balance_after_relaying,
        "Fee should go to relayer"
    );

    let damian_balance_after_unshielding = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    assert_eq!(
        damian_balance_at_start,
        damian_balance_after_unshielding + fee_amount
    );

    Ok(())
}

#[tokio::test]
#[ignore]
#[serial]
async fn shielding_fails_insufficient_balance() -> Result<()> {
    let TestContext {
        shielder,
        token_a,
        connection,
        mut damian,
        ..
    } = TestContext::local().await?;

    let damian_balance_at_start = token_a
        .balance_of(&connection, &damian.account_id)
        .await
        .unwrap();

    let shield_amount = damian_balance_at_start + 1;

    let shield_result = damian.shield(TOKEN_A_ID, shield_amount, &shielder).await;

    // Expected to fail.
    // Can't match on the returned type as we're dry-running calls (aleph-client's behavior)
    // in which case the failure is not encoded as err in the return type
    // but rather Ok(_) and a special flag in the response.
    assert!(shield_result.is_err());

    Ok(())
}
