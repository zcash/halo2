//! Chip implementations for the ECC gadgets.

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk::Error,
};
use pasta_curves::pallas;

use crate::{
    ecc::{
        chip::{BaseFieldElem, EccChip, FixedPoint, FullScalar, ShortScalar},
        FixedPoints,
    },
    utilities::lookup_range_check::DefaultLookupRangeCheck,
};

use super::EccInstructionsOptimized;

pub(crate) mod mul_fixed;
pub(super) mod witness_point;

impl<Fixed: FixedPoints<pallas::Affine>, LookupRangeCheckConfig: DefaultLookupRangeCheck>
    EccInstructionsOptimized<pallas::Affine> for EccChip<Fixed, LookupRangeCheckConfig>
where
    <Fixed as FixedPoints<pallas::Affine>>::Base:
        FixedPoint<pallas::Affine, FixedScalarKind = BaseFieldElem>,
    <Fixed as FixedPoints<pallas::Affine>>::FullScalar:
        FixedPoint<pallas::Affine, FixedScalarKind = FullScalar>,
    <Fixed as FixedPoints<pallas::Affine>>::ShortScalar:
        FixedPoint<pallas::Affine, FixedScalarKind = ShortScalar>,
{
    fn witness_point_from_constant(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: pallas::Affine,
    ) -> Result<Self::Point, Error> {
        let config = self.config().witness_point;
        layouter.assign_region(
            || "witness point (constant)",
            |mut region| config.constant_point(value, 0, &mut region),
        )
    }

    /// Performs variable-base sign-scalar multiplication, returning `[sign] point`
    /// `sign` must be in {-1, 1}.
    fn mul_sign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        sign: &AssignedCell<pallas::Base, pallas::Base>,
        point: &Self::Point,
    ) -> Result<Self::Point, Error> {
        // Multiply point by sign, using the same gate as mul_fixed::short.
        // This also constrains sign to be in {-1, 1}.
        let config_short = self.config().mul_fixed_short.clone();
        config_short.assign_scalar_sign(
            layouter.namespace(|| "variable-base sign-scalar mul"),
            sign,
            point,
        )
    }
}
