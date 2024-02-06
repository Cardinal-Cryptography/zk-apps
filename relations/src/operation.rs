use halo2_base::{utils::BigPrimeField, AssignedValue};

/// A trait representing an operation.
pub trait Operation<F>
where
    Self: Sized,
    F: BigPrimeField,
{
    /// The private data associated with the operation.
    type OpPriv;
    /// The public data associated with the operation.
    type OpPub;

    /// Combines the private and public data to create an instance of the operation.
    ///
    /// # Arguments
    ///
    /// * `op_priv` - The private data of the operation.
    /// * `op_pub` - The public data of the operation.
    ///
    /// # Returns
    ///
    /// An `Option` containing the combined operation, or `None` if the combination fails.
    fn combine(op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self>;
}

pub trait CircuitOperation<F>
where
    Self: Sized,
    F: BigPrimeField,
{
    /// The private data associated with the operation.
    type OpPriv;
    /// The public data associated with the operation.
    type OpPub: Into<Vec<AssignedValue<F>>> + Clone;

    /// Combines the private and public data to create an instance of the operation.
    ///
    /// # Arguments
    ///
    /// * `op_priv` - The private data of the operation.
    /// * `op_pub` - The public data of the operation.
    ///
    /// # Returns
    ///
    /// An `Option` containing the combined operation, or `None` if the combination fails.
    fn combine(op_priv: Self::OpPriv, op_pub: Self::OpPub) -> Option<Self>;
}
