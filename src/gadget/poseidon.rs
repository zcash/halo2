//! Gadget and chips for the Poseidon algebraic hash function.

use std::fmt;
use std::marker::PhantomData;

use crate::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::Error,
};

mod arity3;
mod grain;
mod mds;

pub use arity3::{Arity3Chip, Arity3Config};
use grain::SboxType;

/// A specification for a Poseidon permutation.
pub trait PoseidonSpec<F: FieldExt> {
    /// The arity of this specification.
    fn arity(&self) -> usize;

    /// The number of full rounds for this specification.
    fn full_rounds(&self) -> usize;

    /// The number of partial rounds for this specification.
    fn partial_rounds(&self) -> usize;

    /// Generates `(round_constants, mds, mds^-1)` corresponding to this specification.
    fn constants(&self) -> (Vec<Vec<F>>, Vec<Vec<F>>, Vec<Vec<F>>);
}

/// A little-endian Poseidon specification.
#[derive(Debug)]
pub struct LsbPoseidon<F: FieldExt> {
    sbox: SboxType,
    /// The arity of the Poseidon permutation.
    t: u16,
    /// The number of full rounds.
    r_f: u16,
    /// The number of partial rounds.
    r_p: u16,
    _field: PhantomData<F>,
}

impl<F: FieldExt> LsbPoseidon<F> {
    /// Creates a new Poseidon specification for a field, using the `x^\alpha` S-box.
    pub fn with_pow_sbox(arity: usize, full_rounds: usize, partial_rounds: usize) -> Self {
        LsbPoseidon {
            sbox: SboxType::Pow,
            t: arity as u16,
            r_f: full_rounds as u16,
            r_p: partial_rounds as u16,
            _field: PhantomData::default(),
        }
    }
}

impl<F: FieldExt> PoseidonSpec<F> for LsbPoseidon<F> {
    fn arity(&self) -> usize {
        self.t as usize
    }

    fn full_rounds(&self) -> usize {
        self.r_f as usize
    }

    fn partial_rounds(&self) -> usize {
        self.r_p as usize
    }

    fn constants(&self) -> (Vec<Vec<F>>, Vec<Vec<F>>, Vec<Vec<F>>) {
        let mut grain = grain::Grain::new(self.sbox, self.t, self.r_f, self.r_p);

        let round_constants = (0..(self.r_f + self.r_p))
            .map(|_| (0..self.t).map(|_| grain.next_field_element()).collect())
            .collect();

        let (mds, mds_inv) = mds::generate_mds(&mut grain, self.t as usize);

        (round_constants, mds, mds_inv)
    }
}

/// The set of circuit instructions required to use the [`Poseidon`] gadget.
pub trait PoseidonInstructions: Chip {
    /// Variable representing the state over which the Poseidon permutation operates.
    type State: fmt::Debug;

    /// Applies the Poseidon permutation to the given state.
    fn permute(
        layouter: &mut impl Layouter<Self>,
        initial_state: &Self::State,
    ) -> Result<Self::State, Error>;
}
