//! Gadget and chips for the Poseidon algebraic hash function.

use std::fmt;

use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::Error,
};

mod pow5t3;
pub use pow5t3::{Pow5T3Chip, Pow5T3Config};

use crate::primitives::poseidon::{Domain, Spec, SpongeState, State};

/// The set of circuit instructions required to use the Poseidon permutation.
pub trait PoseidonInstructions<F: FieldExt, S: Spec<F, T, RATE>, const T: usize, const RATE: usize>:
    Chip<F>
{
    /// Variable representing the word over which the Poseidon permutation operates.
    type Word: fmt::Debug;

    /// Applies the Poseidon permutation to the given state.
    fn permute(
        &self,
        layouter: &mut impl Layouter<F>,
        initial_state: &State<Self::Word, T>,
    ) -> Result<State<Self::Word, T>, Error>;
}

/// The set of circuit instructions required to use the [`Duplex`] and [`Hash`] gadgets.
///
/// [`Hash`]: self::Hash
pub trait PoseidonDuplexInstructions<
    F: FieldExt,
    S: Spec<F, T, RATE>,
    const T: usize,
    const RATE: usize,
>: PoseidonInstructions<F, S, T, RATE>
{
    /// Returns the initial empty state for the given domain.
    fn initial_state(
        &self,
        layouter: &mut impl Layouter<F>,
        domain: &impl Domain<F, S, T, RATE>,
    ) -> Result<State<Self::Word, T>, Error>;

    /// Pads the given input (according to the specified domain) and adds it to the state.
    fn pad_and_add(
        &self,
        layouter: &mut impl Layouter<F>,
        domain: &impl Domain<F, S, T, RATE>,
        initial_state: &State<Self::Word, T>,
        input: &SpongeState<Self::Word, RATE>,
    ) -> Result<State<Self::Word, T>, Error>;

    /// Extracts sponge output from the given state.
    fn get_output(state: &State<Self::Word, T>) -> SpongeState<Self::Word, RATE>;
}
