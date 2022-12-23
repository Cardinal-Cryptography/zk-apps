use std::{fs, path::PathBuf};

use aleph_client::{keypair_from_string, Connection, SignedConnection};
use anyhow::Result;
use ark_serialize::CanonicalDeserialize;
use inquire::Password;
use rand::Rng;
use relations::{
    compute_note, serialize, DepositRelation, FrontendNote, FrontendNullifier, FrontendTokenAmount,
    FrontendTokenId, FrontendTrapdoor, Groth16, ProvingSystem,
};

use crate::{app_state::AppState, config::DepositCmd, contract::Shielder};

pub fn do_deposit(
    contract: Shielder,
    connection: Connection,
    cmd: DepositCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let DepositCmd {
        token_id,
        amount: token_amount,
        proving_key_file,
        caller_seed,
        ..
    } = cmd;

    let mut rng = rand::thread_rng();

    let trapdoor: FrontendTrapdoor = rng.gen::<u64>();
    let nullifier: FrontendNullifier = rng.gen::<u64>();
    let note = compute_note(token_id, token_amount, trapdoor, nullifier);

    let seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new("Seed of the depositing account (the tokens owner):")
            .without_confirmation()
            .prompt()?,
    };
    let connection = SignedConnection::from_any_connection(&connection, keypair_from_string(&seed));

    // We generate proof as late as it's possible, so that if any of the lighter procedures fails,
    // we don't waste user's time.
    let proof = generate_proof(
        &proving_key_file,
        note,
        token_id,
        token_amount,
        trapdoor,
        nullifier,
    )?;

    let leaf_idx = contract.deposit(&connection, cmd.token_id, cmd.amount, note, &proof)?;

    app_state.add_deposit(token_id, token_amount, trapdoor, nullifier, leaf_idx);

    Ok(())
}

fn generate_proof(
    proving_key_file: &PathBuf,
    note: FrontendNote,
    token_id: FrontendTokenId,
    token_amount: FrontendTokenAmount,
    trapdoor: FrontendTrapdoor,
    nullifier: FrontendNullifier,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;

    let circuit =
        DepositRelation::with_full_input(note, token_id, token_amount, trapdoor, nullifier);

    Ok(serialize(&Groth16::prove(&pk, circuit)))
}
