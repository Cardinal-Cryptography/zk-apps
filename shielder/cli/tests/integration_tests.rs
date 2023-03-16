#[allow(unused)]
mod psp22;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path, str::FromStr};

    use aleph_client::{AccountId, Connection, KeyPair, SignedConnection};
    use anyhow::Result;
    use liminal_ark_relations::{
        serialize, CanonicalDeserialize, CircuitField, ConstraintSynthesizer, Groth16,
        ProvingSystem,
    };
    use serde::Deserialize;
    use shielder::deposit;
    use tracing::info;
    use tracing_subscriber::EnvFilter;

    use crate::{psp22::*, test_context::*};

    #[tokio::test]
    pub async fn deposit_decreases_balance() -> Result<()> {
        // We need to disable logging in our dependency crates by default.
        let filter = EnvFilter::new("info");

        let subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_target(false)
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

        let damian_balance_before = token_a
            .balance_of(&connection, &damian.account_id)
            .await
            .unwrap();

        let deposit_amount = 100u64;

        info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_before,
            "Balance before shielding");

        deposit::first_deposit(
            TOKEN_A_ID,
            deposit_amount,
            shielder.deposit_pk_file,
            SignedConnection::from_connection(connection.clone(), damian.keypair),
            shielder.instance,
            &mut damian.app_state,
        )
        .await
        .unwrap();

        let damian_balance_after = token_a
            .balance_of(&connection, &damian.account_id)
            .await
            .unwrap();

        info!(token_id = ?TOKEN_A_ID, account = ?damian.account_id, balance = ?damian_balance_after,
            "Balance after shielding");

        assert_eq!(
            damian_balance_after + deposit_amount as u128,
            damian_balance_before,
            "Shielding should decrease balance"
        );
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

    use aleph_client::{AccountId, Connection, KeyPair};
    use anyhow::Result;
    use psp22::PSP22Token;
    use serde::Deserialize;
    use shielder::app_state::AppState;

    use crate::{psp22, shielder::Shielder};

    pub(super) const TOKEN_A_ID: u16 = 0;
    pub(super) const TOKEN_B_ID: u16 = 1;

    #[derive(Debug, Deserialize)]
    pub(super) struct Addresses {
        shielder_address: AccountId,
        token_a_address: AccountId,
        token_b_address: AccountId,
    }

    pub(super) struct User {
        pub(super) account_id: AccountId,
        pub(super) keypair: KeyPair,
        pub(super) app_state: AppState,
    }

    impl User {
        pub(super) fn new(keypair: KeyPair) -> Self {
            let account_id = keypair.account_id().clone();
            let app_state = AppState::default();
            Self {
                account_id,
                keypair,
                app_state,
            }
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
                connection,
                sudo: User::new(sudo),
                damian: User::new(damian),
                hans: User::new(hans),
            })
        }
    }
}
