mod op_pub;
mod scalar;

use ink::storage::Mapping;

pub type Set<T> = Mapping<T, ()>;
pub type Scalar = scalar::Scalar;
pub type OpPub = op_pub::OpPub;
