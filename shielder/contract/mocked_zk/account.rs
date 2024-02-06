use ink::env::hash::{CryptoHash, Sha2x256};

use super::{ops::Operation, traits::Hashable, TOKENS_NUMBER};
use crate::{contract::OpPub, errors::ShielderError, types::Scalar};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Account {
    balances: [(Scalar, Scalar); TOKENS_NUMBER],
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
    pub fn new(test_token_addr: Scalar, tokens: [Scalar; TOKENS_NUMBER]) -> Self {
        let mut balances: [(Scalar, Scalar); TOKENS_NUMBER] =
            [(0_u128.into(), 0_u128.into()); TOKENS_NUMBER];
        for i in 0..TOKENS_NUMBER {
            balances[i] = (tokens[i], 0_u128.into());
        }
        Self {
            balances,
            balance_aleph: 0_u128.into(),
            balance_usdt: 0_u128.into(),
            balance_test_token: 0_u128.into(),
            test_token_addr,
        }
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
                            balance_aleph: self.balance_aleph,
                            balance_usdt: self.balance_usdt,
                            balance_test_token: self.balance_test_token,
                            test_token_addr: self.test_token_addr,
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
                            balance_aleph: self.balance_aleph,
                            balance_usdt: self.balance_usdt,
                            balance_test_token: self.balance_test_token,
                            test_token_addr: self.test_token_addr,
                        });
                    }
                }
                Err(ShielderError::ZkpVerificationFail)
            }
        }
    }
}
