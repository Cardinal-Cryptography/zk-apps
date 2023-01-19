#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

/// Most basic PSP22 token.
#[openbrush::contract]
#[allow(clippy::let_unit_value)] // Clippy shouts about returning anything from messages.
pub mod token {
    use openbrush::{
        contracts::psp22::{self, psp22::Internal, Data},
        traits::Storage,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Token {
        #[storage_field]
        psp22: Data,
    }

    impl psp22::PSP22 for Token {}

    impl Token {
        /// Instantiate the contract with `total_supply` tokens of supply.
        ///
        /// All the created tokens will be minted to the caller.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut instance = Self::default();

            instance
                .psp22
                ._mint_to(Self::env().caller(), total_supply)
                .expect("Should mint");

            instance
        }
    }
}
