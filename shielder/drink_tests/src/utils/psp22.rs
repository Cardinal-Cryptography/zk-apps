use crate::tests::BundleProvider;
use anyhow::Result;
use drink::{
    minimal::MinimalSandbox,
    session::{Session, NO_ENDOWMENT, NO_SALT},
    AccountId32,
};

pub fn deploy_test_token(
    session: &mut Session<MinimalSandbox>,
    supply: u128,
) -> Result<AccountId32> {
    let psp22_bundle = BundleProvider::Psp22.bundle()?;
    let res = session.deploy_bundle(
        psp22_bundle,
        "new",
        &[
            format!("{}", supply).as_str(),
            "Some(\"TST\")",
            "Some(\"TST\")",
            "9",
        ],
        NO_SALT,
        NO_ENDOWMENT,
    )?;
    Ok(res)
}

pub fn deploy_azero_test_token(
    session: &mut Session<MinimalSandbox>,
    supply: u128,
) -> Result<AccountId32> {
    let psp22_bundle = BundleProvider::Psp22.bundle()?;
    let res = session.deploy_bundle(
        psp22_bundle,
        "new",
        &[
            format!("{}", supply).as_str(),
            "Some(\"AZERO\")",
            "Some(\"AZERO\")",
            "9",
        ],
        NO_SALT,
        NO_ENDOWMENT,
    )?;
    Ok(res)
}

pub fn get_psp22_balance(
    session: &mut Session<MinimalSandbox>,
    token: &AccountId32,
    address: &AccountId32,
) -> Result<u128> {
    let res = session.call_with_address(
        token.clone(),
        "PSP22::balance_of",
        &[address.to_string()],
        NO_ENDOWMENT,
    )??;
    Ok(res)
}

pub fn get_psp22_allowance(
    session: &mut Session<MinimalSandbox>,
    token: &AccountId32,
    from: &AccountId32,
    to: &AccountId32,
) -> Result<u128> {
    let res = session.call_with_address(
        token.clone(),
        "PSP22::allowance",
        &[from.to_string(), to.to_string()],
        NO_ENDOWMENT,
    )??;
    Ok(res)
}

pub fn psp22_approve(
    session: &mut Session<MinimalSandbox>,
    token: &AccountId32,
    to: &AccountId32,
    amount: u128,
) -> Result<()> {
    session.call_with_address(
        token.clone(),
        "PSP22::approve",
        &[to.to_string(), format!("{}", amount)],
        NO_ENDOWMENT,
    )??;
    Ok(())
}

pub fn psp22_transfer(
    session: &mut Session<MinimalSandbox>,
    token: &AccountId32,
    to: &AccountId32,
    amount: u128,
) -> Result<()> {
    let empty_arr: [u8; 0] = [];
    session.call_with_address(
        token.clone(),
        "PSP22::transfer",
        &[
            to.to_string(),
            format!("{}", amount),
            format!("{:?}", empty_arr),
        ],
        NO_ENDOWMENT,
    )??;
    Ok(())
}
