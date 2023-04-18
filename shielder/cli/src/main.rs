use std::{env, io};

use aleph_client::{account_from_keypair, keypair_from_string, Connection, SignedConnection};
use anyhow::{anyhow, Result};
use clap::Parser;
use config::{DepositCmd, LoggingFormat, MergeCmd, WithdrawCmd};
use inquire::{CustomType, Password, Select};
use liminal_ark_relations::shielder::types::FrontendTokenAmount;
use shielder::{app_state::AppState, contract::Shielder, deposit::*, merge::*, withdraw::*};
use tracing::info;
use tracing_subscriber::EnvFilter;
use ContractInteractionCommand::{Deposit, Merge, Withdraw};
use StateReadCommand::{PrintState, ShowAssets};
use StateWriteCommand::{SetContractAddress, SetNode};

extern crate shielder;

use crate::{
    config::{
        CliConfig,
        Command::{ContractInteraction, StateRead, StateWrite},
        ContractInteractionCommand, StateReadCommand, StateWriteCommand,
    },
    state_file::{get_app_state, save_app_state},
};

mod config;
mod state_file;

fn perform_state_write_action(app_state: &mut AppState, command: StateWriteCommand) -> Result<()> {
    match command {
        SetNode { node } => {
            app_state.node_address = node;
        }
        SetContractAddress { address } => {
            app_state.contract_address = address;
        }
    };
    Ok(())
}

fn perform_state_read_action(app_state: &AppState, command: StateReadCommand) -> Result<()> {
    match command {
        ShowAssets { token_id } => {
            let assets = match token_id {
                None => app_state.get_all_assets(),
                Some(token_id) => app_state.get_single_asset(token_id),
            };
            info!(?assets)
        }
        PrintState => {
            info!(
                node_address=%app_state.node_address,
                contract_address=%app_state.contract_address,
                deposits=?app_state.deposits()
            )
        }
    };
    Ok(())
}

async fn perform_contract_action(
    app_state: &mut AppState,
    command: ContractInteractionCommand,
) -> Result<()> {
    let connection = Connection::new(&app_state.node_address).await;

    let metadata_file = command.get_metadata_file();
    let contract = Shielder::new(&app_state.contract_address, &metadata_file)?;

    match command {
        Deposit(cmd) => do_deposit(contract, connection, cmd, app_state).await?,
        Withdraw(cmd) => do_withdraw(contract, connection, cmd, app_state).await?,
        Merge(cmd) => do_merge(contract, connection, cmd, app_state).await?,
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config: CliConfig = CliConfig::parse();

    init_logging(cli_config.logging_format)?;

    let password = match cli_config.password {
        Some(password) => password,
        _ => Password::new("Password (for encrypting local state):")
            .without_confirmation()
            .prompt()?,
    };

    let mut app_state = get_app_state(&cli_config.state_file, &password)?;

    match cli_config.command {
        StateWrite(cmd) => {
            perform_state_write_action(&mut app_state, cmd)?;
            save_app_state(&app_state, &cli_config.state_file, &password)?;
        }
        StateRead(cmd) => perform_state_read_action(&app_state, cmd)?,
        ContractInteraction(cmd) => {
            perform_contract_action(&mut app_state, cmd).await?;
            save_app_state(&app_state, &cli_config.state_file, &password)?;
        }
    }

    Ok(())
}

const LOG_CONFIGURATION_ENVVAR: &str = "RUST_LOG";

fn init_logging(format: LoggingFormat) -> Result<()> {
    // We need to disable logging in our dependency crates by default.
    let filter = EnvFilter::new(
        env::var(LOG_CONFIGURATION_ENVVAR)
            .as_deref()
            .unwrap_or("warn,shielder_cli=info"),
    );

    let subscriber = tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_target(false)
        .with_env_filter(filter);

    match format {
        LoggingFormat::Json => subscriber.json().try_init(),
        LoggingFormat::Text => subscriber.try_init(),
    }
    .map_err(|err| anyhow!(err))
}

async fn do_deposit(
    contract: Shielder,
    connection: Connection,
    cmd: DepositCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let DepositCmd {
        token_id,
        amount,
        caller_seed,
        require_new_deposit,
        ..
    } = cmd;

    let seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new("Seed of the depositing account (the tokens owner):")
            .without_confirmation()
            .prompt()?,
    };
    let connection = SignedConnection::from_connection(connection, keypair_from_string(&seed));

    let old_deposit = app_state.get_last_deposit(token_id);

    match (old_deposit, require_new_deposit) {
        (Some(old_deposit), false) => {
            let _ = deposit_and_merge(
                old_deposit,
                amount,
                &cmd.deposit_and_merge_key_file,
                &connection,
                &contract,
                app_state,
            )
            .await?;
            Ok(())
        }
        (_, _) => {
            let _ = new_deposit(
                token_id,
                amount,
                &cmd.deposit_key_file,
                &connection,
                &contract,
                app_state,
            )
            .await?;
            Ok(())
        }
    }
}

async fn do_merge(
    contract: Shielder,
    connection: Connection,
    cmd: MergeCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let MergeCmd {
        first_deposit_id,
        second_deposit_id,
        caller_seed,
        proving_key_file,
        ..
    } = cmd;

    let seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new("Seed of the merging account (the tokens owner):")
            .without_confirmation()
            .prompt()?,
    };
    let connection = SignedConnection::from_connection(connection, keypair_from_string(&seed));

    let first_deposit = app_state
        .get_deposit_by_id(first_deposit_id)
        .ok_or(anyhow!("Cannot match first deposit id to actual deposit!"))?;
    let second_deposit = app_state
        .get_deposit_by_id(second_deposit_id)
        .ok_or(anyhow!("Cannot match second deposit id to actual deposit!"))?;

    anyhow::ensure!(
        first_deposit != second_deposit,
        "Cannot merge a deposit with itself!"
    );

    let first_token_id = first_deposit.token_id;
    let second_token_id = second_deposit.token_id;

    anyhow::ensure!(
        first_token_id == second_token_id,
        "Cannot merge deposits with different token ids!"
    );

    merge(
        first_deposit,
        second_deposit,
        &proving_key_file,
        &connection,
        &contract,
        app_state,
    )
    .await?;

    Ok(())
}

async fn do_withdraw(
    contract: Shielder,
    connection: Connection,
    cmd: WithdrawCmd,
    app_state: &mut AppState,
) -> Result<()> {
    let (deposit, withdraw_amount) = get_deposit_and_withdraw_amount(&cmd, app_state)?;

    let WithdrawCmd {
        recipient,
        caller_seed,
        fee,
        proving_key_file,
        ..
    } = cmd;

    let caller_seed = match caller_seed {
        Some(seed) => seed,
        None => Password::new(
            "Seed of the withdrawing account (the caller, not necessarily recipient):",
        )
        .without_confirmation()
        .prompt()?,
    };

    let signer = keypair_from_string(&caller_seed);
    let recipient = match recipient {
        Some(recipient) => recipient,
        None => account_from_keypair(signer.signer()),
    };

    let connection = SignedConnection::from_connection(connection, signer);

    withdraw(
        &contract,
        &connection,
        deposit,
        withdraw_amount,
        &recipient,
        fee,
        &proving_key_file,
        app_state,
    )
    .await
}

fn get_deposit_and_withdraw_amount(
    cmd: &WithdrawCmd,
    app_state: &AppState,
) -> Result<(shielder::app_state::Deposit, FrontendTokenAmount)> {
    if !cmd.interactive {
        if let Some(deposit) = app_state.get_deposit_by_id(cmd.deposit_id.unwrap()) {
            return Ok((deposit, cmd.amount.unwrap()));
        }
        return Err(anyhow!("Incorrect deposit id"));
    }

    let deposit = Select::new("Select one of your deposits:", app_state.deposits())
        .with_page_size(5)
        .prompt()?;

    let amount =
        CustomType::<FrontendTokenAmount>::new("Specify how many tokens should be withdrawn:")
            .with_default(deposit.token_amount)
            .with_parser(&|a| match str::parse::<FrontendTokenAmount>(a) {
                Ok(amount) if amount <= deposit.token_amount => Ok(amount),
                _ => Err(()),
            })
            .with_error_message(
                "You should provide a valid amount, no more than the whole deposit value",
            )
            .prompt()?;

    Ok((deposit, amount))
}
