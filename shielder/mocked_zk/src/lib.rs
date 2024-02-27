#![cfg_attr(not(feature = "std"), no_std, no_main)]
pub mod account;
pub mod errors;
pub mod note;
pub mod ops;
pub mod relations;
mod scalar;
#[cfg(test)]
mod tests;
pub mod traits;

use ink::env::hash::{CryptoHash, Sha2x256};

pub type Scalar = scalar::Scalar;

pub const MERKLE_TREE_DEPTH: usize = 10;
pub const TOKENS_NUMBER: usize = 2;
pub const MOCKED_TOKEN: Scalar = Scalar::from_bytes([228_u8; 32]);

pub fn mocked_user() -> Scalar {
    1_u128.into()
}

pub fn combine_merkle_hash(first: Scalar, second: Scalar) -> Scalar {
    let mut res = [0x0; 32];
    Sha2x256::hash([first.bytes, second.bytes].concat().as_slice(), &mut res);
    Scalar::from_bytes(res)
}
