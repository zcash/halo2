use super::{
    add, add_incomplete, copy, CellValue, EccBaseFieldElemFixed, EccConfig, EccPoint,
    EccScalarFixed, EccScalarFixedShort, Var,
};
use crate::constants::{
    self,
    load::{OrchardFixedBase, OrchardFixedBasesFull, ValueCommitV, WindowUs},
};

use group::Curve;
use halo2::{
    circuit::Region,
    plonk::{
        Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation, VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

pub mod base_field_elem;
pub mod full_width;
pub mod short;

// A sum type for both full-width and short bases. This enables us to use the
// shared functionality of full-width and short fixed-base scalar multiplication.
#[derive(Copy, Clone, Debug)]
enum OrchardFixedBases {
    Full(OrchardFixedBasesFull),
    ValueCommitV,
}

impl From<OrchardFixedBasesFull> for OrchardFixedBases {
    fn from(full_width_base: OrchardFixedBasesFull) -> Self {
        Self::Full(full_width_base)
    }
}

impl From<ValueCommitV> for OrchardFixedBases {
    fn from(_value_commit_v: ValueCommitV) -> Self {
        Self::ValueCommitV
    }
}

impl OrchardFixedBases {
    pub fn generator(self) -> pallas::Affine {
        match self {
            Self::ValueCommitV => constants::value_commit_v::generator(),
            Self::Full(base) => {
                let base: OrchardFixedBase = base.into();
                base.generator
            }
        }
    }

    pub fn u(self) -> Vec<WindowUs> {
        match self {
            Self::ValueCommitV => ValueCommitV::get().u_short.0.as_ref().to_vec(),
            Self::Full(base) => {
                let base: OrchardFixedBase = base.into();
                base.u.0.as_ref().to_vec()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config<const NUM_WINDOWS: usize> {
    mul_fixed: Column<Fixed>,
    // The fixed Lagrange interpolation coefficients for `x_p`.
    lagrange_coeffs: [Column<Fixed>; constants::H],
    // The fixed `z` for each window such that `y + z = u^2`.
    fixed_z: Column<Fixed>,
    // Decomposition of an `n-1`-bit scalar into `k`-bit windows:
    // a = a_0 + 2^k(a_1) + 2^{2k}(a_2) + ... + 2^{(n-1)k}(a_{n-1})
    window: Column<Advice>,
    // x-coordinate of the multiple of the fixed base at the current window.
    x_p: Column<Advice>,
    // y-coordinate of the multiple of the fixed base at the current window.
    y_p: Column<Advice>,
    // y-coordinate of accumulator (only used in the final row).
    u: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration for `add`
    add_config: add::Config,
    // Configuration for `add_incomplete`
    add_incomplete_config: add_incomplete::Config,
}

impl<const NUM_WINDOWS: usize> From<&EccConfig> for Config<NUM_WINDOWS> {
    fn from(ecc_config: &EccConfig) -> Self {
        let config = Self {
            mul_fixed: ecc_config.mul_fixed,
            lagrange_coeffs: ecc_config.lagrange_coeffs,
            fixed_z: ecc_config.fixed_z,
            x_p: ecc_config.advices[0],
            y_p: ecc_config.advices[1],
            window: ecc_config.advices[4],
            u: ecc_config.advices[5],
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            add_incomplete_config: ecc_config.into(),
        };

        // Check relationships between this config and `add_config`.
        assert_eq!(
            config.x_p, config.add_config.x_p,
            "add is used internally in mul_fixed."
        );
        assert_eq!(
            config.y_p, config.add_config.y_p,
            "add is used internally in mul_fixed."
        );

        // Check relationships between this config and `add_incomplete_config`.
        assert_eq!(
            config.x_p, config.add_incomplete_config.x_p,
            "add_incomplete is used internally in mul_fixed."
        );
        assert_eq!(
            config.y_p, config.add_incomplete_config.y_p,
            "add_incomplete is used internally in mul_fixed."
        );
        for advice in [config.x_p, config.y_p, config.window, config.u].iter() {
            assert_ne!(
                *advice, config.add_config.x_qr,
                "Do not overlap with output columns of add."
            );
            assert_ne!(
                *advice, config.add_config.y_qr,
                "Do not overlap with output columns of add."
            );
        }

        config
    }
}

impl<const NUM_WINDOWS: usize> Config<NUM_WINDOWS> {
    pub(super) fn create_gate_scalar(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate(
            "x_p, y_p checks for ScalarFixed, ScalarFixedShort",
            |meta| {
                let mul_fixed = meta.query_fixed(self.mul_fixed, Rotation::cur());
                let window = meta.query_advice(self.window, Rotation::cur());
                self.coords_check(meta, mul_fixed, window)
            },
        )
    }

    #[allow(clippy::op_ref)]
    fn coords_check(
        &self,
        meta: &mut VirtualCells<'_, pallas::Base>,
        toggle: Expression<pallas::Base>,
        window: Expression<pallas::Base>,
    ) -> Vec<Expression<pallas::Base>> {
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let x_p = meta.query_advice(self.x_p, Rotation::cur());
        let z = meta.query_fixed(self.fixed_z, Rotation::cur());
        let u = meta.query_advice(self.u, Rotation::cur());

        let window_pow: Vec<Expression<pallas::Base>> = (0..constants::H)
            .map(|pow| {
                (0..pow).fold(Expression::Constant(pallas::Base::one()), |acc, _| {
                    acc * window.clone()
                })
            })
            .collect();

        let interpolated_x = window_pow.iter().zip(self.lagrange_coeffs.iter()).fold(
            Expression::Constant(pallas::Base::zero()),
            |acc, (window_pow, coeff)| {
                acc + (window_pow.clone() * meta.query_fixed(*coeff, Rotation::cur()))
            },
        );

        // Check interpolation of x-coordinate
        let x_check = interpolated_x - x_p;
        // Check that `y + z = u^2`, where `z` is fixed and `u`, `y` are witnessed
        let y_check = u.clone() * u - y_p - z;

        vec![toggle.clone() * x_check, toggle * y_check]
    }

    #[allow(clippy::type_complexity)]
    fn assign_region_inner(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        scalar: &ScalarFixed,
        base: OrchardFixedBases,
        coords_check_toggle: Column<Fixed>,
    ) -> Result<(EccPoint, EccPoint), Error> {
        // Assign fixed columns for given fixed base
        self.assign_fixed_constants(region, offset, base, coords_check_toggle)?;

        // Initialize accumulator
        let acc = self.initialize_accumulator(region, offset, base, scalar)?;

        // Process all windows excluding least and most significant windows
        let acc = self.add_incomplete(region, offset, acc, base, scalar)?;

        // Process most significant window using complete addition
        let mul_b = self.process_msb(region, offset, base, scalar)?;

        Ok((acc, mul_b))
    }

    fn assign_fixed_constants(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        fixed_column: Column<Fixed>,
    ) -> Result<(), Error> {
        let (lagrange_coeffs, z) = match base {
            OrchardFixedBases::ValueCommitV => {
                assert_eq!(NUM_WINDOWS, constants::NUM_WINDOWS_SHORT);
                let base = ValueCommitV::get();
                (
                    base.lagrange_coeffs_short.0.as_ref().to_vec(),
                    base.z_short.0.as_ref().to_vec(),
                )
            }
            OrchardFixedBases::Full(base) => {
                assert_eq!(NUM_WINDOWS, constants::NUM_WINDOWS);
                let base: OrchardFixedBase = base.into();
                (
                    base.lagrange_coeffs.0.as_ref().to_vec(),
                    base.z.0.as_ref().to_vec(),
                )
            }
        };

        // Assign fixed columns for given fixed base
        for window in 0..NUM_WINDOWS {
            region.assign_fixed(
                || "Enable coords check",
                fixed_column,
                window + offset,
                || Ok(pallas::Base::one()),
            )?;

            // Assign x-coordinate Lagrange interpolation coefficients
            for k in 0..(constants::H) {
                region.assign_fixed(
                    || {
                        format!(
                            "Lagrange interpolation coeff for window: {:?}, k: {:?}",
                            window, k
                        )
                    },
                    self.lagrange_coeffs[k],
                    window + offset,
                    || Ok(lagrange_coeffs[window].0[k]),
                )?;
            }

            // Assign z-values for each window
            region.assign_fixed(
                || format!("z-value for window: {:?}", window),
                self.fixed_z,
                window + offset,
                || Ok(z[window]),
            )?;
        }

        Ok(())
    }

    fn copy_scalar(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        scalar: &ScalarFixed,
    ) -> Result<(), Error> {
        // Copy the scalar decomposition (`k`-bit windows)
        for (window_idx, window) in scalar.windows().iter().enumerate() {
            copy(
                region,
                || format!("k[{:?}]", window),
                self.window,
                window_idx + offset,
                window,
                &self.perm,
            )?;
        }

        Ok(())
    }

    fn initialize_accumulator(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<EccPoint, Error> {
        // Recall that the message at each window `w` is represented as
        // `m_w = [(k_w + 2) ⋅ 8^w]B`.
        // When `w = 0`, we have `m_0 = [(k_0 + 2)]B`.
        let m0 = {
            let k0 = scalar.windows_field()[0];
            let m0 = k0.map(|k0| base.generator() * (k0 + pallas::Scalar::from_u64(2)));
            let m0 = m0.map(|m0| m0.to_affine().coordinates().unwrap());

            let x = m0.map(|m0| *m0.x());
            let x_cell = region.assign_advice(
                || "m0_x",
                self.x_p,
                offset,
                || x.ok_or(Error::SynthesisError),
            )?;
            let x = CellValue::new(x_cell, x);

            let y = m0.map(|m0| *m0.y());
            let y_cell = region.assign_advice(
                || "m0_y",
                self.y_p,
                offset,
                || y.ok_or(Error::SynthesisError),
            )?;
            let y = CellValue::new(y_cell, y);

            EccPoint { x, y }
        };

        // Assign u = (y_p + z_w).sqrt() for `m0`
        {
            let k0 = scalar.windows_usize()[0];
            let u0 = &base.u()[0];
            let u0 = k0.map(|k0| u0.0[k0]);

            region.assign_advice(|| "u", self.u, offset, || u0.ok_or(Error::SynthesisError))?;
        }

        // Copy `m0` into `x_qr`, `y_qr` cells on row 1 of the incomplete addition.
        let x = copy(
            region,
            || "initialize acc x",
            self.add_incomplete_config.x_qr,
            offset + 1,
            &m0.x,
            &self.perm,
        )?;
        let y = copy(
            region,
            || "initialize acc y",
            self.add_incomplete_config.y_qr,
            offset + 1,
            &m0.y,
            &self.perm,
        )?;

        Ok(EccPoint { x, y })
    }

    fn add_incomplete(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        mut acc: EccPoint,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<EccPoint, Error> {
        // This is 2^w, where w is the window width
        let h = pallas::Scalar::from_u64(constants::H as u64);

        let base_value = base.generator();
        let base_u = base.u();
        let scalar_windows_field = scalar.windows_field();
        let scalar_windows_usize = scalar.windows_usize();

        for (w, k) in scalar_windows_field[1..(scalar_windows_field.len() - 1)]
            .iter()
            .enumerate()
        {
            // Offset window index by 1 since we are starting on k_1
            let w = w + 1;

            // Compute [(k_w + 2) ⋅ 8^w]B
            let mul_b = {
                let mul_b = k.map(|k| {
                    base_value * (k + pallas::Scalar::from_u64(2)) * h.pow(&[w as u64, 0, 0, 0])
                });
                let mul_b = mul_b.map(|mul_b| mul_b.to_affine().coordinates().unwrap());

                let x = mul_b.map(|mul_b| *mul_b.x());
                let x_cell = region.assign_advice(
                    || format!("mul_b_x, window {}", w),
                    self.x_p,
                    offset + w,
                    || x.ok_or(Error::SynthesisError),
                )?;
                let x = CellValue::new(x_cell, x);

                let y = mul_b.map(|mul_b| *mul_b.y());
                let y_cell = region.assign_advice(
                    || format!("mul_b_y, window {}", w),
                    self.y_p,
                    offset + w,
                    || y.ok_or(Error::SynthesisError),
                )?;
                let y = CellValue::new(y_cell, y);

                EccPoint { x, y }
            };

            // Assign u = (y_p + z_w).sqrt()
            let u_val = scalar_windows_usize[w].map(|k| base_u[w].0[k]);
            region.assign_advice(
                || "u",
                self.u,
                offset + w,
                || u_val.ok_or(Error::SynthesisError),
            )?;

            // Add to the accumulator
            acc = self
                .add_incomplete_config
                .assign_region(&mul_b, &acc, offset + w, region)?;
        }
        Ok(acc)
    }

    fn process_msb(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<EccPoint, Error> {
        // This is 2^w, where w is the window width
        let h = pallas::Scalar::from_u64(constants::H as u64);

        // Assign u = (y_p + z_w).sqrt() for the most significant window
        {
            let u_val =
                scalar.windows_usize()[NUM_WINDOWS - 1].map(|k| base.u()[NUM_WINDOWS - 1].0[k]);
            region.assign_advice(
                || "u",
                self.u,
                offset + NUM_WINDOWS - 1,
                || u_val.ok_or(Error::SynthesisError),
            )?;
        }

        // offset_acc = \sum_{j = 0}^{NUM_WINDOWS - 2} 2^{FIXED_BASE_WINDOW_SIZE * j+1}
        let offset_acc = (0..(NUM_WINDOWS - 1)).fold(pallas::Scalar::zero(), |acc, w| {
            acc + pallas::Scalar::from_u64(2).pow(&[
                constants::FIXED_BASE_WINDOW_SIZE as u64 * w as u64 + 1,
                0,
                0,
                0,
            ])
        });

        // `scalar = [k * 8^84 - offset_acc]`, where `offset_acc = \sum_{j = 0}^{83} 2^{FIXED_BASE_WINDOW_SIZE * j + 1}`.
        let scalar = scalar.windows_field()[scalar.windows_field().len() - 1]
            .map(|k| k * h.pow(&[(NUM_WINDOWS - 1) as u64, 0, 0, 0]) - offset_acc);

        let mul_b = {
            let mul_b = scalar.map(|scalar| base.generator() * scalar);
            let mul_b = mul_b.map(|mul_b| mul_b.to_affine().coordinates().unwrap());

            let x = mul_b.map(|mul_b| *mul_b.x());
            let x_cell = region.assign_advice(
                || format!("mul_b_x, window {}", NUM_WINDOWS - 1),
                self.x_p,
                offset + NUM_WINDOWS - 1,
                || x.ok_or(Error::SynthesisError),
            )?;
            let x = CellValue::new(x_cell, x);

            let y = mul_b.map(|mul_b| *mul_b.y());
            let y_cell = region.assign_advice(
                || format!("mul_b_y, window {}", NUM_WINDOWS - 1),
                self.y_p,
                offset + NUM_WINDOWS - 1,
                || y.ok_or(Error::SynthesisError),
            )?;
            let y = CellValue::new(y_cell, y);

            EccPoint { x, y }
        };

        Ok(mul_b)
    }
}

enum ScalarFixed {
    FullWidth(EccScalarFixed),
    Short(EccScalarFixedShort),
    BaseFieldElem(EccBaseFieldElemFixed),
}

impl From<&EccScalarFixed> for ScalarFixed {
    fn from(scalar_fixed: &EccScalarFixed) -> Self {
        Self::FullWidth(scalar_fixed.clone())
    }
}

impl From<&EccScalarFixedShort> for ScalarFixed {
    fn from(scalar_fixed: &EccScalarFixedShort) -> Self {
        Self::Short(scalar_fixed.clone())
    }
}

impl From<&EccBaseFieldElemFixed> for ScalarFixed {
    fn from(base_field_elem: &EccBaseFieldElemFixed) -> Self {
        Self::BaseFieldElem(base_field_elem.clone())
    }
}

impl ScalarFixed {
    fn windows(&self) -> &[CellValue<pallas::Base>] {
        match self {
            ScalarFixed::FullWidth(scalar) => &scalar.windows,
            ScalarFixed::Short(scalar) => &scalar.windows,
            _ => unreachable!("The base field element is not witnessed as windows."),
        }
    }

    // The scalar decomposition was done in the base field. For computation
    // outside the circuit, we now convert them back into the scalar field.
    fn windows_field(&self) -> Vec<Option<pallas::Scalar>> {
        match self {
            Self::BaseFieldElem(scalar) => {
                let mut zs = vec![scalar.base_field_elem];
                zs.extend_from_slice(&scalar.running_sum);

                (0..(zs.len() - 1))
                    .map(|idx| {
                        let z_cur = zs[idx].value();
                        let z_next = zs[idx + 1].value();
                        let word = z_cur.zip(z_next).map(|(z_cur, z_next)| {
                            z_cur - z_next * pallas::Base::from_u64(constants::H as u64)
                        });
                        word.map(|word| pallas::Scalar::from_bytes(&word.to_bytes()).unwrap())
                    })
                    .collect::<Vec<_>>()
            }
            _ => self
                .windows()
                .iter()
                .map(|bits| {
                    bits.value()
                        .map(|value| pallas::Scalar::from_bytes(&value.to_bytes()).unwrap())
                })
                .collect::<Vec<_>>(),
        }
    }

    // The scalar decomposition is guaranteed to be in three-bit windows,
    // so we also cast the least significant byte in their serialisation
    // into usize for convenient indexing into `u`-values
    fn windows_usize(&self) -> Vec<Option<usize>> {
        self.windows_field()
            .iter()
            .map(|window| {
                if let Some(window) = window {
                    let window = window.to_bytes()[0] as usize;
                    assert!(window < constants::H);
                    Some(window)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
