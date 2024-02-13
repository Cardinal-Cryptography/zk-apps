pub mod account;
pub mod hasher;
pub mod note;
pub mod operation;
pub mod proof;
pub mod relations;

#[cfg(test)]
pub mod tests;

/// Represents the available tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    AZERO,
    USDT,
}

pub mod poseidon_consts {
    /// The value of T, which represents a parameter used in the implementation of the Poseidon hash function.
    /// Has to be greater than 1 and equal to RATE + 1, due to the outer Poseidon implementation.
    pub const T: usize = 5;

    /// The value of RATE, which represents the rate of the Poseidon hash function.
    pub const RATE: usize = 4;

    /// The value of R_F, which represents a parameter used in the implementation of the Poseidon hash function.
    pub const R_F: usize = 8;

    /// The value of R_P, which represents a parameter used in the implementation of the Poseidon hash function.
    pub const R_P: usize = 56;
}
