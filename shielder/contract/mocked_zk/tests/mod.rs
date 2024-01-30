pub mod merkle;

use self::merkle::MerkleTree;
use super::{
    account::Account,
    note::Note,
    ops::{OpPriv, Operation},
    relations::ZkProof,
    traits::Hashable,
};
use crate::{contract::OpPub, errors::ShielderError, mocked_zk::USDT_TOKEN, types::Scalar};

fn create_empty_note_proof(id: Scalar, nullifier: Scalar, trapdoor: Scalar) -> (Scalar, ZkProof) {
    let acc_new = Account::new(Scalar { bytes: [0x0; 32] });
    let note = Note::new(id, trapdoor, nullifier, acc_new.hash());
    let proof = ZkProof::new(
        id,
        trapdoor,
        nullifier,
        OpPriv {
            user: 1_u128.into(),
        },
        acc_new,
    );
    (note.hash(), proof)
}

fn update_account(
    id: Scalar,
    nullifier: Scalar,
    trapdoor: Scalar,
    op_pub: OpPub,
    proof: ZkProof,
    merkle_proof: [Scalar; 10],
    merkle_proof_leaf_id: u32,
) -> (Scalar, ZkProof) {
    let op_priv = OpPriv {
        user: 1_u128.into(),
    };
    let operation = Operation::combine(op_pub, op_priv).unwrap();
    let acc_updated = proof.update_account(operation).unwrap();
    let note = Note::new(id, trapdoor, nullifier, acc_updated.hash());
    let new_proof = proof.transition(
        trapdoor,
        nullifier,
        acc_updated,
        op_priv,
        merkle_proof,
        merkle_proof_leaf_id,
    );
    (note.hash(), new_proof)
}

#[test]
fn test_create_note() -> Result<(), ShielderError> {
    let id = 0_u128.into();
    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();
    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note)?;
    Ok(())
}

#[test]
fn test_create_note_fails() -> Result<(), ShielderError> {
    let id = 0_u128.into();
    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();
    let (_, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    let (h_new_note, _) = create_empty_note_proof(1_u128.into(), nullifier, trapdoor);
    assert_eq!(
        ShielderError::ZkpVerificationFail,
        proof.verify_creation(h_new_note).unwrap_err()
    );
    Ok(())
}

#[test]
fn test_update_note() -> Result<(), ShielderError> {
    let id = 0_u128.into();

    let mut merkle_tree = MerkleTree::new();

    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();

    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note)?;
    let merkle_root = merkle_tree.add_leaf(h_new_note)?;
    let merkle_proof = merkle_tree.gen_proof(0)?;

    let nullifier_new = 1_u128.into();
    let trapdoor_new = 1_u128.into();

    let op_pub = crate::contract::OpPub::Deposit {
        amount: 10,
        token: Scalar { bytes: USDT_TOKEN },
        user: 1_u128.into(),
    };

    let (h_new_note, proof) = update_account(
        id,
        nullifier_new,
        trapdoor_new,
        op_pub,
        proof,
        merkle_proof,
        0,
    );
    proof.verify_update(op_pub, h_new_note, merkle_root, nullifier)?;

    Ok(())
}

#[test]
fn test_update_note_fail_op_priv() -> Result<(), ShielderError> {
    let id = 0_u128.into();

    let mut merkle_tree = MerkleTree::new();

    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();

    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note)?;
    let merkle_root = merkle_tree.add_leaf(h_new_note)?;
    let merkle_proof = merkle_tree.gen_proof(0)?;

    let nullifier_new = 1_u128.into();
    let trapdoor_new = 1_u128.into();

    let op_pub = crate::contract::OpPub::Deposit {
        amount: 10,
        token: Scalar { bytes: USDT_TOKEN },
        user: 1_u128.into(),
    };
    let op_pub_fake = crate::contract::OpPub::Deposit {
        amount: 10,
        token: Scalar { bytes: USDT_TOKEN },
        user: 2_u128.into(),
    };

    let (h_new_note, proof) = update_account(
        id,
        nullifier_new,
        trapdoor_new,
        op_pub,
        proof,
        merkle_proof,
        0,
    );

    assert_eq!(
        ShielderError::ZkpVerificationFail,
        proof
            .verify_update(op_pub_fake, h_new_note, merkle_root, nullifier)
            .unwrap_err()
    );

    Ok(())
}
