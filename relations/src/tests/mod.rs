use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use poseidon::Poseidon;

use crate::poseidon_consts::{RATE, T};

pub mod account;
pub mod hasher;
pub mod note;
pub mod operation;
pub mod proof;
pub mod relations;

type PoseidonHasher = Poseidon<Fr, T, RATE>;
