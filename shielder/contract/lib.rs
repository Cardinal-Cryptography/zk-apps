#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;
mod merkle;
mod mocked_zk;
mod types;

#[ink::contract]
#[allow(clippy::large_enum_variant)]
mod contract {

    use ink::storage::Mapping;
    use psp22::PSP22;

    use crate::{
        errors::ShielderError,
        merkle::MerkleTree,
        mocked_zk::{self, ZkProof},
        types::{Scalar, Set},
    };

    #[derive(Clone, Copy, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum OpPub {
        Deposit {
            amount: u128,
            token: AccountId,
            user: AccountId,
        },
        Withdraw {
            amount: u128,
            token: AccountId,
            user: AccountId,
        },
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        nullifier_set: Set<Scalar>,
        notes: MerkleTree,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                nullifier_set: Mapping::new(),
                notes: MerkleTree::new(),
            }
        }

        #[ink(message)]
        pub fn add_note(&mut self, h_note_new: Scalar) -> Result<(), ShielderError> {
            self.notes.add_leaf(h_note_new)?;
            Ok(())
        }

        #[ink(message)]
        pub fn update_note(
            &mut self,
            op_pub: OpPub,
            h_note_new: Scalar,
            merkle_root: Scalar,
            nullifier_old: Scalar,
            proof: ZkProof,
        ) -> Result<(), ShielderError> {
            self.process_operation(op_pub)?;
            self.notes.is_historical_root(merkle_root)?;
            mocked_zk::verify_update(proof, op_pub, h_note_new, merkle_root, nullifier_old)?;
            self.nullify(nullifier_old)?;
            self.notes.add_leaf(h_note_new)?;
            Ok(())
        }

        fn process_operation(&mut self, op_pub: OpPub) -> Result<(), ShielderError> {
            match op_pub {
                OpPub::Deposit {
                    amount,
                    token,
                    user,
                } => {
                    let mut psp22: ink::contract_ref!(PSP22) = token.into();
                    psp22.transfer_from(user, self.env().account_id(), amount, [].to_vec())?;
                }
                OpPub::Withdraw {
                    amount,
                    token,
                    user,
                } => {
                    let mut psp22: ink::contract_ref!(PSP22) = token.into();
                    psp22.transfer(user, amount, [].to_vec())?;
                }
            };
            Ok(())
        }

        fn nullify(&mut self, nullifier: Scalar) -> Result<(), ShielderError> {
            if self.nullifier_set.contains(nullifier) {
                Err(ShielderError::NullifierIsInSet)
            } else {
                self.nullifier_set.insert(nullifier, &());
                Ok(())
            }
        }
    }
}
