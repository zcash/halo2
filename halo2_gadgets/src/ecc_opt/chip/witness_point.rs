use crate::ecc::chip::EccPoint;

use group::prime::PrimeCurveAffine;

use halo2_proofs::{
    circuit::Region,
    plonk::{Assigned, Error},
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

use crate::ecc::chip::witness_point::{Config, Coordinates};

impl Config {
    fn assign_xy_from_constant(
        &self,
        value: (Assigned<pallas::Base>, Assigned<pallas::Base>),
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<Coordinates, Error> {
        // Assign `x` value
        let x_var = region.assign_advice_from_constant(|| "x", self.x, offset, value.0)?;

        // Assign `y` value
        let y_var = region.assign_advice_from_constant(|| "y", self.y, offset, value.1)?;

        Ok((x_var, y_var))
    }

    /// Assigns a constant point that can be the identity.
    pub(crate) fn constant_point(
        &self,
        value: pallas::Affine,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccPoint, Error> {
        // Enable `q_point` selector
        self.q_point.enable(region, offset)?;

        let value = if value == pallas::Affine::identity() {
            // Map the identity to (0, 0).
            (Assigned::Zero, Assigned::Zero)
        } else {
            let value = value.coordinates().unwrap();
            (value.x().into(), value.y().into())
        };

        self.assign_xy_from_constant(value, offset, region)
            .map(|(x, y)| EccPoint::from_coordinates_unchecked(x, y))
    }
}
