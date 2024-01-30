use crate::{contract::OpPub, errors::ShielderError, types::Scalar};

/// empty private operation
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Clone, Copy)]
pub struct OpPriv {
    pub user: Scalar,
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy)]
pub struct Operation {
    pub op_pub: OpPub,
}

impl Operation {
    pub fn combine(op_pub: OpPub, _op_priv: OpPriv) -> Result<Self, ShielderError> {
        match op_pub {
            OpPub::Deposit { user, .. } => {
                if user != _op_priv.user {
                    return Err(ShielderError::ZkpVerificationFail);
                }
            }
            OpPub::Withdraw { user, .. } => {
                if user != _op_priv.user {
                    return Err(ShielderError::ZkpVerificationFail);
                }
            }
        }
        Ok(Operation { op_pub })
    }
}
