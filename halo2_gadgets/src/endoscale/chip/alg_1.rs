use ff::{Field, PrimeFieldBits};
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Layouter, Region, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Constraints, Error, Expression, Selector},
    poly::Rotation,
};

use super::super::util::endoscale_point_pair;
use crate::{
    ecc::chip::{add_incomplete, double, NonIdentityEccPoint},
    utilities::{
        bool_check,
        decompose_running_sum::{RunningSum, RunningSumConfig},
        double_and_add::{DoubleAndAdd, X, Y},
        le_bits_to_field_elem,
    },
};

pub(super) type Bitstring<F> = RunningSum<F, 2>;

/// Config used in Algorithm 1 (endoscaling with a base).
#[derive(Clone, Debug)]
pub(super) struct Alg1Config<C: CurveAffine>
where
    C::Base: PrimeFieldBits,
{
    // Selector for endoscaling checks.
    q_endoscale_base: Selector,
    // Selector for the initial check in double-and-add.
    q_double_and_add_init: Selector,
    // Selector for stead-state double-and-add.
    q_double_and_add: Selector,
    // Selector for the final check in double-and-add.
    q_double_and_add_final: Selector,
    // Configuration used for steady-state double-and-add.
    double_and_add: DoubleAndAdd<C>,
    // Incomplete point doubling config
    double: double::Config<C>,
    // Incomplete point addition config
    add_incomplete: add_incomplete::Config<C>,
    // Bases used in endoscaling.
    base: (Column<Advice>, Column<Advice>),
    // Bits used in endoscaling. These are in (b_0, b_1) pairs.
    pair: (Column<Advice>, Column<Advice>),
    // Configuration for running sum decomposition into pairs of bits.
    pub(super) running_sum_pairs: RunningSumConfig<C::Base, 2>,
}

impl<C: CurveAffine> Alg1Config<C>
where
    C::Base: PrimeFieldBits,
{
    pub(super) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        pair: (Column<Advice>, Column<Advice>),
        base: (Column<Advice>, Column<Advice>),
        (x_a, x_p, lambda_1, lambda_2): (
            Column<Advice>,
            Column<Advice>,
            Column<Advice>,
            Column<Advice>,
        ),
        running_sum_pairs: RunningSumConfig<C::Base, 2>,
    ) -> Self {
        meta.enable_equality(base.0);
        meta.enable_equality(base.1);

        let q_endoscale_base = meta.selector();

        // Initial double-and-add gate
        let q_double_and_add_init = meta.selector();
        // Steady-state double-and-add gate
        let q_double_and_add = meta.complex_selector();
        // Final double-and-add gate
        let q_double_and_add_final = meta.complex_selector();

        let double_and_add = DoubleAndAdd::configure(
            meta,
            x_a,
            x_p,
            lambda_1,
            lambda_2,
            &|meta| {
                let q_double_and_add = meta.query_selector(q_double_and_add);
                let q_double_and_add_final = meta.query_selector(q_double_and_add_final);
                q_double_and_add + q_double_and_add_final
            },
            &|meta| meta.query_selector(q_double_and_add),
        );

        let advices = double_and_add.advices();
        let add_incomplete =
            add_incomplete::Config::configure(meta, advices[2], advices[3], advices[0], advices[1]);
        let double =
            double::Config::configure(meta, advices[0], advices[1], advices[2], advices[3]);

        meta.enable_equality(add_incomplete.x_p);
        meta.enable_equality(add_incomplete.y_p);

        meta.create_gate("init double-and-add", |meta| {
            let selector = meta.query_selector(q_double_and_add_init);
            // The accumulator is initialised to [2](φ(P) + P).

            // Check that the x-coordinate of the inputs to the incomplete addition
            // are related as x, ζx.
            // The y-coordinate is copy-constrained.
            let incomplete_add_x_check = {
                let x_p = meta.query_advice(add_incomplete.x_p, Rotation::prev());
                let phi_x_p = meta.query_advice(add_incomplete.x_qr, Rotation::prev());

                x_p * C::Base::ZETA - phi_x_p
            };

            // Check that the initial accumulator's y-coordinate `y_a` is consistent
            // with the one derived internally by `double_and_add`.
            let init_y_a_check = {
                let y_a = meta.query_advice(double.y_r, Rotation::cur());
                let derived_y_a = double_and_add.y_a(meta, Rotation::next());

                y_a - derived_y_a
            };

            Constraints::with_selector(
                selector,
                [
                    ("incomplete_add_x_check", incomplete_add_x_check),
                    ("init_y_a_check", init_y_a_check),
                ],
            )
        });

        meta.create_gate("final double-and-add", |meta| {
            // Check that the final witnessed y_a is consistent with the y_a
            // derived internally by `double_and_add`.
            let selector = meta.query_selector(q_double_and_add_final);

            // x_{A,i}
            let x_a_prev = meta.query_advice(double_and_add.x_a, Rotation::cur());
            // x_{A,i-1}
            let x_a_cur = meta.query_advice(double_and_add.x_a, Rotation::next());
            // λ_{2,i}
            let lambda2_prev = meta.query_advice(double_and_add.lambda_2, Rotation::cur());
            let y_a_prev = double_and_add.y_a(meta, Rotation::cur());

            let lhs = lambda2_prev * (x_a_prev - x_a_cur);
            let rhs = {
                let y_a_final = meta.query_advice(lambda_1, Rotation::next());
                y_a_prev + y_a_final
            };

            Constraints::with_selector(selector, [lhs - rhs])
        });

        /*
            The accumulator is initialised to [2](φ(P) + P) = (init_x, init_y).

            | pair.0 | pair.1 |    base.0     |    base.1     |  double_and_add.x_a   | double_and_add.lambda_1|   <- column names
            ---------------------------------------------------------------------------------------------------|
            |  b_0   |   b_1  |  init endo_x  |  init endo_y  |      init acc_x       |       init acc_y       |
            |  ...   |   ...  |       ...     |      ...      |          ...          | (acc_y not witnessed)  |
            | b_{n-2}| b_{n-1}|  final endo_x | final endo_y  |      final acc_x      |       final acc_y      |

            (0, 0) -> (P_x, -P_y)
            (0, 1) -> (ζ * P_x, -P_y)
            (1, 0) -> (P_x, P_y)
            (1, 1) -> (ζ * P_x, P_y)
        */
        meta.create_gate("Endoscale base", |meta| {
            let q_endoscale_base = meta.query_selector(q_endoscale_base);

            // Pair of bits from the decomposition.
            let b_0 = meta.query_advice(pair.0, Rotation::cur());
            let b_1 = meta.query_advice(pair.1, Rotation::cur());

            // Boolean-constrain b_0, b_1
            let b_0_check = bool_check(b_0.clone());
            let b_1_check = bool_check(b_1.clone());

            // Check that `b_0, b_1` are consistent with the running sum decomposition.
            let decomposition_check = {
                let word = b_0.clone() + Expression::Constant(C::Base::from(2)) * b_1.clone();
                let expected_word = running_sum_pairs.window_expr_be(meta);

                word - expected_word
            };

            let y_check = {
                let endo_y = double_and_add.y_p(meta, Rotation::cur());
                let p_y = meta.query_advice(base.1, Rotation::cur());
                // If the first bit is set, check that endo_y = P_y
                let b0_set = b_0.clone() * (endo_y.clone() - p_y.clone());

                // If the first bit is not set, check that endo_y = -P_y
                let not_b0 = Expression::Constant(C::Base::one()) - b_0;
                let b0_not_set = not_b0 * (endo_y + p_y);

                b0_set + b0_not_set
            };
            let x_check = {
                let endo_x = meta.query_advice(double_and_add.x_p, Rotation::cur());
                let p_x = meta.query_advice(base.0, Rotation::cur());
                // If the second bit is set, check that endo_x = ζ * P_x
                let zeta = Expression::Constant(C::Base::ZETA);
                let b1_set = b_1.clone() * (endo_x.clone() - zeta * p_x.clone());

                // If the second bit is not set, check that endo_x = P_x
                let not_b1 = Expression::Constant(C::Base::one()) - b_1;
                let b1_not_set = not_b1 * (endo_x - p_x);

                b1_set + b1_not_set
            };

            Constraints::with_selector(
                q_endoscale_base,
                std::array::IntoIter::new([
                    ("b_0_check", b_0_check),
                    ("b_1_check", b_1_check),
                    ("decomposition_check", decomposition_check),
                    ("x_check", x_check),
                    ("y_check", y_check),
                ]),
            )
        });

        Self {
            q_endoscale_base,
            q_double_and_add_init,
            q_double_and_add,
            q_double_and_add_final,
            double_and_add,
            double,
            add_incomplete,
            base,
            pair,
            running_sum_pairs,
        }
    }

    pub(super) fn witness_bitstring(
        &self,
        mut layouter: impl Layouter<C::Base>,
        bits: &[Value<bool>],
    ) -> Result<Bitstring<C::Base>, Error> {
        let alpha = {
            let bits = Value::<Vec<_>>::from_iter(bits.to_vec());
            bits.map(|b| le_bits_to_field_elem(&b))
        };
        let word_num_bits = bits.len();
        let num_windows = word_num_bits / 2;

        layouter.assign_region(
            || "witness bitstring",
            |mut region| {
                let offset = 0;

                self.running_sum_pairs.witness_decompose(
                    &mut region,
                    offset,
                    alpha,
                    true,
                    word_num_bits,
                    num_windows,
                )
            },
        )
    }

    pub(super) fn endoscale_fixed_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Bitstring<C::Base>,
        base: &C,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        layouter.assign_region(
            || "endoscale with fixed base",
            |mut region| {
                let offset = 0;

                let base = {
                    // Assign base_x
                    let x = region.assign_advice_from_constant(
                        || "base_x",
                        self.add_incomplete.x_p,
                        offset,
                        Assigned::from(*base.coordinates().unwrap().x()),
                    )?;

                    // Assign base_y
                    let y = region.assign_advice_from_constant(
                        || "base_y",
                        self.add_incomplete.y_p,
                        offset,
                        Assigned::from(*base.coordinates().unwrap().y()),
                    )?;
                    NonIdentityEccPoint::from_coordinates_unchecked(x, y)
                };

                self.endoscale_base_inner(&mut region, offset, &base, bitstring)
            },
        )
    }

    pub(super) fn endoscale_var_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Bitstring<C::Base>,
        base: &NonIdentityEccPoint<C>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        layouter.assign_region(
            || "endoscale with variable base",
            |mut region| {
                let offset = 0;

                let base = {
                    let x = base.x().copy_advice(
                        || "base_x",
                        &mut region,
                        self.add_incomplete.x_p,
                        offset,
                    )?;
                    let y = base.y().copy_advice(
                        || "base_y",
                        &mut region,
                        self.add_incomplete.y_p,
                        offset,
                    )?;
                    NonIdentityEccPoint::from_coordinates_unchecked(x.into(), y.into())
                };

                self.endoscale_base_inner(&mut region, offset, &base, bitstring)
            },
        )
    }
}

impl<C: CurveAffine> Alg1Config<C>
where
    C::Base: PrimeFieldBits,
{
    #[allow(clippy::type_complexity)]
    fn endoscale_base_init(
        &self,
        region: &mut Region<'_, C::Base>,
        mut offset: usize,
        base: &NonIdentityEccPoint<C>,
    ) -> Result<(usize, (X<C::Base>, Y<C::Base>)), Error> {
        // The accumulator is initialised to [2](φ(P) + P)
        self.q_double_and_add_init.enable(region, offset + 1)?;

        // Incomplete addition of (φ(P) + P), where φ(P) = φ((x, y)) = (ζx, y)
        let sum = {
            let zeta_x = base.x().value().map(|p| Assigned::from(*p * C::Base::ZETA));
            let zeta_x =
                region.assign_advice(|| "ζ * x", self.add_incomplete.x_qr, offset, || zeta_x)?;
            let phi_p = NonIdentityEccPoint::from_coordinates_unchecked(zeta_x, base.y().into());

            self.add_incomplete
                .assign_region(base, &phi_p, offset, region)?
        };
        offset += 1;

        let acc = self
            .double
            .assign_region(&sum, offset, region)
            .map(|acc| (X(acc.x().into()), Y(acc.y().value().copied().into())))?;
        offset += 1;

        Ok((offset, acc))
    }

    #[allow(clippy::type_complexity)]
    fn endoscale_base_main(
        &self,
        region: &mut Region<'_, C::Base>,
        mut offset: usize,
        mut acc: (X<C::Base>, Y<C::Base>),
        base: &NonIdentityEccPoint<C>,
        // Bitstring decomposed into 2-bit windows using a running sum.
        // This internally enables the `q_range_check` selector, which is
        // used in the "Endoscale base" gate.
        bitstring: &Bitstring<C::Base>,
    ) -> Result<(usize, (X<C::Base>, Y<C::Base>)), Error> {
        let running_sum_len = bitstring.zs().len();
        // Copy in running sum
        for (idx, z) in bitstring.zs().iter().rev().enumerate() {
            z.copy_advice(
                || format!("z[{:?}]", running_sum_len - idx),
                region,
                self.running_sum_pairs.z(),
                offset + idx,
            )?;
        }

        // Enable selector for steady-state double-and-add on all but last row
        let num_pairs = bitstring.pairs().len();
        for idx in 0..(num_pairs - 1) {
            self.q_double_and_add.enable(region, offset + idx)?;
        }

        for (pair_idx, pair) in bitstring.pairs().iter().rev().enumerate() {
            self.q_endoscale_base.enable(region, offset)?;

            // Assign base
            base.x()
                .copy_advice(|| "base_x", region, self.base.0, offset)?;
            base.y()
                .copy_advice(|| "base_y", region, self.base.1, offset)?;

            // Assign b_0
            let b_0 = pair.map(|pair| pair[0]);
            region.assign_advice(
                || format!("pair_idx: {}, b_0", num_pairs - pair_idx),
                self.pair.0,
                offset,
                || b_0.map(|b| C::Base::from(b as u64)),
            )?;

            // Assign b_1
            let b_1 = pair.map(|pair| pair[1]);
            region.assign_advice(
                || format!("pair_idx: {}, b_1", num_pairs - pair_idx),
                self.pair.1,
                offset,
                || b_1.map(|b| C::Base::from(b as u64)),
            )?;

            let endo = {
                let base = base.point();
                let endo = pair
                    .zip(base)
                    .map(|(pair, base)| endoscale_point_pair::<C>(pair, base).unwrap());

                let endo_x = endo.map(|endo| *endo.coordinates().unwrap().x());
                let endo_y = endo.map(|endo| *endo.coordinates().unwrap().y());

                (endo_x, endo_y)
            };

            // Add endo to acc.
            acc = self.double_and_add.assign_region(
                region,
                offset,
                (endo.0.map(|v| v.into()), endo.1.map(|v| v.into())),
                acc.0,
                acc.1,
            )?;

            offset += 1;
        }

        Ok((offset, acc))
    }

    #[allow(clippy::type_complexity)]
    fn endoscale_base_final(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        (x, y): (X<C::Base>, Y<C::Base>),
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        self.q_double_and_add_final.enable(region, offset - 1)?;
        let y =
            region.assign_advice(|| "final y_a", self.double_and_add.lambda_1, offset, || *y)?;
        Ok(NonIdentityEccPoint::from_coordinates_unchecked(x.0, y))
    }

    fn endoscale_base_inner(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        base: &NonIdentityEccPoint<C>,
        bitstring: &Bitstring<C::Base>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        let (offset, acc) = self.endoscale_base_init(region, offset, base)?;

        let (offset, (x, y)) = self.endoscale_base_main(region, offset, acc, base, bitstring)?;

        let res = self.endoscale_base_final(region, offset, (x, y))?;

        #[cfg(test)]
        {
            use crate::endoscale::util::endoscale_point;
            let point = base.point();
            let expected_res = bitstring
                .bitstring()
                .zip(point)
                .map(|(bits, point)| endoscale_point(&bits, point));
            res.point()
                .zip(expected_res)
                .map(|(res, expected_res)| assert_eq!(res, expected_res));
        }

        Ok(res)
    }
}

impl<F: FieldExt + PrimeFieldBits> Bitstring<F> {
    fn pairs(&self) -> Vec<Value<[bool; 2]>> {
        self.windows()
            .iter()
            .map(|window| {
                window.map(|window| {
                    window
                        .to_le_bits()
                        .into_iter()
                        .take(2)
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                })
            })
            .collect()
    }

    #[cfg(test)]
    fn bitstring(&self) -> Value<Vec<bool>> {
        let num_bits = self.num_bits();
        self.zs()[0]
            .value()
            .map(|v| v.to_le_bits().iter().by_vals().take(num_bits).collect())
    }
}
