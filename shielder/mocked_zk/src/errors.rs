#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(PartialEq, Debug)]
pub enum ZkpError {
    AccountUpdateError,
    OperationCombineError,
    VerificationError,
}
