use std::path::Path;

use aleph_client::SignedConnection;
use anyhow::Result;
use liminal_ark_relations::shielder::{
    compute_note,
    types::{FrontendNullifier, FrontendTrapdoor},
    MergeRelationWithFullInput,
};
use rand::Rng;

use crate::{
    app_state::{AppState, Deposit},
    contract::Shielder,
    generate_proof, DepositId, MERKLE_PATH_MAX_LEN,
};

/// Performs a merge on two deposits. The leaf index of the first deposit is used to store the
/// merged amount. The second deposit is deleted.
pub async fn merge(
    first_deposit: Deposit,
    second_deposit: Deposit,
    proving_key_file: &Path,
    connection: &SignedConnection,
    contract: &Shielder,
    app_state: &mut AppState,
) -> Result<DepositId> {
    let Deposit {
        deposit_id: first_deposit_id,
        token_id: first_token_id,
        token_amount: first_token_amount,
        trapdoor: first_trapdoor,
        nullifier: first_nullifier,
        leaf_idx: first_leaf_idx,
        note: first_note,
    } = first_deposit;

    let Deposit {
        deposit_id: second_deposit_id,
        token_id: second_token_id,
        token_amount: second_token_amount,
        trapdoor: second_trapdoor,
        nullifier: second_nullifier,
        leaf_idx: second_leaf_idx,
        note: second_note,
    } = second_deposit;

    let matching_tokens = match first_token_id == second_token_id {
        true => Some(first_token_id),
        false => None,
    };

    let token_id = matching_tokens.unwrap_or_else(|| {
        panic!(
            "Cannot merge different tokens. Given: {:?} and {:?}.",
            first_token_id, second_token_id
        )
    });

    let merkle_root = contract.get_merkle_root(connection).await;
    let first_merkle_path = contract
        .get_merkle_path(connection, first_leaf_idx)
        .await
        .unwrap_or_else(|| panic!("Path to given leaf {:?} does not exist!", first_leaf_idx));

    let second_merkle_path = contract
        .get_merkle_path(connection, second_leaf_idx)
        .await
        .unwrap_or_else(|| panic!("Path to given leaf {:?} does not exist!", second_leaf_idx));

    let (new_trapdoor, new_nullifier) =
        rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let new_token_amount = first_token_amount + second_token_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = MergeRelationWithFullInput::new(
        MERKLE_PATH_MAX_LEN,
        token_id,
        first_nullifier,
        second_nullifier,
        new_note,
        merkle_root,
        first_trapdoor,
        second_trapdoor,
        new_trapdoor,
        new_nullifier,
        first_merkle_path,
        second_merkle_path,
        first_leaf_idx.into(),
        second_leaf_idx.into(),
        first_note,
        second_note,
        first_token_amount,
        second_token_amount,
        new_token_amount,
    );

    let proof = generate_proof(circuit, proving_key_file)?;

    let leaf_idx = contract
        .merge(
            connection,
            token_id,
            merkle_root,
            first_nullifier,
            second_nullifier,
            new_note,
            &proof,
        )
        .await?;

    app_state.replace_deposit(
        first_deposit_id,
        new_token_amount,
        new_trapdoor,
        new_nullifier,
        leaf_idx,
        new_note,
    );

    app_state.delete_deposit_by_id(second_deposit_id);

    Ok(first_deposit_id)
}
