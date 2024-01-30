//! Smart contract implementing shielder specification
//! https://docs.alephzero.org/aleph-zero/shielder/introduction-informal

#![cfg_attr(not(feature = "std"), no_std, no_main)]
// #![deny(missing_docs)]

mod errors;
mod merkle;
pub mod mocked_zk;
mod traits;
mod types;

/// Contract module
#[ink::contract]
pub mod contract {

    use crate::{
        errors::ShielderError,
        merkle::MerkleTree,
        mocked_zk::relations::ZkProof,
        traits::psp22::PSP22,
        types::{Scalar, Set},
    };

    /// Enum
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Debug, Clone, Copy)]
    pub enum OpPub {
        /// Deposit PSP-22 token
        Deposit {
            /// amount of deposit
            amount: u128,
            /// PSP-22 token address
            token: Scalar,
            /// User address, from whom tokens are transferred
            user: Scalar,
        },
        /// Withdraw PSP-22 token
        Withdraw {
            /// amount of withdrawal
            amount: u128,
            /// PSP-22 token address
            token: Scalar,
            /// User address, from whom tokens are transferred
            user: Scalar,
        },
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        nullifier_set: Set<Scalar>,
        notes: MerkleTree,
    }

    impl Contract {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        /// Adds empty note to shielder storage
        /// Registers new account with empty balance
        #[ink(message)]
        pub fn add_note(
            &mut self,
            h_note_new: Scalar,
            proof: ZkProof,
        ) -> Result<(), ShielderError> {
            proof.verify_creation(h_note_new)?;
            self.notes.add_leaf(h_note_new)?;
            Ok(())
        }

        /// Updates existing note
        /// Applies operation to private account stored in shielder
        #[ink(message)]
        pub fn update_note(
            &mut self,
            op_pub: OpPub,
            h_note_new: Scalar,
            merkle_root: Scalar,
            nullifier_old: Scalar,
            proof: ZkProof,
        ) -> Result<(), ShielderError> {
            self.notes.is_historical_root(merkle_root)?;
            self.nullify(nullifier_old)?;
            proof.verify_update(op_pub, h_note_new, merkle_root, nullifier_old)?;
            self.notes.add_leaf(h_note_new)?;
            self.process_operation(op_pub)?;
            Ok(())
        }

        fn process_operation(&mut self, op_pub: OpPub) -> Result<(), ShielderError> {
            match op_pub {
                OpPub::Deposit {
                    amount,
                    token,
                    user,
                } => {
                    let mut psp22: ink::contract_ref!(PSP22) = AccountId::from(token.bytes).into();
                    psp22.transfer_from(
                        AccountId::from(user.bytes),
                        self.env().account_id(),
                        amount,
                        [].to_vec()
                    )?;
                }
                OpPub::Withdraw {
                    amount,
                    token,
                    user,
                } => {
                    let mut psp22: ink::contract_ref!(PSP22) = AccountId::from(token.bytes).into();
                    psp22.transfer(
                        AccountId::from(user.bytes),
                        amount,
                        [].to_vec()
                    )?;
                }
            };
            Ok(())
        }

        fn nullify(&mut self, nullifier: Scalar) -> Result<(), ShielderError> {
            self.nullifier_set
                .insert(nullifier, &())
                .map(|_| {})
                .map_or(Ok(()), |_| Err(ShielderError::NullifierIsInSet))
        }
    }
}

#[cfg(test)]
mod tests {
    use drink::{
        contract_api::decode_debug_buffer,
        runtime::MinimalRuntime,
        session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
        ContractBundle,
        AccountId32,
    };

    use crate::{
        mocked_zk::{
            account::Account, 
            note::Note, 
            ops::{OpPriv, Operation}, 
            relations::ZkProof, 
            traits::Hashable,
            tests::merkle::MerkleTree
        },
        types::Scalar,
        contract::OpPub,
    };
    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    fn get_psp22_balance(
        session: &mut Session<MinimalRuntime>,
        token: AccountId32,
        address: AccountId32
    ) -> Result<u128, Box<dyn std::error::Error>> {
        let res: u128 = session.call_with_address(
            token.clone(),
            "PSP22::balance_of",
            &[&*address.to_string()],
            NO_ENDOWMENT
        )??;
        Ok(res)
    }

    fn get_psp22_allowance(
        session: &mut Session<MinimalRuntime>,
        token: AccountId32,
        from: AccountId32,
        to: AccountId32
    ) -> Result<u128, Box<dyn std::error::Error>> {
        let res: u128 = session.call_with_address(
            token.clone(),
            "PSP22::allowance",
            &[&*from.to_string(), &*to.to_string()],
            NO_ENDOWMENT
        )??;
        Ok(res)
    }

    struct ShielderUserEnv {
        id: Scalar,
        proof: ZkProof,
        nullifier: Scalar,
        tree_leaf_id: u32,
    }

    fn create_shielder_account(
        session: &mut Session<MinimalRuntime>,
        shielder_address: AccountId32,
        token: AccountId32,
        merkle_tree: &mut MerkleTree
    ) -> Result<ShielderUserEnv, Box<dyn std::error::Error>> {
        let acc = Account::new(Scalar{bytes: *(token.as_ref())});
        let id = 0_128.into();
        let nullifier = 0_u128.into();
        let trapdoor = 0_u128.into();

        let proof = ZkProof::new(
            id,
            trapdoor,
            nullifier,
            OpPriv {
                user: 0_u128.into(),
            },
            acc,
        );

        let h_note_new = Note::new(
            id,
            trapdoor,
            nullifier,
            acc.hash()
        ).hash();
        
        session.call_with_address(
            shielder_address.clone(),
            "add_note",
            &[
                format!("{:?}", h_note_new),
                format!("{:?}", proof),
            ],
            NO_ENDOWMENT
        )??;
        
        merkle_tree.add_leaf(h_note_new).unwrap();

        Ok(ShielderUserEnv{
            id,
            proof,
            nullifier,
            tree_leaf_id: 0
        })
    }

    struct UpdateOperation {
        op_pub: OpPub,
        op_priv: OpPriv,
    }

    fn shielder_update(
        session: &mut Session<MinimalRuntime>,
        shielder_address: AccountId32,
        upd_op: UpdateOperation,
        user_shielded_data: ShielderUserEnv,
        merkle_tree: &mut MerkleTree
    ) -> Result<ShielderUserEnv, Box<dyn std::error::Error>> {
        let merkle_root = merkle_tree.root();
        let merkle_proof = merkle_tree.gen_proof(
            user_shielded_data.tree_leaf_id as usize
        ).unwrap();
        let nullifier_new = (u128::from(user_shielded_data.nullifier)+1).into();
        let trapdoor_new = 1_u128.into();
    
        let op_pub = upd_op.op_pub;
        let op_priv = upd_op.op_priv;
        let operation = Operation::combine(op_pub, op_priv).unwrap();
        let acc_updated = user_shielded_data.proof.update_account(operation).unwrap();
        let note = Note::new(user_shielded_data.id, trapdoor_new, nullifier_new, acc_updated.hash());
        let new_proof = user_shielded_data.proof.transition(
            trapdoor_new,
            nullifier_new,
            acc_updated,
            op_priv,
            merkle_proof,
            user_shielded_data.tree_leaf_id,
        );
        merkle_tree.add_leaf(note.hash()).unwrap();

        session.call_with_address(
            shielder_address.clone(),
            "update_note",
            &[
                format!("{:?}", op_pub),
                format!("{:?}", note.hash()),
                format!("{:?}", merkle_root),
                format!("{:?}", user_shielded_data.nullifier),
                format!("{:?}", new_proof),
            ],
            NO_ENDOWMENT
        )??;

        Ok(ShielderUserEnv{
            id: user_shielded_data.id,
            proof: new_proof,
            nullifier: nullifier_new,
            tree_leaf_id: user_shielded_data.tree_leaf_id+1
        })
    }

    #[drink::test]
    fn deploy_single_deposit_single_withdraw() -> Result<(), Box<dyn std::error::Error>> {

        const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
        const BOB: AccountId32 = AccountId32::new([2u8; 32]);

        let mut session = Session::<MinimalRuntime>::new()?;
        session.sandbox().mint_into(BOB, 1000000000000000).unwrap();

        let shielder_address = session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            NO_ARGS,
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let psp22_bundle = ContractBundle::load(std::path::Path::new("../PSP22/target/ink/psp22.contract"))?;

        let psp22_address = session.deploy_bundle(
            psp22_bundle,
            "new",
            &["100", "Some(\"TST\")", "Some(\"TST\")", "9"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let mut merkle_tree = MerkleTree::new();

        // CREATE ACCOUNT
        let user_shielded_data = create_shielder_account(
            &mut session,
            shielder_address.clone(),
            psp22_address.clone(),
            &mut merkle_tree
        )?;

        let alice_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            ALICE.clone()
        )?;
        assert_eq!(alice_psp22_balance, 100);
        let bob_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            BOB.clone()
        )?;
        assert_eq!(bob_psp22_balance, 0);


        // APPROVE TRANSFER
        session.call_with_address(
            psp22_address.clone(),
            "PSP22::approve",
            &[&*shielder_address.to_string(), "10"],
            NO_ENDOWMENT
        )??;

        let alice_shielder_allowance: u128 = get_psp22_allowance(
            &mut session, 
            psp22_address.clone(), 
            ALICE.clone(),
            shielder_address.clone()
        )?;
        assert_eq!(alice_shielder_allowance, 10);

        // DEPOSIT
        let user_shielded_data = shielder_update(
            &mut session,
            shielder_address.clone(),
            UpdateOperation {
                op_pub: OpPub::Deposit {
                    amount: 10,
                    token: Scalar{bytes: *(psp22_address.as_ref())},
                    user: Scalar{bytes: *(ALICE.as_ref())},
                },
                op_priv: OpPriv {
                    user: Scalar{bytes: *(ALICE.as_ref())},
                }
            },
            user_shielded_data,
            &mut merkle_tree
        )?;
        
        let alice_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            ALICE.clone()
        )?;
        assert_eq!(alice_psp22_balance, 90);
        let shielder_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            shielder_address.clone()
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
                    token: Scalar{bytes: *(psp22_address.as_ref())},
                    user: Scalar{bytes: *(BOB.as_ref())},
                },
                op_priv: OpPriv {
                    user: Scalar{bytes: *(BOB.as_ref())},
                }
            },
            user_shielded_data,
            &mut merkle_tree
        )?;
        
        let bob_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            BOB.clone()
        )?;
        assert_eq!(bob_psp22_balance, 1);
        let shielder_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            shielder_address.clone()
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
            let acc = AccountId32::new([i as u8;32]);
            withdrawers.push(acc.clone());
            session.sandbox().mint_into(acc, 1000000000000000).unwrap();
        }

        let shielder_address = session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            NO_ARGS,
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let psp22_bundle = ContractBundle::load(std::path::Path::new("../PSP22/target/ink/psp22.contract"))?;

        let psp22_address = session.deploy_bundle(
            psp22_bundle,
            "new",
            &["100", "Some(\"TST\")", "Some(\"TST\")", "9"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let mut merkle_tree = MerkleTree::new();

        // CREATE ACCOUNT
        let mut user_shielded_data = create_shielder_account(
            &mut session,
            shielder_address.clone(),
            psp22_address.clone(),
            &mut merkle_tree
        )?;

        // APPROVE TRANSFER
        session.call_with_address(
            psp22_address.clone(),
            "PSP22::approve",
            &[&*shielder_address.to_string(), "50"],
            NO_ENDOWMENT
        )??;

        let alice_shielder_allowance: u128 = get_psp22_allowance(
            &mut session, 
            psp22_address.clone(), 
            ALICE.clone(),
            shielder_address.clone()
        )?;
        assert_eq!(alice_shielder_allowance, 50);

        // DEPOSIT
        user_shielded_data = shielder_update(
            &mut session,
            shielder_address.clone(),
            UpdateOperation {
                op_pub: OpPub::Deposit {
                    amount: 50,
                    token: Scalar{bytes: *(psp22_address.as_ref())},
                    user: Scalar{bytes: *(ALICE.as_ref())},
                },
                op_priv: OpPriv {
                    user: Scalar{bytes: *(ALICE.as_ref())},
                }
            },
            user_shielded_data,
            &mut merkle_tree
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
                        token: Scalar{bytes: *(psp22_address.as_ref())},
                        user: Scalar{bytes: *(withrawer_addr.as_ref())},
                    },
                    op_priv: OpPriv {
                        user: Scalar{bytes: *(withrawer_addr.as_ref())},
                    }
                },
                user_shielded_data,
                &mut merkle_tree
            )?;
            let psp22_balance: u128 = get_psp22_balance(
                &mut session,
                psp22_address.clone(),
                withrawer_addr.clone()
            )?;
            assert_eq!(psp22_balance, 1);
        }
        let shielder_psp22_balance: u128 = get_psp22_balance(
            &mut session,
            psp22_address.clone(),
            shielder_address.clone()
        )?;
        assert_eq!(shielder_psp22_balance, 42);

        Ok(())
    }

}
