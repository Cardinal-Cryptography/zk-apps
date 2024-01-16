use crate::traits::psp22::PSP22Error;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(PartialEq, Debug)]
pub enum ShielderError {
    PSP22(PSP22Error),
    NullifierIsInSet,
    MerkleTreeVerificationFail,
    MerkleTreeLimitExceeded,
    MerkleTreeProofGenFail,
    ZkpVerificationFail,
    ArithmeticError,
}

impl From<PSP22Error> for ShielderError {
    fn from(inner: PSP22Error) -> Self {
        ShielderError::PSP22(inner)
    }
}
