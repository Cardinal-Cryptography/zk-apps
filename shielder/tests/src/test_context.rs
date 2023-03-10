use std::{fs::File, path::Path};

use aleph_client::{AccountId, Connection, KeyPair};
use anyhow::Result;
use liminal_ark_relations::{CanonicalDeserialize, Groth16, ProvingSystem};
use serde::Deserialize;

use crate::{psp22::PSP22Token, shielder::Shielder};

type ProvingKey = <Groth16 as ProvingSystem>::ProvingKey;

#[derive(Debug, Deserialize)]
struct Addresses {
    shielder_address: AccountId,
    token_a_address: AccountId,
    token_b_address: AccountId,
}

pub struct TestContext {
    pub shielder: Shielder,
    pub token_a: PSP22Token,
    pub token_b: PSP22Token,
    pub connection: Connection,
    pub sudo: KeyPair,
    pub damian: KeyPair,
    pub hans: KeyPair,
    pub deposit_pk: ProvingKey,
    pub deposit_and_merge_pk: ProvingKey,
    pub withdraw_pk: ProvingKey,
}

impl TestContext {
    pub async fn local() -> Result<Self> {
        let resources_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let addresses: Addresses =
            serde_json::from_reader(File::open(resources_path.join("addresses.json"))?)?;

        let shielder = Shielder::new(
            addresses.shielder_address,
            resources_path.join("shielder.json").to_str().unwrap(),
        )?;
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
            shielder,
            token_a,
            token_b,
            connection,
            sudo,
            damian,
            hans,
            deposit_pk,
            deposit_and_merge_pk,
            withdraw_pk,
        })
    }
}
