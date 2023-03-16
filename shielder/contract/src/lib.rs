#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use baby_liminal_extension::{ProvingSystem, VerificationKeyIdentifier};
use ink::storage::Mapping;

mod contract;
mod error;

type Scalar = u64;
pub type Nullifier = Scalar;

/// Tangling output type.
pub type MerkleHash = [u64; 4];
/// Type of the value in the Merkle tree leaf.
pub type Note = MerkleHash;
/// Type of the value in the Merkle tree root.
pub type MerkleRoot = MerkleHash;

/// Short identifier of a registered token contract.
pub type TokenId = u16;
/// `arkworks` does not support serializing `u128` and thus we have to operate on `u64` amounts.
pub type TokenAmount = u64;

type Set<T> = Mapping<T, ()>;

/// Verification key identifier for the `deposit` relation (to be registered in `pallet_snarcos`).
const DEPOSIT_VK_IDENTIFIER: VerificationKeyIdentifier = [b'd', b'p', b's', b't'];
/// Verification key identifier for the `deposit` relation (to be registered in `pallet_snarcos`).
const DEPOSIT_AND_MERGE_VK_IDENTIFIER: VerificationKeyIdentifier = [b'd', b'p', b'm', b'g'];
/// Verification key identifier for the `withdraw` relation (to be registered in `pallet_snarcos`).
const WITHDRAW_VK_IDENTIFIER: VerificationKeyIdentifier = [b'w', b't', b'h', b'd'];
/// The only supported proving system for now.
const SYSTEM: ProvingSystem = ProvingSystem::Groth16;

fn array_to_tuple(a: [u64; 4]) -> (u64, u64, u64, u64) {
    (a[0], a[1], a[2], a[3])
}

fn tuple_to_array(a: (u64, u64, u64, u64)) -> [u64; 4] {
    [a.0, a.1, a.2, a.3]
}
