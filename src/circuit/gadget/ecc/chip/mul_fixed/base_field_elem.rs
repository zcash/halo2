use super::super::{EccBaseFieldElemFixed, EccConfig, EccPoint, OrchardFixedBasesFull};
use super::H_BASE;

use crate::{
    circuit::gadget::utilities::{
        bitrange_subset, copy, lookup_range_check::LookupRangeCheckConfig, range_check, CellValue,
        Var,
    },
    constants::{self, util::decompose_word, T_P},
    primitives::sinsemilla,
};
use halo2::{
    circuit::{Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use arrayvec::ArrayVec;

pub struct Config {
    base_field_fixed_mul: Selector,
    base_field_fixed_canon: Selector,
    canon_advices: [Column<Advice>; 3],
    lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    super_config: super::Config<{ constants::NUM_WINDOWS }>,
}

impl From<&EccConfig> for Config {
    fn from(config: &EccConfig) -> Self {
        let config = Self {
            base_field_fixed_mul: config.base_field_fixed_mul,
            base_field_fixed_canon: config.base_field_fixed_canon,
            canon_advices: [config.advices[6], config.advices[7], config.advices[8]],
            lookup_config: config.lookup_config.clone(),
            super_config: config.into(),
        };

        let add_incomplete_advices = config.super_config.add_incomplete_config.advice_columns();
        for canon_advice in config.canon_advices.iter() {
            assert!(
                !add_incomplete_advices.contains(canon_advice),
                "Deconflict canon_advice columns with incomplete addition columns."
            );
        }

        config
    }
}

impl Config {
    pub fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        // Decompose the base field element α into three-bit windows
        // using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
        // for α = a_0 + 2^3 a_1 + ... + 2^{3*84} a_84.
        //
        // We set z_0 = α, which implies:
        //      z_1 = (α - a_0) / 2^3, (subtract the lowest 3 bits)
        //          = a_1 + 2^3 a_2 + ... + 2^{3*83} a_84,
        //      z_2 = (z_1 - a_1) / 2^3
        //          = a_2 + 2^3 a_3 + ... + 2^{3*82} a_84,
        //      ...,
        //      z_84 = a_84
        //      z_n = (z_84 - a_84) / 2^3
        //          = 0.
        //
        // This gate checks that each a_i = z_i - z_{i+1} * 2^3 is within
        // 3 bits.
        //
        // This gate also checks that this window uses the correct y_p and
        // interpolated x_p.
        meta.create_gate("Decompose base field element", |meta| {
            let base_field_fixed_mul = meta.query_selector(self.base_field_fixed_mul);

            let z_cur = meta.query_advice(self.super_config.window, Rotation::cur());
            let z_next = meta.query_advice(self.super_config.window, Rotation::next());

            //    z_{i+1} = (z_i - a_i) / 2^3
            // => a_i = z_i - z_{i+1} * 2^3
            let word = z_cur - z_next * pallas::Base::from_u64(constants::H as u64);

            // (word - 7) * (word - 6) * ... * (word - 1) * word = 0
            let range_check = range_check(word.clone(), constants::H);

            self.super_config
                .coords_check(meta, base_field_fixed_mul.clone(), word)
                .into_iter()
                .chain(Some((
                    "Decomposition range check",
                    base_field_fixed_mul * range_check,
                )))
        });

        // Check that we get z_85 = 0 as the final output of the three-bit decomposition running sum.
        // Also check that the base field element is canonical.
        meta.create_gate("Canonicity checks", |meta| {
            let base_field_fixed_canon = meta.query_selector(self.base_field_fixed_canon);

            let alpha = meta.query_advice(self.canon_advices[0], Rotation::prev());
            // z_85_alpha is constrained to be zero in this gate.
            let z_85_alpha = meta.query_advice(self.canon_advices[1], Rotation::prev());
            // The last three bits of α.
            let z_84_alpha = meta.query_advice(self.canon_advices[2], Rotation::prev());

            // Decompose α into three pieces, in little-endian order:
            //            α = α_0 (252 bits)  || α_1 (2 bits) || α_2 (1 bit).
            //
            // α_0 is derived, not witnessed.
            let alpha_0 = {
                let two_pow_252 = pallas::Base::from_u128(1 << 126).square();
                alpha - (z_84_alpha.clone() * two_pow_252)
            };
            let alpha_1 = meta.query_advice(self.canon_advices[1], Rotation::cur());
            let alpha_2 = meta.query_advice(self.canon_advices[2], Rotation::cur());

            let alpha_0_prime = meta.query_advice(self.canon_advices[0], Rotation::cur());
            let z_13_alpha_0_prime = meta.query_advice(self.canon_advices[0], Rotation::next());
            let z_44_alpha = meta.query_advice(self.canon_advices[1], Rotation::next());
            let z_43_alpha = meta.query_advice(self.canon_advices[2], Rotation::next());

            let decomposition_checks = {
                // Range-constrain α_1 to be 2 bits
                let alpha_1_range_check = range_check(alpha_1.clone(), 1 << 2);
                // Boolean-constrain α_2
                let alpha_2_range_check = range_check(alpha_2.clone(), 1 << 1);
                // Check that α_1 + 2^2 α_2 = z_84_alpha
                let z_84_alpha_check = z_84_alpha.clone()
                    - (alpha_1.clone() + alpha_2.clone() * pallas::Base::from_u64(1 << 2));

                std::iter::empty()
                    .chain(Some(("alpha_1_range_check", alpha_1_range_check)))
                    .chain(Some(("alpha_2_range_check", alpha_2_range_check)))
                    .chain(Some(("z_84_alpha_check", z_84_alpha_check)))
            };

            // Check α_0_prime = α_0 + 2^130 - t_p
            let alpha_0_prime_check = {
                let two_pow_130 = Expression::Constant(pallas::Base::from_u128(1 << 65).square());
                let t_p = Expression::Constant(pallas::Base::from_u128(T_P));
                alpha_0_prime - (alpha_0 + two_pow_130 - t_p)
            };

            // We want to enforce canonicity of a 255-bit base field element, α.
            // That is, we want to check that 0 ≤ α < p, where p is Pallas base
            // field modulus p = 2^254 + t_p
            //                 = 2^254 + 45560315531419706090280762371685220353.
            // Note that t_p < 2^130.
            //
            // α has been decomposed into three pieces in little-endian order:
            //            α = α_0 (252 bits)  || α_1 (2 bits) || α_2 (1 bit).
            //              = α_0 + 2^252 α_1 + 2^254 α_2.
            //
            // If the MSB α_2 = 1, then:
            //      - α_2 = 1 => α_1 = 0, and
            //      - α_2 = 1 => α_0 < t_p. To enforce this:
            //          - α_2 = 1 => 0 ≤ α_0 < 2^130
            //                - alpha_0_hi_120 = 0 (constrain α_0 to be 132 bits)
            //                - a_43 = 0 or 1 (constrain α_0[130..=131] to be 0)
            //          - α_2 = 1 => 0 ≤ α_0 + 2^130 - t_p < 2^130
            //                    => 13 ten-bit lookups of α_0 + 2^130 - t_p
            //                    => z_13_alpha_0_prime = 0
            //
            let canon_checks = {
                // alpha_0_hi_120 = z_44 - 2^120 z_84
                let alpha_0_hi_120 = {
                    let two_pow_120 =
                        Expression::Constant(pallas::Base::from_u128(1 << 60).square());
                    z_44_alpha.clone() - z_84_alpha * two_pow_120
                };
                // a_43 = z_43 - (2^3)z_44
                let a_43 = z_43_alpha - z_44_alpha * *H_BASE;

                std::iter::empty()
                    .chain(Some(("MSB = 1 => alpha_1 = 0", alpha_2.clone() * alpha_1)))
                    .chain(Some((
                        "MSB = 1 => alpha_0_hi_120 = 0",
                        alpha_2.clone() * alpha_0_hi_120,
                    )))
                    .chain(Some((
                        "MSB = 1 => a_43 = 0 or 1",
                        alpha_2.clone() * range_check(a_43, 2),
                    )))
                    .chain(Some((
                        "MSB = 1 => z_13_alpha_0_prime = 0",
                        alpha_2 * z_13_alpha_0_prime,
                    )))
            };

            canon_checks
                .chain(decomposition_checks)
                .chain(Some(("z_85_alpha = 0", z_85_alpha)))
                .chain(Some(("alpha_0_prime check", alpha_0_prime_check)))
                .map(move |(name, poly)| (name, base_field_fixed_canon.clone() * poly))
        });
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        scalar: CellValue<pallas::Base>,
        base: OrchardFixedBasesFull,
    ) -> Result<EccPoint, Error> {
        let (scalar, acc, mul_b) = layouter.assign_region(
            || "Base-field elem fixed-base mul (incomplete addition)",
            |mut region| {
                let offset = 0;

                // Decompose scalar
                let scalar = self.decompose_base_field_elem(scalar, offset, &mut region)?;

                let (acc, mul_b) = self.super_config.assign_region_inner(
                    &mut region,
                    offset,
                    &(&scalar).into(),
                    base.into(),
                    self.base_field_fixed_mul,
                )?;

                Ok((scalar, acc, mul_b))
            },
        )?;

        // Add to the accumulator and return the final result as `[scalar]B`.
        let result = layouter.assign_region(
            || "Base-field elem fixed-base mul (complete addition)",
            |mut region| {
                self.super_config
                    .add_config
                    .assign_region(&mul_b, &acc, 0, &mut region)
            },
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

        // We want to enforce canonicity of a 255-bit base field element, α.
        // That is, we want to check that 0 ≤ α < p, where p is Pallas base
        // field modulus p = 2^254 + t_p
        //                 = 2^254 + 45560315531419706090280762371685220353.
        // Note that t_p < 2^130.
        //
        // α has been decomposed into three pieces in little-endian order:
        //            α = α_0 (252 bits)  || α_1 (2 bits) || α_2 (1 bit).
        //              = α_0 + 2^252 α_1 + 2^254 α_2.
        //
        // If the MSB α_2 = 1, then:
        //      - α_2 = 1 => α_1 = 0, and
        //      - α_2 = 1 => α_0 < t_p. To enforce this:
        //      - α_2 = 1 => 0 ≤ α_0 < 2^130
        //                => 13 ten-bit lookups of α_0
        //      - α_2 = 1 => 0 ≤ α_0 + 2^130 - t_p < 2^130
        //                => 13 ten-bit lookups of α_0 + 2^130 - t_p
        //                => z_13_alpha_0_prime = 0
        //
        let (alpha, running_sum) = (scalar.base_field_elem, &scalar.running_sum);
        let z_43_alpha = running_sum[42];
        let z_44_alpha = running_sum[43];
        let z_84_alpha = running_sum[83];
        let z_85_alpha = running_sum[84];

        // α_0 = α - z_84_alpha * 2^252
        let alpha_0 = alpha
            .value()
            .zip(z_84_alpha.value())
            .map(|(alpha, z_84_alpha)| {
                let two_pow_252 = pallas::Base::from_u128(1 << 126).square();
                alpha - z_84_alpha * two_pow_252
            });

        let (alpha_0_prime, z_13_alpha_0_prime) = {
            // alpha_0_prime = alpha + 2^130 - t_p.
            let alpha_0_prime = alpha_0.map(|alpha_0| {
                let two_pow_130 = pallas::Base::from_u128(1 << 65).square();
                let t_p = pallas::Base::from_u128(T_P);
                alpha_0 + two_pow_130 - t_p
            });
            let (alpha_0_prime, zs) = self.lookup_config.witness_check(
                layouter.namespace(|| "Lookup range check alpha_0 + 2^130 - t_p"),
                alpha_0_prime,
                13,
                false,
            )?;

            (alpha_0_prime, zs[13])
        };

        layouter.assign_region(
            || "Canonicity checks",
            |mut region| {
                let perm = &self.super_config.perm;

                // Activate canonicity check gate
                self.base_field_fixed_canon.enable(&mut region, 1)?;

                // Offset 0
                {
                    let offset = 0;

                    // Copy α
                    copy(
                        &mut region,
                        || "Copy α",
                        self.canon_advices[0],
                        offset,
                        &alpha,
                        perm,
                    )?;

                    // z_85_alpha is constrained to be zero in the custom gate.
                    copy(
                        &mut region,
                        || "Copy z_85_alpha",
                        self.canon_advices[1],
                        offset,
                        &z_85_alpha,
                        perm,
                    )?;

                    // z_84_alpha = the top three bits of alpha.
                    copy(
                        &mut region,
                        || "Copy z_84_alpha",
                        self.canon_advices[2],
                        offset,
                        &z_84_alpha,
                        perm,
                    )?;
                }

                // Offset 1
                {
                    let offset = 1;
                    // Copy alpha_0_prime = alpha_0 + 2^130 - t_p.
                    // We constrain this in the custom gate to be derived correctly.
                    copy(
                        &mut region,
                        || "Copy α_0 + 2^130 - t_p",
                        self.canon_advices[0],
                        offset,
                        &alpha_0_prime,
                        perm,
                    )?;

                    // Decompose α into three pieces,
                    //     α = α_0 (252 bits)  || α_1 (2 bits) || α_2 (1 bit).
                    // We only need to witness α_1 and α_2. α_0 is derived in the gate.
                    // Witness α_1 = α[252..=253]
                    let alpha_1 = alpha.value().map(|alpha| bitrange_subset(alpha, 252..254));
                    region.assign_advice(
                        || "α_1 = α[252..=253]",
                        self.canon_advices[1],
                        offset,
                        || alpha_1.ok_or(Error::SynthesisError),
                    )?;

                    // Witness the MSB α_2 = α[254]
                    let alpha_2 = alpha.value().map(|alpha| bitrange_subset(alpha, 254..255));
                    region.assign_advice(
                        || "α_2 = α[254]",
                        self.canon_advices[2],
                        offset,
                        || alpha_2.ok_or(Error::SynthesisError),
                    )?;
                }

                // Offset 2
                {
                    let offset = 2;
                    // Copy z_13_alpha_0_prime
                    copy(
                        &mut region,
                        || "Copy z_13_alpha_0_prime",
                        self.canon_advices[0],
                        offset,
                        &z_13_alpha_0_prime,
                        perm,
                    )?;

                    // Copy z_44_alpha
                    copy(
                        &mut region,
                        || "Copy z_44_alpha",
                        self.canon_advices[1],
                        offset,
                        &z_44_alpha,
                        perm,
                    )?;

                    // Copy z_43_alpha
                    copy(
                        &mut region,
                        || "Copy z_43_alpha",
                        self.canon_advices[2],
                        offset,
                        &z_43_alpha,
                        perm,
                    )?;
                }

                Ok(())
            },
        )?;

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
                decompose_word::<pallas::Base>(
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
                    || format!("z_{:?}", idx + 1),
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

        Ok(EccBaseFieldElemFixed {
            base_field_elem,
            running_sum,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use group::Curve;
    use halo2::{
        circuit::{Chip, Layouter},
        plonk::Error,
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use crate::circuit::gadget::{
        ecc::{
            chip::{EccChip, OrchardFixedBasesFull},
            FixedPoint, Point,
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
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "commit_ivk_r"),
            FixedPoint::from_inner(chip.clone(), commit_ivk_r),
            commit_ivk_r.generator(),
        )?;

        // note_commit_r
        let note_commit_r = OrchardFixedBasesFull::NoteCommitR;
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "note_commit_r"),
            FixedPoint::from_inner(chip.clone(), note_commit_r),
            note_commit_r.generator(),
        )?;

        // nullifier_k
        let nullifier_k = OrchardFixedBasesFull::NullifierK;
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "nullifier_k"),
            FixedPoint::from_inner(chip.clone(), nullifier_k),
            nullifier_k.generator(),
        )?;

        // value_commit_r
        let value_commit_r = OrchardFixedBasesFull::ValueCommitR;
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "value_commit_r"),
            FixedPoint::from_inner(chip.clone(), value_commit_r),
            value_commit_r.generator(),
        )?;

        // spend_auth_g
        let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "spend_auth_g"),
            FixedPoint::from_inner(chip, spend_auth_g),
            spend_auth_g.generator(),
        )?;

        Ok(())
    }

    #[allow(clippy::op_ref)]
    fn test_single_base(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        base: FixedPoint<pallas::Affine, EccChip>,
        base_val: pallas::Affine,
    ) -> Result<(), Error> {
        let column = chip.config().advices[0];
        fn constrain_equal(
            chip: EccChip,
            mut layouter: impl Layouter<pallas::Base>,
            base_val: pallas::Affine,
            scalar_val: pallas::Base,
            result: Point<pallas::Affine, EccChip>,
        ) -> Result<(), Error> {
            // Move scalar from base field into scalar field (which always fits for Pallas).
            let scalar = pallas::Scalar::from_bytes(&scalar_val.to_bytes()).unwrap();
            let expected = Point::new(
                chip,
                layouter.namespace(|| "expected point"),
                Some((base_val * scalar).to_affine()),
            )?;
            result.constrain_equal(layouter.namespace(|| "constrain result"), &expected)
        }

        // [a]B
        {
            let scalar_fixed = pallas::Base::rand();
            let result = {
                let scalar_fixed = chip.load_private(
                    layouter.namespace(|| "random base field element"),
                    column,
                    Some(scalar_fixed),
                )?;
                base.mul_base_field_elem(layouter.namespace(|| "random [a]B"), scalar_fixed)?
            };
            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "random [a]B"),
                base_val,
                scalar_fixed,
                result,
            )?;
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
            let result = {
                let scalar_fixed = chip.load_private(
                    layouter.namespace(|| "mul with double"),
                    column,
                    Some(scalar_fixed),
                )?;
                base.mul_base_field_elem(layouter.namespace(|| "mul with double"), scalar_fixed)?
            };
            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "mul with double"),
                base_val,
                scalar_fixed,
                result,
            )?;
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = pallas::Base::zero();
            let result = {
                let scalar_fixed =
                    chip.load_private(layouter.namespace(|| "zero"), column, Some(scalar_fixed))?;
                base.mul_base_field_elem(layouter.namespace(|| "mul by zero"), scalar_fixed)?
            };
            constrain_equal(
                chip.clone(),
                layouter.namespace(|| "mul by zero"),
                base_val,
                scalar_fixed,
                result,
            )?;
        }

        // [-1]B is the largest base field element
        {
            let scalar_fixed = -pallas::Base::one();
            let result = {
                let scalar_fixed =
                    chip.load_private(layouter.namespace(|| "-1"), column, Some(scalar_fixed))?;
                base.mul_base_field_elem(layouter.namespace(|| "mul by -1"), scalar_fixed)?
            };
            constrain_equal(
                chip,
                layouter.namespace(|| "mul by -1"),
                base_val,
                scalar_fixed,
                result,
            )?;
        }

        Ok(())
    }
}
