#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod merkle;
mod psp22;
mod types;

#[ink::contract]
mod contract {
    use ink::storage::Mapping;

    use crate::{merkle::MerkleTree, psp22::PSP22, types::{Set, Scalar}};

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std", 
        derive(scale_info::TypeInfo)
    )]
    pub enum OpPub {
        Deposit(u128, AccountId, AccountId),
        Withdraw(u128, AccountId, AccountId)
    }

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std", 
        derive(scale_info::TypeInfo)
    )]

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Contract {
        nullifier_set: Set<u128>,
        notes: MerkleTree,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                nullifier_set: Mapping::default(),
                notes: MerkleTree::new(),
            }
        }

        #[ink(message)]
        pub fn add_note(&mut self, op_pub: OpPub, h_note_new: Scalar) {
            self.process_operation(op_pub);
            self.notes.add_leaf(h_note_new);
        }

        #[ink(message)]
        pub fn update_note(
            &mut self,
            op_pub: OpPub,
            h_note_new: Scalar,
            merkle_root: Scalar,
            nullifier_old: u128,
        ) {
            self.process_operation(op_pub);
            assert!(self.notes.is_historical_root(merkle_root));
            assert!(!self.nullifier_set.contains(nullifier_old));
            self.notes.add_leaf(h_note_new);
            self.nullifier_set.insert(nullifier_old, &());
        }

        fn process_operation(&mut self, op_pub: OpPub) {
            match op_pub {
                OpPub::Deposit(amount, token_id, user) => {
                    let mut psp22: ink::contract_ref!(PSP22) = token_id.into();
                    psp22.transfer_from(
                        user,
                        self.env().account_id(),
                        amount,
                        [].to_vec()
                    ).unwrap();
                },
                OpPub::Withdraw(amount, token_id, user) => {
                    let mut psp22: ink::contract_ref!(PSP22) = token_id.into();
                    psp22.transfer(
                        user,
                        amount,
                        [].to_vec()
                    ).unwrap();
                }
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        // /// Imports all the definitions from the outer scope so we can use them here.
        // use super::*;

        // /// We test if the default constructor does its job.
        // #[ink::test]
        // fn default_works() {
        //     let contract = Contract::default();
        //     assert_eq!(contract.get(), false);
        // }

        // /// We test a simple use case of our contract.
        // #[ink::test]
        // fn it_works() {
        //     let mut contract = Contract::new(false);
        //     assert_eq!(contract.get(), false);
        //     contract.flip();
        //     assert_eq!(contract.get(), true);
        // }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ContractRef::default();

            // When
            let contract_account_id = client
                .instantiate("contract", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<ContractRef>(contract_account_id.clone())
                .call(|contract| contract.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = ContractRef::new(false);
            let contract_account_id = client
                .instantiate("contract", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<ContractRef>(contract_account_id.clone())
                .call(|contract| contract.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<ContractRef>(contract_account_id.clone())
                .call(|contract| contract.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<ContractRef>(contract_account_id.clone())
                .call(|contract| contract.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
