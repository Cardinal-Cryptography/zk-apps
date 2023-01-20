use std::path::Path;

use aleph_client::{
    contract::{event::get_contract_events, ContractInstance, ConvertibleValue},
    AccountId, AsConnection, SignedConnection,
};
use anyhow::{anyhow, Result};
use relations::{
    bytes_from_note, FrontendMerklePath, FrontendMerkleRoot, FrontendNote, FrontendNullifier,
    FrontendTokenAmount, FrontendTokenId,
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct Shielder {
    contract: ContractInstance,
}

impl Shielder {
    pub fn new(address: &AccountId, metadata_path: &Path) -> Result<Self> {
        Ok(Self {
            contract: ContractInstance::new(address.clone(), metadata_path.to_str().unwrap())?,
        })
    }

    /// Call `deposit` message of the contract. If successful, return leaf idx.
    pub async fn deposit(
        &self,
        connection: &SignedConnection,
        token_id: FrontendTokenId,
        token_amount: FrontendTokenAmount,
        note: FrontendNote,
        proof: &[u8],
    ) -> Result<u32> {
        let note_bytes = bytes_from_note(&note);

        let args = [
            &*token_id.to_string(),
            &*token_amount.to_string(),
            &*format!("0x{}", hex::encode(note_bytes)),
            &*format!("0x{}", hex::encode(proof)),
        ];

        debug!("Calling deposit tx with arguments {:?}", &args);

        let tx_info = self
            .contract
            .contract_exec(connection, "deposit", &args)
            .await?;
        let events =
            get_contract_events(connection.as_connection(), &self.contract, tx_info).await?;
        let event = match &*events {
            [event] if event.name == Some("Deposited".into()) => Ok(event),
            _ => Err(anyhow!(
                "Expected a single deposit event to be emitted. Found: {events:?}"
            )),
        }?;

        if let Some(leaf_idx) = event.data.get("leaf_idx") {
            let leaf_idx = ConvertibleValue(leaf_idx.clone()).try_into()?;
            info!("Successfully deposited tokens.");
            Ok(leaf_idx)
        } else {
            Err(anyhow!("Failed to read event data"))
        }
    }

    /// Call `withdraw` message of the contract. If successful, return leaf idx.
    #[allow(clippy::too_many_arguments)]
    pub async fn withdraw(
        &self,
        connection: &SignedConnection,
        token_id: FrontendTokenId,
        value: FrontendTokenAmount,
        recipient: AccountId,
        fee_for_caller: FrontendTokenAmount,
        merkle_root: FrontendMerkleRoot,
        old_nullifier: FrontendNullifier,
        new_note: FrontendNote,
        proof: &[u8],
    ) -> Result<u32> {
        let new_note_bytes = bytes_from_note(&new_note);
        let merkle_root_bytes = bytes_from_note(&merkle_root);

        let args = [
            &*token_id.to_string(),
            &*value.to_string(),
            &*recipient.to_string(),
            &*format!("{:?}", Some(fee_for_caller)),
            &*format!("0x{}", hex::encode(merkle_root_bytes)),
            &*old_nullifier.to_string(),
            &*format!("0x{}", hex::encode(new_note_bytes)),
            &*format!("0x{}", hex::encode(proof)),
        ];

        debug!("Calling withdraw tx with arguments {:?}", &args);

        let tx_info = self
            .contract
            .contract_exec(connection, "withdraw", &args)
            .await?;

        let events =
            get_contract_events(connection.as_connection(), &self.contract, tx_info).await?;
        let event = match &*events {
            [event] if event.name == Some("Withdrawn".into()) => Ok(event),
            _ => Err(anyhow!(
                "Expected a single withdrawal event to be emitted. Found: {events:?}"
            )),
        }?;

        if let Some(leaf_idx) = event.data.get("leaf_idx") {
            let leaf_idx = ConvertibleValue(leaf_idx.clone()).try_into()?;
            info!("Successfully withdrawn tokens.");
            Ok(leaf_idx)
        } else {
            Err(anyhow!("Failed to read event data"))
        }
    }

    /// Fetch the current merkle root.
    pub async fn get_merkle_root(&self, connection: &SignedConnection) -> FrontendMerkleRoot {
        self.contract
            .contract_read0(connection, "current_merkle_root")
            .await
            .unwrap()
    }

    /// Fetch the current merkle root.
    pub async fn get_merkle_path(
        &self,
        connection: &SignedConnection,
        leaf_idx: u32,
    ) -> Option<FrontendMerklePath> {
        self.contract
            .contract_read(connection, "merkle_path", &[&*leaf_idx.to_string()])
            .await
            .unwrap()
    }
}
