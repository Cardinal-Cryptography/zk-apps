use crate::{
    account::Account, errors::ZkpError, mocked_user, note::Note, ops::OpPriv, relations::ZkProof,
    traits::Hashable, Scalar, MOCKED_TOKEN, TOKENS_NUMBER,
};

fn supported_tokens() -> [Scalar; TOKENS_NUMBER] {
    let mut tokens: [Scalar; TOKENS_NUMBER] = [0_u128.into(); TOKENS_NUMBER];
    tokens[0] = MOCKED_TOKEN;
    tokens
}

fn create_empty_note_proof(id: Scalar, nullifier: Scalar, trapdoor: Scalar) -> (Scalar, ZkProof) {
    let acc_new = Account::new(supported_tokens());
    let note = Note::new(id, trapdoor, nullifier, acc_new.hash());
    let proof = ZkProof::new(
        id,
        trapdoor,
        nullifier,
        OpPriv {
            user: mocked_user(),
        },
        acc_new,
    );
    (note.hash(), proof)
}

#[test]
fn test_create_note() -> Result<(), ZkpError> {
    let id = 0_u128.into();
    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();
    let (h_new_note, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    proof.verify_creation(h_new_note, supported_tokens())?;
    Ok(())
}

#[test]
fn test_create_note_fails() -> Result<(), ZkpError> {
    let id = 0_u128.into();
    let nullifier = 0_u128.into();
    let trapdoor = 0_u128.into();
    let (_, proof) = create_empty_note_proof(id, nullifier, trapdoor);
    let (h_new_note, _) = create_empty_note_proof(1_u128.into(), nullifier, trapdoor);
    assert_eq!(
        ZkpError::VerificationError,
        proof
            .verify_creation(h_new_note, supported_tokens())
            .unwrap_err()
    );
    Ok(())
}
