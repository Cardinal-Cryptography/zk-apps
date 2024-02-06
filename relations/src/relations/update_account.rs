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
    hasher::InnerHasher,
    poseidon_consts::{RATE, R_F, R_P, T},
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

#[allow(dead_code)]
pub fn update_account_circuit<F, A>(
    ctx: &mut Context<F>,
    input: UpdateAccountInput<F, A>,
    make_public: &mut Vec<AssignedValue<F>>,
) where
    F: BigPrimeField,
    A: CircuitAccount<F>,
{
    let old_account_hash = input.old_account_hash;
    let new_account_hash = input.new_account_hash;

    make_public.extend([old_account_hash, new_account_hash]);

    let gate = GateChip::<F>::default();

    let mut poseidon =
        PoseidonHasher::<F, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);

    let old_account = input.old_account;

    let inner_old_account_hash = poseidon.hash_account(ctx, &gate, &old_account);

    let new_account = old_account.update(input.operation, ctx, &gate);

    let inner_new_account_hash = poseidon.hash_account(ctx, &gate, &new_account);

    let eq = gate.is_equal(ctx, old_account_hash, inner_old_account_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);

    let eq = gate.is_equal(ctx, new_account_hash, inner_new_account_hash);
    gate.assert_is_const(ctx, &eq, &F::ONE);
}
