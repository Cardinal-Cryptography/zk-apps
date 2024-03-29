use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::{spec::OptimizedPoseidonSpec, PoseidonHasher},
    utils::BigPrimeField,
    AssignedValue, Context,
};

use super::update_account::{update_account_circuit, UpdateAccountInput};
use crate::{
    account::{Account, CircuitAccount},
    merkle_proof::{CircuitMerkleProof, MerkleProof},
    note::{CircuitNote, Note},
    operation::{CircuitOperation, Operation},
    poseidon_consts::{RATE, R_F, R_P, T_WIDTH},
    CloneToVec,
};

type OpFor<A, F> = <<A as Account<F>>::CircuitAccount as CircuitAccount<F>>::Op;

pub struct UpdateNoteInput<F, A, const TREE_HEIGHT: usize>
where
    F: BigPrimeField,
    A: Account<F>,
{
    //public inputs
    pub op_pub: <OpFor<A, F> as CircuitOperation<F>>::OpPub,
    pub new_note_hash: AssignedValue<F>,
    pub merkle_root: AssignedValue<F>,
    //old_note.nullifier is also a public input

    //witnesses
    pub new_note: CircuitNote<F>,
    pub old_note: CircuitNote<F>,
    pub merkle_proof: CircuitMerkleProof<F, TREE_HEIGHT>,
    pub op_priv: <OpFor<A, F> as CircuitOperation<F>>::OpPriv,

    pub old_account: <A as Account<F>>::CircuitAccount,
}

//helper functions
#[allow(clippy::too_many_arguments)]
impl<F, A, const TREE_HEIGHT: usize> UpdateNoteInput<F, A, TREE_HEIGHT>
where
    F: BigPrimeField,
    A: Account<F>,
{
    pub fn new(
        ctx: &mut Context<F>,
        op_pub: <A::Op as Operation<F>>::OpPub,
        new_note_hash: F,
        merkle_root: F,
        new_note: Note<F>,
        old_note: Note<F>,
        merkle_proof: MerkleProof<F, TREE_HEIGHT>,
        op_priv: <A::Op as Operation<F>>::OpPriv,
        old_account: A,
    ) -> Self {
        let op_pub = op_pub
            .into()
            .iter()
            .map(|x| ctx.load_witness(*x))
            .collect::<Vec<AssignedValue<F>>>()
            .into();
        let new_note_hash = ctx.load_witness(new_note_hash);
        let merkle_root = ctx.load_witness(merkle_root);
        let new_note = new_note.load(ctx);
        let old_note = old_note.load(ctx);
        let merkle_proof = merkle_proof.load(ctx);
        let op_priv = op_priv
            .into()
            .iter()
            .map(|x| ctx.load_witness(*x))
            .collect::<Vec<AssignedValue<F>>>()
            .into();

        let old_account = old_account.load(ctx);

        Self {
            op_pub,
            new_note_hash,
            merkle_root,
            new_note,
            old_note,
            merkle_proof,
            op_priv,
            old_account,
        }
    }
}

fn verify_note_circuit<F>(
    ctx: &mut Context<F>,
    gate: &GateChip<F>,
    poseidon: &mut PoseidonHasher<F, T_WIDTH, RATE>,
    note: &CircuitNote<F>,
    note_hash: AssignedValue<F>,
) where
    F: BigPrimeField,
{
    let inner_note_hash = poseidon.hash_fix_len_array(ctx, gate, &note.clone_to_vec());
    let eq = gate.is_equal(ctx, note_hash, inner_note_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);
}

#[allow(dead_code)]
pub fn update_note_circuit<F, A, const TREE_HEIGHT: usize>(
    ctx: &mut Context<F>,
    input: UpdateNoteInput<F, A, TREE_HEIGHT>,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
    A: Account<F>,
{
    let gate = GateChip::<F>::default();
    let mut poseidon =
        PoseidonHasher::<F, T_WIDTH, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    let op_pub = input.op_pub;

    make_public.extend(op_pub.clone().into());

    let new_note_hash = input.new_note_hash;
    let merkle_root = input.merkle_root;
    let old_nullifier = input.old_note.nullifier;

    make_public.extend([new_note_hash, merkle_root, old_nullifier]);

    verify_note_circuit(ctx, &gate, &mut poseidon, &input.new_note, new_note_hash);

    let old_note_hash = poseidon.hash_fix_len_array(ctx, &gate, &input.old_note.clone_to_vec());

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

    update_account_circuit(ctx, update_account_input);
}
