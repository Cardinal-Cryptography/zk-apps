//! Smart contract implementing shielder specification
//! https://docs.alephzero.org/aleph-zero/shielder/introduction-informal

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![deny(missing_docs)]

mod errors;
mod merkle;
mod mocked_zk;
mod traits;
mod types;

/// Contract module
#[ink::contract]
pub mod contract {

    use crate::{
        errors::ShielderError,
        merkle::MerkleTree,
        mocked_zk::relations::ZkProof,
        traits::psp22::PSP22,
        types::{Scalar, Set},
    };

    /// Enum
    #[derive(Clone, Copy, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum OpPub {
        /// Deposit PSP-22 token
        Deposit {
            /// amount of deposit
            amount: u128,
            /// PSP-22 token address
            token: AccountId,
            /// User address, from whom tokens are transferred
            user: AccountId,
        },
        /// Withdraw PSP-22 token
        Withdraw {
            /// amount of withdrawal
            amount: u128,
            /// PSP-22 token address
            token: AccountId,
            /// User address, from whom tokens are transferred
            user: AccountId,
        },
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        nullifier_set: Set<Scalar>,
        notes: MerkleTree,
    }

    impl Contract {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        /// Adds empty note to shielder storage
        /// Registers new account with empty balance
        #[ink(message)]
        pub fn add_note(
            &mut self,
            h_note_new: Scalar,
            proof: ZkProof,
        ) -> Result<(), ShielderError> {
            proof.verify_creation(h_note_new)?;
            self.notes.add_leaf(h_note_new)?;
            Ok(())
        }

        /// Updates existing note
        /// Applies operation to private account stored in shielder
        #[ink(message)]
        pub fn update_note(
            &mut self,
            op_pub: OpPub,
            h_note_new: Scalar,
            merkle_root: Scalar,
            nullifier_old: Scalar,
            proof: ZkProof,
        ) -> Result<(), ShielderError> {
            self.notes.is_historical_root(merkle_root)?;
            self.nullify(nullifier_old)?;
            proof.verify_update(op_pub, h_note_new, merkle_root, nullifier_old)?;
            self.notes.add_leaf(h_note_new)?;
            self.process_operation(op_pub)?;
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
            self.nullifier_set
                .insert(nullifier, &())
                .map(|_| {})
                .ok_or(ShielderError::NullifierIsInSet)
        }
    }
}
