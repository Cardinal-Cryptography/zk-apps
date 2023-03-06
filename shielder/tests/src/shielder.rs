use aleph_client::{
    contract::ContractInstance, AccountId, ConnectionApi, SignedConnection, TxInfo,
};
use anyhow::Result;
use scale::{Decode, Encode};
use shielder::{MerkleHash, MerkleRoot, Note, Nullifier, TokenAmount, TokenId};

pub type MerklePath = Vec<MerkleHash>;

#[derive(Encode)]
pub enum Relation {
    Deposit,
    DepositAndMerge,
    Withdraw,
}

#[derive(Debug)]
pub struct Shielder {
    instance: ContractInstance,
}

impl Shielder {
    pub fn new(address: AccountId, metadata_path: &str) -> Result<Self> {
        Ok(Shielder {
            instance: ContractInstance::new(address, metadata_path)?,
        })
    }

    pub async fn deposit(
        &self,
        conn: &SignedConnection,
        token_id: TokenId,
        value: TokenAmount,
        note: Note,
        proof: Vec<u8>,
    ) -> Result<TxInfo> {
        let args = [
            &*token_id.to_string(),
            &*value.to_string(),
            &*format!("0x{}", hex::encode(Encode::encode(&note))),
            &*format!("0x{}", hex::encode(proof)),
        ];

        self.instance.contract_exec(conn, "deposit", &args).await
    }

    pub async fn withdraw(
        &self,
        conn: &SignedConnection,
        token_id: TokenId,
        value: TokenAmount,
        recipient: AccountId,
        fee_for_caller: Option<TokenAmount>,
        merkle_root: MerkleRoot,
        nullifier: Nullifier,
        new_note: Note,
        proof: Vec<u8>,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec(
                conn,
                "withdraw",
                &[
                    token_id.to_string(),
                    value.to_string(),
                    recipient.to_string(),
                    hex::encode(Encode::encode(&fee_for_caller)),
                    hex::encode(Encode::encode(&merkle_root)),
                    hex::encode(Encode::encode(&nullifier)),
                    hex::encode(Encode::encode(&new_note)),
                    hex::encode(proof),
                ],
            )
            .await
    }

    pub async fn current_merkle_root<C: ConnectionApi>(&self, conn: &C) -> Result<MerkleRoot> {
        self.instance
            .contract_read0(conn, "current_merkle_root")
            .await
    }

    pub async fn merkle_path<C: ConnectionApi>(&self, conn: &C) -> Result<Option<MerklePath>> {
        self.instance.contract_read0(conn, "merkle_path").await
    }

    pub async fn contains_nullifier<C: ConnectionApi>(
        &self,
        conn: &C,
        nullifier: Nullifier,
    ) -> Result<bool> {
        self.instance
            .contract_read(conn, "contains_nullifier", &[nullifier.to_string()])
            .await
    }

    pub async fn register_vk(
        &self,
        conn: &SignedConnection,
        relation: Relation,
        vk: Vec<u8>,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec(
                conn,
                "register_vk",
                &[
                    hex::encode(Encode::encode(&relation)),
                    format!("0x{}", hex::encode(vk)),
                ],
            )
            .await
    }

    pub async fn registered_token_address<C: ConnectionApi>(
        &self,
        conn: &C,
        token_id: TokenId,
    ) -> Result<Option<AccountId>> {
        self.instance
            .contract_read(conn, "registered_token_address", &[token_id.to_string()])
            .await
    }

    pub async fn register_new_token(
        &self,
        conn: &SignedConnection,
        token_id: TokenId,
        token_address: AccountId,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec(
                conn,
                "register_new_token",
                &[token_id.to_string(), token_address.to_string()],
            )
            .await
    }

    pub async fn deposit_and_merge(
        &self,
        conn: &SignedConnection,
        token_id: TokenId,
        token_amount: TokenAmount,
        merkle_root: MerkleRoot,
        old_nullifier: Nullifier,
        new_note: Note,
        proof: Vec<u8>,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec(
                conn,
                "deposit_and_merge",
                &[
                    token_id.to_string(),
                    token_amount.to_string(),
                    format!("0x{}", hex::encode(Encode::encode(&merkle_root))),
                    old_nullifier.to_string(),
                    format!("0x{}", hex::encode(Encode::encode(&new_note))),
                    format!("0x{}", hex::encode(proof)),
                ],
            )
            .await
    }
}
