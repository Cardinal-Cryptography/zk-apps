use anyhow::Result;
use drink::{
    runtime::MinimalRuntime,
    session::{Session, NO_ENDOWMENT, NO_SALT},
    AccountId32, ContractBundle,
};

pub fn deploy_test_token(
    session: &mut Session<MinimalRuntime>,
    supply: u128,
) -> Result<AccountId32> {
    let psp22_bundle =
        ContractBundle::load(std::path::Path::new("../PSP22/target/ink/psp22.contract"))?;
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

pub fn get_psp22_balance(
    session: &mut Session<MinimalRuntime>,
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
    session: &mut Session<MinimalRuntime>,
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
    session: &mut Session<MinimalRuntime>,
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
    session: &mut Session<MinimalRuntime>,
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
