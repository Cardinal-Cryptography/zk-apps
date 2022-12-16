use std::{env, io};

use aleph_client::{keypair_from_string, SignedConnection};
use anyhow::{anyhow, Result};
use clap::Parser;
use config::LoggingFormat;
use inquire::Password;
use tracing::info;
use tracing_subscriber::EnvFilter;
use zeroize::Zeroize;
use ContractInteractionCommand::{Deposit, RegisterToken, Withdraw};
use StateReadCommand::{PrintState, ShowAssets};
use StateWriteCommand::{SetContractAddress, SetNode};

use crate::{
    app_state::AppState,
    config::{
        CliConfig,
        Command::{ContractInteraction, StateRead, StateWrite},
        ContractInteractionCommand, SetContractAddressCmd, SetNodeCmd, ShowAssetsCmd,
        StateReadCommand, StateWriteCommand,
    },
    contract::Shielder,
    deposit::do_deposit,
    state_file::{get_app_state, save_app_state},
    withdraw::do_withdraw,
};

type DepositId = u16;

mod app_state;
mod config;
mod contract;
mod deposit;
mod state_file;
mod withdraw;

fn perform_state_write_action(app_state: &mut AppState, command: StateWriteCommand) -> Result<()> {
    match command {
        SetNode(SetNodeCmd { node }) => {
            app_state.node_address = node;
        }
        SetContractAddress(SetContractAddressCmd { address }) => {
            app_state.contract_address = address;
        }
    };
    Ok(())
}

fn perform_state_read_action(app_state: &mut AppState, command: StateReadCommand) -> Result<()> {
    match command {
        ShowAssets(ShowAssetsCmd { token_id }) => {
            let assets = match token_id {
                None => app_state.get_all_assets(),
                Some(token_id) => app_state.get_single_asset(token_id),
            };
            info!(?assets)
        }
        PrintState => {
            info!(caller_seed=?app_state.caller_seed, 
                node_address=%app_state.node_address, 
                contract_address=%app_state.contract_address,
                deposits=?app_state.deposits())
        }
    };
    Ok(())
}

fn perform_contract_action(
    app_state: &mut AppState,
    command: ContractInteractionCommand,
) -> Result<()> {
    let signer = keypair_from_string(&app_state.caller_seed);
    let connection = SignedConnection::new(&app_state.node_address, signer);

    let metadata_file = command.get_metadata_file();
    let contract = Shielder::new(&app_state.contract_address, &metadata_file)?;

    match command {
        Deposit(cmd) => do_deposit(contract, connection, cmd, app_state)?,
        Withdraw(cmd) => do_withdraw(contract, connection, cmd, app_state)?,
        RegisterToken(cmd) => {
            contract.register_new_token(&connection, cmd.token_id, cmd.token_address)?
        }
    };
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config: CliConfig = CliConfig::parse();

    init_logging(cli_config.logging_format)?;

    let seed = match cli_config.seed {
        Some(seed) => seed,
        _ => Password::new("Password (account seed):")
            .without_confirmation()
            .prompt()?,
    };

    let mut app_state = get_app_state(&cli_config.state_file, &seed)?;
    app_state.caller_seed = seed;

    match cli_config.command {
        StateWrite(cmd) => perform_state_write_action(&mut app_state, cmd)?,
        StateRead(cmd) => perform_state_read_action(&mut app_state, cmd)?,
        ContractInteraction(cmd) => perform_contract_action(&mut app_state, cmd)?,
    }

    save_app_state(&app_state, &cli_config.state_file, &app_state.caller_seed)?;

    app_state.caller_seed.zeroize();
    // `cli_config.seed` and `seed` are already moved

    Ok(())
}

const LOG_CONFIGURATION_ENVVAR: &str = "RUST_LOG";

fn init_logging(format: LoggingFormat) -> Result<()> {
    // We need to disable logging in our dependency crates by default.
    let filter = EnvFilter::new(
        env::var(LOG_CONFIGURATION_ENVVAR)
            .as_deref()
            .unwrap_or("warn,blender_cli=info"),
    );

    match format {
        LoggingFormat::Text => tracing_subscriber::fmt()
            .with_writer(io::stdout)
            .with_target(false)
            .with_env_filter(filter)
            .try_init(),
        LoggingFormat::Json => tracing_subscriber::fmt()
            .with_writer(io::stdout)
            .with_target(false)
            .with_env_filter(filter)
            .json()
            .try_init(),
    }
    .map_err(|err| anyhow!(err))
}
