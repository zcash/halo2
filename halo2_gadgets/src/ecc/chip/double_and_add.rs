//! Helper implementing double-and-add (incomplete addition).

use std::{array, collections::HashSet, marker::PhantomData};

use super::NonIdentityEccPoint;
use ff::Field;
use group::Curve;
use halo2_proofs::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

/// Configuration implementing double-and-add.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config<C: CurveAffine> {
    q_dbl_and_add: Selector,
    /// x-coordinate of P in P + 2Q = R
    x_p: Column<Advice>,
    /// y-coordinate of P in P + 2Q = R
    y_p: Column<Advice>,
    /// The gradient 3(x_q)^2 / 2(y_q)
    gradient: Column<Advice>,
    /// x-coordinate of Q or R in P + 2Q = R
    x_qr: Column<Advice>,
    /// y-coordinate of Q or R in P + 2Q = R
    y_qr: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> Config<C> {
    /// Configures this configuration.
    pub fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
        gradient: Column<Advice>,
        x_qr: Column<Advice>,
        y_qr: Column<Advice>,
    ) -> Self {
        meta.enable_equality(x_p);
        meta.enable_equality(y_p);
        meta.enable_equality(x_qr);
        meta.enable_equality(y_qr);

        let config = Self {
            q_dbl_and_add: meta.selector(),
            x_p,
            y_p,
            gradient,
            x_qr,
            y_qr,
            _marker: PhantomData,
        };

        config.create_gate(meta);

        config
    }

    pub(crate) fn advice_columns(&self) -> HashSet<Column<Advice>> {
        core::array::IntoIter::new([self.x_p, self.y_p, self.x_qr, self.y_qr]).collect()
    }

    fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        meta.create_gate("double-and-add gates", |meta| {
            let q_dbl_and_add = meta.query_selector(self.q_dbl_and_add);
            let x_p = meta.query_advice(self.x_p, Rotation::cur());
            let y_p = meta.query_advice(self.y_p, Rotation::cur());
            let x_q = meta.query_advice(self.x_qr, Rotation::cur());
            let y_q = meta.query_advice(self.y_qr, Rotation::cur());
            let gradient = meta.query_advice(self.gradient, Rotation::cur());
            let x_r = meta.query_advice(self.x_qr, Rotation::next());
            let y_r = meta.query_advice(self.y_qr, Rotation::next());

            let two = C::Base::from(2);
            let three = C::Base::from(3);
            //             gradient = 3(x_q)^2 / 2(y_q)
            // => 2(y_q) * gradient = 3(x_q)^2
            let gradient_check = {
                let lhs = y_q.clone() * two * gradient.clone();
                let rhs = x_q.clone().square() * three;
                lhs - rhs
            };
            // ([2]Q)_x = (3(x_q)^2 / 2(y_q))^2 - 2x_q
            let x_dblq = gradient.clone().square() - x_q.clone() * two;
            // ([2]Q)_y =  (3(x_q)^2 / 2(y_q))(x_q - x_dblq) - y_q
            let y_dblq = gradient * (x_q - x_dblq.clone()) - y_q;

            // (x_r + x_dblq + x_p)⋅(x_p − x_dblq)^2 − (y_p − y_dblq)^2 = 0
            let poly1 = {
                (x_r.clone() + x_dblq.clone() + x_p.clone())
                    * (x_p.clone() - x_dblq.clone()).square()
                    - (y_p.clone() - y_dblq.clone()).square()
            };

            // (y_r + y_dblq)(x_p − x_dblq) − (y_p − y_dblq)(x_dblq − x_r) = 0
            // FIXME degree too high, witness gradient instead of inv_yq
            let poly2 =
                (y_r + y_dblq.clone()) * (x_p - x_dblq.clone()) - (y_p - y_dblq) * (x_dblq - x_r);

            array::IntoIter::new([
                ("gradient_check", gradient_check),
                ("x_r", poly1),
                ("y_r", poly2),
            ])
            .map(move |(name, poly)| (name, q_dbl_and_add.clone() * poly))
        });
    }

    /// Assign region for double-and-add. P + 2Q
    pub fn assign_region(
        &self,
        p: &NonIdentityEccPoint<C>,
        q: &NonIdentityEccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        // Enable `q_dbl_and_add` selector
        self.q_dbl_and_add.enable(region, offset)?;

        // Handle exceptional cases
        p.point()
            .zip(q.point())
            .map(|(p, q)| {
                let q_dbl = q * C::Scalar::from(2);
                // P is point at infinity
                if p.is_identity().into()
                // Q is point at infinity
                || q.is_identity().into()
            // P = 2Q or P = -2Q
            || (p == q_dbl.to_affine() || -p == q_dbl.to_affine())
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

        // Witness `gradient = 3(x_q)^2 / 2(y_q)` in `gradient` column.
        region.assign_advice(
            || "gradient",
            self.gradient,
            offset,
            || {
                q.x.value()
                    .zip(q.y.value())
                    .map(|(&x, &y)| {
                        x.square() * C::Base::from(3) * C::Base::TWO_INV * y.invert().unwrap()
                    })
                    .ok_or(Error::Synthesis)
            },
        )?;

        // Copy point `q` into `x_qr`, `y_qr` columns
        q.x.copy_advice(|| "x_q", region, self.x_qr, offset)?;
        q.y.copy_advice(|| "y_q", region, self.y_qr, offset)?;

        // Compute the sum `P + 2Q = R`
        let r = {
            let p = p.point();
            let q = q.point();
            let r = p
                .zip(q)
                .map(|(p, q)| (p + q + q).to_affine().coordinates().unwrap());
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
    use crate::ecc::chip::witness_point;
    use group::{Curve, Group};
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{
        arithmetic::{CurveAffine, FieldExt},
        pallas,
    };
    use rand::rngs::OsRng;

    #[test]
    pub fn test_add_incomplete() {
        struct MyCircuit<C: CurveAffine> {
            p: Option<C>,
            q: Option<C>,
        }

        impl<C: CurveAffine> Circuit<C::Base> for MyCircuit<C> {
            type Config = (super::Config<C>, witness_point::Config<C>);
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self { p: None, q: None }
            }

            fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
                let x_p = meta.advice_column();
                let y_p = meta.advice_column();
                let gradient = meta.advice_column();
                let x_qr = meta.advice_column();
                let y_qr = meta.advice_column();

                let add_incomplete_config =
                    super::Config::configure(meta, x_p, y_p, gradient, x_qr, y_qr);
                let witness_point_config = witness_point::Config::configure(meta, x_p, y_p);

                (add_incomplete_config, witness_point_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<C::Base>,
            ) -> Result<(), Error> {
                layouter.assign_region(
                    || "P + 2Q",
                    |mut region| {
                        let p = config.1.point_non_id(self.p, 0, &mut region)?;
                        let q = config.1.point_non_id(self.q, 1, &mut region)?;
                        let res = config.0.assign_region(&p, &q, 2, &mut region)?;

                        if let Some(point) = res.point() {
                            assert_eq!(
                                point.to_curve(),
                                self.p.unwrap().to_curve() + self.q.unwrap() + self.q.unwrap()
                            );
                        }

                        Ok(())
                    },
                )
            }
        }

        // P + 2Q
        {
            let circuit = MyCircuit {
                p: Some(pallas::Point::random(OsRng).to_affine()),
                q: Some(pallas::Point::random(OsRng).to_affine()),
            };
            let prover = MockProver::<pallas::Base>::run(8, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // P + 2Q = P + P should return an error
        {
            let p = pallas::Point::random(OsRng);
            // 2Q = P
            let q = p * pallas::Scalar::TWO_INV;
            let circuit = MyCircuit {
                p: Some(p.to_affine()),
                q: Some(q.to_affine()),
            };
            MockProver::<pallas::Base>::run(8, &circuit, vec![])
                .expect_err("P + P should return an error");
        }

        // P + 2Q = P + (-P) should return an error
        {
            let p = pallas::Point::random(OsRng);
            // 2Q = -P
            let q = -p * pallas::Scalar::TWO_INV;
            let circuit = MyCircuit {
                p: Some(p.to_affine()),
                q: Some(q.to_affine()),
            };
            MockProver::<pallas::Base>::run(8, &circuit, vec![])
                .expect_err("P + (-P) should return an error");
        }
    }
}
