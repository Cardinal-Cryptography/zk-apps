use std::{fs, path::Path};

use anyhow::Result;
use ark_serialize::CanonicalDeserialize;
use liminal_ark_relations::{
    environment::{CircuitField, Groth16, ProvingSystem},
    serialization::serialize,
    ConstraintSynthesizer,
};

pub type DepositId = u16;

pub type LeafIdx = u32;

pub const MERKLE_PATH_MAX_LEN: u8 = 16;

pub mod app_state;
pub mod contract;
pub mod deposit;
pub mod ink_contract;
pub mod merge;
pub mod withdraw;

/// Generates a Groth16 proof for the given `circuit` using proving key from the file.
/// Returns an error when either reading file or deserialization of the proving key fails.
pub fn generate_proof(
    circuit: impl ConstraintSynthesizer<CircuitField>,
    proving_key_file: &Path,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;

    Ok(serialize(&Groth16::prove(&pk, circuit)))
}
