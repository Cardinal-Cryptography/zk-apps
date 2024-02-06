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
    note::CircuitNote,
    operation::CircuitOperation,
    poseidon_consts::{RATE, R_F, R_P, T},
    proof::CircuitMerkleProof,
};

pub struct UpdateNoteInput<F, A>
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
    pub merkle_proof: CircuitMerkleProof<F>,
    pub op_priv: <A::Op as CircuitOperation<F>>::OpPriv,
    pub id: AssignedValue<F>,
    pub old_account: A,
}

impl<F, A> UpdateNoteInput<F, A>
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    //helper functions
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
        merkle_proof: CircuitMerkleProof<F>,
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

//    1. h_note_new = hash(note_new)
//    2. note_new = Note { id, trapdoor_new, nullifier_new, h_acc_new }
//    3. h_note_old = hash(note_old)
//    4. note_old = Note { id, trapdoor_old, nullifier_old, h_acc_old }
//    5. verify_merkle_proof(merkle_root, h_note_old, proof)
//    6. op = combine(op_pub, op_priv)
//    7. R_update_account(op, h_acc_old, h_acc_new)

#[allow(dead_code)]
pub fn update_note_circuit<F, A>(
    ctx: &mut Context<F>,
    input: UpdateNoteInput<F, A>,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    //let op_pub = ctx.load_witness(input.op_pub().into());
    //let outer_new_note_hash = ctx.load_witness(input.new_note_hash());
    //let merkle_root = ctx.load_witness(input.merkle_root());
    //let old_nullifier = ctx.load_witness(input.old_nullifier());

    let op_pub = input.op_pub;

    make_public.extend(op_pub.clone().into());

    let outer_new_note_hash = input.new_note_hash;
    let merkle_root = input.merkle_root;
    let old_nullifier = input.old_nullifier;

    make_public.extend([outer_new_note_hash, merkle_root, old_nullifier]);

    //let new_trapdoor = ctx.load_witness(input.new_trapdoor());
    //let new_nullifier = ctx.load_witness(input.new_nullifier());

    let new_trapdoor = input.new_trapdoor;
    let new_nullifier = input.new_nullifier;

    let gate = GateChip::<F>::default();

    let mut poseidon =
        PoseidonHasher::<F, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    let inner_new_note_hash = poseidon.hash_note(ctx, &gate, &input.new_note);

    let eq = gate.is_equal(ctx, inner_new_note_hash, outer_new_note_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);

    let inner_old_note_hash = poseidon.hash_note(ctx, &gate, &input.old_note);

    let eq = gate.is_equal(ctx, outer_new_note_hash, inner_old_note_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);

    let merkle_proof = input.merkle_proof;

    merkle_proof.verify(ctx, &gate, &mut poseidon, merkle_root, inner_old_note_hash);

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
