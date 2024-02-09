pub mod account;
pub mod note;
pub mod ops;
pub mod relations;
#[cfg(test)]
mod tests;
pub mod traits;

use crate::types::Scalar;

pub const TOKENS_NUMBER: usize = 10;
pub const MOCKED_TOKEN: Scalar = Scalar {
    bytes: [228_u8; 32],
};
