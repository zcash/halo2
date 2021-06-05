use super::super::{EccConfig, EccPoint, EccScalarFixed, OrchardFixedBasesFull};

use halo2::{circuit::Region, plonk::Error};
use pasta_curves::pallas;

pub struct Config<const NUM_WINDOWS: usize>(super::Config<NUM_WINDOWS>);

impl<const NUM_WINDOWS: usize> From<&EccConfig> for Config<NUM_WINDOWS> {
    fn from(config: &EccConfig) -> Self {
        Self(config.into())
    }
}

impl<const NUM_WINDOWS: usize> Config<NUM_WINDOWS> {
    pub fn assign_region(
        &self,
        scalar: &EccScalarFixed,
        base: OrchardFixedBasesFull,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccPoint, Error> {
        let (acc, mul_b) =
            self.0
                .assign_region_inner(region, offset, &scalar.into(), base.into())?;

        // Add to the accumulator and return the final result as `[scalar]B`.
        let result = self
            .0
            .add_config
            .assign_region(&mul_b, &acc, offset + NUM_WINDOWS, region)?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;
            use halo2::arithmetic::FieldExt;

            let base: super::OrchardFixedBases = base.into();
            let scalar = scalar
                .value
                .map(|scalar| pallas::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
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

    use crate::circuit::gadget::ecc::{
        chip::{EccChip, OrchardFixedBasesFull},
        FixedPoint, ScalarFixed,
    };
    use crate::constants;

    pub fn test_mul_fixed(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        // commit_ivk_r
        let commit_ivk_r = OrchardFixedBasesFull::CommitIvkR;
        let commit_ivk_r = FixedPoint::from_inner(chip.clone(), commit_ivk_r);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "commit_ivk_r"),
            commit_ivk_r,
        )?;

        // note_commit_r
        let note_commit_r = OrchardFixedBasesFull::NoteCommitR;
        let note_commit_r = FixedPoint::from_inner(chip.clone(), note_commit_r);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "note_commit_r"),
            note_commit_r,
        )?;

        // nullifier_k
        let nullifier_k = OrchardFixedBasesFull::NullifierK;
        let nullifier_k = FixedPoint::from_inner(chip.clone(), nullifier_k);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "nullifier_k"),
            nullifier_k,
        )?;

        // value_commit_r
        let value_commit_r = OrchardFixedBasesFull::ValueCommitR;
        let value_commit_r = FixedPoint::from_inner(chip.clone(), value_commit_r);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "value_commit_r"),
            value_commit_r,
        )?;

        // spend_auth_g
        let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
        let spend_auth_g = FixedPoint::from_inner(chip.clone(), spend_auth_g);
        test_single_base(chip, layouter.namespace(|| "spend_auth_g"), spend_auth_g)?;

        Ok(())
    }

    #[allow(clippy::op_ref)]
    fn test_single_base(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        base: FixedPoint<pallas::Affine, EccChip>,
    ) -> Result<(), Error>
    where
        pallas::Scalar: PrimeFieldBits,
    {
        // [a]B
        {
            let scalar_fixed = pallas::Scalar::rand();

            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;

            base.mul(layouter.namespace(|| "mul"), &scalar_fixed)?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333333333333333333333333333333333333333333333333333333333333333333334 in octal.
        // (There is another *non-canonical* sequence
        // 5333333333333333333333333333333333333333332711161673731021062440252244051273333333333 in octal.)
        {
            let h = pallas::Scalar::from_u64(constants::H as u64);
            let scalar_fixed = "1333333333333333333333333333333333333333333333333333333333333333333333333333333333334"
                        .chars()
                        .fold(pallas::Scalar::zero(), |acc, c| {
                            acc * &h + &pallas::Scalar::from_u64(c.to_digit(8).unwrap().into())
                        });

            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;

            base.mul(layouter.namespace(|| "mul with double"), &scalar_fixed)?;
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = pallas::Scalar::zero();
            let scalar_fixed = ScalarFixed::new(
                chip,
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;
            base.mul(layouter.namespace(|| "mul by zero"), &scalar_fixed)?;
        }

        Ok(())
    }
}
