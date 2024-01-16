use ink::env::hash::{CryptoHash, Sha2x256};

use super::traits::Hashable;
use crate::types::Scalar;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy)]
pub struct Note {
    id: Scalar,
    trapdoor: Scalar,
    nullifier: Scalar,
    account_hash: Scalar,
}

impl Note {
    pub fn new(id: Scalar, trapdoor: Scalar, nullifier: Scalar, account_hash: Scalar) -> Self {
        Self {
            id,
            trapdoor,
            nullifier,
            account_hash,
        }
    }
}

impl Hashable for Note {
    fn hash(&self) -> Scalar {
        let mut res = [0x0; 32];
        Sha2x256::hash(
            [
                self.id.bytes,
                self.trapdoor.bytes,
                self.nullifier.bytes,
                self.account_hash.bytes,
            ]
            .concat()
            .as_slice(),
            &mut res,
        );
        Scalar { bytes: res }
    }
}
