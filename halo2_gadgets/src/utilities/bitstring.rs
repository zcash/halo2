//! Bitstring gadget

use std::ops::Range;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter},
    plonk::Error,
};

use super::RangeConstrained;

/// Instructions to constrain and subset a bitstring.
pub trait BitstringInstructions<F: FieldExt> {
    /// Constrains the witnessed field element to be no longer
    /// than `num_bits`.
    fn constrain(
        &self,
        layouter: impl Layouter<F>,
        witnessed: &AssignedCell<F, F>,
        num_bits: usize,
    ) -> Result<RangeConstrained<F, AssignedCell<F, F>>, Error>;

    /// Takes a specified subsequence of the little-endian bit representation of a field element.
    /// The bits are numbered from 0 for the LSB.
    fn extract_bitrange(
        &self,
        layouter: impl Layouter<F>,
        witnessed: &AssignedCell<F, F>,
        range: Range<usize>,
    ) -> Result<RangeConstrained<F, AssignedCell<F, F>>, Error>;
}

/// A Bitstring gadget.
///
/// TODO: Deduplicate with the utilities::RangeConstrained struct.
#[derive(Debug)]
pub struct Bitstring<F: FieldExt, BitstringChip: BitstringInstructions<F>> {
    chip: BitstringChip,
    inner: AssignedCell<F, F>,
}

impl<F: FieldExt, BitstringChip: BitstringInstructions<F>> Bitstring<F, BitstringChip> {
    /// Constructs a Bitstring gadget from a witnessed field element.
    pub fn new(chip: BitstringChip, inner: AssignedCell<F, F>) -> Self {
        Self { chip, inner }
    }

    /// Constrain this bitstring to `num_bits`.
    pub fn constrain(
        &self,
        mut layouter: impl Layouter<F>,
        num_bits: usize,
    ) -> Result<RangeConstrained<F, AssignedCell<F, F>>, Error> {
        self.chip.constrain(layouter, &self.inner, num_bits)
    }

    /// Take a bitrange of this Bitstring.
    pub fn bitrange_of(
        &self,
        mut layouter: impl Layouter<F>,
        bitrange: Range<usize>,
    ) -> Result<RangeConstrained<F, AssignedCell<F, F>>, Error> {
        self.chip.extract_bitrange(layouter, &self.inner, bitrange)
    }
}
