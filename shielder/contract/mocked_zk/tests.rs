use super::{
    account::Account,
    note::Note,
    ops::{OpPriv, Operation},
    relations::ZkProof,
    traits::Hashable,
};
use crate::{
    contract::OpPub,
    errors::ShielderError,
    merkle::MerkleTree,
    mocked_zk::{mocked_user, MOCKED_TOKEN, TOKENS_NUMBER},
    types::Scalar,
};
use ink::primitives::AccountId;

fn create_empty_note_proof(id: Scalar, nullifier: Scalar, trapdoor: Scalar) -> (Scalar, ZkProof) {
    let mut tokens: [Scalar; TOKENS_NUMBER] = [0_u128.into(); TOKENS_NUMBER];
    tokens[0] = MOCKED_TOKEN;

    let acc_new = Account::new(tokens);
    let note = Note::new(id, trapdoor, nullifier, acc_new.hash());
    let proof = ZkProof::new(
        id,
        trapdoor,
        nullifier,
        OpPriv {
            user: mocked_user(),
        },
        acc_new,
    );
    (note.hash(), proof)
}

fn update_account(
    nullifier: Scalar,
    trapdoor: Scalar,
    op_pub: OpPub,
    proof: ZkProof,
    merkle_proof: [Scalar; 10],
    merkle_proof_leaf_id: u32,
) -> (Scalar, ZkProof) {
    let op_priv = OpPriv {
        user: mocked_user(),
    };
    let operation = Operation::combine(op_pub, op_priv).unwrap();
    proof
        .update_account(
            operation,
            trapdoor,
            nullifier,
            merkle_proof,
            merkle_proof_leaf_id,
        )
        .unwrap()
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
    // need this because MerkleTree is called
    ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));

    let id = 0_u128.into();

    let mut merkle_tree = MerkleTree::default();

    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();

    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note)?;
    merkle_tree.add_leaf(h_new_note)?;
    let merkle_root = merkle_tree.root();
    let merkle_proof = merkle_tree.gen_proof(0)?;

    let nullifier_new = 1_u128.into();
    let trapdoor_new = 1_u128.into();

    let op_pub = crate::contract::OpPub::Deposit {
        amount: 10,
        token: MOCKED_TOKEN,
        user: mocked_user(),
    };

    let (h_new_note, proof) =
        update_account(nullifier_new, trapdoor_new, op_pub, proof, merkle_proof, 0);
    proof.verify_update(op_pub, h_new_note, merkle_root, nullifier)?;

    Ok(())
}

#[test]
fn test_update_note_fail_op_priv() -> Result<(), ShielderError> {
    // need this because merkle tree is called
    ink::env::test::set_callee::<ink::env::DefaultEnvironment>(AccountId::from([0x1; 32]));

    let id = 0_u128.into();

    let mut merkle_tree = MerkleTree::default();

    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();

    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note)?;
    merkle_tree.add_leaf(h_new_note)?;
    let merkle_root = merkle_tree.root();
    let merkle_proof = merkle_tree.gen_proof(0)?;

    let nullifier_new = 1_u128.into();
    let trapdoor_new = 1_u128.into();

    let op_pub = crate::contract::OpPub::Deposit {
        amount: 10,
        token: MOCKED_TOKEN,
        user: mocked_user(),
    };
    let op_pub_fake = crate::contract::OpPub::Deposit {
        amount: 10,
        token: MOCKED_TOKEN,
        user: 2_u128.into(),
    };

    let (h_new_note, proof) =
        update_account(nullifier_new, trapdoor_new, op_pub, proof, merkle_proof, 0);

    assert_eq!(
        ShielderError::ZkpVerificationFail,
        proof
            .verify_update(op_pub_fake, h_new_note, merkle_root, nullifier)
            .unwrap_err()
    );

    Ok(())
}
