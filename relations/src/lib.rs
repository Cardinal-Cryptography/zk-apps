pub mod account;
pub mod merkle_proof;
pub mod note;
pub mod operation;
pub mod relations;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    AZERO,
    USDT,
}

pub trait CloneToVec<T> {
    fn clone_to_vec(&self) -> Vec<T>;
}

pub mod poseidon_consts {
    /// Has to be greater than 1 and equal to RATE + 1, due to the outer Poseidon implementation.
    pub const T_WIDTH: usize = RATE + 1;

    pub const RATE: usize = 4;

    pub const R_F: usize = 8;

    pub const R_P: usize = 56;
}
