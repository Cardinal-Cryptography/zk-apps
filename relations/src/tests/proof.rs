use halo2_base::{
    gates::GateChip,
    halo2_proofs::halo2curves::{bn256::Fr, ff::PrimeField},
    poseidon::hasher::{spec::OptimizedPoseidonSpec, PoseidonHasher},
    utils::testing::base_test,
    AssignedValue, Context,
};
use poseidon::Poseidon;

use crate::{
    poseidon_consts::{RATE, R_F, R_P, T},
    proof::{CircuitMerkleProof, MerkleProof},
};

fn test_merkle_proof_circuit<const MAX_PATH_LEN: usize>(
    ctx: &mut Context<Fr>,
    merkle_proof: CircuitMerkleProof<Fr, MAX_PATH_LEN>,
    root: AssignedValue<Fr>,
    leaf: AssignedValue<Fr>,
    make_public: &mut Vec<AssignedValue<Fr>>,
) {
    make_public.extend([root]);

    let gate = GateChip::<Fr>::default();

    let mut poseidon =
        PoseidonHasher::<Fr, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    merkle_proof.verify(ctx, &gate, &mut poseidon, root, leaf);
}

#[test]
fn test_verify_correct() {
    //                                          merkle root
    //                placeholder                                        x
    //        1                          x                     x                         x
    //   2        3                x          x            x       x                 x       x
    // 4  *5*   6   7            x   x      x   x        x   x   x   x             x   x   x   x

    let mut poseidon: Poseidon<Fr, T, RATE> = Poseidon::new(R_F, R_P);

    let zero_note = Fr::zero(); // x
    let leaf = Fr::from_u128(5); // 5

    let sibling_note = Fr::from_u128(4); // 4
    poseidon.update(&[sibling_note, leaf]);
    let parent_note = poseidon.squeeze(); // 2

    let mut poseidon: Poseidon<Fr, T, RATE> = Poseidon::new(R_F, R_P);

    let uncle_note = Fr::from_u128(3); // 3
    poseidon.update(&[parent_note, uncle_note]);

    let grandpa_root = poseidon.squeeze(); // 1
    let mut poseidon: Poseidon<Fr, T, RATE> = Poseidon::new(R_F, R_P);
    poseidon.update(&[grandpa_root, zero_note]);

    let placeholder = poseidon.squeeze();

    let mut poseidon: Poseidon<Fr, T, RATE> = Poseidon::new(R_F, R_P);
    poseidon.update(&[placeholder, zero_note]);
    let merkle_root = poseidon.squeeze();

    let path_shape = vec![false, true];
    let merkle_path = vec![sibling_note, uncle_note];

    let proof: MerkleProof<Fr, 4> = MerkleProof::new(path_shape.clone(), merkle_path.clone());

    let proof_result = proof.verify(merkle_root, leaf);
    assert!(proof_result);

    let mut make_public = Vec::new();

    base_test()
        .k(9)
        .expect_satisfied(proof_result)
        .run(|ctx, _| {
            let circuit_proof = proof.load(ctx);
            let merkle_root = ctx.load_witness(merkle_root);
            let leaf = ctx.load_witness(leaf);

            test_merkle_proof_circuit(ctx, circuit_proof, merkle_root, leaf, &mut make_public);
        });
}

#[test]
fn test_verify_incorrect() {
    let path_shape = vec![true, false, true];
    let path = vec![
        Fr::from_u128(1u128),
        Fr::from_u128(2u128),
        Fr::from_u128(3u128),
    ];

    let proof: MerkleProof<Fr, 4> = MerkleProof::new(path_shape.clone(), path.clone());
    let root = Fr::from_u128(4u128);
    let leaf = Fr::from_u128(5u128);

    let mut make_public = Vec::new();

    let proof_result = proof.verify(root, leaf);
    assert!(!proof_result);

    base_test()
        .k(9)
        .expect_satisfied(proof_result)
        .run(|ctx, _| {
            let circuit_proof = proof.load(ctx);
            let root = ctx.load_witness(root);
            let leaf = ctx.load_witness(leaf);

            test_merkle_proof_circuit(ctx, circuit_proof, root, leaf, &mut make_public);
        });
}
