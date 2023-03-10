#[allow(unused)]
mod psp22;
#[allow(unused)]
mod shielder;
#[cfg(test)]
mod test_context;

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

    use crate::{psp22::PSP22Token, shielder::Shielder, test_context::*};

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
            deposit_pk,
            deposit_and_merge_pk,
            withdraw_pk,
        } = TestContext::local().await?;
        let dbalanceA = token_a
            .balance_of(&connection, &damian.account_id())
            .await
            .unwrap();
        println!("TokenA.balance_of(DAMIAN)={:?}", dbalanceA);
        Ok(())
    }
}
