#[allow(unused)]
mod shielder;

#[allow(unused)]
mod psp22;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path, str::FromStr};

    use aleph_client::{AccountId, Connection, KeyPair};
    use anyhow::Result;
    use serde::Deserialize;

    use crate::{psp22::PSP22Token, shielder::Shielder};

    #[derive(Debug, Deserialize)]
    struct Addresses {
        shielder_address: AccountId,
        token_a_address: AccountId,
        token_b_address: AccountId,
    }

    struct TestContext {
        shielder: Shielder,
        token_a: PSP22Token,
        token_b: PSP22Token,
        connection: Connection,
        sudo: KeyPair,
        damian: KeyPair,
        hans: KeyPair,
    }

    impl TestContext {
        async fn local() -> Result<Self> {
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

    #[tokio::test]
    pub async fn dummy() -> Result<()> {
        let TestContext {
            shielder,
            token_a,
            token_b,
            connection,
            sudo,
            damian,
            hans,
        } = TestContext::local().await?;
        let dbalance = token_a.balance_of(&connection, &damian.account_id()).await;
        println!("TokenA.balance_of(DAMIAN)={:?}", dbalance);
        Ok(())
    }
}
