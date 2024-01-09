use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use aleph_client::AccountId;
use itertools::Itertools;
use liminal_ark_relations::shielder::types::{
    FrontendNote, FrontendNullifier, FrontendTokenAmount, FrontendTokenId, FrontendTrapdoor,
};
use serde::{Deserialize, Serialize};

use crate::DepositId;

/// Full information about a single deposit.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Deposit {
    pub deposit_id: DepositId,
    pub token_id: FrontendTokenId,
    pub token_amount: FrontendTokenAmount,
    pub leaf_idx: u32,
    pub trapdoor: FrontendTrapdoor,
    pub nullifier: FrontendNullifier,
    pub note: FrontendNote,
}

impl Display for Deposit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ TokenID: {}, Amount: {} }}",
            self.token_id, self.token_amount
        )
    }
}

const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";

/// Deposit data narrowed to the most important part (for the user).
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Asset {
    pub token_id: FrontendTokenId,
    pub token_amount: FrontendTokenAmount,
    pub deposit_id: DepositId,
}

impl PartialOrd<Self> for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self.token_id < other.token_id
            || (self.token_id == other.token_id && self.token_amount > other.token_amount)
        {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl From<&Deposit> for Asset {
    fn from(d: &Deposit) -> Self {
        Asset {
            token_id: d.token_id,
            token_amount: d.token_amount,
            deposit_id: d.deposit_id,
        }
    }
}

/// Application info that is kept locally.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct AppState {
    pub node_address: String,
    pub contract_address: AccountId,

    deposit_counter: DepositId,
    deposits: Vec<Deposit>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            node_address: DEFAULT_NODE_ADDRESS.to_string(),
            contract_address: AccountId::new([0u8; 32]),
            deposit_counter: 0,
            deposits: Default::default(),
        }
    }
}

impl AppState {
    pub fn get_all_assets(&self) -> Vec<Asset> {
        self.deposits.iter().map(Asset::from).sorted().collect()
    }

    pub fn get_single_asset(&self, token_id: FrontendTokenId) -> Vec<Asset> {
        self.deposits
            .iter()
            .filter_map(|d| (token_id == d.token_id).then(|| Asset::from(d)))
            .sorted()
            .collect()
    }

    pub fn add_deposit(
        &mut self,
        token_id: FrontendTokenId,
        token_amount: FrontendTokenAmount,
        trapdoor: FrontendTrapdoor,
        nullifier: FrontendNullifier,
        leaf_idx: u32,
        note: FrontendNote,
    ) -> DepositId {
        let deposit_id = self.deposit_counter;
        self.deposits.push(Deposit {
            deposit_id,
            token_id,
            token_amount,
            leaf_idx,
            trapdoor,
            nullifier,
            note,
        });
        self.deposit_counter += 1;
        deposit_id
    }

    pub fn deposits(&self) -> Vec<Deposit> {
        self.deposits
            .clone()
            .into_iter()
            .sorted_by_key(|d| Asset::from(d))
            .collect()
    }

    pub fn get_deposit_by_id(&self, deposit_id: DepositId) -> Option<Deposit> {
        self.deposits
            .iter()
            .find(|d| d.deposit_id == deposit_id)
            .map(Clone::clone)
    }

    pub fn delete_deposit_by_id(&mut self, deposit_id: DepositId) {
        self.deposits.retain(|d| d.deposit_id != deposit_id)
    }

    pub fn replace_deposit(
        &mut self,
        deposit_id: DepositId,
        token_amount: FrontendTokenAmount,
        trapdoor: FrontendTrapdoor,
        nullifier: FrontendNullifier,
        leaf_idx: u32,
        note: FrontendNote,
    ) {
        for deposit in &mut self.deposits {
            if deposit.deposit_id == deposit_id {
                let new_deposit = Deposit {
                    deposit_id,
                    token_id: deposit.token_id,
                    token_amount,
                    leaf_idx,
                    trapdoor,
                    nullifier,
                    note,
                };
                *deposit = new_deposit;
            }
        }
    }

    pub fn get_last_deposit(&self, token_id: FrontendTokenId) -> Option<Deposit> {
        self.deposits
            .iter()
            .filter(|d| d.token_id == token_id)
            .last()
            .cloned()
    }
}
