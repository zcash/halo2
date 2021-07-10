use std::array;

use super::super::{copy, CellValue, EccConfig, EccPoint, EccScalarFixedShort, Var};
use crate::constants::{ValueCommitV, L_VALUE, NUM_WINDOWS_SHORT};

use halo2::{
    circuit::{Layouter, Region},
    plonk::{ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

pub struct Config {
    // Selector used for fixed-base scalar mul with short signed exponent.
    q_mul_fixed_short: Selector,
    q_scalar_fixed_short: Selector,
    super_config: super::Config<NUM_WINDOWS_SHORT>,
}

impl From<&EccConfig> for Config {
    fn from(config: &EccConfig) -> Self {
        Self {
            q_mul_fixed_short: config.q_mul_fixed_short,
            q_scalar_fixed_short: config.q_scalar_fixed_short,
            super_config: config.into(),
        }
    }
}

impl Config {
    // We reuse the constraints in the `mul_fixed` gate so exclude them here.
    // Here, we add some new constraints specific to the short signed case.
    pub(crate) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        // Check that sign is either 1 or -1.
        // Check that last window is either 0 or 1.
        meta.create_gate("Check sign and last window", |meta| {
            let q_scalar_fixed_short = meta.query_selector(self.q_scalar_fixed_short);
            let last_window = meta.query_advice(self.super_config.window, Rotation::prev());
            let sign = meta.query_advice(self.super_config.window, Rotation::cur());

            let one = Expression::Constant(pallas::Base::one());

            let last_window_check = last_window.clone() * (one.clone() - last_window);
            let sign_check = sign.clone() * sign - one;

            vec![
                q_scalar_fixed_short.clone() * last_window_check,
                q_scalar_fixed_short * sign_check,
            ]
        });

        meta.create_gate("Short fixed-base mul gate", |meta| {
            let q_mul_fixed_short = meta.query_selector(self.q_mul_fixed_short);
            let y_p = meta.query_advice(self.super_config.y_p, Rotation::cur());
            let y_a = meta.query_advice(self.super_config.add_config.y_qr, Rotation::cur());
            let sign = meta.query_advice(self.super_config.window, Rotation::cur());

            // `(x_a, y_a)` is the result of `[m]B`, where `m` is the magnitude.
            // We conditionally negate this result using `y_p = y_a * s`, where `s` is the sign.

            // Check that the final `y_p = y_a` or `y_p = -y_a`
            let y_check = q_mul_fixed_short.clone()
                * (y_p.clone() - y_a.clone())
                * (y_p.clone() + y_a.clone());

            // Check that the correct sign is witnessed s.t. sign * y_p = y_a
            let negation_check = sign * y_p - y_a;

            array::IntoIter::new([y_check, negation_check])
                .map(move |poly| q_mul_fixed_short.clone() * poly)
        });
    }

    fn witness(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        value: Option<pallas::Scalar>,
    ) -> Result<EccScalarFixedShort, Error> {
        // Enable `q_scalar_fixed_short`
        self.q_scalar_fixed_short
            .enable(region, offset + NUM_WINDOWS_SHORT)?;

        // Compute the scalar's sign and magnitude
        let sign = value.map(|value| {
            // t = (p - 1)/2
            let t = (pallas::Scalar::zero() - &pallas::Scalar::one()) * &pallas::Scalar::TWO_INV;
            if value > t {
                -pallas::Scalar::one()
            } else {
                pallas::Scalar::one()
            }
        });

        let magnitude = sign.zip(value).map(|(sign, value)| sign * &value);

        // Decompose magnitude into `k`-bit windows
        let windows = self
            .super_config
            .decompose_scalar_fixed::<L_VALUE>(magnitude, offset, region)?;

        // Assign the sign and enable `q_scalar_fixed_short`
        let sign = sign.map(|sign| {
            assert!(sign == pallas::Scalar::one() || sign == -pallas::Scalar::one());
            if sign == pallas::Scalar::one() {
                pallas::Base::one()
            } else {
                -pallas::Base::one()
            }
        });
        let sign_cell = region.assign_advice(
            || "sign",
            self.super_config.window,
            offset + NUM_WINDOWS_SHORT,
            || sign.ok_or(Error::SynthesisError),
        )?;

        Ok(EccScalarFixedShort {
            magnitude,
            sign: CellValue::<pallas::Base>::new(sign_cell, sign),
            windows,
        })
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        scalar: Option<pallas::Scalar>,
        base: &ValueCommitV,
    ) -> Result<(EccPoint, EccScalarFixedShort), Error> {
        let (scalar, acc, mul_b) = layouter.assign_region(
            || "Short fixed-base mul (incomplete addition)",
            |mut region| {
                let offset = 0;

                // Copy the scalar decomposition
                let scalar = self.witness(&mut region, offset, scalar)?;

                let (acc, mul_b) = self.super_config.assign_region_inner(
                    &mut region,
                    offset,
                    &(&scalar).into(),
                    base.clone().into(),
                    self.super_config.q_mul_fixed,
                )?;

                Ok((scalar, acc, mul_b))
            },
        )?;

        let result = layouter.assign_region(
            || "Short fixed-base mul (most significant word)",
            |mut region| {
                let offset = 0;
                // Add to the cumulative sum to get `[magnitude]B`.
                let magnitude_mul = self.super_config.add_config.assign_region(
                    &mul_b,
                    &acc,
                    offset,
                    &mut region,
                )?;

                // Increase offset by 1 after complete addition
                let offset = offset + 1;

                // Assign sign to `window` column
                let sign = copy(
                    &mut region,
                    || "sign",
                    self.super_config.window,
                    offset,
                    &scalar.sign,
                    &self.super_config.perm,
                )?;

                // Conditionally negate `y`-coordinate
                let y_val = if let Some(sign) = sign.value() {
                    if sign == -pallas::Base::one() {
                        magnitude_mul.y.value().map(|y: pallas::Base| -y)
                    } else {
                        magnitude_mul.y.value()
                    }
                } else {
                    None
                };

                // Enable mul_fixed_short selector on final row
                self.q_mul_fixed_short.enable(&mut region, offset)?;

                // Assign final `y` to `y_p` column and return final point
                let y_var = region.assign_advice(
                    || "y_var",
                    self.super_config.y_p,
                    offset,
                    || y_val.ok_or(Error::SynthesisError),
                )?;

                Ok(EccPoint {
                    x: magnitude_mul.x,
                    y: CellValue::new(y_var, y_val),
                })
            },
        )?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;

            let base: super::OrchardFixedBases = base.clone().into();

            let scalar = scalar
                .magnitude
                .zip(scalar.sign.value())
                .map(|(magnitude, sign)| {
                    let sign = if sign == pallas::Base::one() {
                        pallas::Scalar::one()
                    } else if sign == -pallas::Base::one() {
                        -pallas::Scalar::one()
                    } else {
                        panic!("Sign should be 1 or -1.")
                    };
                    magnitude * sign
                });
            let real_mul = scalar.map(|scalar| base.generator() * scalar);
            let result = result.point();

            if let (Some(real_mul), Some(result)) = (real_mul, result) {
                assert_eq!(real_mul.to_affine(), result);
            }
        }

        Ok((result, scalar))
    }
}

#[cfg(test)]
pub mod tests {
    use group::Curve;
    use halo2::{circuit::Layouter, plonk::Error};
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use crate::circuit::gadget::ecc::{chip::EccChip, FixedPointShort, Point};
    use crate::constants::load::ValueCommitV;

    #[allow(clippy::op_ref)]
    pub fn test_mul_fixed_short(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        // value_commit_v
        let value_commit_v = ValueCommitV::get();
        let base_val = value_commit_v.generator;
        let value_commit_v = FixedPointShort::from_inner(chip.clone(), value_commit_v);

        fn constrain_equal(
            chip: EccChip,
            mut layouter: impl Layouter<pallas::Base>,
            base_val: pallas::Affine,
            scalar_val: pallas::Scalar,
            result: Point<pallas::Affine, EccChip>,
        ) -> Result<(), Error> {
            let expected = Point::new(
                chip,
                layouter.namespace(|| "expected point"),
                Some((base_val * scalar_val).to_affine()),
            )?;
            result.constrain_equal(layouter.namespace(|| "constrain result"), &expected)
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed_short = pallas::Scalar::zero();
            let (result, _) = value_commit_v.mul(
                layouter.namespace(|| "mul by zero"),
                Some(scalar_fixed_short),
            )?;

            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "mul by zero"),
                base_val,
                scalar_fixed_short,
                result,
            )?;
        }

        // Random [a]B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(rand::random::<u64>());
            let mut sign = pallas::Scalar::one();
            if rand::random::<bool>() {
                sign = -sign;
            }
            let scalar_fixed_short = sign * &scalar_fixed_short;
            let (result, _) = value_commit_v.mul(
                layouter.namespace(|| "random short scalar"),
                Some(scalar_fixed_short),
            )?;

            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "random [a]B"),
                base_val,
                scalar_fixed_short,
                result,
            )?;
        }

        // [2^64 - 1]B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);
            let (result, _) = value_commit_v.mul(
                layouter.namespace(|| "[2^64 - 1]B"),
                Some(scalar_fixed_short),
            )?;

            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "[2^64 - 1]B"),
                base_val,
                scalar_fixed_short,
                result,
            )?;
        }

        // [-(2^64 - 1)]B
        {
            let scalar_fixed_short = -pallas::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);
            let (result, _) = value_commit_v.mul(
                layouter.namespace(|| "-[2^64 - 1]B"),
                Some(scalar_fixed_short),
            )?;

            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "[-2^64 - 1]B"),
                base_val,
                scalar_fixed_short,
                result,
            )?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333334 in octal.
        // [0xB6DB_6DB6_DB6D_B6DC] B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(0xB6DB_6DB6_DB6D_B6DCu64);

            let (result, _) = value_commit_v.mul(
                layouter.namespace(|| "mul with double"),
                Some(scalar_fixed_short),
            )?;

            constrain_equal(
                chip,
                layouter.namespace(|| "mul with double"),
                base_val,
                scalar_fixed_short,
                result,
            )?;
        }

        Ok(())
    }
}
