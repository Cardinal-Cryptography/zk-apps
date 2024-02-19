use halo2_base::{
    gates::{GateChip, GateInstructions},
    poseidon::hasher::{spec::OptimizedPoseidonSpec, PoseidonHasher},
    utils::BigPrimeField,
    AssignedValue,
};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};

use crate::{
    account::CircuitAccount,
    poseidon_consts::{RATE, R_F, R_P, T_WIDTH},
};

pub struct UpdateAccountInput<F, A>
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    //public inputs
    pub old_account_hash: AssignedValue<F>,
    pub new_account_hash: AssignedValue<F>,
    pub operation: A::Op,

    //witnesses
    pub old_account: A,
}

impl<F, A> UpdateAccountInput<F, A>
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    pub fn new(
        old_account_hash: AssignedValue<F>,
        new_account_hash: AssignedValue<F>,
        operation: A::Op,
        old_account: A,
    ) -> Self {
        Self {
            old_account_hash,
            new_account_hash,
            operation,
            old_account,
        }
    }
}

pub fn verify_account_circuit<F, A>(
    ctx: &mut Context<F>,
    gate: &GateChip<F>,
    poseidon: &mut PoseidonHasher<F, T_WIDTH, RATE>,
    account: &A,
    account_hash: AssignedValue<F>,
) where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    let inner_account_hash = poseidon.hash_fix_len_array(ctx, gate, &account.to_array());
    let eq = gate.is_equal(ctx, account_hash, inner_account_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);
}

#[allow(dead_code)]
pub fn update_account_circuit<F, A>(ctx: &mut Context<F>, input: UpdateAccountInput<F, A>)
where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    let gate = GateChip::<F>::default();
    let mut poseidon =
        PoseidonHasher::<F, T_WIDTH, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    let old_account = input.old_account;
    verify_account_circuit(
        ctx,
        &gate,
        &mut poseidon,
        &old_account,
        input.old_account_hash,
    );

    let new_account = old_account.update(input.operation, ctx, &gate);
    verify_account_circuit(
        ctx,
        &gate,
        &mut poseidon,
        &new_account,
        input.new_account_hash,
    );
}
