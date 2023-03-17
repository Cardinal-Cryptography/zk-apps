#[allow(unused)]
mod psp22;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{env, fs::File, path::Path, str::FromStr};

    use aleph_client::{AccountId, Connection, KeyPair, SignedConnection};
    use anyhow::Result;
    use liminal_ark_relations::{
        serialize, CanonicalDeserialize, CircuitField, ConstraintSynthesizer, Groth16,
        ProvingSystem,
    };
    use serde::Deserialize;
    use shielder::{deposit, withdraw};
    use tracing::info;
    use tracing_subscriber::EnvFilter;

    use crate::{psp22::*, test_context::*};

    const LOG_CONFIGURATION_ENVVAR: &str = "RUST_LOG";

    #[tokio::test]
    async fn basic_interaction() -> Result<()> {
        // We need to disable logging in our dependency crates by default.
        let filter = EnvFilter::new(
            env::var(LOG_CONFIGURATION_ENVVAR)
                .as_deref()
                .unwrap_or("warn,shielder_cli=info,integration_tests::tests=debug"),
        );

        let subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_target(true)
            .with_env_filter(filter);

        subscriber.try_init().unwrap();

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
    async fn deposit_and_merge() -> Result<()> {
        // We need to disable logging in our dependency crates by default.
        let filter = EnvFilter::new(env::var(LOG_CONFIGURATION_ENVVAR).as_deref().unwrap_or(
            "warn,shielder_cli=info,integration_tests::tests=debug,aleph_client::contract=debug",
        ));

        let subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_target(true)
            .with_env_filter(filter);

        subscriber.try_init().unwrap();

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
}

mod shielder {
    use std::path::{Path, PathBuf};

    use aleph_client::AccountId;
    use anyhow::Result;
    use shielder::contract::Shielder as ShielderContract;

    #[allow(unused)]
    pub(super) struct Shielder {
        pub(super) instance: ShielderContract,
        pub(super) deposit_pk_file: PathBuf,
        pub(super) deposit_and_merge_pk_file: PathBuf,
        pub(super) withdraw_pk_file: PathBuf,
    }

    impl Shielder {
        pub(super) fn new(shielder_address: &AccountId, resources_path: &Path) -> Result<Self> {
            let shielder =
                ShielderContract::new(shielder_address, &resources_path.join("shielder.json"))?;

            Ok(Self {
                instance: shielder,
                deposit_pk_file: resources_path.join("deposit.pk.bytes"),
                deposit_and_merge_pk_file: resources_path.join("deposit_and_merge.pk.bytes"),
                withdraw_pk_file: resources_path.join("withdraw.pk.bytes"),
            })
        }
    }
}

mod test_context {
    use std::{fs::File, path::Path};

    use aleph_client::{AccountId, Connection, KeyPair, SignedConnection};
    use anyhow::Result;
    use liminal_ark_relations::{FrontendTokenAmount, FrontendTokenId};
    use psp22::PSP22Token;
    use serde::Deserialize;
    use shielder::{
        app_state::{AppState, Deposit},
        deposit, withdraw, DepositId,
    };

    use crate::{psp22, shielder::Shielder};

    pub(super) const TOKEN_A_ID: u16 = 0;

    #[derive(Debug, Deserialize)]
    pub(super) struct Addresses {
        shielder_address: AccountId,
        token_a_address: AccountId,
        token_b_address: AccountId,
    }

    pub(super) struct User {
        pub(super) account_id: AccountId,
        pub(super) app_state: AppState,
        pub(super) conn: SignedConnection,
    }

    impl User {
        pub(super) fn new(keypair: KeyPair, conn: Connection) -> Self {
            let account_id = keypair.account_id().clone();
            let app_state = AppState::default();
            let conn = SignedConnection::from_connection(conn, keypair.clone());
            Self {
                account_id,
                app_state,
                conn,
            }
        }

        pub(super) fn get_deposit(&self, deposit_id: DepositId) -> Option<Deposit> {
            self.app_state.get_deposit_by_id(deposit_id)
        }

        pub(super) async fn shield(
            &mut self,
            token_id: FrontendTokenId,
            token_amount: FrontendTokenAmount,
            shielder: &Shielder,
        ) -> Result<DepositId> {
            let deposit_id = deposit::first_deposit(
                token_id,
                token_amount,
                &shielder.deposit_pk_file,
                &self.conn,
                &shielder.instance,
                &mut self.app_state,
            )
            .await?;
            Ok(deposit_id)
        }

        pub(super) async fn unshield(
            &mut self,
            shielder: &Shielder,
            deposit: Deposit,
            amount: Option<FrontendTokenAmount>,
            fee: u64,
        ) -> Result<()> {
            let withdraw_amount = amount.unwrap_or(deposit.token_amount);
            withdraw::withdraw(
                &shielder.instance,
                &self.conn,
                deposit,
                withdraw_amount,
                &self.account_id,
                fee,
                &shielder.withdraw_pk_file,
                &mut self.app_state,
            )
            .await
        }
    }

    pub(super) struct TestContext {
        pub shielder: Shielder,
        pub token_a: PSP22Token,
        pub token_b: PSP22Token,
        pub connection: Connection,
        pub sudo: User,
        pub damian: User,
        pub hans: User,
    }

    impl TestContext {
        pub(super) async fn local() -> Result<Self> {
            let resources_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("resources");
            let addresses: Addresses =
                serde_json::from_reader(File::open(resources_path.join("addresses.json"))?)?;

            let shielder = Shielder::new(&addresses.shielder_address, &resources_path)?;

            let token_a = PSP22Token::new(
                addresses.token_a_address,
                resources_path.join("public_token.json").to_str().unwrap(),
            )?;
            let token_b = PSP22Token::new(
                addresses.token_b_address,
                resources_path.join("public_token.json").to_str().unwrap(),
            )?;

            let node_address = option_env!("NODE_WS")
                .unwrap_or_else(|| "ws://127.0.0.1:9944")
                .to_string();

            let connection = Connection::new(&node_address).await;

            let sudo = aleph_client::keypair_from_string("//Alice");
            let damian = aleph_client::keypair_from_string("//0");
            let hans = aleph_client::keypair_from_string("//1");

            Ok(Self {
                shielder,
                token_a,
                token_b,
                connection: connection.clone(),
                sudo: User::new(sudo, connection.clone()),
                damian: User::new(damian, connection.clone()),
                hans: User::new(hans, connection.clone()),
            })
        }
    }
}
