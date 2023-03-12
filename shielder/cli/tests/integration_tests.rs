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

    use crate::{psp22::*, TestContext};

    #[tokio::test]
    pub async fn deposit_decreases_balance() -> Result<()> {
        let TestContext {
            shielder,
            token_a,
            token_b,
            connection,
            sudo,
            damian,
            hans,
        } = TestContext::local().await?;
        let dbalanceA = token_a
            .balance_of(&connection, &damian.account_id())
            .await
            .unwrap();
        println!("TokenA.balance_of(DAMIAN)={:?}", dbalanceA);
        Ok(())
    }
}

mod shielder {
    use std::path::Path;

    use aleph_client::AccountId;
    use anyhow::Result;
    use liminal_ark_relations::{CanonicalDeserialize, Groth16, ProvingSystem};
    use shielder::contract::Shielder as ShielderContract;

    use crate::ProvingKey;

    #[allow(unused)]
    pub(super) struct Shielder {
        instance: ShielderContract,
        deposit_pk: ProvingKey,
        deposit_and_merge_pk: ProvingKey,
        withdraw_pk: ProvingKey,
    }

    impl Shielder {
        pub(super) fn new(shielder_address: &AccountId, resources_path: &Path) -> Result<Self> {
            let shielder =
                ShielderContract::new(shielder_address, &resources_path.join("shielder.json"))?;

            let deposit_pk = {
                let pk_bytes = std::fs::read(resources_path.join("deposit.groth16.vk.bytes"))?;
                <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?
            };

            let deposit_and_merge_pk = {
                let pk_bytes =
                    std::fs::read(resources_path.join("deposit_and_merge.groth16.vk.bytes"))?;
                <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?
            };

            let withdraw_pk = {
                let pk_bytes = std::fs::read(resources_path.join("withdraw.groth16.vk.bytes"))?;
                <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?
            };

            Ok(Self {
                instance: shielder,
                deposit_pk,
                deposit_and_merge_pk,
                withdraw_pk,
            })
        }
    }
}

use std::{fs::File, path::Path};

use aleph_client::{AccountId, Connection, KeyPair};
use anyhow::Result;
use liminal_ark_relations::{Groth16, ProvingSystem};
use psp22::PSP22Token;
use serde::Deserialize;

use crate::shielder::Shielder;

type ProvingKey = <Groth16 as ProvingSystem>::ProvingKey;

#[derive(Debug, Deserialize)]
struct Addresses {
    shielder_address: AccountId,
    token_a_address: AccountId,
    token_b_address: AccountId,
}

struct TestContext {
    pub shielder: Shielder,
    pub token_a: PSP22Token,
    pub token_b: PSP22Token,
    pub connection: Connection,
    pub sudo: KeyPair,
    pub damian: KeyPair,
    pub hans: KeyPair,
}

impl TestContext {
    async fn local() -> Result<Self> {
        let resources_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
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
            sudo,
            damian,
            hans,
        })
    }
}
