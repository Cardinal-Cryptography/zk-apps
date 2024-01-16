use crate::types::Scalar;

pub trait Hashable {
    fn hash(&self) -> Scalar;
}
