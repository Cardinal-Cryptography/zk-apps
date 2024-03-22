//! Smart contract implementing shielder specification
//! https://docs.alephzero.org/aleph-zero/shielder/introduction-informal

#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod errors;
mod merkle;
mod traits;
mod types;

/// Contract module
#[ink::contract]
pub mod contract {

    use crate::{errors::ShielderError, merkle::MerkleTree, traits::psp22::PSP22Error, types::Set};
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use mocked_zk::{ops::OpPub, relations::ZkProof, Scalar};

    pub const MERKLE_TREE_DEPTH: usize = mocked_zk::MERKLE_TREE_DEPTH;
    pub const TOKENS_NUMBER: usize = mocked_zk::TOKENS_NUMBER;

    /// Contract storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        nullifier_set: Set<Scalar>,
        notes: MerkleTree<{ MERKLE_TREE_DEPTH }>,
        supported_tokens: [Scalar; TOKENS_NUMBER],
    }

    impl Contract {
        /// Constructor
        #[ink(constructor)]
        pub fn new(supported_tokens: [Scalar; TOKENS_NUMBER]) -> Self {
            Self {
                supported_tokens,
                ..Default::default()
            }
        }

        /// Adds empty note to shielder storage
        /// Registers new account with empty balance
        /// Returns id of the note in shielder's storage
        #[ink(message)]
        pub fn add_note(
            &mut self,
            h_note_new: Scalar,
            proof: ZkProof,
        ) -> Result<u32, ShielderError> {
            proof.verify_creation(h_note_new, self.supported_tokens)?;
            self.notes.add_leaf(h_note_new)
        }

        /// Updates existing note
        /// Applies operation to private account stored in shielder
        /// Returns id of the note in shielder's storage
        #[ink(message)]
        pub fn update_note(
            &mut self,
            op_pub: OpPub,
            h_note_new: Scalar,
            merkle_root: Scalar,
            nullifier_old: Scalar,
            proof: ZkProof,
        ) -> Result<u32, ShielderError> {
            self.notes.is_historical_root(merkle_root)?;
            self.nullify(nullifier_old)?;
            proof.verify_update(op_pub, h_note_new, merkle_root, nullifier_old)?;
            let leaf_id = self.notes.add_leaf(h_note_new)?;
            self.process_operation(op_pub)?;
            Ok(leaf_id)
        }

        fn process_operation(&mut self, op_pub: OpPub) -> Result<(), ShielderError> {
            match op_pub {
                OpPub::Deposit {
                    amount,
                    token,
                    user,
                } => build_call::<DefaultEnvironment>()
                    .call(AccountId::from(token.bytes))
                    .call_v1()
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!(
                            "PSP22::transfer_from"
                        )))
                        .push_arg(AccountId::from(user.bytes))
                        .push_arg(self.env().account_id())
                        .push_arg(amount)
                        .push_arg([].to_vec() as ink::prelude::vec::Vec<u8>),
                    )
                    .returns::<Result<(), PSP22Error>>()
                    .invoke()?,
                OpPub::Withdraw {
                    amount,
                    token,
                    user,
                } => build_call::<DefaultEnvironment>()
                    .call(AccountId::from(token.bytes))
                    .call_v1()
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer")))
                            .push_arg(AccountId::from(user.bytes))
                            .push_arg(amount)
                            .push_arg([].to_vec() as ink::prelude::vec::Vec<u8>),
                    )
                    .returns::<Result<(), PSP22Error>>()
                    .invoke()?,
            };
            Ok(())
        }

        /// Returns merkle root of notes storage
        #[ink(message)]
        pub fn notes_merkle_root(&self) -> Result<Scalar, ShielderError> {
            self.notes.root()
        }

        /// Returns merkle path
        /// WARNING: that might expose identity of caller!
        #[ink(message)]
        pub fn notes_merkle_path(
            &self,
            note_id: u32,
        ) -> Result<[Scalar; MERKLE_TREE_DEPTH], ShielderError> {
            self.notes.gen_proof(note_id)
        }

        #[ink(message)]
        pub fn supported_tokens(&self) -> [Scalar; TOKENS_NUMBER] {
            self.supported_tokens
        }

        fn nullify(&mut self, nullifier: Scalar) -> Result<(), ShielderError> {
            self.nullifier_set
                .insert(nullifier, &())
                .map(|_| {})
                .map_or(Ok(()), |_| Err(ShielderError::NullifierIsInSet))
        }
    }
}
