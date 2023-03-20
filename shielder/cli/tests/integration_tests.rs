pub mod psp22;
#[allow(unused)]
pub mod shielder_wrapper;
#[allow(unused)]
pub mod utils;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serial_test::serial;
    use shielder::{deposit, withdraw};
    use tracing::info;

    use crate::utils::{TestContext, TOKEN_A_ID};

    #[tokio::test]
    #[serial]
    async fn basic_interaction() -> Result<()> {
        let TestContext {
            shielder,
            token_a,
            token_b,
            connection,
            mut sudo,
            mut damian,
            mut hans,
        } = TestContext::local().await?;

        let damian_balance_before_shield = token_a
            .balance_of(&connection, &damian.account_id)
            .await
            .unwrap();

        let shield_amount = 100u64;

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
            damian_balance_after_shield + shield_amount as u128,
            damian_balance_before_shield,
            "Shielding should decrease balance"
        );

        let prev_deposit = damian
            .get_deposit(deposit_id)
            .expect("deposit to exist since we just created it");
        let deposit_amount = prev_deposit.token_amount;

        damian
            .unshield(&shielder, prev_deposit, None, 0)
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
    #[serial]
    async fn deposit_and_merge() -> Result<()> {
        let TestContext {
            shielder,
            token_a,
            token_b,
            connection,
            mut sudo,
            mut damian,
            mut hans,
        } = TestContext::local().await?;

        let damian_balance_at_start = token_a
            .balance_of(&connection, &damian.account_id)
            .await
            .unwrap();

        info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_at_start,
                "Balance before shielding");

        let first_shield_amount = 100u64;

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
        let second_shield_amount = 50u64;

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
        let res = damian.unshield(&shielder, first_deposit, None, 0).await;
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
            .unshield(&shielder, merged_deposit, None, 0)
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
    #[serial]
    async fn withdraw_partial() -> Result<()> {
        todo!()
    }

    #[tokio::test]
    #[serial]
    async fn fee_for_relayer() -> Result<()> {
        todo!()
    }

    #[tokio::test]
    #[serial]
    async fn withdraw_to_different_recipient() -> Result<()> {
        todo!()
    }

    #[tokio::test]
    #[serial]
    async fn withdraw_through_relayer() -> Result<()> {
        todo!()
    }
}
