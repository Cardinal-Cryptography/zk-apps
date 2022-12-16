use std::path::PathBuf;

use aleph_client::AccountId;
use clap::{Args, Parser, Subcommand, ValueEnum};
use relations::{FrontendTokenAmount, FrontendTokenId};

use crate::DepositId;

#[derive(Clone, Eq, PartialEq, Parser)]
pub(super) struct CliConfig {
    /// Path to the file containing application state.
    #[clap(long, default_value = "~/.shielder-state", value_parser = parsing::parse_path)]
    pub state_file: PathBuf,

    /// Logging configuration.
    #[clap(short = 'l', value_enum, default_value = "text")]
    pub logging_format: LoggingFormat,

    /// Account seed, which is used both for submitting transactions and decrypting `state_file`.
    ///
    /// If not provided, will be prompted.
    #[clap(long)]
    pub seed: Option<String>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub(super) enum Command {
    #[clap(flatten)]
    StateWrite(StateWriteCommand),
    #[clap(flatten)]
    StateRead(StateReadCommand),
    #[clap(flatten)]
    ContractInteraction(ContractInteractionCommand),
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub(super) enum StateWriteCommand {
    SetNode(SetNodeCmd),
    SetContractAddress(SetContractAddressCmd),
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub(super) enum StateReadCommand {
    ShowAssets(ShowAssetsCmd),
    PrintState,
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub(super) enum ContractInteractionCommand {
    Deposit(DepositCmd),
    Withdraw(WithdrawCmd),
    RegisterToken(RegisterTokenCmd),
}

#[derive(Clone, Eq, PartialEq, Debug, ValueEnum)]
pub(super) enum LoggingFormat {
    Text,
    Json,
}

impl ContractInteractionCommand {
    pub fn get_metadata_file(&self) -> PathBuf {
        match self {
            ContractInteractionCommand::Deposit(DepositCmd { metadata_file, .. }) => {
                metadata_file.clone()
            }
            ContractInteractionCommand::Withdraw(WithdrawCmd { metadata_file, .. }) => {
                metadata_file.clone()
            }
            ContractInteractionCommand::RegisterToken(RegisterTokenCmd {
                metadata_file, ..
            }) => metadata_file.clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct SetNodeCmd {
    /// WS endpoint address of the node to connect to.
    pub node: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct SetContractAddressCmd {
    /// Address of the Shielder contract.
    pub address: AccountId,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct ShowAssetsCmd {
    /// Which token type to display. All, if `None`.
    pub token_id: Option<FrontendTokenId>,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct DepositCmd {
    /// Registered token id.
    pub token_id: FrontendTokenId,

    /// Amount of the token to deposit.
    pub amount: FrontendTokenAmount,

    /// Contract metadata file.
    #[clap(default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,

    /// raw pk bytes file.
    #[clap(default_value = "deposit.pk.bytes", value_parser = parsing::parse_path)]
    pub proving_key_file: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct WithdrawCmd {
    /// Which note should be spent.
    #[clap(long, required_unless_present("interactive"))]
    pub deposit_id: Option<DepositId>,

    /// How many tokens should be withdrawn.
    #[clap(long, required_unless_present("interactive"))]
    pub amount: Option<FrontendTokenAmount>,

    /// Perform action interactively.
    #[clap(short, conflicts_with_all(["deposit_id", "amount"]))]
    pub interactive: bool,

    /// The destination account. If `None`, the tokens will be transferred to the main seed account.
    #[clap(long)]
    pub recipient: Option<AccountId>,

    /// Seed for submitting the transaction. If `None`, the main seed is used.
    #[clap(long)]
    pub caller_seed: Option<String>,

    /// Fee for the caller.
    #[clap(long)]
    pub fee: Option<FrontendTokenAmount>,

    /// Contract metadata file.
    #[clap(long, default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,

    /// raw pk bytes file.
    #[clap(default_value = "withdraw.pk.bytes", value_parser = parsing::parse_path)]
    pub proving_key_file: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub(super) struct RegisterTokenCmd {
    /// Token ID to register this particular token contract under.
    #[clap(long)]
    pub token_id: u16,

    /// Address where the token contract can be found.
    #[clap(long)]
    pub token_address: AccountId,

    /// Contract metadata file.
    #[clap(long, default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,
}

mod parsing {
    use std::{path::PathBuf, str::FromStr};

    use anyhow::{anyhow, Result};

    pub fn parse_path(path: &str) -> Result<PathBuf> {
        let expanded_path =
            shellexpand::full(path).map_err(|e| anyhow!("Failed to expand path: {e:?}"))?;
        PathBuf::from_str(expanded_path.as_ref())
            .map_err(|e| anyhow!("Failed to interpret path: {e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CliConfig::command().debug_assert()
    }
}
