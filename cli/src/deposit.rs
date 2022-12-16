use std::fs;

use aleph_client::SignedConnection;
use anyhow::Result;
use ark_serialize::CanonicalDeserialize;
use rand::Rng;
use relations::{
    compute_note, serialize, DepositRelation, FrontendNullifier, FrontendTrapdoor, Groth16,
    NonUniversalSystem, ProvingSystem,
};

use crate::{app_state::AppState, config::DepositCmd, contract::Shielder};

pub(super) fn do_deposit(
    contract: Shielder,
    connection: SignedConnection,
    cmd: DepositCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let DepositCmd {
        token_id,
        amount: token_amount,
        proving_key_file,
        ..
    } = cmd;

    let mut rng = rand::thread_rng();

    let trapdoor: FrontendTrapdoor = rng.gen::<u64>();
    let nullifier: FrontendNullifier = rng.gen::<u64>();
    let note = compute_note(token_id, token_amount, trapdoor, nullifier);

    let circuit = DepositRelation::new(note, token_id, token_amount, trapdoor, nullifier);

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
    let leaf_idx = contract.deposit(&connection, cmd.token_id, cmd.amount, note, &proof)?;

    app_state.add_deposit(cmd.token_id, cmd.amount, trapdoor, nullifier, leaf_idx);

    Ok(())
}
