use crate::Scalar;

pub trait Hashable {
    fn hash(&self) -> Scalar;
}
