use anyhow::Result;
use drink::{
    runtime::MinimalRuntime,
    session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    AccountId32,
};

use crate::{
    contract::MERKLE_TREE_DEPTH,
    drink_tests::{BundleProvider, UpdateOperation},
    errors::ShielderError,
    mocked_zk::{
        account::Account,
        note::Note,
        ops::{OpPriv, Operation},
        relations::ZkProof,
        traits::Hashable,
        TOKENS_NUMBER,
    },
    types::Scalar,
};

#[derive(Clone, Copy, Debug)]
pub struct ShielderUserEnv {
    pub proof: ZkProof,
    pub nullifier: Scalar,
    pub tree_leaf_id: u32,
}

pub fn deploy_shielder(session: &mut Session<MinimalRuntime>) -> Result<AccountId32> {
    let res = session.deploy_bundle(
        BundleProvider::local()?,
        "new",
        NO_ARGS,
        NO_SALT,
        NO_ENDOWMENT,
    )?;
    Ok(res)
}

pub fn create_shielder_account(
    session: &mut Session<MinimalRuntime>,
    shielder_address: &AccountId32,
    token: &AccountId32,
    nullifier: Scalar,
) -> Result<ShielderUserEnv> {
    let mut tokens: [Scalar; TOKENS_NUMBER] = [0_u128.into(); TOKENS_NUMBER];
    tokens[0] = Scalar::from_bytes(*((*token).as_ref()));

    let acc = Account::new(tokens);

    let id = 0_128.into();
    let trapdoor = 0_u128.into();
    let op_priv = OpPriv {
        user: 0_u128.into(),
    };

    let proof = ZkProof::new(id, trapdoor, nullifier, op_priv, acc);

    let h_note_new = Note::new(id, trapdoor, nullifier, acc.hash()).hash();

    let leaf_id_res: Result<u32, ShielderError> = session.call_with_address(
        shielder_address.clone(),
        "add_note",
        &[format!("{:?}", h_note_new), format!("{:?}", proof)],
        NO_ENDOWMENT,
    )??;

    Ok(ShielderUserEnv {
        proof,
        nullifier,
        tree_leaf_id: leaf_id_res.unwrap(),
    })
}

pub fn shielder_update(
    session: &mut Session<MinimalRuntime>,
    shielder_address: &AccountId32,
    upd_op: UpdateOperation,
    user_shielded_data: ShielderUserEnv,
    nullifier: Scalar,
) -> Result<ShielderUserEnv> {
    let merkle_root_res: Result<Scalar, ShielderError> = session.call_with_address(
        shielder_address.clone(),
        "notes_merkle_root",
        NO_ARGS,
        NO_ENDOWMENT,
    )??;
    let merkle_root = merkle_root_res.unwrap();
    let merkle_proof_res: Result<[Scalar; MERKLE_TREE_DEPTH], ShielderError> = session
        .call_with_address(
            shielder_address.clone(),
            "notes_merkle_path",
            &[format!("{:?}", user_shielded_data.tree_leaf_id)],
            NO_ENDOWMENT,
        )??;
    let merkle_proof = merkle_proof_res.unwrap();
    let trapdoor_new = 1_u128.into();

    let op_pub = upd_op.op_pub;
    let op_priv = upd_op.op_priv;
    let operation = Operation::combine(op_pub, op_priv).unwrap();
    let (note_hash, new_proof) = user_shielded_data
        .proof
        .update_account(
            operation,
            trapdoor_new,
            nullifier,
            merkle_proof,
            user_shielded_data.tree_leaf_id,
        )
        .unwrap();

    let new_leaf_id_res: Result<u32, ShielderError> = session.call_with_address(
        shielder_address.clone(),
        "update_note",
        &[
            format!("{:?}", op_pub),
            format!("{:?}", note_hash),
            format!("{:?}", merkle_root),
            format!("{:?}", user_shielded_data.nullifier),
            format!("{:?}", new_proof),
        ],
        NO_ENDOWMENT,
    )??;

    Ok(ShielderUserEnv {
        proof: new_proof,
        nullifier,
        tree_leaf_id: new_leaf_id_res.unwrap(),
    })
}
