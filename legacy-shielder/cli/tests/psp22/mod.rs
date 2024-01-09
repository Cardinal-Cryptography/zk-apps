use aleph_client::{
    contract::ContractInstance, AccountId, Balance, ConnectionApi, SignedConnection, TxInfo,
};
use anyhow::Result;

#[derive(Debug)]
pub struct PSP22Token {
    pub instance: ContractInstance,
}

impl PSP22Token {
    pub fn new(address: AccountId, metadata_path: &str) -> Result<Self> {
        Ok(PSP22Token {
            instance: ContractInstance::new(address, metadata_path)?,
        })
    }

    pub async fn transfer(
        &self,
        conn: &SignedConnection,
        recipient: AccountId,
        value: Balance,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec_value(conn, "PSP22::transfer", &[recipient.to_string()], value)
            .await
    }

    pub async fn transfer_from(
        &self,
        conn: &SignedConnection,
        from: AccountId,
        to: AccountId,
        value: Balance,
    ) -> Result<TxInfo> {
        self.instance
            .contract_exec_value(
                conn,
                "PSP22::transfer_from",
                &[from.to_string(), to.to_string()],
                value,
            )
            .await
    }

    pub async fn balance_of<C: ConnectionApi>(
        &self,
        conn: &C,
        account: &AccountId,
    ) -> Result<Balance> {
        self.instance
            .contract_read(conn, "PSP22::balance_of", &[account.to_string()])
            .await
    }
}
