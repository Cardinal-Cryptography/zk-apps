use std::path::PathBuf;

use aleph_client::{keypair_from_string, Connection, SignedConnection};
use anyhow::Result;
use inquire::Password;
use rand::Rng;
use relations::{
    compute_note, DepositAndMergeRelation, DepositRelationWithFullInput, FrontendNullifier,
    FrontendTokenAmount, FrontendTokenId, FrontendTrapdoor,
};

use crate::{
    app_state::{AppState, Deposit},
    config::DepositCmd,
    contract::Shielder,
    generate_proof, MERKLE_PATH_MAX_LEN,
};

pub async fn do_deposit(
    contract: Shielder,
    connection: Connection,
    cmd: DepositCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let DepositCmd {
        token_id,
        amount,
        caller_seed,
        ..
    } = cmd;

    let seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new("Seed of the depositing account (the tokens owner):")
            .without_confirmation()
            .prompt()?,
    };
    let connection = SignedConnection::from_connection(connection, keypair_from_string(&seed));

    let old_deposit = app_state.get_last_deposit(token_id);
    match old_deposit {
        Some(old_deposit) => {
            deposit_and_merge(
                old_deposit,
                amount,
                cmd.deposit_and_merge_key_file,
                connection,
                contract,
                app_state,
            )
            .await
        }
        None => {
            first_deposit(
                token_id,
                amount,
                cmd.deposit_key_file,
                connection,
                contract,
                app_state,
            )
            .await
        }
    }
}

async fn first_deposit(
    token_id: FrontendTokenId,
    token_amount: FrontendTokenAmount,
    proving_key_file: PathBuf,
    connection: SignedConnection,
    contract: Shielder,
    app_state: &mut AppState,
) -> Result<()> {
    let (trapdoor, nullifier) = rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let note = compute_note(token_id, token_amount, trapdoor, nullifier);

    // We generate proof as late as it's possible, so that if any of the lighter procedures fails,
    // we don't waste user's time.
    let circuit =
        DepositRelationWithFullInput::new(note, token_id, token_amount, trapdoor, nullifier);
    let proof = generate_proof(circuit, proving_key_file)?;

    let leaf_idx = contract
        .deposit(&connection, token_id, token_amount, note, &proof)
        .await?;

    app_state.add_deposit(token_id, token_amount, trapdoor, nullifier, leaf_idx, note);

    Ok(())
}

async fn deposit_and_merge(
    deposit: Deposit,
    token_amount: FrontendTokenAmount,
    proving_key_file: PathBuf,
    connection: SignedConnection,
    contract: Shielder,
    app_state: &mut AppState,
) -> Result<()> {
    let Deposit {
        token_id,
        token_amount: old_token_amount,
        trapdoor: old_trapdoor,
        nullifier: old_nullifier,
        leaf_idx,
        note: old_note,
        ..
    } = deposit;
    let merkle_root = contract.get_merkle_root(&connection).await;
    let merkle_path = contract
        .get_merkle_path(&connection, leaf_idx)
        .await
        .expect("Path does not exist");

    let (new_trapdoor, new_nullifier) =
        rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let new_token_amount = old_token_amount + token_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = DepositAndMergeRelation::with_full_input(
        MERKLE_PATH_MAX_LEN,
        token_id,
        token_amount,
        old_nullifier,
        merkle_root,
        new_note,
        old_trapdoor,
        new_trapdoor,
        new_nullifier,
        merkle_path,
        leaf_idx.into(),
        old_note,
        old_token_amount,
        new_token_amount,
    );

    let proof = generate_proof(circuit, proving_key_file)?;

    let leaf_idx = contract
        .deposit_and_merge(
            &connection,
            token_id,
            token_amount,
            merkle_root,
            old_nullifier,
            new_note,
            &proof,
        )
        .await?;

    app_state.replace_deposit(
        deposit.deposit_id,
        token_amount,
        new_trapdoor,
        new_nullifier,
        leaf_idx,
        new_note,
    );

    Ok(())
}
