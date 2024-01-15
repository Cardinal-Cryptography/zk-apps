use ink::storage::Mapping;

pub type Set<T> = Mapping<T, ()>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, scale::Decode, scale::Encode)]
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

#[cfg(test)]
mod tests {
    use crate::types::Scalar;

    #[test]
    fn test_scalar_from_u128() {
        let mut bytes = [0x0; 32];
        bytes[2] = 0x01;
        bytes[1] = 0xE2;
        bytes[0] = 0x40;
        assert_eq!(Scalar::from(123456_u128), Scalar { bytes });
    }

    #[test]
    fn test_u128_from_scalar() {
        let expected = 987654321_u128;
        let mut bytes = [0x0; 32];
        bytes[3] = 0x3A;
        bytes[2] = 0xDE;
        bytes[1] = 0x68;
        bytes[0] = 0xB1;
        assert_eq!(expected, u128::from(Scalar { bytes }));
    }
}
