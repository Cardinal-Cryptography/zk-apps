use std::{
    path::Path,
    sync::{mpsc::channel, Arc},
    thread,
    time::Duration,
};

use aleph_client::{
    contract::{
        event::{listen_contract_events, subscribe_events, ContractEvent},
        util::to_u128,
        ContractInstance,
    },
    AccountId, SignedConnection,
};
use anyhow::{anyhow, Result};
use contract_transcode::Value;
use relations::{
    bytes_from_note, FrontendMerklePath, FrontendMerkleRoot, FrontendNote, FrontendNullifier,
    FrontendTokenAmount, FrontendTokenId,
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct Shielder {
    contract: Arc<ContractInstance>,
}

impl Shielder {
    pub fn new(address: &AccountId, metadata_path: &Path) -> Result<Self> {
        Ok(Self {
            contract: Arc::new(ContractInstance::new(
                address.clone(),
                metadata_path.to_str().unwrap(),
            )?),
        })
    }

    /// Call `deposit` message of the contract. If successful, return leaf idx.
    pub fn deposit(
        &self,
        connection: &SignedConnection,
        token_id: FrontendTokenId,
        token_amount: FrontendTokenAmount,
        note: FrontendNote,
        proof: &[u8],
    ) -> Result<u32> {
        let subscription = subscribe_events(connection)?;
        let (cancel_tx, cancel_rx) = channel();
        let (leaf_tx, leaf_rx) = channel();

        let contract_clone = self.contract.clone();
        thread::spawn(move || {
            listen_contract_events(
                subscription,
                &[contract_clone.as_ref()],
                Some(cancel_rx),
                |event_or_error| {
                    debug!("{:?}", event_or_error);
                    if let Ok(ContractEvent { ident, data, .. }) = event_or_error {
                        if Some(String::from("Deposited")) == ident {
                            let event_note: Value = data.get("note").unwrap().clone();
                            let decoded_note: [u64; 4] =
                                to_seq(&event_note).unwrap().try_into().unwrap();
                            // check the `note` in the event as well to identify unambiguously
                            if note.eq(&decoded_note) {
                                let leaf_idx = data.get("leaf_idx").unwrap().clone();
                                leaf_tx.send(to_u128(leaf_idx).unwrap()).unwrap();
                            }
                        }
                    }
                },
            );
        });

        let note_bytes = bytes_from_note(&note);

        let args = [
            &*token_id.to_string(),
            &*token_amount.to_string(),
            &*format!("0x{}", hex::encode(note_bytes)),
            &*format!("0x{}", hex::encode(proof)),
        ];

        debug!("Calling deposit tx with arguments {:?}", &args);

        self.contract
            .contract_exec(connection, "deposit", &args)
            .map_err(|e| {
                cancel_tx.send(()).unwrap();
                e
            })?;

        thread::sleep(Duration::from_secs(3));
        cancel_tx.send(()).unwrap();

        if let Ok(leaf_idx) = leaf_rx.try_recv() {
            info!("Successfully deposited tokens.");
            Ok(leaf_idx as u32)
        } else {
            Err(anyhow!(
                "Failed to observe expected event. And actually I do not know where your tokens are."
            ))
        }
    }

    /// Call `withdraw` message of the contract.
    #[allow(clippy::too_many_arguments)]
    pub fn withdraw(
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
        let subscription = subscribe_events(connection)?;
        let (cancel_tx, cancel_rx) = channel();
        let (leaf_tx, leaf_rx) = channel();

        let contract_ptr = self.contract.clone();

        thread::spawn(move || {
            listen_contract_events(
                subscription,
                &[&contract_ptr],
                Some(cancel_rx),
                |event_or_error| {
                    debug!("{:?}", event_or_error);
                    if let Ok(ContractEvent { ident, data, .. }) = event_or_error {
                        if Some(String::from("Withdrawn")) == ident {
                            let event_note: Value = data.get("new_note").unwrap().clone();
                            let decoded_note: [u64; 4] =
                                to_seq(&event_note).unwrap().try_into().unwrap();
                            // check the `new_note` in the event as well to identify it unambiguously
                            if new_note.eq(&decoded_note) {
                                let leaf_idx = data.get("leaf_idx").unwrap().clone();
                                leaf_tx.send(to_u128(leaf_idx).unwrap()).unwrap();
                            }
                        }
                    }
                },
            );
        });

        let new_note_bytes = bytes_from_note(&new_note);
        // NOTE: a bit of a misnomer but types fit (and root is also a note)
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

        self.contract
            .contract_exec(connection, "withdraw", &args)
            .map_err(|e| {
                cancel_tx.send(()).unwrap();
                e
            })?;

        thread::sleep(Duration::from_secs(3));
        cancel_tx.send(()).unwrap();

        if let Ok(leaf_idx) = leaf_rx.try_recv() {
            info!("Successfully withdrawn tokens.");
            Ok(leaf_idx as u32)
        } else {
            Err(anyhow!(
                "Failed to observe expected event. Funds may not be SAFU."
            ))
        }
    }

    /// Fetch the current merkle root.
    pub fn get_merkle_root(&self, connection: &SignedConnection) -> FrontendMerkleRoot {
        let root = self
            .contract
            .contract_read0(connection, "current_merkle_root")
            .unwrap();
        let decoded_root = to_seq(&root).unwrap();
        decoded_root.try_into().unwrap()
    }

    /// Fetch the current merkle root.
    pub fn get_merkle_path(
        &self,
        connection: &SignedConnection,
        leaf_idx: u32,
    ) -> Option<FrontendMerklePath> {
        let value = self
            .contract
            .contract_read(connection, "merkle_path", &[&*leaf_idx.to_string()])
            .unwrap();

        match value {
            Value::Tuple(value) => match value.ident() {
                Some(ident) => match ident.as_str() {
                    "Some" => match value.values().next().unwrap() {
                        Value::Seq(seq) => {
                            let mut path: Vec<[u64; 4]> = vec![];
                            seq.elems().iter().for_each(|value| {
                                let note = to_seq(value).unwrap();
                                path.push(note.try_into().unwrap());
                            });

                            Some(path)
                        }

                        _ => panic!("Unexpected value: {:?}", value),
                    },
                    "None" => None,
                    _ => panic!("Unexpected string value: {:?}", value),
                },
                None => None,
            },
            _ => panic!("Expected {:?} to be a Tuple", &value),
        }
    }
}

// TODO: could be made generic over elements
// TODO: move to aleph-client
fn to_seq(value: &Value) -> Result<Vec<u64>> {
    match value {
        Value::Seq(seq) => {
            let mut result = vec![];
            for element in seq.elems() {
                match element {
                    Value::UInt(integer) => result.push(*integer as u64),
                    _ => panic!("Expected {:?} to be an UInt", &element),
                }
            }
            Ok(result)
        }
        _ => Err(anyhow!("Expected {:?} to be a sequence", value)),
    }
}
