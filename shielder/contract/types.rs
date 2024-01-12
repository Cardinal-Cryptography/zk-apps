use ink::storage::Mapping;

pub type Set<T> = Mapping<T, ()>;

#[derive(Default, Clone, Copy, PartialEq, Eq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Scalar {
    pub bytes: [u8; 32],
}

impl From<u128> for Scalar {
    fn from(value: u128) -> Self {
        Self {
            bytes: [value.to_le_bytes(), [0x0; 16]]
                .concat()
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }
}

impl From<Scalar> for u128 {
    fn from(value: Scalar) -> Self {
        u128::from_le_bytes(value.bytes[0..16].try_into().unwrap())
    }
}
