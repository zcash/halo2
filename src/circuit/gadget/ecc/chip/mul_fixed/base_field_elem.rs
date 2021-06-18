use super::super::{EccBaseFieldElemFixed, EccConfig, EccPoint, OrchardFixedBasesFull};

use crate::{
    circuit::gadget::utilities::{copy, CellValue, Var},
    constants::{self, util::decompose_scalar_fixed, NUM_WINDOWS},
};
use halo2::{
    circuit::Region,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use arrayvec::ArrayVec;

pub struct Config {
    base_field_fixed: Column<Fixed>,
    super_config: super::Config<{ constants::NUM_WINDOWS }>,
}

impl From<&EccConfig> for Config {
    fn from(config: &EccConfig) -> Self {
        Self {
            base_field_fixed: config.base_field_fixed,
            super_config: config.into(),
        }
    }
}

impl Config {
    pub fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        // Decompose the base field element α into three-bit windows
        // using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
        // for α = a_0 + (2^3) a_1 + ... + (2^{3(n-1)}) a_{n-1}.
        // For a Pallas base field element (255 bits), n = 255 / 3 = 85.
        //
        // We set z_0 = α, which implies:
        //      z_1 = (α - a_0) / 2^3, (subtract the lowest 3 bits)
        //          = a_1 + (2^3) a_2 + ... + 2^{3(n-1)} a_{n-1},
        //      z_2 = (z_1 - a_1) / 2^3
        //          = a_2 + (2^3) a_3 + ... + 2^{3(n-2)} a_{n-1},
        //      ...,
        //      z_{n-1} = a_{n-1}
        //      z_n = (z_{n-1} - a_{n-1}) / 2^3
        //          = 0.
        //
        // This gate checks that each a_i = z_i - z_{i+1} * (2^3) is within
        // 3 bits.
        meta.create_gate("Decompose base field element", |meta| {
            // This gate is activated when base_field_fixed = 1
            let fixed_is_one = {
                let base_field_fixed = meta.query_fixed(self.base_field_fixed, Rotation::cur());
                let two = Expression::Constant(pallas::Base::from_u64(2));
                base_field_fixed.clone() * (two - base_field_fixed)
            };

            let z_cur = meta.query_advice(self.super_config.window, Rotation::cur());
            let z_next = meta.query_advice(self.super_config.window, Rotation::next());

            //    z_{i+1} = (z_i - a_i) / (2^3)
            // => a_i = z_i - (z_{i+1} * (2^3))
            let word = z_cur - z_next * pallas::Base::from_u64(constants::H as u64);

            // (word - 7) * (word - 6) * ... * (word - 1) * word = 0
            let range_check =
                (0..constants::H).fold(Expression::Constant(pallas::Base::one()), |acc, i| {
                    acc * (word.clone() - Expression::Constant(pallas::Base::from_u64(i as u64)))
                });

            vec![fixed_is_one * range_check]
        });

        meta.create_gate("x_p, y_p checks for BaseFieldElemFixed", |meta| {
            // This gate is activated when base_field_fixed = 1
            let fixed_is_one = {
                let base_field_fixed = meta.query_fixed(self.base_field_fixed, Rotation::cur());
                let two = Expression::Constant(pallas::Base::from_u64(2));
                base_field_fixed.clone() * (base_field_fixed - two)
            };

            let z_cur = meta.query_advice(self.super_config.window, Rotation::cur());
            let z_next = meta.query_advice(self.super_config.window, Rotation::next());
            let window = z_cur - z_next * pallas::Base::from_u64(constants::H as u64);
            self.super_config.coords_check(meta, fixed_is_one, window)
        });

        // Check that we get z_85 = 0 as the final output of the running sum.
        meta.create_gate("z_85 = 0", |meta| {
            // This gate is activated when base_field_fixed = 2
            let fixed_is_two = {
                let base_field_fixed = meta.query_fixed(self.base_field_fixed, Rotation::cur());
                let one = Expression::Constant(pallas::Base::one());
                base_field_fixed.clone() * (base_field_fixed - one)
            };

            let z_85 = meta.query_advice(self.super_config.window, Rotation::cur());

            vec![fixed_is_two * z_85]
        });
    }

    pub fn assign_region(
        &self,
        scalar: CellValue<pallas::Base>,
        base: OrchardFixedBasesFull,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccPoint, Error> {
        // Decompose scalar
        let scalar = self.decompose_base_field_elem(scalar, offset, region)?;

        let (acc, mul_b) = self.super_config.assign_region_inner(
            region,
            offset,
            &(&scalar).into(),
            base.into(),
            self.base_field_fixed,
        )?;

        // Increase offset by 1 because the running sum decomposition takes
        // up 86 rows (1 more than the number of windows.)
        let offset = offset + 1;

        // Add to the accumulator and return the final result as `[scalar]B`.
        let result = self.super_config.add_config.assign_region(
            &mul_b,
            &acc,
            offset + NUM_WINDOWS,
            region,
        )?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;

            let base: super::OrchardFixedBases = base.into();
            let scalar = &scalar
                .base_field_elem()
                .value()
                .map(|scalar| pallas::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
            let real_mul = scalar.map(|scalar| base.generator() * scalar);
            let result = result.point();

            if let (Some(real_mul), Some(result)) = (real_mul, result) {
                assert_eq!(real_mul.to_affine(), result);
            }
        }

        Ok(result)
    }

    fn decompose_base_field_elem(
        &self,
        base_field_elem: CellValue<pallas::Base>,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccBaseFieldElemFixed, Error> {
        // Decompose base field element into 3-bit words.
        let words: Vec<Option<u8>> = {
            let words = base_field_elem.value().map(|base_field_elem| {
                decompose_scalar_fixed::<pallas::Base>(
                    base_field_elem,
                    constants::L_ORCHARD_BASE,
                    constants::FIXED_BASE_WINDOW_SIZE,
                )
            });

            if let Some(words) = words {
                words.into_iter().map(Some).collect()
            } else {
                vec![None; constants::NUM_WINDOWS]
            }
        };

        // Initialize empty ArrayVec to store running sum values [z_1, ..., z_85].
        let mut running_sum: ArrayVec<CellValue<pallas::Base>, { constants::NUM_WINDOWS }> =
            ArrayVec::new();

        // Assign running sum `z_i`, i = 0..=n, where z_{i+1} = (z_i - a_i) / (2^3)
        // and `z_0` is initialized as `base_field_elem`.
        let mut z = copy(
            region,
            || "z_0 = base_field_elem",
            self.super_config.window,
            offset,
            &base_field_elem,
            &self.super_config.perm,
        )?;

        for idx in 0..words.len() {
            region.assign_fixed(
                || "Decomposition check",
                self.base_field_fixed,
                offset + idx,
                || Ok(pallas::Base::from_u64(1)),
            )?;
        }

        let offset = offset + 1;

        let eight_inv = pallas::Base::TWO_INV.square() * pallas::Base::TWO_INV;
        for (idx, word) in words.iter().enumerate() {
            // z_next = (z_cur - word) / (2^3)
            let z_next = {
                let word = word.map(|word| pallas::Base::from_u64(word as u64));
                let z_next_val = z
                    .value()
                    .zip(word)
                    .map(|(z_cur_val, word)| (z_cur_val - word) * eight_inv);
                let cell = region.assign_advice(
                    || format!("word {:?}", idx),
                    self.super_config.window,
                    offset + idx,
                    || z_next_val.ok_or(Error::SynthesisError),
                )?;
                CellValue::new(cell, z_next_val)
            };

            // Update `z`.
            z = z_next;
            running_sum.push(z);
        }

        let offset = offset + words.len() - 1;
        region.assign_fixed(
            || "Check z_85 = 0",
            self.base_field_fixed,
            offset,
            || Ok(pallas::Base::from_u64(2)),
        )?;

        Ok(EccBaseFieldElemFixed {
            base_field_elem,
            running_sum,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use halo2::{
        circuit::{Chip, Layouter},
        plonk::Error,
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use crate::circuit::gadget::{
        ecc::{
            chip::{EccChip, OrchardFixedBasesFull},
            FixedPoint,
        },
        utilities::{CellValue, UtilitiesInstructions},
    };
    use crate::constants;

    pub fn test_mul_fixed_base_field(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        impl UtilitiesInstructions<pallas::Base> for EccChip {
            type Var = CellValue<pallas::Base>;
        }

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
    ) -> Result<(), Error> {
        let column = chip.config().advices[0];

        // [a]B
        {
            let scalar_fixed = pallas::Base::rand();
            let scalar_fixed = chip.load_private(
                layouter.namespace(|| "witness random base field element"),
                column,
                Some(scalar_fixed),
            )?;

            base.mul_base_field_elem(layouter.namespace(|| "mul"), scalar_fixed)?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333333333333333333333333333333333333333333333333333333333333333333334 in octal.
        // (There is another *non-canonical* sequence
        // 5333333333333333333333333333333333333333332711161673731021062440252244051273333333333 in octal.)
        {
            let h = pallas::Base::from_u64(constants::H as u64);
            let scalar_fixed = "1333333333333333333333333333333333333333333333333333333333333333333333333333333333334"
                        .chars()
                        .fold(pallas::Base::zero(), |acc, c| {
                            acc * &h + &pallas::Base::from_u64(c.to_digit(8).unwrap().into())
                        });

            let scalar_fixed = chip.load_private(
                layouter.namespace(|| "mul with double"),
                column,
                Some(scalar_fixed),
            )?;

            base.mul_base_field_elem(layouter.namespace(|| "mul with double"), scalar_fixed)?;
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = pallas::Base::zero();
            let scalar_fixed = chip.load_private(
                layouter.namespace(|| "mul by zero"),
                column,
                Some(scalar_fixed),
            )?;
            base.mul_base_field_elem(layouter.namespace(|| "mul by zero"), scalar_fixed)?;
        }

        // [-1]B is the largest base field element
        {
            let scalar_fixed = -pallas::Base::one();
            let scalar_fixed = chip.load_private(
                layouter.namespace(|| "mul by -1"),
                column,
                Some(scalar_fixed),
            )?;
            base.mul_base_field_elem(layouter.namespace(|| "mul by -1"), scalar_fixed)?;
        }

        Ok(())
    }
}
