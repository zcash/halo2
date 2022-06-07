use std::marker::PhantomData;

use super::NonIdentityEccPoint;
use ff::Field;
use group::{Curve, Group};
use halo2_proofs::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::CurveAffine;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config<C: CurveAffine> {
    q_double: Selector,
    // x-coordinate of P in R = [2]P
    pub x_p: Column<Advice>,
    // y-coordinate of P in R = [2]P
    pub y_p: Column<Advice>,
    // x-coordinate of R in R = [2]P
    pub x_r: Column<Advice>,
    // y-coordinate of R in R = [2]P
    pub y_r: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> Config<C> {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
        x_r: Column<Advice>,
        y_r: Column<Advice>,
    ) -> Self {
        meta.enable_equality(x_p);
        meta.enable_equality(y_p);
        meta.enable_equality(x_r);
        meta.enable_equality(y_r);

        let config = Self {
            q_double: meta.selector(),
            x_p,
            y_p,
            x_r,
            y_r,
            _marker: PhantomData,
        };

        config.create_gate(meta);

        config
    }

    fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        // https://p.z.cash/halo2-0.1:ecc-incomplete-doubling
        meta.create_gate("incomplete doubling", |meta| {
            let q_double = meta.query_selector(self.q_double);
            let x_p = meta.query_advice(self.x_p, Rotation::cur());
            let y_p = meta.query_advice(self.y_p, Rotation::cur());
            let x_r = meta.query_advice(self.x_r, Rotation::cur());
            let y_r = meta.query_advice(self.y_r, Rotation::cur());

            // (x_r + 2x_p) * 4y_p^2 - 9x_p^4 = 0
            let poly1 = {
                (x_r.clone() + x_p.clone() * C::Base::from(2))
                    * y_p.clone().square()
                    * C::Base::from(4)
                    - x_p.clone().square().square() * C::Base::from(9)
            };

            // (y_r + y_p) * 2y_p - 3x_p^2 * (x_p - x_r) = 0
            let poly2 = (y_r + y_p.clone()) * y_p * C::Base::from(2)
                - x_p.clone().square() * C::Base::from(3) * (x_p - x_r);

            Constraints::with_selector(q_double, [("x_r", poly1), ("y_r", poly2)])
        });
    }

    pub(super) fn assign_region(
        &self,
        p: &NonIdentityEccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        // Enable `q_double` selector
        self.q_double.enable(region, offset)?;

        // Handle exceptional cases
        p.y.value()
            .map(|y_p| {
                if y_p.is_zero_vartime() {
                    Err(Error::Synthesis)
                } else {
                    Ok(())
                }
            })
            .transpose()?;

        // Copy point `p` into `x_p`, `y_p` columns
        p.x.copy_advice(|| "x_p", region, self.x_p, offset)?;
        p.y.copy_advice(|| "y_p", region, self.y_p, offset)?;

        // Compute the doubling `[2]P = R`
        let r = {
            let r = p
                .point()
                .map(|p| p.to_curve().double().to_affine().coordinates().unwrap());
            let r_x = r.map(|r| *r.x());
            let r_y = r.map(|r| *r.y());

            (r_x, r_y)
        };

        // Assign `r` to `x_r`, `y_r` columns
        let x_r = r.0;
        let x_r =
            region.assign_advice(|| "x_r", self.x_r, offset, || x_r.ok_or(Error::Synthesis))?;

        let y_r = r.1;
        let y_r =
            region.assign_advice(|| "y_r", self.y_r, offset, || y_r.ok_or(Error::Synthesis))?;

        let result = NonIdentityEccPoint { x: x_r, y: y_r };

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use group::{prime::PrimeCurveAffine, Curve, Group};
    use halo2_proofs::{circuit::Layouter, plonk::Error};
    use pasta_curves::pallas;

    use crate::ecc::{EccInstructions, NonIdentityPoint};

    #[allow(clippy::too_many_arguments)]
    pub fn test_double<EccChip: EccInstructions<pallas::Affine> + Clone + Eq + std::fmt::Debug>(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        p_val: pallas::Affine,
        p: &NonIdentityPoint<pallas::Affine, EccChip>,
    ) -> Result<(), Error> {
        // [2]P
        {
            let result = p.double(layouter.namespace(|| "[2]P"))?;
            let witnessed_result = NonIdentityPoint::new(
                chip,
                layouter.namespace(|| "witnessed [2]P"),
                Some((p_val.to_curve().double()).to_affine()),
            )?;
            result.constrain_equal(layouter.namespace(|| "constrain [2]P"), &witnessed_result)?;
        }

        Ok(())
    }
}
