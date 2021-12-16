use super::{
    add, add_incomplete, EccBaseFieldElemFixed, EccScalarFixed, EccScalarFixedShort,
    NonIdentityEccPoint,
};
use crate::circuit::gadget::utilities::decompose_running_sum::RunningSumConfig;
use crate::constants::{
    self,
    load::{NullifierK, OrchardFixedBase, OrchardFixedBasesFull, ValueCommitV, WindowUs},
};

use group::{ff::PrimeField, Curve};
use halo2::{
    circuit::{AssignedCell, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Selector, VirtualCells},
    poly::Rotation,
};
use lazy_static::lazy_static;
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

pub mod base_field_elem;
pub mod full_width;
pub mod short;

lazy_static! {
    static ref TWO_SCALAR: pallas::Scalar = pallas::Scalar::from(2);
    // H = 2^3 (3-bit window)
    static ref H_SCALAR: pallas::Scalar = pallas::Scalar::from(constants::H as u64);
    static ref H_BASE: pallas::Base = pallas::Base::from(constants::H as u64);
}

// A sum type for both full-width and short bases. This enables us to use the
// shared functionality of full-width and short fixed-base scalar multiplication.
#[derive(Copy, Clone, Debug)]
enum OrchardFixedBases {
    Full(OrchardFixedBasesFull),
    NullifierK,
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

impl From<NullifierK> for OrchardFixedBases {
    fn from(_nullifier_k: NullifierK) -> Self {
        Self::NullifierK
    }
}

impl OrchardFixedBases {
    pub fn generator(self) -> pallas::Affine {
        match self {
            Self::ValueCommitV => constants::value_commit_v::generator(),
            Self::NullifierK => constants::nullifier_k::generator(),
            Self::Full(base) => base.generator(),
        }
    }

    pub fn u(self) -> Vec<WindowUs> {
        match self {
            Self::ValueCommitV => ValueCommitV::get().u_short.0.as_ref().to_vec(),
            Self::NullifierK => NullifierK.u().0.as_ref().to_vec(),
            Self::Full(base) => base.u().0.as_ref().to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    running_sum_config: RunningSumConfig<pallas::Base, { constants::FIXED_BASE_WINDOW_SIZE }>,
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
    // Configuration for `add`
    add_config: add::Config,
    // Configuration for `add_incomplete`
    add_incomplete_config: add_incomplete::Config,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        lagrange_coeffs: [Column<Fixed>; constants::H],
        window: Column<Advice>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
        u: Column<Advice>,
        add_config: add::Config,
        add_incomplete_config: add_incomplete::Config,
    ) -> Self {
        meta.enable_equality(window.into());
        meta.enable_equality(u.into());

        let q_running_sum = meta.selector();
        let running_sum_config = RunningSumConfig::configure(meta, q_running_sum, window);

        let config = Self {
            running_sum_config,
            lagrange_coeffs,
            fixed_z: meta.fixed_column(),
            window,
            x_p,
            y_p,
            u,
            add_config,
            add_incomplete_config,
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

        config.running_sum_coords_gate(meta);

        config
    }

    /// Check that each window in the running sum decomposition uses the correct y_p
    /// and interpolated x_p.
    ///
    /// This gate is used both in the mul_fixed::base_field_elem and mul_fixed::short
    /// helpers, which decompose the scalar using a running sum.
    ///
    /// This gate is not used in the mul_fixed::full_width helper, since the full-width
    /// scalar is witnessed directly as three-bit windows instead of being decomposed
    /// via a running sum.
    fn running_sum_coords_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("Running sum coordinates check", |meta| {
            let q_mul_fixed_running_sum =
                meta.query_selector(self.running_sum_config.q_range_check);

            let z_cur = meta.query_advice(self.window, Rotation::cur());
            let z_next = meta.query_advice(self.window, Rotation::next());

            //    z_{i+1} = (z_i - a_i) / 2^3
            // => a_i = z_i - z_{i+1} * 2^3
            let word = z_cur - z_next * pallas::Base::from(constants::H as u64);

            self.coords_check(meta, q_mul_fixed_running_sum, word)
        });
    }

    #[allow(clippy::op_ref)]
    fn coords_check(
        &self,
        meta: &mut VirtualCells<'_, pallas::Base>,
        toggle: Expression<pallas::Base>,
        window: Expression<pallas::Base>,
    ) -> Vec<(&'static str, Expression<pallas::Base>)> {
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
        let x_check = interpolated_x - x_p.clone();
        // Check that `y + z = u^2`, where `z` is fixed and `u`, `y` are witnessed
        let y_check = u.square() - y_p.clone() - z;
        // Check that (x, y) is on the curve
        let on_curve =
            y_p.square() - x_p.clone().square() * x_p - Expression::Constant(pallas::Affine::b());

        vec![
            ("check x", toggle.clone() * x_check),
            ("check y", toggle.clone() * y_check),
            ("on-curve", toggle * on_curve),
        ]
    }

    #[allow(clippy::type_complexity)]
    fn assign_region_inner<const NUM_WINDOWS: usize>(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        scalar: &ScalarFixed,
        base: OrchardFixedBases,
        coords_check_toggle: Selector,
    ) -> Result<(NonIdentityEccPoint, NonIdentityEccPoint), Error> {
        // Assign fixed columns for given fixed base
        self.assign_fixed_constants::<NUM_WINDOWS>(region, offset, base, coords_check_toggle)?;

        // Initialize accumulator
        let acc = self.initialize_accumulator(region, offset, base, scalar)?;

        // Process all windows excluding least and most significant windows
        let acc = self.add_incomplete(region, offset, acc, base, scalar)?;

        // Process most significant window using complete addition
        let mul_b = self.process_msb::<NUM_WINDOWS>(region, offset, base, scalar)?;

        Ok((acc, mul_b))
    }

    fn assign_fixed_constants<const NUM_WINDOWS: usize>(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        coords_check_toggle: Selector,
    ) -> Result<(), Error> {
        let mut constants = None;

        let build_constants = || match base {
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
            OrchardFixedBases::NullifierK => {
                assert_eq!(NUM_WINDOWS, constants::NUM_WINDOWS);
                let base: OrchardFixedBase = NullifierK.into();
                (
                    base.lagrange_coeffs.0.as_ref().to_vec(),
                    base.z.0.as_ref().to_vec(),
                )
            }
        };

        // Assign fixed columns for given fixed base
        for window in 0..NUM_WINDOWS {
            coords_check_toggle.enable(region, window + offset)?;

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
                    || {
                        if constants.as_ref().is_none() {
                            constants = Some(build_constants());
                        }
                        let lagrange_coeffs = &constants.as_ref().unwrap().0;
                        Ok(lagrange_coeffs[window].0[k])
                    },
                )?;
            }

            // Assign z-values for each window
            region.assign_fixed(
                || format!("z-value for window: {:?}", window),
                self.fixed_z,
                window + offset,
                || {
                    let z = &constants.as_ref().unwrap().1;
                    Ok(z[window])
                },
            )?;
        }

        Ok(())
    }

    fn process_window(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        w: usize,
        k: Option<pallas::Scalar>,
        k_usize: Option<usize>,
        base: OrchardFixedBases,
    ) -> Result<NonIdentityEccPoint, Error> {
        let base_value = base.generator();
        let base_u = base.u();

        // Compute [(k_w + 2) ⋅ 8^w]B
        let mul_b = {
            let mul_b =
                k.map(|k| base_value * (k + *TWO_SCALAR) * H_SCALAR.pow(&[w as u64, 0, 0, 0]));
            let mul_b = mul_b.map(|mul_b| mul_b.to_affine().coordinates().unwrap());

            let x = mul_b.map(|mul_b| {
                let x = *mul_b.x();
                assert!(x != pallas::Base::zero());
                x
            });
            let x = region.assign_advice(
                || format!("mul_b_x, window {}", w),
                self.x_p,
                offset + w,
                || x.ok_or(Error::Synthesis),
            )?;

            let y = mul_b.map(|mul_b| {
                let y = *mul_b.y();
                assert!(y != pallas::Base::zero());
                y
            });
            let y = region.assign_advice(
                || format!("mul_b_y, window {}", w),
                self.y_p,
                offset + w,
                || y.ok_or(Error::Synthesis),
            )?;

            NonIdentityEccPoint { x, y }
        };

        // Assign u = (y_p + z_w).sqrt()
        let u_val = k_usize.map(|k| base_u[w].0[k]);
        region.assign_advice(|| "u", self.u, offset + w, || u_val.ok_or(Error::Synthesis))?;

        Ok(mul_b)
    }

    fn initialize_accumulator(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<NonIdentityEccPoint, Error> {
        // Recall that the message at each window `w` is represented as
        // `m_w = [(k_w + 2) ⋅ 8^w]B`.
        // When `w = 0`, we have `m_0 = [(k_0 + 2)]B`.
        let w = 0;
        let k0 = scalar.windows_field()[0];
        let k0_usize = scalar.windows_usize()[0];
        self.process_window(region, offset, w, k0, k0_usize, base)
    }

    fn add_incomplete(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        mut acc: NonIdentityEccPoint,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<NonIdentityEccPoint, Error> {
        let scalar_windows_field = scalar.windows_field();
        let scalar_windows_usize = scalar.windows_usize();

        for (w, (k, k_usize)) in scalar_windows_field[..(scalar_windows_field.len() - 1)]
            .iter()
            .zip(scalar_windows_usize[..(scalar_windows_field.len() - 1)].iter())
            .enumerate()
            // Skip k_0 (already processed).
            .skip(1)
        {
            // Compute [(k_w + 2) ⋅ 8^w]B
            let mul_b = self.process_window(region, offset, w, *k, *k_usize, base)?;

            // Add to the accumulator
            acc = self
                .add_incomplete_config
                .assign_region(&mul_b, &acc, offset + w, region)?;
        }
        Ok(acc)
    }

    fn process_msb<const NUM_WINDOWS: usize>(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: OrchardFixedBases,
        scalar: &ScalarFixed,
    ) -> Result<NonIdentityEccPoint, Error> {
        // Assign u = (y_p + z_w).sqrt() for the most significant window
        {
            let u_val =
                scalar.windows_usize()[NUM_WINDOWS - 1].map(|k| base.u()[NUM_WINDOWS - 1].0[k]);
            region.assign_advice(
                || "u",
                self.u,
                offset + NUM_WINDOWS - 1,
                || u_val.ok_or(Error::Synthesis),
            )?;
        }

        // offset_acc = \sum_{j = 0}^{NUM_WINDOWS - 2} 2^{FIXED_BASE_WINDOW_SIZE*j + 1}
        let offset_acc = (0..(NUM_WINDOWS - 1)).fold(pallas::Scalar::zero(), |acc, w| {
            acc + (*TWO_SCALAR).pow(&[
                constants::FIXED_BASE_WINDOW_SIZE as u64 * w as u64 + 1,
                0,
                0,
                0,
            ])
        });

        // `scalar = [k * 8^84 - offset_acc]`, where `offset_acc = \sum_{j = 0}^{83} 2^{FIXED_BASE_WINDOW_SIZE*j + 1}`.
        let scalar = scalar.windows_field()[scalar.windows_field().len() - 1]
            .map(|k| k * (*H_SCALAR).pow(&[(NUM_WINDOWS - 1) as u64, 0, 0, 0]) - offset_acc);

        let mul_b = {
            let mul_b = scalar.map(|scalar| base.generator() * scalar);
            let mul_b = mul_b.map(|mul_b| mul_b.to_affine().coordinates().unwrap());

            let x = mul_b.map(|mul_b| {
                let x = *mul_b.x();
                assert!(x != pallas::Base::zero());
                x
            });
            let x = region.assign_advice(
                || format!("mul_b_x, window {}", NUM_WINDOWS - 1),
                self.x_p,
                offset + NUM_WINDOWS - 1,
                || x.ok_or(Error::Synthesis),
            )?;

            let y = mul_b.map(|mul_b| {
                let y = *mul_b.y();
                assert!(y != pallas::Base::zero());
                y
            });
            let y = region.assign_advice(
                || format!("mul_b_y, window {}", NUM_WINDOWS - 1),
                self.y_p,
                offset + NUM_WINDOWS - 1,
                || y.ok_or(Error::Synthesis),
            )?;

            NonIdentityEccPoint { x, y }
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
    // The scalar decomposition was done in the base field. For computation
    // outside the circuit, we now convert them back into the scalar field.
    fn windows_field(&self) -> Vec<Option<pallas::Scalar>> {
        let running_sum_to_windows = |zs: Vec<AssignedCell<pallas::Base, pallas::Base>>| {
            (0..(zs.len() - 1))
                .map(|idx| {
                    let z_cur = zs[idx].value();
                    let z_next = zs[idx + 1].value();
                    let word = z_cur
                        .zip(z_next)
                        .map(|(z_cur, z_next)| z_cur - z_next * *H_BASE);
                    word.map(|word| pallas::Scalar::from_repr(word.to_repr()).unwrap())
                })
                .collect::<Vec<_>>()
        };
        match self {
            Self::BaseFieldElem(scalar) => running_sum_to_windows(scalar.running_sum.to_vec()),
            Self::Short(scalar) => running_sum_to_windows(scalar.running_sum.to_vec()),
            Self::FullWidth(scalar) => scalar
                .windows
                .iter()
                .map(|bits| {
                    bits.value()
                        .map(|value| pallas::Scalar::from_repr(value.to_repr()).unwrap())
                })
                .collect::<Vec<_>>(),
        }
    }

    // The scalar decomposition is guaranteed to be in three-bit windows,
    // so we also cast the least significant 4 bytes in their serialisation
    // into usize for convenient indexing into `u`-values
    fn windows_usize(&self) -> Vec<Option<usize>> {
        self.windows_field()
            .iter()
            .map(|window| {
                if let Some(window) = window {
                    let window = window.get_lower_32() as usize;
                    assert!(window < constants::H);
                    Some(window)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
