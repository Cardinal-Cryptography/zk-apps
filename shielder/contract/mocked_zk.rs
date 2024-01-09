// mocked zk relation


#![cfg_attr(not(feature = "std"), no_std, no_main)]

use crate::{errors::ShielderError, types::Scalar, contract::OpPub, merkle::{self, compute_hash}};

trait Hashable {
    fn hash(&self) -> Scalar;
}

#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std", 
    derive(scale_info::TypeInfo)
)]

// empty private operation
struct OpPriv {
}

#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std", 
    derive(scale_info::TypeInfo)
)]

struct Operation {
    op_pub: OpPub,
}

impl Operation {
    fn combine(op_pub: OpPub, _op_priv: OpPriv) -> Self {
        Operation{op_pub}
    }
}

#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std", 
    derive(scale_info::TypeInfo)
)]
struct Note {
    id: Scalar,
    trapdoor: Scalar,
    nullifier: Scalar,
    account_hash: Scalar,
}

impl Hashable for Note {
    fn hash(&self) -> Scalar {
        merkle::compute_hash(
            self.id,
            merkle::compute_hash(
                self.trapdoor,
                merkle::compute_hash(
                    self.nullifier, 
                    self.account_hash,
                )
            )
        )
    }
}

#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std", 
    derive(scale_info::TypeInfo)
)]
struct Account {
    balance_aleph: Scalar,
    balance_usdt: Scalar,
}

impl Hashable for Account {
    fn hash(&self) -> Scalar {
        merkle::compute_hash(
            self.balance_aleph,

            self.balance_usdt
        )
    }
}

const USDT_TOKEN: [u8; 32] = [0x2 as u8; 32];

impl Account {
    // TODO: increase and decrease balances
    fn update(&self, operation: Operation) -> Self {
        match operation.op_pub {
            OpPub::Deposit(amount, token, _) => {
                let balance_usdt = self.balance_usdt;
                if token.as_ref() == USDT_TOKEN {
                    // decrease scalar by amount
                }
                Self {
                    balance_aleph: self.balance_aleph,
                    balance_usdt,
                }
            },
            OpPub::Withdraw(amount, token, _) => {
                let balance_usdt = self.balance_usdt;
                if token.as_ref() == USDT_TOKEN {
                    // increase scalar by amount
                }
                Self {
                    balance_aleph: self.balance_aleph,
                    balance_usdt,
                }
            }
        }
    }
}


#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std", 
    derive(scale_info::TypeInfo)
)]

pub struct ZkProof {
    id: Scalar,
    trapdoor_new: Scalar,
    trapdoor_old: Scalar,
    nullifier_new: Scalar,
    acc_old: Account,
    op_priv: OpPriv,
    merkle_proof: [Scalar; merkle::DEPTH],
    merkle_proof_leaf_id: u32,
}

fn verify_hash<T: Hashable>(to_hash: T, hash: Scalar) -> Result<Scalar, ShielderError> {
    let real_hash = to_hash.hash();
    if real_hash != hash {
        return Err(ShielderError::ZkpVerificationFail)
    }
    Ok(real_hash)
}

fn verify_acccount_update(
    proof: ZkProof,
    op: Operation,
    h_acc_old: Scalar,
) -> Result<Account, ShielderError> {
    let acc_new = proof.acc_old.update(op);
    verify_hash(proof.acc_old, h_acc_old)?;
    Ok(acc_new)
}

fn verify_merkle_proof(
    proof: ZkProof,
    h_note_old: Scalar,
    merkle_root: Scalar
) -> Result<(), ShielderError> {
    let mut id = proof.merkle_proof_leaf_id;
    let mut scalar = h_note_old;
    for node in proof.merkle_proof {
        if id % 2 == 0{
            scalar = compute_hash(scalar, node);
        }
        else {
            scalar = compute_hash(node, scalar);
        }
        id /= 2;
    }
    if scalar != merkle_root {
        return Err(ShielderError::ZkpVerificationFail)
    }
    Ok(())
}

pub fn verify_update (
    proof: ZkProof,
    op_pub: OpPub,
    h_note_new: Scalar,
    merkle_root: Scalar,
    nullifier_old: Scalar
) -> Result<(), ShielderError> {
    let h_acc_old = proof.acc_old.hash();
    let op = Operation::combine(op_pub, proof.op_priv);
    let acc_new = verify_acccount_update(proof, op, h_acc_old)?;
    let h_acc_new = acc_new.hash();
    let note_new = Note {
        id: proof.id,
        trapdoor: proof.trapdoor_new,
        nullifier: proof.nullifier_new,
        account_hash: h_acc_new,
    };
    verify_hash(note_new, h_note_new)?;
    let note_old = Note {
        id: proof.id,
        trapdoor: proof.trapdoor_old,
        nullifier: nullifier_old,
        account_hash: h_acc_old,
    };
    let h_note_old = note_old.hash();
    verify_merkle_proof(proof, h_note_old, merkle_root)?;
    Ok(())
}