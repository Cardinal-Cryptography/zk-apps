mod utils;

use drink::{
    runtime::MinimalRuntime,
    session::Session,
    AccountId32,
};

use crate::{
    contract::OpPub,
    drink_tests::utils::{psp22::*, shielder::*},
    mocked_zk::{ops::OpPriv, tests::merkle::MerkleTree},
    types::Scalar,
};

#[drink::contract_bundle_provider]
pub enum BundleProvider {}

#[drink::test]
fn deploy_single_deposit_single_withdraw() -> Result<(), Box<dyn std::error::Error>> {
    const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
    const BOB: AccountId32 = AccountId32::new([2u8; 32]);

    let mut session = Session::<MinimalRuntime>::new()?;
    session.sandbox().mint_into(BOB, 1000000000000000).unwrap();

    let shielder_address = deploy_shielder(&mut session)?;
    let psp22_address = deploy_test_token(&mut session, 100)?;

    let mut merkle_tree = MerkleTree::new();

    // CREATE ACCOUNT
    let user_shielded_data = create_shielder_account(
        &mut session,
        shielder_address.clone(),
        psp22_address.clone(),
        &mut merkle_tree,
    )?;

    // APPROVE TRANSFER
    psp22_approve(
        &mut session,
        psp22_address.clone(),
        shielder_address.clone(),
        10,
    )?;

    // DEPOSIT
    let user_shielded_data = shielder_update(
        &mut session,
        shielder_address.clone(),
        UpdateOperation {
            op_pub: OpPub::Deposit {
                amount: 10,
                token: Scalar {
                    bytes: *(psp22_address.as_ref()),
                },
                user: Scalar {
                    bytes: *(ALICE.as_ref()),
                },
            },
            op_priv: OpPriv {
                user: Scalar {
                    bytes: *(ALICE.as_ref()),
                },
            },
        },
        user_shielded_data,
        &mut merkle_tree,
    )?;

    let alice_psp22_balance: u128 =
        get_psp22_balance(&mut session, psp22_address.clone(), ALICE.clone())?;
    assert_eq!(alice_psp22_balance, 90);
    let shielder_psp22_balance: u128 = get_psp22_balance(
        &mut session,
        psp22_address.clone(),
        shielder_address.clone(),
    )?;
    assert_eq!(shielder_psp22_balance, 10);

    // SWITCH TO BOB
    session = session.with_actor(BOB.clone());

    // WITHDRAW
    let _ = shielder_update(
        &mut session,
        shielder_address.clone(),
        UpdateOperation {
            op_pub: OpPub::Withdraw {
                amount: 1,
                token: Scalar {
                    bytes: *(psp22_address.as_ref()),
                },
                user: Scalar {
                    bytes: *(BOB.as_ref()),
                },
            },
            op_priv: OpPriv {
                user: Scalar {
                    bytes: *(BOB.as_ref()),
                },
            },
        },
        user_shielded_data,
        &mut merkle_tree,
    )?;

    let bob_psp22_balance: u128 =
        get_psp22_balance(&mut session, psp22_address.clone(), BOB.clone())?;
    assert_eq!(bob_psp22_balance, 1);
    let shielder_psp22_balance: u128 = get_psp22_balance(
        &mut session,
        psp22_address.clone(),
        shielder_address.clone(),
    )?;
    assert_eq!(shielder_psp22_balance, 9);

    Ok(())
}

#[drink::test]
fn deploy_single_deposit_multiple_withdraw() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::<MinimalRuntime>::new()?;

    const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
    let mut withdrawers: Vec<AccountId32> = vec![];
    for i in 2..10 {
        let acc = AccountId32::new([i as u8; 32]);
        withdrawers.push(acc.clone());
        session.sandbox().mint_into(acc, 1000000000000000).unwrap();
    }

    let shielder_address = deploy_shielder(&mut session)?;
    let psp22_address = deploy_test_token(&mut session, 100)?;

    let mut merkle_tree = MerkleTree::new();

    // CREATE ACCOUNT
    let mut user_shielded_data = create_shielder_account(
        &mut session,
        shielder_address.clone(),
        psp22_address.clone(),
        &mut merkle_tree,
    )?;

    // APPROVE TRANSFER
    psp22_approve(
        &mut session,
        psp22_address.clone(),
        shielder_address.clone(),
        50,
    )?;

    let alice_shielder_allowance: u128 = get_psp22_allowance(
        &mut session,
        psp22_address.clone(),
        ALICE.clone(),
        shielder_address.clone(),
    )?;
    assert_eq!(alice_shielder_allowance, 50);

    // DEPOSIT
    user_shielded_data = shielder_update(
        &mut session,
        shielder_address.clone(),
        UpdateOperation {
            op_pub: OpPub::Deposit {
                amount: 50,
                token: Scalar {
                    bytes: *(psp22_address.as_ref()),
                },
                user: Scalar {
                    bytes: *(ALICE.as_ref()),
                },
            },
            op_priv: OpPriv {
                user: Scalar {
                    bytes: *(ALICE.as_ref()),
                },
            },
        },
        user_shielded_data,
        &mut merkle_tree,
    )?;

    // SWITCH TO BOB
    for withrawer_addr in withdrawers {
        session = session.with_actor(withrawer_addr.clone());

        // WITHDRAW
        user_shielded_data = shielder_update(
            &mut session,
            shielder_address.clone(),
            UpdateOperation {
                op_pub: OpPub::Withdraw {
                    amount: 1,
                    token: Scalar {
                        bytes: *(psp22_address.as_ref()),
                    },
                    user: Scalar {
                        bytes: *(withrawer_addr.as_ref()),
                    },
                },
                op_priv: OpPriv {
                    user: Scalar {
                        bytes: *(withrawer_addr.as_ref()),
                    },
                },
            },
            user_shielded_data,
            &mut merkle_tree,
        )?;
        let psp22_balance: u128 =
            get_psp22_balance(&mut session, psp22_address.clone(), withrawer_addr.clone())?;
        assert_eq!(psp22_balance, 1);
    }
    let shielder_psp22_balance: u128 = get_psp22_balance(
        &mut session,
        psp22_address.clone(),
        shielder_address.clone(),
    )?;
    assert_eq!(shielder_psp22_balance, 42);

    Ok(())
}
