use std::{fs::File, path::Path};

use aleph_client::{AccountId, Connection, KeyPair, SignedConnection};
use anyhow::Result;
use liminal_ark_relations::shielder::types::{FrontendTokenAmount, FrontendTokenId};
use serde::Deserialize;
use shielder::{
    app_state::{AppState, Deposit},
    deposit, withdraw, DepositId,
};

use crate::{psp22::PSP22Token, shielder_wrapper::ShielderWrapper};

// Token ID under which we've registered a PSP22 token in the setup phase.
pub const TOKEN_A_ID: u16 = 0;

#[derive(Debug, Deserialize)]
pub struct Addresses {
    shielder_address: AccountId,
    token_a_address: AccountId,
    token_b_address: AccountId,
}

pub struct User {
    pub account_id: AccountId,
    pub app_state: AppState,
    pub conn: SignedConnection,
}

impl User {
    pub fn new(keypair: KeyPair, conn: Connection) -> Self {
        let account_id = keypair.account_id().clone();
        let app_state = AppState::default();
        let conn = SignedConnection::from_connection(conn, keypair.clone());
        Self {
            account_id,
            app_state,
            conn,
        }
    }

    pub fn get_deposit(&self, deposit_id: DepositId) -> Option<Deposit> {
        self.app_state.get_deposit_by_id(deposit_id)
    }

    pub async fn shield(
        &mut self,
        token_id: FrontendTokenId,
        token_amount: FrontendTokenAmount,
        shielder: &ShielderWrapper,
    ) -> Result<DepositId> {
        let deposit_id = deposit::new_deposit(
            token_id,
            token_amount,
            &shielder.deposit_pk_file,
            &self.conn,
            &shielder.instance,
            &mut self.app_state,
        )
        .await?;
        Ok(deposit_id)
    }

    pub async fn unshield(
        &mut self,
        shielder: &ShielderWrapper,
        deposit: Deposit,
        amount: Option<FrontendTokenAmount>,
        fee: u128,
        recipient: Option<AccountId>,
    ) -> Result<()> {
        let withdraw_amount = amount.unwrap_or(deposit.token_amount);
        let recipient = recipient.unwrap_or(self.account_id.clone());
        withdraw::withdraw(
            &shielder.instance,
            &self.conn,
            deposit,
            withdraw_amount,
            &recipient,
            fee,
            &shielder.withdraw_pk_file,
            &mut self.app_state,
        )
        .await
    }
}

#[allow(unused)]
pub struct TestContext {
    pub shielder: ShielderWrapper,
    pub token_a: PSP22Token,
    pub token_b: PSP22Token,
    pub connection: Connection,
    pub sudo: User,
    pub damian: User,
    pub hans: User,
}

impl TestContext {
    pub async fn local() -> Result<Self> {
        init_logger().expect("Logger failed to properly initialized");

        let resources_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("resources");
        let addresses: Addresses =
            serde_json::from_reader(File::open(resources_path.join("addresses.json"))?)?;

        let shielder = ShielderWrapper::new(&addresses.shielder_address, &resources_path)?;

        let token_a = PSP22Token::new(
            addresses.token_a_address,
            resources_path.join("public_token.json").to_str().unwrap(),
        )?;
        let token_b = PSP22Token::new(
            addresses.token_b_address,
            resources_path.join("public_token.json").to_str().unwrap(),
        )?;

        let node_address = option_env!("NODE_WS")
            .unwrap_or_else(|| "ws://127.0.0.1:9944")
            .to_string();

        let connection = Connection::new(&node_address).await;

        let sudo = aleph_client::keypair_from_string("//Alice");
        let damian = aleph_client::keypair_from_string("//0");
        let hans = aleph_client::keypair_from_string("//1");

        Ok(Self {
            shielder,
            token_a,
            token_b,
            connection: connection.clone(),
            sudo: User::new(sudo, connection.clone()),
            damian: User::new(damian, connection.clone()),
            hans: User::new(hans, connection.clone()),
        })
    }
}

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use tracing_subscriber::EnvFilter;

const LOG_CONFIGURATION_ENVVAR: &str = "RUST_LOG";

// Flag determining whether tracing subscriber has been initialized already.
static LOGGER: AtomicBool = AtomicBool::new(false);

// Initialize tracing subscriber (logging).
// Makes sure it's initialized only once, globally. Otherwise subscribtions fails with an error.
pub fn init_logger() -> Result<()> {
    match LOGGER.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
        Ok(true) => {
            // Should not happen: it's only possible if LOGGER=true and we tried to set it to false.
            panic!("[{:?}] Unexpected LOGGER state", thread::current().id())
        }
        Ok(false) => {
            // For LOGGER=false and new=true
            // No logger yet, subscribing a new one
            {}
        }
        Err(true) => {
            // For LOGGER=true
            // There is a logger already, don't create a new one.
            return Ok(());
        }
        Err(false) => {
            // For LOGGER=false and new=true
            panic!("[{:?}] Failed to create a logger", thread::current().id());
        }
    }

    let filter = EnvFilter::new(
        std::env::var(LOG_CONFIGURATION_ENVVAR)
            .as_deref()
            .unwrap_or("warn,shielder_cli=info,integration_tests::tests=debug"),
    );

    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_env_filter(filter);

    subscriber.try_init().unwrap();
    Ok(())
}
