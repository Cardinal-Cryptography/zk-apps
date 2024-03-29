use halo2_base::{utils::ScalarField, AssignedValue, Context};

use crate::CloneToVec;

#[derive(Clone, Copy, Debug)]
pub struct Note<F: ScalarField> {
    pub zk_id: F,
    pub trapdoor: F,
    pub nullifier: F,
    pub account_hash: F,
}

impl<F: ScalarField> Note<F> {
    pub fn new(note_id: F, trapdoor: F, nullifier: F, account_hash: F) -> Self {
        Self {
            zk_id: note_id,
            trapdoor,
            nullifier,
            account_hash,
        }
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitNote<F> {
        CircuitNote {
            zk_id: ctx.load_witness(self.zk_id),
            trapdoor: ctx.load_witness(self.trapdoor),
            nullifier: ctx.load_witness(self.nullifier),
            account_hash: ctx.load_witness(self.account_hash),
        }
    }
}

impl<F: ScalarField> CloneToVec<F> for Note<F> {
    fn clone_to_vec(&self) -> Vec<F> {
        vec![self.zk_id, self.trapdoor, self.nullifier, self.account_hash]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CircuitNote<F: ScalarField> {
    pub zk_id: AssignedValue<F>,
    pub trapdoor: AssignedValue<F>,
    pub nullifier: AssignedValue<F>,
    pub account_hash: AssignedValue<F>,
}

impl<F: ScalarField> CloneToVec<AssignedValue<F>> for CircuitNote<F> {
    fn clone_to_vec(&self) -> Vec<AssignedValue<F>> {
        vec![self.zk_id, self.trapdoor, self.nullifier, self.account_hash]
    }
}
