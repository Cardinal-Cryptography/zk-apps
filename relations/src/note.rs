use halo2_base::{utils::ScalarField, AssignedValue, Context};

/// Represents a note in a shielder.
#[derive(Clone, Copy, Debug)]
pub struct Note<F: ScalarField> {
    pub id: F,
    pub trapdoor: F,
    pub nullifier: F,
    pub account_hash: F,
}

impl<F: ScalarField> Note<F> {
    /// Creates a new Note instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the note.
    /// * `trapdoor` - The trapdoor associated with the note.
    /// * `nullifier` - The nullifier of the note.
    /// * `account_hash` - The account hash associated with the note.
    ///
    /// # Returns
    ///
    /// A new Note instance.
    pub fn new(id: F, trapdoor: F, nullifier: F, account_hash: F) -> Self {
        Self {
            id,
            trapdoor,
            nullifier,
            account_hash,
        }
    }

    /// Converts the Note instance to an array of elements from the field.
    ///
    /// # Returns
    ///
    /// An array containing the ID, trapdoor, nullifier, and account hash of the note.
    pub fn to_array(&self) -> [F; 4] {
        [self.id, self.trapdoor, self.nullifier, self.account_hash]
    }

    pub fn load(&self, ctx: &mut Context<F>) -> CircuitNote<F> {
        CircuitNote {
            id: ctx.load_witness(self.id),
            trapdoor: ctx.load_witness(self.trapdoor),
            nullifier: ctx.load_witness(self.nullifier),
            account_hash: ctx.load_witness(self.account_hash),
        }
    }
}

/// Represents a note in a shielder's circuit.
#[derive(Clone, Copy, Debug)]
pub struct CircuitNote<F: ScalarField> {
    pub id: AssignedValue<F>,
    pub trapdoor: AssignedValue<F>,
    pub nullifier: AssignedValue<F>,
    pub account_hash: AssignedValue<F>,
}

impl<F: ScalarField> CircuitNote<F> {
    pub fn to_array(&self) -> [AssignedValue<F>; 4] {
        [self.id, self.trapdoor, self.nullifier, self.account_hash]
    }
}
