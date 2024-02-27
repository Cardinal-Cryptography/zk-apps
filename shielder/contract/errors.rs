use crate::traits::psp22::PSP22Error;
use mocked_zk::errors::ZkpError;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(PartialEq, Debug)]
pub enum ShielderError {
    PSP22(PSP22Error),
    NullifierIsInSet,
    MerkleTreeVerificationFail,
    MerkleTreeLimitExceeded,
    MerkleTreeProofGenFail,
    MerkleTreeNonExistingNode,
    ZkpVerificationFail,
    ArithmeticError,
}

impl From<PSP22Error> for ShielderError {
    fn from(inner: PSP22Error) -> Self {
        ShielderError::PSP22(inner)
    }
}

impl From<ZkpError> for ShielderError {
    fn from(_inner: ZkpError) -> Self {
        ShielderError::ZkpVerificationFail
    }
}
