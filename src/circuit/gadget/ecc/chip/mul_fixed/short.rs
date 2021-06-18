use std::array;

use super::super::{copy, CellValue, EccConfig, EccPoint, EccScalarFixedShort, Var};
use crate::constants::ValueCommitV;

use halo2::{
    circuit::Region,
    plonk::{ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use pasta_curves::pallas;

pub struct Config<const NUM_WINDOWS: usize> {
    // Selector used for fixed-base scalar mul with short signed exponent.
    q_mul_fixed_short: Selector,
    super_config: super::Config<NUM_WINDOWS>,
}

impl<const NUM_WINDOWS: usize> From<&EccConfig> for Config<NUM_WINDOWS> {
    fn from(config: &EccConfig) -> Self {
        Self {
            q_mul_fixed_short: config.q_mul_fixed_short,
            super_config: config.into(),
        }
    }
}

impl<const NUM_WINDOWS: usize> Config<NUM_WINDOWS> {
    // We reuse the constraints in the `mul_fixed` gate so exclude them here.
    // Here, we add some new constraints specific to the short signed case.
    pub(crate) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
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

    pub fn assign_region(
        &self,
        scalar: &EccScalarFixedShort,
        base: &ValueCommitV,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccPoint, Error> {
        // Copy the scalar decomposition
        self.super_config
            .copy_scalar(region, offset, &scalar.into())?;

        let (acc, mul_b) = self.super_config.assign_region_inner(
            region,
            offset,
            &scalar.into(),
            base.clone().into(),
            self.super_config.mul_fixed,
        )?;

        // Add to the cumulative sum to get `[magnitude]B`.
        let magnitude_mul = self.super_config.add_config.assign_region(
            &mul_b,
            &acc,
            offset + NUM_WINDOWS,
            region,
        )?;

        // Increase offset by 1 after complete addition
        let offset = offset + 1;

        // Assign sign to `window` column
        let sign = copy(
            region,
            || "sign",
            self.super_config.window,
            offset + NUM_WINDOWS,
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
        self.q_mul_fixed_short
            .enable(region, offset + NUM_WINDOWS)?;

        // Assign final `x, y` to `x_p, y_p` columns and return final point
        let x_val = magnitude_mul.x.value();
        let x_var = region.assign_advice(
            || "x_var",
            self.super_config.x_p,
            offset + NUM_WINDOWS,
            || x_val.ok_or(Error::SynthesisError),
        )?;
        let y_var = region.assign_advice(
            || "y_var",
            self.super_config.y_p,
            offset + NUM_WINDOWS,
            || y_val.ok_or(Error::SynthesisError),
        )?;

        let result = EccPoint {
            x: CellValue::new(x_var, x_val),
            y: CellValue::new(y_var, y_val),
        };

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

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use ff::PrimeFieldBits;
    use halo2::{circuit::Layouter, plonk::Error};
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use crate::circuit::gadget::ecc::{chip::EccChip, FixedPointShort, ScalarFixedShort};
    use crate::constants::load::ValueCommitV;

    #[allow(clippy::op_ref)]
    pub fn test_mul_fixed_short(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error>
    where
        pallas::Scalar: PrimeFieldBits,
    {
        // value_commit_v
        let value_commit_v = ValueCommitV::get();
        let value_commit_v = FixedPointShort::from_inner(chip.clone(), value_commit_v);

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = pallas::Scalar::zero();
            let scalar_fixed = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul"), &scalar_fixed)?;
        }

        // Random [a]B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(rand::random::<u64>());
            let mut sign = pallas::Scalar::one();
            if rand::random::<bool>() {
                sign = -sign;
            }
            let scalar_fixed_short = sign * &scalar_fixed_short;

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // [2^64 - 1]B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // [-(2^64 - 1)]B
        {
            let scalar_fixed_short = -pallas::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333334 in octal.
        // [0xB6DB_6DB6_DB6D_B6DC] B
        {
            let scalar_fixed_short = pallas::Scalar::from_u64(0xB6DB_6DB6_DB6D_B6DCu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip,
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        Ok(())
    }
}
