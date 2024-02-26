use ink::env::hash::{CryptoHash, Sha2x256};

use super::{
    ops::{OpPub, Operation},
    traits::Hashable,
    TOKENS_NUMBER,
};
use crate::{errors::ShielderError, types::Scalar};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Account {
    balances: [(Scalar, Scalar); TOKENS_NUMBER],
}

impl Hashable for Account {
    fn hash(&self) -> Scalar {
        let mut res = [0x0; 32];
        for i in 1..TOKENS_NUMBER {
            Sha2x256::hash(self.balances[i].1.bytes.as_slice(), &mut res);
        }
        Scalar::from_bytes(res)
    }
}

impl Account {
    pub fn new(tokens: [Scalar; TOKENS_NUMBER]) -> Self {
        let mut balances: [(Scalar, Scalar); TOKENS_NUMBER] =
            [(0_u128.into(), 0_u128.into()); TOKENS_NUMBER];
        for i in 0..TOKENS_NUMBER {
            balances[i] = (tokens[i], 0_u128.into());
        }
        Self { balances }
    }

    pub fn update(&self, operation: Operation) -> Result<Self, ShielderError> {
        match operation.op_pub {
            OpPub::Deposit {
                amount: op_amount,
                token: op_token,
                ..
            } => {
                for (i, (token, balance)) in self.balances.into_iter().enumerate() {
                    if token == op_token {
                        let balance_upd: Scalar = ((u128::from(balance))
                            .checked_add(op_amount)
                            .ok_or(ShielderError::ArithmeticError)?)
                        .into();
                        let mut balances_upd = self.balances;
                        balances_upd[i] = (token, balance_upd);
                        return Ok(Self {
                            balances: balances_upd,
                        });
                    }
                }
                Err(ShielderError::ZkpVerificationFail)
            }
            OpPub::Withdraw {
                amount: op_amount,
                token: op_token,
                ..
            } => {
                for (i, (token, balance)) in self.balances.into_iter().enumerate() {
                    if token == op_token {
                        let balance_upd: Scalar = ((u128::from(balance))
                            .checked_sub(op_amount)
                            .ok_or(ShielderError::ArithmeticError)?)
                        .into();
                        let mut balances_upd = self.balances;
                        balances_upd[i] = (token, balance_upd);
                        return Ok(Self {
                            balances: balances_upd,
                        });
                    }
                }
                Err(ShielderError::ZkpVerificationFail)
            }
        }
    }
}
