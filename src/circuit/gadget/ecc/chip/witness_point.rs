use super::{CellValue, EccConfig, NonIdentityEccPoint, Var};

use group::prime::PrimeCurveAffine;

use halo2::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

#[derive(Clone, Debug)]
pub struct Config {
    q_point_non_id: Selector,
    // x-coordinate
    pub x: Column<Advice>,
    // y-coordinate
    pub y: Column<Advice>,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_point_non_id: ecc_config.q_point_non_id,
            x: ecc_config.advices[0],
            y: ecc_config.advices[1],
        }
    }
}

impl Config {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("witness non-identity point", |meta| {
            // Check that the point being witnessed is a valid curve point y^2 = x^3 + b,
            // where b = 5 in the Pallas equation

            let q_point_non_id = meta.query_selector(self.q_point_non_id);

            let x = meta.query_advice(self.x, Rotation::cur());
            let y = meta.query_advice(self.y, Rotation::cur());

            // y^2 = x^3 + b
            let curve_eqn =
                y.square() - (x.clone().square() * x) - Expression::Constant(pallas::Affine::b());

            vec![("on_curve", q_point_non_id * curve_eqn)]
        });
    }

    /// Assigns a non-identity point.
    pub(super) fn point_non_id(
        &self,
        value: Option<pallas::Affine>,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<NonIdentityEccPoint, Error> {
        // Enable `q_point_non_id` selector
        self.q_point_non_id.enable(region, offset)?;

        // Return an error if the point is the identity.
        if value == Some(pallas::Affine::identity()) {
            return Err(Error::SynthesisError);
        };

        let value = value.map(|value| {
            let value = value.coordinates().unwrap();
            (*value.x(), *value.y())
        });

        // Assign `x` value
        let x_val = value.map(|value| value.0);
        let x_var = region.assign_advice(
            || "x",
            self.x,
            offset,
            || x_val.ok_or(Error::SynthesisError),
        )?;

        // Assign `y` value
        let y_val = value.map(|value| value.1);
        let y_var = region.assign_advice(
            || "y",
            self.y,
            offset,
            || y_val.ok_or(Error::SynthesisError),
        )?;

        Ok(NonIdentityEccPoint {
            x: CellValue::<pallas::Base>::new(x_var, x_val),
            y: CellValue::<pallas::Base>::new(y_var, y_val),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use halo2::circuit::Layouter;
    use pasta_curves::pallas;

    use super::*;
    use crate::circuit::gadget::ecc::{EccInstructions, NonIdentityPoint};

    pub fn test_witness_non_id<
        EccChip: EccInstructions<pallas::Affine> + Clone + Eq + std::fmt::Debug,
    >(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
    ) {
        // Witnessing the identity should return an error.
        NonIdentityPoint::new(
            chip,
            layouter.namespace(|| "witness identity"),
            Some(pallas::Affine::identity()),
        )
        .expect_err("witnessing 𝒪 should return an error");
    }
}
