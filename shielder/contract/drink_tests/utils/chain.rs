use anyhow::Result;
use drink::{
    minimal::MinimalSandbox, sandbox_api::balance_api::BalanceAPI, session::Session, AccountId32,
};

use super::ACCOUNT_INITIAL_AMOUNT;

pub fn init_acc_with_balance(
    session: &mut Session<MinimalSandbox>,
    acc: &AccountId32,
) -> Result<()> {
    session
        .sandbox()
        .mint_into(&acc.clone(), ACCOUNT_INITIAL_AMOUNT)
        .unwrap();
    Ok(())
}

pub fn init_alice(session: &mut Session<MinimalSandbox>) -> Result<AccountId32> {
    let res = AccountId32::new([2u8; 32]);
    init_acc_with_balance(session, &res)?;
    Ok(res)
}

pub fn init_bob(session: &mut Session<MinimalSandbox>) -> Result<AccountId32> {
    let res = AccountId32::new([3u8; 32]);
    init_acc_with_balance(session, &res)?;
    Ok(res)
}
