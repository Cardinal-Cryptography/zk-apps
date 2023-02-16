use baby_liminal_extension::BabyLiminalError;
use ink::prelude::{format, string::String};
use openbrush::contracts::{psp22::PSP22Error, traits::ownable::OwnableError};
use scale::{Decode, Encode};

#[derive(Eq, PartialEq, Debug, Decode, Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ShielderError {
    /// Caller is missing some permission.
    InsufficientPermission(OwnableError),
    /// Merkle tree is full - no new notes can be created.
    TooManyNotes,
    /// There was no such merkle root.
    UnknownMerkleRoot,
    /// Cannot reuse nullifier.
    NullifierAlreadyUsed,
    /// Fee exceeds the withdrawn amount.
    TooHighFee,

    /// Pallet returned an error (through chain extension).
    ChainExtension(BabyLiminalError),

    /// PSP22 related error (e.g. insufficient allowance).
    Psp22(PSP22Error),
    /// Environment error (e.g. non-existing token contract).
    InkEnv(String),

    /// This token id is already taken.
    TokenIdAlreadyRegistered,
    /// There is no registered token under this token id.
    TokenIdNotRegistered,
}

impl From<BabyLiminalError> for ShielderError {
    fn from(e: BabyLiminalError) -> Self {
        ShielderError::ChainExtension(e)
    }
}

impl From<PSP22Error> for ShielderError {
    fn from(e: PSP22Error) -> Self {
        ShielderError::Psp22(e)
    }
}

impl From<OwnableError> for ShielderError {
    fn from(e: OwnableError) -> Self {
        ShielderError::InsufficientPermission(e)
    }
}

impl From<ink::env::Error> for ShielderError {
    fn from(e: ink::env::Error) -> Self {
        ShielderError::InkEnv(format!("{:?}", e))
    }
}
