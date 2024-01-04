#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

use baby_liminal_extension::{ProvingSystem, VerificationKeyIdentifier};
use ink::storage::Mapping;

mod contract;
mod error;

/// Tangling output type.
type MerkleHash = [u64; 4];
/// Type of the value in the Merkle tree leaf.
type Note = MerkleHash;
/// Type of the value in the Merkle tree root.
type MerkleRoot = MerkleHash;
/// Type of the nullifier.
type Nullifier = MerkleHash;

/// Short identifier of a registered token contract.
type TokenId = u16;
/// Type for the amount of deposited / withdrawn tokens.
type TokenAmount = u128;

type Set<T> = Mapping<T, ()>;

/// Verification key identifier for the `deposit` relation (to be registered in `pallet_snarcos`).
const DEPOSIT_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'd', b'd', b'e', b'p', b'o', b's', b'i', b't'];
/// Verification key identifier for the `deposit` relation (to be registered in `pallet_snarcos`).
const DEPOSIT_AND_MERGE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'd', b'e', b'p', b'o', b'n', b'm', b'r', b'g'];
/// Verification key identifier for the `merge` relation (to be registered in `pallet_baby_liminal`).
const MERGE_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'm', b'e', b'r', b'g', b'e', b'r', b'e', b'l'];
/// Verification key identifier for the `withdraw` relation (to be registered in `pallet_snarcos`).
const WITHDRAW_VK_IDENTIFIER: VerificationKeyIdentifier =
    [b'w', b'i', b't', b'h', b'd', b'r', b'a', b'w'];
/// The only supported proving system for now.
const SYSTEM: ProvingSystem = ProvingSystem::Groth16;

/// PSP22 standard selector for transferring on behalf.
const PSP22_TRANSFER_FROM_SELECTOR: [u8; 4] = [0x54, 0xb3, 0xc7, 0x6e];
/// PSP22 standard selector for transferring own tokens.
const PSP22_TRANSFER_SELECTOR: [u8; 4] = [0xdb, 0x20, 0xf9, 0xf5];

fn array_to_tuple(a: [u64; 4]) -> (u64, u64, u64, u64) {
    (a[0], a[1], a[2], a[3])
}

fn tuple_to_array(a: (u64, u64, u64, u64)) -> [u64; 4] {
    [a.0, a.1, a.2, a.3]
}
