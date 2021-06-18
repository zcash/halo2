use super::super::{CellValue, EccConfig, EccScalarFixedShort, Var};
use crate::constants::{L_VALUE, NUM_WINDOWS_SHORT};
use halo2::{
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

pub struct Config {
    q_scalar_fixed_short: Selector,
    super_config: super::Config,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_scalar_fixed_short: ecc_config.q_scalar_fixed_short,
            super_config: ecc_config.into(),
        }
    }
}

impl Config {
    pub fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        // Check that sign is either 1 or -1.
        // Check that last window is either 0 or 1.
        meta.create_gate("Check sign and last window", |meta| {
            let q_scalar_fixed_short = meta.query_selector(self.q_scalar_fixed_short);
            let last_window = meta.query_advice(self.super_config.window, Rotation::prev());
            let sign = meta.query_advice(self.super_config.window, Rotation::cur());

            let one = Expression::Constant(pallas::Base::one());

            let last_window_check = last_window.clone() * (one.clone() - last_window);
            let sign_check = (one.clone() + sign.clone()) * (one - sign);

            vec![
                q_scalar_fixed_short.clone() * last_window_check,
                q_scalar_fixed_short * sign_check,
            ]
        });
    }
}

impl Config {
    #[allow(clippy::op_ref)]
    pub fn assign_region(
        &self,
        value: Option<pallas::Scalar>,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
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
            .decompose_scalar_fixed::<NUM_WINDOWS_SHORT, L_VALUE>(magnitude, offset, region)?;

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
            NUM_WINDOWS_SHORT,
            || sign.ok_or(Error::SynthesisError),
        )?;

        Ok(EccScalarFixedShort {
            magnitude,
            sign: CellValue::<pallas::Base>::new(sign_cell, sign),
            windows,
        })
    }
}
