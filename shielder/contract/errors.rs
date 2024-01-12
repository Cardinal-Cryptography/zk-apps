use psp22::PSP22Error;

#[derive(PartialEq, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ShielderError {
    PSP22(PSP22Error),
    NullifierIsInSet,
    MerkleTreeVerificationFail,
    MerkleTreeLimitExceeded,
    MerkleTreeProofGenFail,
    ZkpVerificationFail,
}

impl From<PSP22Error> for ShielderError {
    fn from(inner: PSP22Error) -> Self {
        ShielderError::PSP22(inner)
    }
}
