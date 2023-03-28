use std::path::{Path, PathBuf};

use aleph_client::AccountId;
use anyhow::Result;
use shielder::contract::Shielder as ShielderContract;

#[allow(unused)]
pub struct ShielderWrapper {
    pub instance: ShielderContract,
    pub deposit_pk_file: PathBuf,
    pub deposit_and_merge_pk_file: PathBuf,
    pub merge_pk_file: PathBuf,
    pub withdraw_pk_file: PathBuf,
}

impl ShielderWrapper {
    pub fn new(shielder_address: &AccountId, resources_path: &Path) -> Result<Self> {
        let shielder =
            ShielderContract::new(shielder_address, &resources_path.join("shielder.json"))?;

        Ok(Self {
            instance: shielder,
            deposit_pk_file: resources_path.join("deposit.pk.bytes"),
            deposit_and_merge_pk_file: resources_path.join("deposit_and_merge.pk.bytes"),
            merge_pk_file: resources_path.join("merge.pk.bytes"),
            withdraw_pk_file: resources_path.join("withdraw.pk.bytes"),
        })
    }
}
