use aleph_client::{account_from_keypair, keypair_from_string, Connection, SignedConnection};
use anyhow::{anyhow, Result};
use inquire::{CustomType, Password, Select};
use rand::Rng;
use relations::{
    compute_note, FrontendNullifier, FrontendTokenAmount, FrontendTrapdoor, WithdrawRelation,
};

use crate::{
    app_state::{AppState, Deposit},
    config::WithdrawCmd,
    contract::Shielder,
    generate_proof, MERKLE_PATH_MAX_LEN,
};

pub fn do_withdraw(
    contract: Shielder,
    connection: Connection,
    cmd: WithdrawCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let (deposit, withdraw_amount) = get_deposit_and_withdraw_amount(&cmd, app_state)?;

    let WithdrawCmd {
        recipient,
        caller_seed,
        fee,
        proving_key_file,
        ..
    } = cmd;

    let Deposit {
        token_id,
        token_amount: whole_token_amount,
        trapdoor: old_trapdoor,
        nullifier: old_nullifier,
        leaf_idx,
        ..
    } = deposit;

    let old_note = compute_note(token_id, whole_token_amount, old_trapdoor, old_nullifier);

    let caller_seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new(
            "Seed of the withdrawing account (the caller, not necessarily recipient):",
        )
        .without_confirmation()
        .prompt()?,
    };
    let signer = keypair_from_string(&caller_seed);
    let recipient = match recipient {
        Some(recipient) => recipient,
        None => account_from_keypair(&signer),
    };
    let connection = SignedConnection::from_any_connection(&connection, signer);

    let recipient_bytes: [u8; 32] = recipient.clone().into();

    let merkle_root = contract.get_merkle_root(&connection);
    let merkle_path = contract
        .get_merkle_path(&connection, leaf_idx)
        .expect("Path does not exist");

    let (new_trapdoor, new_nullifier) =
        rand::thread_rng().gen::<(FrontendTrapdoor, FrontendNullifier)>();
    let new_token_amount = whole_token_amount - withdraw_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = WithdrawRelation::with_full_input(
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

    let proof = generate_proof(circuit, proving_key_file)?;

    let leaf_idx = contract.withdraw(
        &connection,
        token_id,
        withdraw_amount,
        recipient,
        fee,
        merkle_root,
        old_nullifier,
        new_note,
        &proof,
    )?;

    app_state.delete_deposit_by_id(deposit.deposit_id);

    // save new deposit to the state
    if new_token_amount > 0 {
        app_state.add_deposit(
            token_id,
            new_token_amount,
            new_trapdoor,
            new_nullifier,
            leaf_idx,
        );
    }

    Ok(())
}

fn get_deposit_and_withdraw_amount(
    cmd: &WithdrawCmd,
    app_state: &AppState,
) -> Result<(Deposit, FrontendTokenAmount)> {
    if !cmd.interactive {
        if let Some(deposit) = app_state.get_deposit_by_id(cmd.deposit_id.unwrap()) {
            return Ok((deposit, cmd.amount.unwrap()));
        }
        return Err(anyhow!("Incorrect deposit id"));
    }

    let deposit = Select::new("Select one of your deposits:", app_state.deposits())
        .with_page_size(5)
        .prompt()?;

    let amount =
        CustomType::<FrontendTokenAmount>::new("Specify how many tokens should be withdrawn:")
            .with_default(deposit.token_amount)
            .with_parser(&|a| match str::parse::<FrontendTokenAmount>(a) {
                Ok(amount) if amount <= deposit.token_amount => Ok(amount),
                _ => Err(()),
            })
            .with_error_message(
                "You should provide a valid amount, no more than the whole deposit value",
            )
            .prompt()?;

    Ok((deposit, amount))
}
