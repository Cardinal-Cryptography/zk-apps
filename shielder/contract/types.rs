#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::storage::Mapping;

pub type Scalar = [u8; 32];
pub type Set<T> = Mapping<T, ()>;
