//! Gadget and chips for the Poseidon algebraic hash function.

use std::fmt;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::Error,
};

mod pow5t3;
pub use pow5t3::{Pow5T3Chip, Pow5T3Config};

/// The set of circuit instructions required to use the [`Poseidon`] gadget.
pub trait PoseidonInstructions<F: FieldExt>: Chip<F> {
    /// Variable representing the state over which the Poseidon permutation operates.
    type State: fmt::Debug;

    /// Applies the Poseidon permutation to the given state.
    fn permute(
        &self,
        layouter: &mut impl Layouter<F>,
        initial_state: &Self::State,
    ) -> Result<Self::State, Error>;
}
