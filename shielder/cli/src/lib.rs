use std::{fs, path::PathBuf};

use anyhow::Result;
use ark_serialize::CanonicalDeserialize;
use liminal_ark_relations::{
    serialize, CircuitField, ConstraintSynthesizer, Groth16, ProvingSystem,
};

pub type DepositId = u16;

pub const MERKLE_PATH_MAX_LEN: u8 = 16;

pub mod contract;

/// Generates a Groth16 proof for the given `circuit` using proving from the file.
/// Returns an error when deserialization of the proving key fails.
pub fn generate_proof(
    circuit: impl ConstraintSynthesizer<CircuitField>,
    proving_key_file: PathBuf,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;

    Ok(serialize(&Groth16::prove(&pk, circuit)))
}
