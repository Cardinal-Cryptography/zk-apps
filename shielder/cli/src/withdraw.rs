use std::path::Path;

use aleph_client::{sp_runtime::AccountId32, SignedConnection};
use anyhow::Result;
use liminal_ark_relations::shielder::{
    compute_note,
    types::{FrontendNullifier, FrontendTokenAmount, FrontendTrapdoor},
    WithdrawRelationWithFullInput,
};
use rand::Rng;

use crate::{
    app_state::{AppState, Deposit},
    contract::Shielder,
    generate_proof, MERKLE_PATH_MAX_LEN,
};

#[allow(clippy::too_many_arguments)]
pub async fn withdraw(
    contract: &Shielder,
    connection: &SignedConnection,
    deposit: Deposit,
    withdraw_amount: FrontendTokenAmount,
    recipient: &AccountId32,
    fee: u128,
    withdraw_pk_file: &Path,
    app_state: &mut AppState,
) -> Result<()> {
    let Deposit {
        token_id,
        token_amount: whole_token_amount,
        trapdoor: old_trapdoor,
        nullifier: old_nullifier,
        leaf_idx,
        note: old_note,
        ..
    } = deposit;

    let recipient_bytes: [u8; 32] = recipient.clone().into();

    let merkle_root = contract.get_merkle_root(connection).await;
    let merkle_path = contract
        .get_merkle_path(connection, leaf_idx)
        .await
        .expect("Path does not exist");

    let (new_trapdoor, new_nullifier) =
        rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let new_token_amount = whole_token_amount - withdraw_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = WithdrawRelationWithFullInput::new(
        MERKLE_PATH_MAX_LEN,
        fee,
        recipient_bytes,
        token_id,
        old_nullifier,
        new_note,
        withdraw_amount,
        merkle_root,
        old_trapdoor,
        new_trapdoor,
        new_nullifier,
        merkle_path,
        leaf_idx.into(),
        old_note,
        whole_token_amount,
        new_token_amount,
    );

    let proof = generate_proof(circuit, withdraw_pk_file)?;

    let leaf_idx = contract
        .withdraw(
            connection,
            token_id,
            withdraw_amount,
            recipient,
            fee,
            merkle_root,
            old_nullifier,
            new_note,
            &proof,
        )
        .await?;

    // save new deposit to the state
    if new_token_amount > 0 {
        app_state.replace_deposit(
            deposit.deposit_id,
            new_token_amount,
            new_trapdoor,
            new_nullifier,
            leaf_idx,
            new_note,
        );
    } else {
        app_state.delete_deposit_by_id(deposit.deposit_id);
    }

    Ok(())
}
