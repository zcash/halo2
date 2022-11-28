use std::marker::PhantomData;

use halo2_gadgets::{
    ecc::{EccInstructions, Point, ScalarVar},
    utilities::{bitstring::BitstringInstructions, RangeConstrained},
};
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{AssignedCell, Layouter, Value},
    plonk::Error,
};

pub trait DuplexInstructions<F: FieldExt> {
    fn absorb(
        &mut self,
        layouter: impl Layouter<F>,
        value: AssignedCell<F, F>,
    ) -> Result<(), Error>;
    fn squeeze(&mut self, layouter: impl Layouter<F>) -> Result<AssignedCell<F, F>, Error>;
}

pub trait TranscriptInstructions<C: CurveAffine>:
    BitstringInstructions<C::Base> + DuplexInstructions<C::Base> + EccInstructions<C>
{
}

/// A Transcript gadget
pub struct Transcript<C, TranscriptChip>
where
    C: CurveAffine,
    TranscriptChip: TranscriptInstructions<C>,
{
    transcript_chip: TranscriptChip,
    _marker: PhantomData<C>,
}

impl<C, TranscriptChip> Transcript<C, TranscriptChip>
where
    C: CurveAffine,
    TranscriptChip: TranscriptInstructions<C>,
{
    pub fn new(transcript_chip: TranscriptChip) -> Self {
        Self {
            transcript_chip,
            _marker: PhantomData,
        }
    }

    /// Hashes a point into the transcript.
    pub fn common_point(
        &mut self,
        mut layouter: impl Layouter<C::Base>,
        point: Value<C>,
    ) -> Result<Point<C, TranscriptChip>, Error> {
        // Witness point
        let point = Point::new(
            self.transcript_chip.clone(),
            layouter.namespace(|| "witness points"),
            point,
        )?;

        // TODO: absorb POSEIDON_PREFIX_POINT
        self.transcript_chip
            .absorb(layouter.namespace(|| "x-coordinate"), point.x())?;
        self.transcript_chip
            .absorb(layouter.namespace(|| "y-coordinate"), point.y())?;

        Ok(point)
    }

    /// Reads a scalar field element from the transcript.
    ///
    /// This instruction does the following steps:
    /// - Constrains the next sequence of proof bits to be the representation of a scalar
    ///   field element.
    /// - Assigns the scalar field element into the circuit.
    /// - Updates the transcript's internal state with this scalar field element.
    /// - Returns the assigned scalar field element.
    pub fn common_scalar(
        &mut self,
        layouter: impl Layouter<C::Base>,
        scalar: Value<C::Scalar>,
    ) -> Result<ScalarVar<C, TranscriptChip>, Error> {
        // TODO: absorb POSEIDON_PREFIX_SCALAR
        // TODO: absorb scalar

        ScalarVar::new(self.transcript_chip.clone(), layouter, scalar)
    }

    /// Squeezes a `LENGTH`-bit challenge from the transcript.
    pub fn squeeze_challenge<const LENGTH: usize>(
        &mut self,
        mut layouter: impl Layouter<C::Base>,
    ) -> Result<RangeConstrained<C::Base, AssignedCell<C::Base, C::Base>>, Error> {
        let challenge = self
            .transcript_chip
            .squeeze(layouter.namespace(|| "squeeze"))?;
        self.transcript_chip.extract_bitrange(
            layouter.namespace(|| "extract bitrange"),
            &challenge,
            0..LENGTH,
        )
    }
}
