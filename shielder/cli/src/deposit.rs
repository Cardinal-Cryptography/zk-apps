use std::path::Path;

use aleph_client::SignedConnection;
use anyhow::Result;
use liminal_ark_relations::shielder::{
    compute_note,
    types::{FrontendNullifier, FrontendTokenAmount, FrontendTokenId, FrontendTrapdoor},
    DepositAndMergeRelationWithFullInput, DepositRelationWithFullInput,
};
use rand::Rng;

use crate::{
    app_state::{AppState, Deposit},
    contract::Shielder,
    generate_proof, DepositId, MERKLE_PATH_MAX_LEN,
};

pub async fn new_deposit(
    token_id: FrontendTokenId,
    token_amount: FrontendTokenAmount,
    proving_key_file: &Path,
    connection: &SignedConnection,
    contract: &Shielder,
    app_state: &mut AppState,
) -> Result<DepositId> {
    let (trapdoor, nullifier) = rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let note = compute_note(token_id, token_amount, trapdoor, nullifier);

    // We generate proof as late as it's possible, so that if any of the lighter procedures fails,
    // we don't waste user's time.
    let circuit =
        DepositRelationWithFullInput::new(note, token_id, token_amount, trapdoor, nullifier);
    let proof = generate_proof(circuit, proving_key_file)?;

    let leaf_idx = contract
        .deposit(connection, token_id, token_amount, note, &proof)
        .await?;

    let deposit_id =
        app_state.add_deposit(token_id, token_amount, trapdoor, nullifier, leaf_idx, note);

    Ok(deposit_id)
}

pub async fn deposit_and_merge(
    deposit: Deposit,
    token_amount: FrontendTokenAmount,
    proving_key_file: &Path,
    connection: &SignedConnection,
    contract: &Shielder,
    app_state: &mut AppState,
) -> Result<DepositId> {
    let Deposit {
        token_id,
        token_amount: old_token_amount,
        trapdoor: old_trapdoor,
        nullifier: old_nullifier,
        leaf_idx,
        note: old_note,
        ..
    } = deposit;
    let merkle_root = contract.get_merkle_root(connection).await;
    let merkle_path = contract
        .get_merkle_path(connection, leaf_idx)
        .await
        .expect("Path does not exist");

    let (new_trapdoor, new_nullifier) =
        rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let new_token_amount = old_token_amount + token_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = DepositAndMergeRelationWithFullInput::new(
        MERKLE_PATH_MAX_LEN,
        token_id,
        old_nullifier,
        new_note,
        token_amount,
        merkle_root,
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
            connection,
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
        new_token_amount,
        new_trapdoor,
        new_nullifier,
        leaf_idx,
        new_note,
    );

    Ok(deposit.deposit_id)
}
