use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::{spec::OptimizedPoseidonSpec, PoseidonHasher},
    utils::BigPrimeField,
    AssignedValue, Context,
};

use super::update_account::{update_account_circuit, UpdateAccountInput};
use crate::{
    account::CircuitAccount,
    hasher::InnerHasher,
    merkle_proof::CircuitMerkleProof,
    note::CircuitNote,
    operation::CircuitOperation,
    poseidon_consts::{RATE, R_F, R_P, T},
};

pub struct UpdateNoteInput<F, A, const MAX_PATH_LEN: usize>
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    //public inputs
    pub op_pub: <A::Op as CircuitOperation<F>>::OpPub,
    pub new_note_hash: AssignedValue<F>,
    pub merkle_root: AssignedValue<F>,
    pub old_nullifier: AssignedValue<F>,

    //witnesses
    pub new_note: CircuitNote<F>,
    pub old_note: CircuitNote<F>,
    pub new_trapdoor: AssignedValue<F>,
    pub old_trapdoor: AssignedValue<F>,
    pub new_nullifier: AssignedValue<F>,
    pub merkle_proof: CircuitMerkleProof<F, MAX_PATH_LEN>,
    pub op_priv: <A::Op as CircuitOperation<F>>::OpPriv,
    pub id: AssignedValue<F>,

    pub old_account: A,
}

//helper functions
#[allow(clippy::too_many_arguments)]
impl<F, A, const MAX_PATH_LEN: usize> UpdateNoteInput<F, A, MAX_PATH_LEN>
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    pub fn new(
        op_pub: <A::Op as CircuitOperation<F>>::OpPub,
        new_note_hash: AssignedValue<F>,
        merkle_root: AssignedValue<F>,
        old_nullifier: AssignedValue<F>,
        new_note: CircuitNote<F>,
        old_note: CircuitNote<F>,
        new_trapdoor: AssignedValue<F>,
        old_trapdoor: AssignedValue<F>,
        new_nullifier: AssignedValue<F>,
        merkle_proof: CircuitMerkleProof<F, MAX_PATH_LEN>,
        op_priv: <A::Op as CircuitOperation<F>>::OpPriv,
        id: AssignedValue<F>,
        old_account: A,
    ) -> Self {
        Self {
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
        }
    }
}

fn verify_note_circuit<F>(
    ctx: &mut Context<F>,
    gate: &GateChip<F>,
    poseidon: &mut PoseidonHasher<F, T, RATE>,
    note: &CircuitNote<F>,
    note_hash: AssignedValue<F>,
) where
    F: BigPrimeField,
{
    let inner_note_hash = poseidon.hash_note(ctx, gate, note);
    let eq = gate.is_equal(ctx, note_hash, inner_note_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);
}

#[allow(dead_code)]
pub fn update_note_circuit<F, A, const MAX_PATH_LEN: usize>(
    ctx: &mut Context<F>,
    input: UpdateNoteInput<F, A, MAX_PATH_LEN>,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    let op_pub = input.op_pub;

    make_public.extend(op_pub.clone().into());

    let outer_new_note_hash = input.new_note_hash;
    let merkle_root = input.merkle_root;
    let old_nullifier = input.old_nullifier;

    make_public.extend([outer_new_note_hash, merkle_root, old_nullifier]);

    let _new_trapdoor = input.new_trapdoor;
    let _new_nullifier = input.new_nullifier;

    let gate = GateChip::<F>::default();

    let mut poseidon =
        PoseidonHasher::<F, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    verify_note_circuit(
        ctx,
        &gate,
        &mut poseidon,
        &input.new_note,
        outer_new_note_hash,
    );

    let old_note_hash = poseidon.hash_note(ctx, &gate, &input.old_note);

    let old_account_hash = poseidon.hash_account(ctx, &gate, &input.old_account);
    let outer_old_note = CircuitNote {
        id: input.id,
        trapdoor: input.old_trapdoor,
        nullifier: input.old_nullifier,
        account_hash: old_account_hash,
    };
    verify_note_circuit(ctx, &gate, &mut poseidon, &outer_old_note, old_note_hash);

    let merkle_proof = input.merkle_proof;

    merkle_proof.verify(ctx, &gate, &mut poseidon, merkle_root, old_note_hash);

    let op_priv = input.op_priv;

    let operation = CircuitOperation::combine(op_priv, op_pub).unwrap();

    let update_account_input = UpdateAccountInput::new(
        input.old_note.account_hash,
        input.new_note.account_hash,
        operation,
        input.old_account,
    );

    update_account_circuit(ctx, update_account_input, make_public);
}
