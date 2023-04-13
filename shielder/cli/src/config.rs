use std::path::PathBuf;

use aleph_client::AccountId;
use clap::{Args, Parser, Subcommand, ValueEnum};
use liminal_ark_relations::shielder::types::{FrontendTokenAmount, FrontendTokenId};
use shielder::DepositId;

#[derive(Clone, Eq, PartialEq, Parser)]
pub struct CliConfig {
    /// Path to the file containing application state.
    #[clap(long, default_value = "~/.shielder-state", value_parser = parsing::parse_path)]
    pub state_file: PathBuf,

    /// Logging configuration.
    #[clap(short = 'l', value_enum, default_value = "text")]
    pub logging_format: LoggingFormat,

    /// Password for `state_file` encryption and decryption.
    ///
    /// If not provided, will be prompted.
    #[clap(long)]
    pub password: Option<String>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub enum Command {
    #[clap(flatten)]
    StateWrite(StateWriteCommand),
    #[clap(flatten)]
    StateRead(StateReadCommand),
    #[clap(flatten)]
    ContractInteraction(ContractInteractionCommand),
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub enum StateWriteCommand {
    /// Set WS address of the node that we will be connecting to.
    SetNode {
        /// WS endpoint address of the node to connect to.
        node: String,
    },
    /// Set address of the Shielder contract.
    SetContractAddress {
        /// Address of the Shielder contract.
        address: AccountId,
    },
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub enum StateReadCommand {
    /// Display all available deposits.
    ShowAssets {
        /// Which token type to display. All, if `None`.
        token_id: Option<FrontendTokenId>,
    },
    /// Display full `state_file` content.
    PrintState,
}

#[derive(Clone, Eq, PartialEq, Debug, Subcommand)]
pub enum ContractInteractionCommand {
    /// Shield some tokens.
    Deposit(DepositCmd),
    /// Unshield some tokens.
    Withdraw(WithdrawCmd),
    /// Merge two tokens.
    Merge(MergeCmd),
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
            ContractInteractionCommand::Merge(MergeCmd { metadata_file, .. }) => {
                metadata_file.clone()
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub struct DepositCmd {
    /// Token id (must already be registered in the contract).
    pub token_id: FrontendTokenId,

    /// Amount of the token to be shielded.
    pub amount: FrontendTokenAmount,

    /// When provided, a new deposit is to be created even if a previous one exists.
    #[clap(long, action)]
    pub require_new_deposit: bool,

    /// Seed for submitting the transaction.
    ///
    /// If not provided, will be prompted.
    #[clap(long)]
    pub caller_seed: Option<String>,

    /// File with contract metadata.
    #[clap(default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,

    /// File with raw proving key bytes for first deposit.
    ///
    /// If not found, command will fail - the tool won't generate it for you.
    #[clap(default_value = "deposit.pk.bytes", value_parser = parsing::parse_path)]
    pub deposit_key_file: PathBuf,

    /// File with raw proving key bytes for subsequent deposits.
    ///
    /// If not found, command will fail - the tool won't generate it for you.
    #[clap(default_value = "deposit_and_merge.pk.bytes", value_parser = parsing::parse_path)]
    pub deposit_and_merge_key_file: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub struct WithdrawCmd {
    /// Which note should be spent, last created if none provided.
    #[clap(long, required_unless_present("interactive"))]
    pub deposit_id: Option<DepositId>,

    /// How many tokens should be withdrawn.
    #[clap(long, required_unless_present("interactive"))]
    pub amount: Option<FrontendTokenAmount>,

    /// Perform action interactively.
    #[clap(short, conflicts_with_all(["deposit_id", "amount"]))]
    pub interactive: bool,

    /// The destination account. If `None`, the tokens will be transferred to the caller seed account.
    #[clap(long)]
    pub recipient: Option<AccountId>,

    /// Seed for submitting the transaction.
    ///
    /// If not provided, will be prompted.
    #[clap(long)]
    pub caller_seed: Option<String>,

    /// Fee for the caller. Zero, if not provided.
    #[clap(long, default_value = "0")]
    pub fee: FrontendTokenAmount,

    /// File with contract metadata.
    #[clap(long, default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,

    /// File with raw proving key bytes.
    ///
    /// If not found, command will fail - the tool won't generate it for you.
    #[clap(default_value = "withdraw.pk.bytes", value_parser = parsing::parse_path)]
    pub proving_key_file: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Debug, Args)]
pub struct MergeCmd {
    /// First of the notes that should be spent. The merged amount will be stored under the leaf
    /// index of the first deposit. The second deposit will be deleted.
    pub first_deposit_id: DepositId,

    /// Second of the notes that should be spent.
    pub second_deposit_id: DepositId,

    /// Seed for submitting the transaction.
    ///
    /// If not provided, will be prompted.
    #[clap(long)]
    pub caller_seed: Option<String>,

    /// File with contract metadata.
    #[clap(long, default_value = "shielder-metadata.json", value_parser = parsing::parse_path)]
    pub metadata_file: PathBuf,

    /// File with raw proving key bytes for the merging of deposits.
    ///
    /// If not found, command will fail - the tool won't generate it for you.
    #[clap(default_value = "merge.pk.bytes", value_parser = parsing::parse_path)]
    pub proving_key_file: PathBuf,
}

#[derive(Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum LoggingFormat {
    Text,
    Json,
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
