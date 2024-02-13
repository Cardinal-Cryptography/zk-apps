use halo2_base::{
    halo2_proofs::{
        arithmetic::Field,
        halo2curves::{bn256::Fr, ff::PrimeField},
    },
    utils::testing::base_test,
};
use proptest::result;

use crate::{
    account::Account,
    hasher::tests::OuterHasher,
    note::Note,
    operation::Operation,
    proof::MerkleProof,
    relations::update_note::{update_note_circuit, UpdateNoteInput},
    tests::{account::DummyAccount, operation::DummyOperation, PoseidonHasher},
    Token,
};

fn prepare_accounts() -> (
    DummyAccount<Fr>,
    DummyOperation<Fr, [u8; 32]>,
    DummyAccount<Fr>,
) {
    let old_account = DummyAccount::<Fr>::new(Fr::zero(), Fr::zero());

    let op_pub = DummyOperation::Deposit(Fr::from_u128(100u128), Token::AZERO, [0u8; 32]);
    let operation = DummyOperation::combine((), op_pub).unwrap();

    let new_account = old_account.update(operation);

    (old_account, operation, new_account)
}

fn prepare_notes(
    old_account_hash: Fr,
    new_account_hash: Fr,
) -> (Fr, Fr, Fr, Note<Fr>, Fr, Fr, Note<Fr>) {
    let id = Fr::ZERO;

    let old_trapdoor = Fr::ZERO;
    let old_nullifier = Fr::ZERO;
    let old_note = Note::new(id, old_trapdoor, old_nullifier, old_account_hash);

    let new_trapdoor = Fr::ONE;
    let new_nullifier = Fr::ONE;
    let new_note = Note::new(id, new_trapdoor, new_nullifier, new_account_hash);

    (
        id,
        old_trapdoor,
        old_nullifier,
        old_note,
        new_trapdoor,
        new_nullifier,
        new_note,
    )
}

fn prepare_merkle_proof(leaf: Fr) -> (Fr, MerkleProof<Fr, 30>) {
    let zero_note = Fr::zero(); // x

    let sibling_note = Fr::from_u128(4); // 4
    let parent_note = PoseidonHasher::hash_two_to_one(&[sibling_note, leaf]); // 2

    let uncle_note = Fr::from_u128(3); // 3

    let grandpa_root = PoseidonHasher::hash_two_to_one(&[parent_note, uncle_note]); // 1

    let mut merkle_root = PoseidonHasher::hash_two_to_one(&[grandpa_root, zero_note]);

    for _ in 0..27 {
        merkle_root = PoseidonHasher::hash_two_to_one(&[merkle_root, zero_note]);
    }

    let path_shape = vec![false, true];
    let merkle_path = vec![sibling_note, uncle_note];

    let merkle_proof = MerkleProof::new(path_shape.clone(), merkle_path.clone());

    (merkle_root, merkle_proof)
}

#[test]
fn test_correct_note_update_passes() {
    let usable_rows = 11;
    let result = true;

    let (old_account, operation, new_account) = prepare_accounts();

    let old_account_hash = PoseidonHasher::hash_account(&old_account);
    let new_account_hash = PoseidonHasher::hash_account(&new_account);

    let (id, old_trapdoor, old_nullifier, old_note, new_trapdoor, new_nullifier, new_note) =
        prepare_notes(old_account_hash, new_account_hash);

    let old_note_hash = PoseidonHasher::hash_note(&old_note);
    let new_note_hash = PoseidonHasher::hash_note(&new_note);

    let (merkle_root, merkle_proof) = prepare_merkle_proof(old_note_hash);

    let mut make_public = Vec::new();

    base_test()
        .unusable_rows(usable_rows)
        .k(9)
        .expect_satisfied(result)
        .run(|ctx, _| {
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
            let _op_priv = ctx.load_zero();

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
                (),
                id,
                old_account,
            );
            update_note_circuit(ctx, input, &mut make_public);
        });
}
