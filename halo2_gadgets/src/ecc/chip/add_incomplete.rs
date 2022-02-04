use std::{array, collections::HashSet};

use super::NonIdentityEccPoint;
use ff::Field;
use group::Curve;
use halo2_proofs::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    q_add_incomplete: Selector,
    // x-coordinate of P in P + Q = R
    pub x_p: Column<Advice>,
    // y-coordinate of P in P + Q = R
    pub y_p: Column<Advice>,
    // x-coordinate of Q or R in P + Q = R
    pub x_qr: Column<Advice>,
    // y-coordinate of Q or R in P + Q = R
    pub y_qr: Column<Advice>,
}

impl Config {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
        x_qr: Column<Advice>,
        y_qr: Column<Advice>,
    ) -> Self {
        meta.enable_equality(x_p);
        meta.enable_equality(y_p);
        meta.enable_equality(x_qr);
        meta.enable_equality(y_qr);

        let config = Self {
            q_add_incomplete: meta.selector(),
            x_p,
            y_p,
            x_qr,
            y_qr,
        };

        config.create_gate(meta);

        config
    }

    pub(crate) fn advice_columns(&self) -> HashSet<Column<Advice>> {
        core::array::IntoIter::new([self.x_p, self.y_p, self.x_qr, self.y_qr]).collect()
    }

    fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("incomplete addition gates", |meta| {
            let q_add_incomplete = meta.query_selector(self.q_add_incomplete);
            let x_p = meta.query_advice(self.x_p, Rotation::cur());
            let y_p = meta.query_advice(self.y_p, Rotation::cur());
            let x_q = meta.query_advice(self.x_qr, Rotation::cur());
            let y_q = meta.query_advice(self.y_qr, Rotation::cur());
            let x_r = meta.query_advice(self.x_qr, Rotation::next());
            let y_r = meta.query_advice(self.y_qr, Rotation::next());

            // (x_r + x_q + x_p)⋅(x_p − x_q)^2 − (y_p − y_q)^2 = 0
            let poly1 = {
                (x_r.clone() + x_q.clone() + x_p.clone())
                    * (x_p.clone() - x_q.clone())
                    * (x_p.clone() - x_q.clone())
                    - (y_p.clone() - y_q.clone()).square()
            };

            // (y_r + y_q)(x_p − x_q) − (y_p − y_q)(x_q − x_r) = 0
            let poly2 = (y_r + y_q.clone()) * (x_p - x_q.clone()) - (y_p - y_q) * (x_q - x_r);

            array::IntoIter::new([("x_r", poly1), ("y_r", poly2)])
                .map(move |(name, poly)| (name, q_add_incomplete.clone() * poly))
        });
    }

    pub(super) fn assign_region(
        &self,
        p: &NonIdentityEccPoint,
        q: &NonIdentityEccPoint,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<NonIdentityEccPoint, Error> {
        // Enable `q_add_incomplete` selector
        self.q_add_incomplete.enable(region, offset)?;

        // Handle exceptional cases
        let (x_p, y_p) = (p.x.value(), p.y.value());
        let (x_q, y_q) = (q.x.value(), q.y.value());
        x_p.zip(y_p)
            .zip(x_q)
            .zip(y_q)
            .map(|(((x_p, y_p), x_q), y_q)| {
                // P is point at infinity
                if (x_p.is_zero_vartime() && y_p.is_zero_vartime())
                // Q is point at infinity
                || (x_q.is_zero_vartime() && y_q.is_zero_vartime())
                // x_p = x_q
                || (x_p == x_q)
                {
                    Err(Error::Synthesis)
                } else {
                    Ok(())
                }
            })
            .transpose()?;

        // Copy point `p` into `x_p`, `y_p` columns
        p.x.copy_advice(|| "x_p", region, self.x_p, offset)?;
        p.y.copy_advice(|| "y_p", region, self.y_p, offset)?;

        // Copy point `q` into `x_qr`, `y_qr` columns
        q.x.copy_advice(|| "x_q", region, self.x_qr, offset)?;
        q.y.copy_advice(|| "y_q", region, self.y_qr, offset)?;

        // Compute the sum `P + Q = R`
        let r = {
            let p = p.point();
            let q = q.point();
            let r = p
                .zip(q)
                .map(|(p, q)| (p + q).to_affine().coordinates().unwrap());
            let r_x = r.map(|r| *r.x());
            let r_y = r.map(|r| *r.y());

            (r_x, r_y)
        };

        // Assign the sum to `x_qr`, `y_qr` columns in the next row
        let x_r = r.0;
        let x_r_var = region.assign_advice(
            || "x_r",
            self.x_qr,
            offset + 1,
            || x_r.ok_or(Error::Synthesis),
        )?;

        let y_r = r.1;
        let y_r_var = region.assign_advice(
            || "y_r",
            self.y_qr,
            offset + 1,
            || y_r.ok_or(Error::Synthesis),
        )?;

        let result = NonIdentityEccPoint {
            x: x_r_var,
            y: y_r_var,
        };

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use group::Curve;
    use halo2_proofs::{circuit::Layouter, plonk::Error};
    use pasta_curves::pallas;

    use crate::ecc::{EccInstructions, NonIdentityPoint};

    #[allow(clippy::too_many_arguments)]
    pub fn test_add_incomplete<
        EccChip: EccInstructions<pallas::Affine> + Clone + Eq + std::fmt::Debug,
    >(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        p_val: pallas::Affine,
        p: &NonIdentityPoint<pallas::Affine, EccChip>,
        q_val: pallas::Affine,
        q: &NonIdentityPoint<pallas::Affine, EccChip>,
        p_neg: &NonIdentityPoint<pallas::Affine, EccChip>,
        test_errors: bool,
    ) -> Result<(), Error> {
        // P + Q
        {
            let result = p.add_incomplete(layouter.namespace(|| "P + Q"), q)?;
            let witnessed_result = NonIdentityPoint::new(
                chip,
                layouter.namespace(|| "witnessed P + Q"),
                Some((p_val + q_val).to_affine()),
            )?;
            result.constrain_equal(layouter.namespace(|| "constrain P + Q"), &witnessed_result)?;
        }

        if test_errors {
            // P + P should return an error
            p.add_incomplete(layouter.namespace(|| "P + P"), p)
                .expect_err("P + P should return an error");

            // P + (-P) should return an error
            p.add_incomplete(layouter.namespace(|| "P + (-P)"), p_neg)
                .expect_err("P + (-P) should return an error");
        }

        Ok(())
    }
}
