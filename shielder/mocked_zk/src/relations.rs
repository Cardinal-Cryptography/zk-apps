use crate::{
    account::Account,
    combine_merkle_hash,
    errors::ZkpError,
    note::Note,
    ops::{OpPriv, OpPub, Operation},
    traits::Hashable,
    Scalar, MERKLE_TREE_DEPTH, TOKENS_NUMBER,
};

/// mocked proof of knowledge, not ZK
/// you can imagine ZkProof object as someone's "knowledge"
/// functions starting with verify_ are mocks of relation
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ZkProof {
    id: Scalar,
    trapdoor_new: Scalar,
    trapdoor_old: Scalar,
    nullifier_new: Scalar,
    acc_old: Account,
    acc_new: Account,
    op_priv: OpPriv,
    merkle_proof: [Scalar; MERKLE_TREE_DEPTH],
    merkle_proof_leaf_id: u32,
}

pub fn verify_hash<T: Hashable>(to_hash: T, hash: Scalar) -> Result<Scalar, ZkpError> {
    let real_hash = to_hash.hash();
    if real_hash != hash {
        return Err(ZkpError::VerificationError);
    }
    Ok(real_hash)
}

impl ZkProof {
    pub fn new(
        id: Scalar,
        trapdoor: Scalar,
        nullifier: Scalar,
        op_priv: OpPriv,
        acc: Account,
    ) -> Self {
        Self {
            id,
            trapdoor_new: trapdoor,
            nullifier_new: nullifier,
            acc_new: acc,
            trapdoor_old: 0_u128.into(),
            acc_old: acc,
            op_priv,
            merkle_proof: [0_u128.into(); MERKLE_TREE_DEPTH],
            merkle_proof_leaf_id: 0,
        }
    }

    fn transition(
        &self,
        trapdoor: Scalar,
        nullifier: Scalar,
        acc: Account,
        op_priv: OpPriv,
        merkle_proof: [Scalar; MERKLE_TREE_DEPTH],
        merkle_proof_leaf_id: u32,
    ) -> Self {
        Self {
            id: self.id,
            trapdoor_new: trapdoor,
            trapdoor_old: self.trapdoor_new,
            nullifier_new: nullifier,
            acc_new: acc,
            acc_old: self.acc_new,
            op_priv,
            merkle_proof,
            merkle_proof_leaf_id,
        }
    }

    pub fn update_account(
        &self,
        operation: Operation,
        trapdoor: Scalar,
        nullifier: Scalar,
        merkle_proof: [Scalar; MERKLE_TREE_DEPTH],
        merkle_proof_leaf_id: u32,
    ) -> Result<(Scalar, Self), ZkpError> {
        let acc_updated = self.acc_new.update(operation)?;
        let note = Note::new(self.id, trapdoor, nullifier, acc_updated.hash());
        let new_proof = self.transition(
            trapdoor,
            nullifier,
            acc_updated,
            operation.op_priv,
            merkle_proof,
            merkle_proof_leaf_id,
        );
        Ok((note.hash(), new_proof))
    }

    pub fn verify_acccount_update(
        &self,
        op: Operation,
        h_acc_old: Scalar,
    ) -> Result<Account, ZkpError> {
        let acc_new = self.acc_old.update(op)?;
        verify_hash(self.acc_old, h_acc_old)?;
        Ok(acc_new)
    }

    fn verify_merkle_proof(&self, h_note_old: Scalar, merkle_root: Scalar) -> Result<(), ZkpError> {
        let mut id = self.merkle_proof_leaf_id;
        let mut scalar = h_note_old;
        for node in self.merkle_proof {
            if id % 2 == 0 {
                scalar = combine_merkle_hash(scalar, node);
            } else {
                scalar = combine_merkle_hash(node, scalar);
            }
            id /= 2;
        }
        if scalar != merkle_root {
            return Err(ZkpError::VerificationError);
        }
        Ok(())
    }

    pub fn verify_creation(
        &self,
        h_note_new: Scalar,
        tokens_list: [Scalar; TOKENS_NUMBER],
    ) -> Result<(), ZkpError> {
        let h_acc_new = Account::new(tokens_list).hash();
        let note_new = Note::new(self.id, self.trapdoor_new, self.nullifier_new, h_acc_new);
        verify_hash(note_new, h_note_new)?;
        Ok(())
    }

    pub fn verify_update(
        &self,
        op_pub: OpPub,
        h_note_new: Scalar,
        merkle_root: Scalar,
        nullifier_old: Scalar,
    ) -> Result<(), ZkpError> {
        let h_acc_old = self.acc_old.hash();
        let op = Operation::combine(op_pub, self.op_priv)?;
        let acc_new = self.verify_acccount_update(op, h_acc_old)?;
        let h_acc_new = acc_new.hash();
        let note_new = Note::new(self.id, self.trapdoor_new, self.nullifier_new, h_acc_new);
        verify_hash(note_new, h_note_new)?;
        let note_old = Note::new(self.id, self.trapdoor_old, nullifier_old, h_acc_old);
        let h_note_old = note_old.hash();
        self.verify_merkle_proof(h_note_old, merkle_root)?;
        Ok(())
    }
}
