use drink::{
    runtime::MinimalRuntime,
    session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    AccountId32,
};

use crate::{
    contract::OpPub,
    drink_tests::BundleProvider,
    mocked_zk::{
        account::Account,
        note::Note,
        ops::{OpPriv, Operation},
        relations::ZkProof,
        tests::merkle::MerkleTree,
        traits::Hashable,
    },
    types::Scalar,
};

pub struct ShielderUserEnv {
    pub id: Scalar,
    pub proof: ZkProof,
    pub nullifier: Scalar,
    pub tree_leaf_id: u32,
}

pub struct UpdateOperation {
    pub op_pub: OpPub,
    pub op_priv: OpPriv,
}

pub fn deploy_shielder(
    session: &mut Session<MinimalRuntime>,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
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
    shielder_address: AccountId32,
    token: AccountId32,
    merkle_tree: &mut MerkleTree,
) -> Result<ShielderUserEnv, Box<dyn std::error::Error>> {
    let acc = Account::new(Scalar {
        bytes: *(token.as_ref()),
    });
    let id = 0_128.into();
    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();

    let proof = ZkProof::new(
        id,
        trapdoor,
        nullifier,
        OpPriv {
            user: 0_u128.into(),
        },
        acc,
    );

    let h_note_new = Note::new(id, trapdoor, nullifier, acc.hash()).hash();

    session.call_with_address(
        shielder_address.clone(),
        "add_note",
        &[format!("{:?}", h_note_new), format!("{:?}", proof)],
        NO_ENDOWMENT,
    )??;

    merkle_tree.add_leaf(h_note_new).unwrap();

    Ok(ShielderUserEnv {
        id,
        proof,
        nullifier,
        tree_leaf_id: 0,
    })
}

pub fn shielder_update(
    session: &mut Session<MinimalRuntime>,
    shielder_address: AccountId32,
    upd_op: UpdateOperation,
    user_shielded_data: ShielderUserEnv,
    merkle_tree: &mut MerkleTree,
) -> Result<ShielderUserEnv, Box<dyn std::error::Error>> {
    let merkle_root = merkle_tree.root();
    let merkle_proof = merkle_tree
        .gen_proof(user_shielded_data.tree_leaf_id as usize)
        .unwrap();
    let nullifier_new = (u128::from(user_shielded_data.nullifier) + 1).into();
    let trapdoor_new = 1_u128.into();

    let op_pub = upd_op.op_pub;
    let op_priv = upd_op.op_priv;
    let operation = Operation::combine(op_pub, op_priv).unwrap();
    let acc_updated = user_shielded_data.proof.update_account(operation).unwrap();
    let note = Note::new(
        user_shielded_data.id,
        trapdoor_new,
        nullifier_new,
        acc_updated.hash(),
    );
    let new_proof = user_shielded_data.proof.transition(
        trapdoor_new,
        nullifier_new,
        acc_updated,
        op_priv,
        merkle_proof,
        user_shielded_data.tree_leaf_id,
    );
    merkle_tree.add_leaf(note.hash()).unwrap();

    session.call_with_address(
        shielder_address.clone(),
        "update_note",
        &[
            format!("{:?}", op_pub),
            format!("{:?}", note.hash()),
            format!("{:?}", merkle_root),
            format!("{:?}", user_shielded_data.nullifier),
            format!("{:?}", new_proof),
        ],
        NO_ENDOWMENT,
    )??;

    Ok(ShielderUserEnv {
        id: user_shielded_data.id,
        proof: new_proof,
        nullifier: nullifier_new,
        tree_leaf_id: user_shielded_data.tree_leaf_id + 1,
    })
}
