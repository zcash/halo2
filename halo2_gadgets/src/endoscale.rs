//! Gadget for endoscaling.
use ff::PrimeFieldBits;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Assigned, Error},
};
use pasta_curves::arithmetic::CurveAffine;
use std::fmt::Debug;

pub mod util;

/// Instructions to map bitstrings to and from endoscalars.
pub trait EndoscaleInstructions<C: CurveAffine>
where
    C::Base: PrimeFieldBits,
{
    /// A non-identity point.
    type NonIdentityPoint: Clone + Debug;
    /// A bitstring up to `MAX_BITSTRING_LENGTH` bits.
    type Bitstring: Clone + Debug;
    /// Enumeration of fixed bases used in endoscaling.
    type FixedBases;
    /// The maximum number of bits that can be represented by [`Self::Bitstring`].
    /// When endoscaling with a base, each unique base can only support up to
    /// `MAX_BITSTRING_LENGTH` bits.
    const MAX_BITSTRING_LENGTH: usize;
    /// The number of fixed bases available.
    const NUM_FIXED_BASES: usize;

    /// Witnesses a slice of bools as a vector of [`Self::Bitstring`]s.
    fn witness_bitstring(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bits: &[Value<bool>],
        for_base: bool,
    ) -> Result<Vec<Self::Bitstring>, Error>;

    /// Computes commitment (Alg 1) to a variable-length bitstring using the endoscaling
    /// algorithm. Uses the fixed bases defined in [`Self::FixedBases`].
    ///
    /// # Panics
    /// Panics if bitstring.len() exceeds NUM_FIXED_BASES.
    #[allow(clippy::type_complexity)]
    fn endoscale_fixed_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: Vec<Self::Bitstring>,
        bases: Vec<Self::FixedBases>,
    ) -> Result<Vec<Self::NonIdentityPoint>, Error>;

    /// Computes commitment (Alg 1) to a variable-length bitstring using the endoscaling
    /// algorithm. Uses variable bases witnessed elsewhere in the circuit.
    ///
    /// # Panics
    /// Panics if bitstring.len() exceeds bases.len().
    #[allow(clippy::type_complexity)]
    fn endoscale_var_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: Vec<Self::Bitstring>,
        bases: Vec<Self::NonIdentityPoint>,
    ) -> Result<Vec<Self::NonIdentityPoint>, Error>;

    /// Computes endoscalar (Alg 2) mapping to a variable-length bitstring using
    /// the endoscaling algorithm.
    fn compute_endoscalar(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Self::Bitstring,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error>;

    /// Check that a witnessed bitstring corresponds to a range of endoscalars
    /// provided as public inputs.
    fn constrain_bitstring(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Self::Bitstring,
        pub_input_rows: Vec<usize>,
    ) -> Result<(), Error>;
}
