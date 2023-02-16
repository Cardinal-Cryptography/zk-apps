use std::{env, fs, io, path::PathBuf};

use aleph_client::Connection;
use anyhow::{anyhow, Result};
use ark_serialize::CanonicalDeserialize;
use clap::Parser;
use config::LoggingFormat;
use inquire::Password;
use liminal_ark_relations::{
    serialize, CircuitField, ConstraintSynthesizer, Groth16, ProvingSystem,
};
use tracing::info;
use tracing_subscriber::EnvFilter;
use ContractInteractionCommand::{Deposit, Withdraw};
use StateReadCommand::{PrintState, ShowAssets};
use StateWriteCommand::{SetContractAddress, SetNode};

use crate::{
    app_state::AppState,
    config::{
        CliConfig,
        Command::{ContractInteraction, StateRead, StateWrite},
        ContractInteractionCommand, StateReadCommand, StateWriteCommand,
    },
    contract::Shielder,
    deposit::do_deposit,
    state_file::{get_app_state, save_app_state},
    withdraw::do_withdraw,
};

type DepositId = u16;

const MERKLE_PATH_MAX_LEN: u8 = 16;

mod app_state;
mod config;
mod contract;
mod deposit;
mod state_file;
mod withdraw;

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

fn generate_proof(
    circuit: impl ConstraintSynthesizer<CircuitField>,
    proving_key_file: PathBuf,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;

    Ok(serialize(&Groth16::prove(&pk, circuit)))
}
