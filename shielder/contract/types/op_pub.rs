use super::Scalar;

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
}
