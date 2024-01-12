#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;
mod merkle;
mod mocked_zk;
mod psp22;
mod types;

#[ink::contract]
mod contract {

    use ink::storage::Mapping;

    use crate::{
        errors::ShielderError,
        merkle::MerkleTree,
        mocked_zk::{self, MockedZkProof},
        psp22::PSP22,
        types::{Scalar, Set},
    };

    #[derive(Clone, Copy, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum OpPub {
        Deposit(u128, AccountId, AccountId),
        Withdraw(u128, AccountId, AccountId),
    }

    #[ink(storage)]
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
        pub fn add_note(&mut self, op_pub: OpPub, h_note_new: Scalar) -> Result<(), ShielderError> {
            self.process_operation(op_pub)?;
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
            proof: MockedZkProof,
        ) -> Result<(), ShielderError> {
            self.process_operation(op_pub)?;
            self.notes.is_historical_root(merkle_root)?;
            (!self.nullifier_set.contains(nullifier_old))
                .then_some(())
                .ok_or(ShielderError::NullifierIsInSet)?;
            mocked_zk::verify_update(proof, op_pub, h_note_new, merkle_root, nullifier_old)?;
            self.notes.add_leaf(h_note_new)?;
            self.nullifier_set.insert(nullifier_old, &());
            Ok(())
        }

        fn process_operation(&mut self, op_pub: OpPub) -> Result<(), ShielderError> {
            match op_pub {
                OpPub::Deposit(amount, token_id, user) => {
                    let mut psp22: ink::contract_ref!(PSP22) = token_id.into();
                    psp22.transfer_from(user, self.env().account_id(), amount, [].to_vec())?;
                }
                OpPub::Withdraw(amount, token_id, user) => {
                    let mut psp22: ink::contract_ref!(PSP22) = token_id.into();
                    psp22.transfer(user, amount, [].to_vec())?;
                }
            };
            Ok(())
        }
    }
}
