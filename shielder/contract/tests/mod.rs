use drink::{
    contract_api::decode_debug_buffer,
    runtime::MinimalRuntime,
    session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
};
use ink::{primitives::AccountId, scale::Encode};

use crate::mocked_zk::{account::Account, note::Note, ops::OpPriv, relations::ZkProof, traits::Hashable};

#[drink::contract_bundle_provider]
enum BundleProvider {}

#[drink::test]
fn deploy_and_call_a_contract() -> Result<(), Box<dyn std::error::Error>>  {

    let mut session = Session::<MinimalRuntime>::new()?;
    
    // We can now deploy the contract.
    let _contract_address = session.deploy_bundle(
        // The bundle that we want to deploy.
        BundleProvider::local()?,
        // The constructor that we want to call.
        "new",
        // The constructor arguments (as stringish objects).
        NO_ARGS,
        // Salt for the contract address derivation.
        NO_SALT,
        // Initial endowment (the amount of tokens that we want to transfer to the contract).
        NO_ENDOWMENT,
    )?;

    let proof = ZkProof::new(
        0_u128.into(),
        0_u128.into(),
        0_u128.into(),
        OpPriv { user: AccountId::from([0x0;32]) },
        Account::new()
    );

    let h_note_new = Note::new(
        0_u128.into(),
        0_u128.into(),
        0_u128.into(),
        Account::new().hash()
    ).hash();

    let result = session.call(
        "add_note",
        &[h_note_new.into(), proof.into()],
        NO_ENDOWMENT
    );

    Ok(())
}