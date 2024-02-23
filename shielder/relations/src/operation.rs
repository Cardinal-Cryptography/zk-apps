use halo2_base::{utils::BigPrimeField, AssignedValue};

pub trait Operation<F>
where
    Self: Sized,
    F: BigPrimeField,
{
    type OpPriv: Into<Vec<F>>;
    type OpPub: Into<Vec<F>>;

    fn combine(op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self>;
}

pub trait CircuitOperation<F>
where
    Self: Sized,
    F: BigPrimeField,
{
    type OpPriv: From<Vec<AssignedValue<F>>>;
    type OpPub: From<Vec<AssignedValue<F>>> + Into<Vec<AssignedValue<F>>> + Clone;

    fn combine(op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self>;
}
