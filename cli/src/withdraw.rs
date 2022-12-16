use std::fs;

use aleph_client::{account_from_keypair, keypair_from_string, SignedConnection};
use anyhow::{anyhow, Result};
use ark_serialize::CanonicalDeserialize;
use inquire::{CustomType, Select};
use rand::Rng;
use relations::{
    compute_note, serialize, FrontendNullifier, FrontendTokenAmount, FrontendTrapdoor, Groth16,
    NonUniversalSystem, ProvingSystem, WithdrawRelation,
};
use tracing::debug;

use crate::{
    app_state::{AppState, Deposit},
    config::WithdrawCmd,
    contract::Shielder,
};

pub(super) fn do_withdraw(
    contract: Shielder,
    mut connection: SignedConnection,
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

    if let Some(seed) = caller_seed {
        connection = SignedConnection::new(&app_state.node_address, keypair_from_string(&seed));
    }
    let recipient = match recipient {
        None => account_from_keypair(&keypair_from_string(&app_state.caller_seed)),
        Some(recipient) => recipient,
    };
    let recipient_bytes: [u8; 32] = recipient.clone().into();
    debug!(?recipient_bytes, "recipient_bytes");

    let merkle_root = contract.get_merkle_root(&connection);
    let merkle_path = contract
        .get_merkle_path(&connection, leaf_idx)
        .expect("Path does not exist");

    debug!(?merkle_path, "retrieved merkle path");

    let mut rng = rand::thread_rng();
    let new_trapdoor: FrontendTrapdoor = rng.gen::<u64>();
    let new_nullifier: FrontendNullifier = rng.gen::<u64>();
    let new_token_amount = whole_token_amount - withdraw_amount;
    let new_note = compute_note(token_id, new_token_amount, new_trapdoor, new_nullifier);

    let circuit = WithdrawRelation::new(
        old_nullifier,
        merkle_root,
        new_note,
        token_id,
        withdraw_amount,
        old_trapdoor,
        new_trapdoor,
        new_nullifier,
        merkle_path,
        leaf_idx.into(),
        old_note,
        whole_token_amount,
        new_token_amount,
        fee.unwrap_or_default(),
        recipient_bytes,
    );

    let pk = match fs::read(proving_key_file) {
        Ok(bytes) => <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*bytes)?,
        Err(_e) => {
            let (pk, vk) = Groth16::generate_keys(circuit.clone());

            fs::write("deposit.pk.bytes", serialize(&pk)).unwrap();
            // NOTE: not needed here but for registering in the snarcos pallet
            fs::write("deposit.vk.bytes", serialize(&vk)).unwrap();

            pk
        }
    };

    let proof = serialize(&Groth16::prove(&pk, circuit));

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
