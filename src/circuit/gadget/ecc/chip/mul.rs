use super::{add, copy, CellValue, EccConfig, EccPoint, Var};
use crate::constants::{NUM_COMPLETE_BITS, T_Q};
use std::ops::{Deref, Range};

use ff::{PrimeField, PrimeFieldBits};
use halo2::{
    arithmetic::FieldExt,
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

use pasta_curves::pallas;

mod complete;
mod incomplete;

// Bits used in incomplete addition. k_{254} to k_{4} inclusive
const INCOMPLETE_LEN: usize = pallas::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS;
const INCOMPLETE_RANGE: Range<usize> = 0..INCOMPLETE_LEN;

// Bits k_{254} to k_{4} inclusive are used in incomplete addition.
// The `hi` half is k_{254} to k_{130} inclusive (length 125 bits).
const INCOMPLETE_HI_RANGE: Range<usize> = 0..(INCOMPLETE_LEN / 2);

// Bits k_{254} to k_{4} inclusive are used in incomplete addition.
// The `lo` half is k_{129} to k_{4} inclusive (length 126 bits).
const INCOMPLETE_LO_RANGE: Range<usize> = (INCOMPLETE_LEN / 2)..INCOMPLETE_LEN;

// Bits k_{3} to k_{1} inclusive are used in complete addition.
// Bit k_{0} is handled separately.
const COMPLETE_RANGE: Range<usize> = INCOMPLETE_LEN..(INCOMPLETE_LEN + NUM_COMPLETE_BITS);

pub struct Config {
    // Selector used to constrain the cells used in complete addition.
    q_mul_complete: Selector,
    // Selector used to check recovery of the original scalar after decomposition.
    q_mul_decompose_var: Selector,
    // Selector used to constrain the initialization of the running sum to be zero.
    q_init_z: Selector,
    // Advice column used to decompose scalar in complete addition.
    z_complete: Column<Advice>,
    // Advice column where the scalar is copied for use in the final recovery check.
    scalar: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration used in complete addition
    add_config: add::Config,
    // Configuration used for `hi` bits of the scalar
    hi_config: incomplete::Config,
    // Configuration used for `lo` bits of the scalar
    lo_config: incomplete::Config,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        let config = Self {
            q_mul_complete: ecc_config.q_mul_complete,
            q_mul_decompose_var: ecc_config.q_mul_decompose_var,
            q_init_z: ecc_config.q_init_z,
            z_complete: ecc_config.advices[9],
            scalar: ecc_config.advices[1],
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            hi_config: incomplete::Config::hi_config(ecc_config),
            lo_config: incomplete::Config::lo_config(ecc_config),
        };

        assert_eq!(
            config.hi_config.x_p, config.lo_config.x_p,
            "x_p is shared across hi and lo halves."
        );
        assert_eq!(
            config.hi_config.y_p, config.lo_config.y_p,
            "y_p is shared across hi and lo halves."
        );

        let add_config_advices = config.add_config.advice_columns();
        assert!(
            !add_config_advices.contains(&config.z_complete),
            "z_complete cannot overlap with complete addition columns."
        );
        assert!(
            !add_config_advices.contains(&config.hi_config.z),
            "hi_config z cannot overlap with complete addition columns."
        );

        config
    }
}

impl Config {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        self.hi_config.create_gate(meta);
        self.lo_config.create_gate(meta);

        let complete_config: complete::Config = self.into();
        complete_config.create_gate(meta);

        self.create_init_scalar_gate(meta);
        self.create_final_scalar_gate(meta);
    }

    /// Gate used to check that the running sum for scalar decomposition is initialized to zero.
    fn create_init_scalar_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("Initialize running sum for variable-base mul", |meta| {
            let q_init_z = meta.query_selector(self.q_init_z);
            let z = meta.query_advice(self.hi_config.z, Rotation::cur());

            vec![q_init_z * z]
        });
    }

    /// Gate used to check final scalar is recovered.
    fn create_final_scalar_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("Decompose scalar for variable-base mul", |meta| {
            let q_mul_decompose_var = meta.query_selector(self.q_mul_decompose_var);
            let scalar = meta.query_advice(self.scalar, Rotation::cur());
            let z_cur = meta.query_advice(self.z_complete, Rotation::cur());

            // The scalar field `F_q = 2^254 + t_q`.
            // -((2^127)^2) = -(2^254) = t_q (mod q)
            let t_q = -(pallas::Scalar::from_u128(1u128 << 127).square());
            let t_q = pallas::Base::from_bytes(&t_q.to_bytes()).unwrap();

            // Check that `k = scalar + t_q`
            vec![q_mul_decompose_var * (scalar + Expression::Constant(t_q) - z_cur)]
        });
    }

    pub(super) fn assign_region(
        &self,
        scalar: &CellValue<pallas::Base>,
        base: &EccPoint,
        offset: usize,
        region: &mut Region<'_, pallas::Base>,
    ) -> Result<EccPoint, Error> {
        // Decompose the scalar bitwise (big-endian bit order).
        let bits = decompose_for_scalar_mul(scalar.value());
        let bits_incomplete_hi = &bits[INCOMPLETE_HI_RANGE];
        let bits_incomplete_lo = &bits[INCOMPLETE_LO_RANGE];
        let lsb = bits[pallas::Scalar::NUM_BITS as usize - 1];

        // Initialize the accumulator `acc = [2]base`
        let acc = self
            .add_config
            .assign_region(&base, &base, offset, region)?;

        // Increase the offset by 1 after complete addition.
        let offset = offset + 1;

        // Initialize the running sum for scalar decomposition to zero
        let z_init = {
            // Constrain the initialization of `z` to equal zero.
            self.q_init_z.enable(region, offset)?;

            let z_val = pallas::Base::zero();
            let z_cell =
                region.assign_advice(|| "initial z", self.hi_config.z, offset, || Ok(z_val))?;

            Z(CellValue::new(z_cell, Some(z_val)))
        };

        // Increase the offset by 1 after initializing `z`.
        let offset = offset + 1;

        // Double-and-add (incomplete addition) for the `hi` half of the scalar decomposition
        let (x_a, y_a, zs_incomplete_hi) = self.hi_config.double_and_add(
            region,
            offset,
            &base,
            bits_incomplete_hi,
            (X(acc.x), Y(acc.y.value()), z_init),
        )?;

        // Double-and-add (incomplete addition) for the `lo` half of the scalar decomposition
        let z = &zs_incomplete_hi[zs_incomplete_hi.len() - 1];
        let (x_a, y_a, zs_incomplete_lo) = self.lo_config.double_and_add(
            region,
            offset,
            &base,
            bits_incomplete_lo,
            (x_a, y_a, *z),
        )?;

        // Move from incomplete addition to complete addition.
        // Inside incomplete::double_and_add, the offset was increase once after initialization
        // of the running sum.
        // Then, the final assignment of double-and-add was made on row + offset + 1.
        // Outside of incomplete addition, we must account for these offset increases by adding
        // 2 to the incomplete addition length.
        let offset = offset + INCOMPLETE_LO_RANGE.len() + 2;

        // Complete addition
        let (acc, zs_complete) = {
            let z = &zs_incomplete_lo[zs_incomplete_lo.len() - 1];
            let complete_config: complete::Config = self.into();
            // Bits used in complete addition. k_{3} to k_{1} inclusive
            // The LSB k_{0} is handled separately.
            let bits_complete = &bits[COMPLETE_RANGE];
            complete_config.assign_region(region, offset, bits_complete, base, x_a, y_a, *z)?
        };

        // Each iteration of the complete addition uses two rows.
        let offset = offset + COMPLETE_RANGE.len() * 2;

        // Process the least significant bit
        let z = &zs_complete[zs_complete.len() - 1];
        let (result, z_final) = self.process_lsb(region, offset, scalar, base, acc, lsb, *z)?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;

            let base = base.point();
            let scalar = scalar
                .value()
                .map(|scalar| pallas::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
            let real_mul = base.zip(scalar).map(|(base, scalar)| base * scalar);
            let result = result.point();

            if let (Some(real_mul), Some(result)) = (real_mul, result) {
                assert_eq!(real_mul.to_affine(), result);
            }
        }

        let mut zs = vec![z_init];
        zs.extend_from_slice(&zs_incomplete_hi);
        zs.extend_from_slice(&zs_incomplete_lo);
        zs.extend_from_slice(&zs_complete);
        zs.extend_from_slice(&[z_final]);

        // This reverses zs to give us [z_0, z_1, ..., z_{254}, z_{255}].
        zs.reverse();

        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_lsb(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        _scalar: &CellValue<pallas::Base>,
        base: &EccPoint,
        acc: EccPoint,
        lsb: Option<bool>,
        z: Z<pallas::Base>,
    ) -> Result<(EccPoint, Z<pallas::Base>), Error> {
        // Copy in the penultimate `z_1` value.
        copy(
            region,
            || "penultimate z_1 running sum",
            self.z_complete,
            offset,
            &z.0,
            &self.perm,
        )?;

        // Increase offset after copying in `z_1`.
        let offset = offset + 1;

        // Assign the final `z_0` value.
        let z = {
            let z_val = z.0.value().zip(lsb).map(|(z_val, lsb)| {
                pallas::Base::from_u64(2) * z_val + pallas::Base::from_u64(lsb as u64)
            });
            let z_cell = region.assign_advice(
                || "final z_0",
                self.z_complete,
                offset,
                || z_val.ok_or(Error::SynthesisError),
            )?;
            Z(CellValue::new(z_cell, z_val))
        };

        // Enforce that the final bit decomposition from `z_1` to `z_0` was done correctly.
        self.q_mul_complete.enable(region, offset)?;

        // If `lsb` is 0, return `Acc + (-P)`. If `lsb` is 1, simply return `Acc + 0`.
        let x_p = if let Some(lsb) = lsb {
            if !lsb {
                base.x.value()
            } else {
                Some(pallas::Base::zero())
            }
        } else {
            None
        };
        let y_p = if let Some(lsb) = lsb {
            if !lsb {
                base.y.value().map(|y_p| -y_p)
            } else {
                Some(pallas::Base::zero())
            }
        } else {
            None
        };

        let x_p_cell = region.assign_advice(
            || "x_p",
            self.add_config.x_p,
            offset + 1,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        let y_p_cell = region.assign_advice(
            || "y_p",
            self.add_config.y_p,
            offset + 1,
            || y_p.ok_or(Error::SynthesisError),
        )?;

        let p = EccPoint {
            x: CellValue::<pallas::Base>::new(x_p_cell, x_p),
            y: CellValue::<pallas::Base>::new(y_p_cell, y_p),
        };

        // Return the result of the final complete addition as `[scalar]B`
        let result = self
            .add_config
            .assign_region(&p, &acc, offset + 1, region)?;

        Ok((result, z))
    }
}

#[derive(Clone, Debug)]
// `x`-coordinate of the accumulator.
struct X<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for X<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
// `y`-coordinate of the accumulator.
struct Y<F: FieldExt>(Option<F>);
impl<F: FieldExt> Deref for Y<F> {
    type Target = Option<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
// Cumulative sum `z` used to decompose the scalar.
struct Z<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for Z<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn decompose_for_scalar_mul(scalar: Option<pallas::Base>) -> Vec<Option<bool>> {
    let bits = scalar.map(|scalar| {
        // We use `k = scalar + t_q` in the double-and-add algorithm, where
        // the scalar field `F_q = 2^254 + t_q`.
        let k = {
            let scalar = pallas::Scalar::from_bytes(&scalar.to_bytes()).unwrap();
            scalar + pallas::Scalar::from_u128(T_Q)
        };

        // `k` is decomposed bitwise (big-endian) into `[k_n, ..., lsb]`, where
        // each `k_i` is a bit and `scalar = k_n * 2^n + ... + k_1 * 2 + lsb`.
        let mut bits: Vec<bool> = k
            .to_le_bits()
            .into_iter()
            .take(pallas::Scalar::NUM_BITS as usize)
            .collect();
        bits.reverse();
        assert_eq!(bits.len(), pallas::Scalar::NUM_BITS as usize);

        bits
    });

    if let Some(bits) = bits {
        bits.into_iter().map(Some).collect()
    } else {
        vec![None; pallas::Scalar::NUM_BITS as usize]
    }
}

#[cfg(test)]
pub mod tests {
    use halo2::{circuit::Layouter, plonk::Error};
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use crate::circuit::gadget::ecc::{EccInstructions, Point, ScalarVar};

    pub fn test_mul<EccChip: EccInstructions<pallas::Affine> + Clone + Eq + std::fmt::Debug>(
        chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        zero: &Point<pallas::Affine, EccChip>,
        p: &Point<pallas::Affine, EccChip>,
    ) -> Result<(), Error> {
        let scalar_val = pallas::Base::rand();
        let scalar = ScalarVar::new(
            chip.clone(),
            layouter.namespace(|| "ScalarVar"),
            Some(scalar_val),
        )?;

        // [a]B
        p.mul(layouter.namespace(|| "mul"), &scalar)?;

        // [a]𝒪 should return an error since variable-base scalar multiplication
        // uses incomplete addition at the beginning of its double-and-add.
        zero.mul(layouter.namespace(|| "mul"), &scalar)
            .expect_err("[a]𝒪 should return an error");

        // [0]B should return (0,0) since variable-base scalar multiplication
        // uses complete addition for the final bits of the scalar.
        let scalar_val = pallas::Base::zero();
        let scalar = ScalarVar::new(chip, layouter.namespace(|| "ScalarVar"), Some(scalar_val))?;
        p.mul(layouter.namespace(|| "mul"), &scalar)?;

        Ok(())
    }
}
