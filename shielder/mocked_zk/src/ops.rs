use crate::{errors::ZkpError, Scalar};

/// Enum
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum OpPub {
    /// Deposit PSP-22 token
    Deposit {
        /// amount of deposit
        amount: u128,
        /// PSP-22 token address
        token: Scalar,
        /// User address, from whom tokens are transferred
        user: Scalar,
    },
    /// Withdraw PSP-22 token
    Withdraw {
        /// amount of withdrawal
        amount: u128,
        /// PSP-22 token address
        token: Scalar,
        /// User address, to who the tokens are transferred
        user: Scalar,
    },
    /// Deposit PSP-22 token through relayer
    DepositRelayer {
        /// amount of deposit
        amount: u128,
        /// PSP-22 token address
        token: Scalar,
        /// User address, from whom tokens are transferred
        user: Scalar,
        /// Fee amount for relayer
        fee: u128,
        /// PSP-22 token address
        fee_token: Scalar,
        /// Relayer address, from whom the transaction is initiated
        relayer: Scalar,
    },
    /// Withdraw PSP-22 token
    WithdrawRelayer {
        /// amount of withdrawal
        amount: u128,
        /// PSP-22 token address
        token: Scalar,
        /// User address, to who the tokens are transferred
        user: Scalar,
        /// Fee amount for relayer
        fee: u128,
        /// PSP-22 token address
        fee_token: Scalar,
        /// Relayer address, from whom the transaction is initiated
        relayer: Scalar,
    },
}

/// empty private operation
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct OpPriv {
    pub user: Scalar,
}

impl OpPriv {
    pub fn new(user: Scalar) -> Self {
        Self { user }
    }
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Operation {
    pub op_pub: OpPub,
    pub op_priv: OpPriv,
}

impl Operation {
    pub fn combine(op_pub: OpPub, op_priv: OpPriv) -> Result<Self, ZkpError> {
        match op_pub {
            OpPub::Deposit { user, .. } => {
                if user != op_priv.user {
                    return Err(ZkpError::OperationCombineError);
                }
            }
            OpPub::Withdraw { user, .. } => {
                if user != op_priv.user {
                    return Err(ZkpError::OperationCombineError);
                }
            }
            OpPub::DepositRelayer { user, .. } => {
                if user != op_priv.user {
                    return Err(ZkpError::OperationCombineError);
                }
            }
            OpPub::WithdrawRelayer { user, .. } => {
                if user != op_priv.user {
                    return Err(ZkpError::OperationCombineError);
                }
            }
        }
        Ok(Operation { op_pub, op_priv })
    }
}
