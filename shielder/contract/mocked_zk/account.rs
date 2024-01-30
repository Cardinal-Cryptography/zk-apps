use ink::env::hash::{CryptoHash, Sha2x256};

use super::{ops::Operation, traits::Hashable, USDT_TOKEN};
use crate::{contract::OpPub, errors::ShielderError, types::Scalar};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Account {
    balance_aleph: Scalar,
    balance_usdt: Scalar,
    balance_test_token: Scalar,
    test_token_addr: Scalar,
}

impl Hashable for Account {
    fn hash(&self) -> Scalar {
        let mut res = [0x0; 32];
        Sha2x256::hash(
            [self.balance_aleph.bytes, self.balance_usdt.bytes]
                .concat()
                .as_slice(),
            &mut res,
        );
        Scalar { bytes: res }
    }
}

impl Account {
    pub fn new(test_token_addr: Scalar) -> Self {
        Self {
            balance_aleph: 0_u128.into(),
            balance_usdt: 0_u128.into(),
            balance_test_token: 0_u128.into(),
            test_token_addr,
        }
    }
    pub fn update(&self, operation: Operation) -> Result<Self, ShielderError> {
        match operation.op_pub {
            OpPub::Deposit { amount, token, .. } => {
                let mut balance_usdt = self.balance_usdt;
                let mut balance_test_token = self.balance_test_token;
                if token.bytes == USDT_TOKEN {
                    balance_usdt = (u128::from(balance_usdt)
                        .checked_add(amount)
                        .ok_or(ShielderError::ArithmeticError)?)
                    .into();
                }
                if token.bytes == self.test_token_addr.bytes {
                    balance_test_token = (u128::from(balance_test_token)
                        .checked_add(amount)
                        .ok_or(ShielderError::ArithmeticError)?)
                    .into();
                }
                Ok(Self {
                    balance_aleph: self.balance_aleph,
                    balance_usdt,
                    balance_test_token,
                    test_token_addr: self.test_token_addr,
                })
            }
            OpPub::Withdraw { amount, token, .. } => {
                let mut balance_usdt = self.balance_usdt;
                let mut balance_test_token = self.balance_test_token;
                if token.bytes == USDT_TOKEN {
                    balance_usdt = (u128::from(balance_usdt)
                        .checked_sub(amount)
                        .ok_or(ShielderError::ArithmeticError)?)
                    .into();
                }
                if token.bytes == self.test_token_addr.bytes {
                    balance_test_token = (u128::from(balance_test_token)
                        .checked_sub(amount)
                        .ok_or(ShielderError::ArithmeticError)?)
                    .into();
                }
                Ok(Self {
                    balance_aleph: self.balance_aleph,
                    balance_usdt,
                    balance_test_token,
                    test_token_addr: self.test_token_addr,
                })
            }
        }
    }
}
