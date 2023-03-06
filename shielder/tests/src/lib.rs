#[allow(unused)]
mod shielder;

#[allow(unused)]
mod psp22;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use aleph_client::AccountId;
    use anyhow::Result;
    use serde::Deserialize;

    use crate::{psp22::PSP22Token, shielder::Shielder};

    #[derive(Debug, Deserialize)]
    struct Addresses {
        shielder_address: AccountId,
        token_a_address: AccountId,
        token_b_address: AccountId,
    }

    #[derive(Debug)]
    struct TestContext {
        shielder: Shielder,
        token_a: PSP22Token,
        token_b: PSP22Token,
    }

    impl TestContext {
        fn local() -> Result<Self> {
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

            Ok(Self {
                shielder,
                token_a,
                token_b,
            })
        }
    }

    #[tokio::test]
    pub async fn dummy() -> Result<()> {
        let test_context = TestContext::local()?;
        println!("{:?}", test_context);
        Ok(())
    }
}
