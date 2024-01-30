use drink::{
    runtime::MinimalRuntime,
    session::{Session, NO_ENDOWMENT, NO_SALT},
    AccountId32, ContractBundle,
};

pub fn deploy_test_token(
    session: &mut Session<MinimalRuntime>,
    supply: u128,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let formatted_supply: String = format!("{}", supply);

    let psp22_bundle =
        ContractBundle::load(std::path::Path::new("../PSP22/target/ink/psp22.contract"))?;

    let res = session.deploy_bundle(
        psp22_bundle,
        "new",
        &[
            formatted_supply.as_str(),
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
    token: AccountId32,
    address: AccountId32,
) -> Result<u128, Box<dyn std::error::Error>> {
    let res: u128 = session.call_with_address(
        token.clone(),
        "PSP22::balance_of",
        &[&*address.to_string()],
        NO_ENDOWMENT,
    )??;
    Ok(res)
}

pub fn get_psp22_allowance(
    session: &mut Session<MinimalRuntime>,
    token: AccountId32,
    from: AccountId32,
    to: AccountId32,
) -> Result<u128, Box<dyn std::error::Error>> {
    let res: u128 = session.call_with_address(
        token.clone(),
        "PSP22::allowance",
        &[&*from.to_string(), &*to.to_string()],
        NO_ENDOWMENT,
    )??;
    Ok(res)
}

pub fn psp22_approve(
    session: &mut Session<MinimalRuntime>,
    token: AccountId32,
    to: AccountId32,
    amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    let formatted_amount: String = format!("{}", amount);
    session.call_with_address(
        token.clone(),
        "PSP22::approve",
        &[&*to.to_string(), formatted_amount.as_str()],
        NO_ENDOWMENT,
    )??;
    Ok(())
}
