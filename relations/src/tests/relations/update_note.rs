use halo2_base::{
    halo2_proofs::{
        arithmetic::Field,
        halo2curves::{bn256::Fr, ff::PrimeField},
    },
    utils::testing::base_test,
};
use poseidon::Poseidon;

use crate::{
    account::Account,
    hasher::OuterHasher,
    note::Note,
    operation::{self, Operation},
    poseidon_consts::{RATE, R_F, R_P, T},
    proof::MerkleProof,
    relations::{
        update_account::{update_account_circuit, UpdateAccountInput},
        update_note::{update_note_circuit, UpdateNoteInput},
    },
    tests::{account::DummyAccount, operation::DummyOperation},
    Token,
};

#[test]
fn test() {
    let old_account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());
    let old_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&old_account);

    let op_pub = DummyOperation::Deposit(Fr::from_u128(100u128), Token::AZERO, [0u8; 32]);
    let op_priv = ();
    let operation = DummyOperation::combine(op_priv, op_pub).unwrap();

    let new_account = old_account.update(operation);
    let new_account_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_account(&new_account);

    let id = Fr::ZERO;

    let old_trapdoor = Fr::ZERO;
    let old_nullifier = Fr::ZERO;
    let old_note = Note::new(id, old_trapdoor, old_nullifier, old_account_hash);

    let new_trapdoor = Fr::ONE;
    let new_nullifier = Fr::ONE;
    let new_note = Note::new(id, new_trapdoor, new_nullifier, new_account_hash);
    let new_note_hash = Poseidon::<Fr, T, RATE>::new(R_F, R_P).hash_note(&new_note);

    let merkle_root = Fr::ZERO;

    let merkle_proof = MerkleProof::new(vec![], vec![], 0);

    let mut make_public = Vec::new();

    base_test().k(9).expect_satisfied(true).run(|ctx, _| {
        let op_pub = operation.load(ctx);
        let new_note_hash = ctx.load_witness(new_note_hash);
        let merkle_root = ctx.load_witness(merkle_root);
        let old_nullifier = ctx.load_witness(old_nullifier);
        let new_note = new_note.load(ctx);
        let old_note = old_note.load(ctx);
        let new_trapdoor = ctx.load_witness(new_trapdoor);
        let old_trapdoor = ctx.load_witness(old_trapdoor);
        let new_nullifier = ctx.load_witness(new_nullifier);
        let merkle_proof = merkle_proof.load(ctx);
        let op_priv = ctx.load_zero();

        let op_priv = ();
        let id = ctx.load_witness(id);
        let old_account = old_account.load(ctx);

        let input = UpdateNoteInput::new(
            op_pub,
            new_note_hash,
            merkle_root,
            old_nullifier,
            new_note,
            old_note,
            new_trapdoor,
            old_trapdoor,
            new_nullifier,
            merkle_proof,
            op_priv,
            id,
            old_account,
        );
        update_note_circuit(ctx, input, &mut make_public);
    });
}
