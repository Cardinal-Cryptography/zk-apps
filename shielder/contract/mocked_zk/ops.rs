use ink::primitives::AccountId;

use crate::{contract::OpPub, errors::ShielderError};

/// empty private operation
#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OpPriv {
    pub user: AccountId,
}

#[derive(Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
