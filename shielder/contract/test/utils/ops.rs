use drink::AccountId32;

use mocked_zk::{
    ops::{OpPriv, OpPub},
    Scalar,
};

pub struct UpdateOperation {
    pub op_pub: OpPub,
    pub op_priv: OpPriv,
}

pub fn deposit_op(
    psp22_address: &AccountId32,
    user: &AccountId32,
    amount: u128,
) -> UpdateOperation {
    UpdateOperation {
        op_pub: OpPub::Deposit {
            amount,
            token: Scalar::from_bytes(*((*psp22_address).as_ref())),
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
        op_priv: OpPriv {
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
    }
}

pub fn withdraw_op(
    psp22_address: &AccountId32,
    user: &AccountId32,
    amount: u128,
) -> UpdateOperation {
    UpdateOperation {
        op_pub: OpPub::Withdraw {
            amount,
            token: Scalar::from_bytes(*((*psp22_address).as_ref())),
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
        op_priv: OpPriv {
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
    }
}
