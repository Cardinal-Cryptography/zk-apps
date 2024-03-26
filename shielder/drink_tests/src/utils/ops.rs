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

pub fn deposit_op_relayer(
    psp22_address: &AccountId32,
    user: &AccountId32,
    amount: u128,
    azero_address: &AccountId32,
    relayer: &AccountId32,
    fee: u128,
) -> UpdateOperation {
    UpdateOperation {
        op_pub: OpPub::DepositRelayer {
            amount,
            token: Scalar::from_bytes(*((*psp22_address).as_ref())),
            user: Scalar::from_bytes(*((*user).as_ref())),
            fee,
            fee_token: Scalar::from_bytes(*((*azero_address).as_ref())),
            relayer: Scalar::from_bytes(*((*relayer).as_ref())),
        },
        op_priv: OpPriv {
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
    }
}

pub fn withdraw_op_relayer(
    psp22_address: &AccountId32,
    user: &AccountId32,
    amount: u128,
    azero_address: &AccountId32,
    relayer: &AccountId32,
    fee: u128,
) -> UpdateOperation {
    UpdateOperation {
        op_pub: OpPub::WithdrawRelayer {
            amount,
            token: Scalar::from_bytes(*((*psp22_address).as_ref())),
            user: Scalar::from_bytes(*((*user).as_ref())),
            fee,
            fee_token: Scalar::from_bytes(*((*azero_address).as_ref())),
            relayer: Scalar::from_bytes(*((*relayer).as_ref())),
        },
        op_priv: OpPriv {
            user: Scalar::from_bytes(*((*user).as_ref())),
        },
    }
}
