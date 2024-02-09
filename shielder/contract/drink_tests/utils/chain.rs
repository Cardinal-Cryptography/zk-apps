use crate::drink_tests::utils::ACCOUNT_INITIAL_AMOUNT;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};

pub fn init_acc_with_balance(
    session: &mut Session<MinimalRuntime>,
    acc: &AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    session
        .sandbox()
        .mint_into(acc.clone(), ACCOUNT_INITIAL_AMOUNT)
        .unwrap();
    Ok(())
}

pub fn init_alice(
    session: &mut Session<MinimalRuntime>,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let res = AccountId32::new([2u8; 32]);
    init_acc_with_balance(session, &res)?;
    Ok(res)
}

pub fn init_bob(
    session: &mut Session<MinimalRuntime>,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let res = AccountId32::new([3u8; 32]);
    init_acc_with_balance(session, &res)?;
    Ok(res)
}
