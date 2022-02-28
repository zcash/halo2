//! Helper implementing point witnessing.

use super::{EccPoint, NonIdentityEccPoint};

use group::ff::Field;
use halo2_proofs::{
    circuit::{AssignedCell, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector, VirtualCells},
    poly::Rotation,
};
use pasta_curves::arithmetic::CurveAffine;
use std::marker::PhantomData;

type Coordinates<F> = (AssignedCell<F, F>, AssignedCell<F, F>);

/// Configuration implementing point witnessing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config<C: CurveAffine> {
    q_point: Selector,
    q_point_non_id: Selector,
    /// x-coordinate
    pub x: Column<Advice>,
    /// y-coordinate
    pub y: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> Config<C> {
    /// Configures this configuration.
    pub fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        x: Column<Advice>,
        y: Column<Advice>,
    ) -> Self {
        let config = Self {
            q_point: meta.selector(),
            q_point_non_id: meta.selector(),
            x,
            y,
            _marker: PhantomData,
        };

        config.create_gate(meta);

        config
    }

    fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let curve_eqn = |meta: &mut VirtualCells<C::Base>| {
            let x = meta.query_advice(self.x, Rotation::cur());
            let y = meta.query_advice(self.y, Rotation::cur());

            // y^2 = x^3 + b
            y.square() - (x.clone().square() * x) - Expression::Constant(C::b())
        };

        meta.create_gate("witness point", |meta| {
            // Check that the point being witnessed is either:
            // - the identity, which is mapped to (0, 0) in affine coordinates; or
            // - a valid curve point y^2 = x^3 + b, where b = 5 in the Pallas equation

            let q_point = meta.query_selector(self.q_point);
            let x = meta.query_advice(self.x, Rotation::cur());
            let y = meta.query_advice(self.y, Rotation::cur());

            vec![
                ("x == 0 v on_curve", q_point.clone() * x * curve_eqn(meta)),
                ("y == 0 v on_curve", q_point * y * curve_eqn(meta)),
            ]
        });

        meta.create_gate("witness non-identity point", |meta| {
            // Check that the point being witnessed is a valid curve point y^2 = x^3 + b,
            // where b = 5 in the Pallas equation

            let q_point_non_id = meta.query_selector(self.q_point_non_id);

            vec![("on_curve", q_point_non_id * curve_eqn(meta))]
        });
    }

    fn assign_xy(
        &self,
        value: Option<(C::Base, C::Base)>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<Coordinates<C::Base>, Error> {
        // Assign `x` value
        let x_val = value.map(|value| value.0);
        let x_var =
            region.assign_advice(|| "x", self.x, offset, || x_val.ok_or(Error::Synthesis))?;

        // Assign `y` value
        let y_val = value.map(|value| value.1);
        let y_var =
            region.assign_advice(|| "y", self.y, offset, || y_val.ok_or(Error::Synthesis))?;

        Ok((x_var, y_var))
    }

    /// Assigns a point that can be the identity.
    pub(super) fn point(
        &self,
        value: Option<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Enable `q_point` selector
        self.q_point.enable(region, offset)?;

        let value = value.map(|value| {
            // Map the identity to (0, 0).
            if value == C::identity() {
                (C::Base::zero(), C::Base::zero())
            } else {
                let value = value.coordinates().unwrap();
                (*value.x(), *value.y())
            }
        });

        self.assign_xy(value, offset, region)
            .map(|(x, y)| EccPoint { x, y })
    }

    /// Assigns a non-identity point.
    pub fn point_non_id(
        &self,
        value: Option<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        // Enable `q_point_non_id` selector
        self.q_point_non_id.enable(region, offset)?;

        if let Some(value) = value {
            // Return an error if the point is the identity.
            if value == C::identity() {
                return Err(Error::Synthesis);
            }
        };

        let value = value.map(|value| {
            let value = value.coordinates().unwrap();
            (*value.x(), *value.y())
        });

        self.assign_xy(value, offset, region)
            .map(|(x, y)| NonIdentityEccPoint { x, y })
    }
}

#[cfg(test)]
pub mod tests {
    use halo2_proofs::circuit::Layouter;
    use pasta_curves::pallas;

    use crate::ecc::{EccInstructions, NonIdentityPoint};
    use group::prime::PrimeCurveAffine;

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
