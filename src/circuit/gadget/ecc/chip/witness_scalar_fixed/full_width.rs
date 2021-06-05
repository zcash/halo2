use super::super::{EccConfig, EccScalarFixed};
use crate::constants::{L_ORCHARD_SCALAR, NUM_WINDOWS};
use halo2::{circuit::Region, plonk::Error};
use pasta_curves::pallas;

pub struct Config(super::Config);

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self(ecc_config.into())
    }
}

impl Config {
    pub fn assign_region(
        &self,
        value: Option<pallas::Scalar>,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccScalarFixed, Error> {
        let windows = self
            .0
            .decompose_scalar_fixed::<NUM_WINDOWS, L_ORCHARD_SCALAR>(value, offset, region)?;

        Ok(EccScalarFixed { value, windows })
    }
}
