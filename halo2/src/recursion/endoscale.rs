//! Gadget for endoscaling.

use ff::PrimeFieldBits;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    plonk::Error,
};
use pasta_curves::arithmetic::CurveAffine;

mod chip;
pub mod primitive;

/// Instructions to map bitstrings to and from endoscalars.
pub trait EndoscaleInstructions<C: CurveAffine>
where
    C::Base: PrimeFieldBits,
{
    /// A bitstring to be used in endoscaling.
    type Bitstring: std::fmt::Debug + Clone;
    /// The maximum number of bits that can be represented by [`Self::Bitstring`].
    const MAX_BITSTRING_LENGTH: usize;

    /// Witnesses a slice of bools as a vector of [`Self::Bitstring`]s.
    fn witness_bitstring(bits: &[bool]) -> Vec<Self::Bitstring>;

    /// Computes commitment (Alg 1) to a variable-length bitstring using the endoscaling
    /// algorithm. Uses a fixed base.
    ///
    /// The bitstring is decomposed into pairs of bits using a running sum outside of
    /// this gadget.
    ///
    /// # Panics
    /// Panics if `NUM_BITS` is larger than `MAX_BITSTRING_LENGTH`.
    /// Panics if `NUM_BITS` is an odd number.
    #[allow(clippy::type_complexity)]
    fn endoscale_fixed_base<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        layouter: L,
        base: C,
        bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    >;

    /// Computes commitment (Alg 1) to a variable-length bitstring using the endoscaling
    /// algorithm. Uses a variable base witnessed elsewhere in the circuit.
    ///
    /// The bitstring is decomposed into pairs of bits using a running sum outside of
    /// this gadget.
    ///
    /// # Panics
    /// Panics if `NUM_BITS` is larger than `MAX_BITSTRING_LENGTH`.
    /// Panics if `NUM_BITS` is an odd number.
    #[allow(clippy::type_complexity)]
    fn endoscale_var_base<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        layouter: L,
        base: (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    >;

    /// Computes endoscalar (Alg 2) mapping to a variable-length bitstring using
    /// the endoscaling algorithm.
    ///
    /// The bitstring is decomposed into windows using a running sum outside of
    /// this gadget.
    ///
    /// # Panics
    /// Panics if `NUM_BITS` is larger than `MAX_BITSTRING_LENGTH`.
    /// Panics if `NUM_BITS` is an odd number.
    fn endoscale_scalar<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        layouter: L,
        bitstring: &Self::Bitstring,
    ) -> Result<AssignedCell<C::Base, C::Base>, Error>;

    /// Check that a witnessed bitstring corresponds to a range of endoscalars
    /// provided as public inputs.
    ///
    /// # Panics
    /// Panics if `NUM_BITS` is larger than `MAX_BITSTRING_LENGTH`.
    /// Panics if `NUM_BITS` is an odd number.
    fn recover_bitstring<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        layouter: L,
        bitstring: &Self::Bitstring,
        pub_input_rows: [usize; NUM_WINDOWS],
    ) -> Result<(), Error>;
}
