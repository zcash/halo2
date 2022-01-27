use super::{EccInstructions, FixedPoints};
use crate::{
    circuit::gadget::utilities::{
        lookup_range_check::LookupRangeCheckConfig, UtilitiesInstructions,
    },
    primitives::sinsemilla,
};
use arrayvec::ArrayVec;

use ff::Field;
use group::prime::PrimeCurveAffine;
use halo2::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

use std::convert::TryInto;

pub(super) mod add;
pub(super) mod add_incomplete;
pub mod constants;
pub(super) mod mul;
pub(super) mod mul_fixed;
pub(super) mod witness_point;

pub use constants::*;

/// A curve point represented in affine (x, y) coordinates, or the
/// identity represented as (0, 0).
/// Each coordinate is assigned to a cell.
#[derive(Clone, Debug)]
pub struct EccPoint {
    /// x-coordinate
    x: AssignedCell<pallas::Base, pallas::Base>,
    /// y-coordinate
    y: AssignedCell<pallas::Base, pallas::Base>,
}

impl EccPoint {
    /// Constructs a point from its coordinates, without checking they are on the curve.
    ///
    /// This is an internal API that we only use where we know we have a valid curve point
    /// (specifically inside Sinsemilla).
    pub(in crate::circuit::gadget) fn from_coordinates_unchecked(
        x: AssignedCell<pallas::Base, pallas::Base>,
        y: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Self {
        EccPoint { x, y }
    }

    /// Returns the value of this curve point, if known.
    pub fn point(&self) -> Option<pallas::Affine> {
        match (self.x.value(), self.y.value()) {
            (Some(x), Some(y)) => {
                if x.is_zero_vartime() && y.is_zero_vartime() {
                    Some(pallas::Affine::identity())
                } else {
                    Some(pallas::Affine::from_xy(*x, *y).unwrap())
                }
            }
            _ => None,
        }
    }
    /// The cell containing the affine short-Weierstrass x-coordinate,
    /// or 0 for the zero point.
    pub fn x(&self) -> AssignedCell<pallas::Base, pallas::Base> {
        self.x.clone()
    }
    /// The cell containing the affine short-Weierstrass y-coordinate,
    /// or 0 for the zero point.
    pub fn y(&self) -> AssignedCell<pallas::Base, pallas::Base> {
        self.y.clone()
    }

    #[cfg(test)]
    fn is_identity(&self) -> Option<bool> {
        self.x.value().map(|x| x.is_zero_vartime())
    }
}

/// A non-identity point represented in affine (x, y) coordinates.
/// Each coordinate is assigned to a cell.
#[derive(Clone, Debug)]
pub struct NonIdentityEccPoint {
    /// x-coordinate
    x: AssignedCell<pallas::Base, pallas::Base>,
    /// y-coordinate
    y: AssignedCell<pallas::Base, pallas::Base>,
}

impl NonIdentityEccPoint {
    /// Constructs a point from its coordinates, without checking they are on the curve.
    ///
    /// This is an internal API that we only use where we know we have a valid non-identity
    /// curve point (specifically inside Sinsemilla).
    pub(in crate::circuit::gadget) fn from_coordinates_unchecked(
        x: AssignedCell<pallas::Base, pallas::Base>,
        y: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Self {
        NonIdentityEccPoint { x, y }
    }

    /// Returns the value of this curve point, if known.
    pub fn point(&self) -> Option<pallas::Affine> {
        match (self.x.value(), self.y.value()) {
            (Some(x), Some(y)) => {
                assert!(!x.is_zero_vartime() && !y.is_zero_vartime());
                Some(pallas::Affine::from_xy(*x, *y).unwrap())
            }
            _ => None,
        }
    }
    /// The cell containing the affine short-Weierstrass x-coordinate.
    pub fn x(&self) -> AssignedCell<pallas::Base, pallas::Base> {
        self.x.clone()
    }
    /// The cell containing the affine short-Weierstrass y-coordinate.
    pub fn y(&self) -> AssignedCell<pallas::Base, pallas::Base> {
        self.y.clone()
    }
}

impl From<NonIdentityEccPoint> for EccPoint {
    fn from(non_id_point: NonIdentityEccPoint) -> Self {
        Self {
            x: non_id_point.x,
            y: non_id_point.y,
        }
    }
}

/// Configuration for the ECC chip
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct EccConfig<FixedPoints: super::FixedPoints<pallas::Affine>> {
    /// Advice columns needed by instructions in the ECC chip.
    pub advices: [Column<Advice>; 10],

    /// Incomplete addition
    add_incomplete: add_incomplete::Config,

    /// Complete addition
    add: add::Config,

    /// Variable-base scalar multiplication
    mul: mul::Config,

    /// Fixed-base full-width scalar multiplication
    mul_fixed_full: mul_fixed::full_width::Config<FixedPoints>,
    /// Fixed-base signed short scalar multiplication
    mul_fixed_short: mul_fixed::short::Config<FixedPoints>,
    /// Fixed-base mul using a base field element as a scalar
    mul_fixed_base_field: mul_fixed::base_field_elem::Config<FixedPoints>,

    /// Witness point
    witness_point: witness_point::Config,

    /// Lookup range check using 10-bit lookup table
    pub lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
}

/// A trait representing the kind of scalar used with a particular `FixedPoint`.
///
/// This trait exists because of limitations around const generics.
pub trait ScalarKind {
    const NUM_WINDOWS: usize;
}

/// Type marker representing a full-width scalar for use in fixed-base scalar
/// multiplication.
pub enum FullScalar {}
impl ScalarKind for FullScalar {
    const NUM_WINDOWS: usize = NUM_WINDOWS;
}

/// Type marker representing a signed 64-bit scalar for use in fixed-base scalar
/// multiplication.
pub enum ShortScalar {}
impl ScalarKind for ShortScalar {
    const NUM_WINDOWS: usize = NUM_WINDOWS_SHORT;
}

/// Type marker representing a base field element being used as a scalar in fixed-base
/// scalar multiplication.
pub enum BaseFieldElem {}
impl ScalarKind for BaseFieldElem {
    const NUM_WINDOWS: usize = NUM_WINDOWS;
}

/// Returns information about a fixed point.
///
/// TODO: When associated consts can be used as const generics, introduce a
/// `const NUM_WINDOWS: usize` associated const, and return `NUM_WINDOWS`-sized
/// arrays instead of `Vec`s.
pub trait FixedPoint<C: CurveAffine>: std::fmt::Debug + Eq + Clone {
    type ScalarKind: ScalarKind;

    fn generator(&self) -> C;
    fn u(&self) -> Vec<[[u8; 32]; H]>;
    fn z(&self) -> Vec<u64>;

    fn lagrange_coeffs(&self) -> Vec<[C::Base; H]> {
        compute_lagrange_coeffs(self.generator(), Self::ScalarKind::NUM_WINDOWS)
    }
}

/// A chip implementing EccInstructions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EccChip<FixedPoints: super::FixedPoints<pallas::Affine>> {
    config: EccConfig<FixedPoints>,
}

impl<FixedPoints: super::FixedPoints<pallas::Affine>> Chip<pallas::Base> for EccChip<FixedPoints> {
    type Config = EccConfig<FixedPoints>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Fixed: super::FixedPoints<pallas::Affine>> UtilitiesInstructions<pallas::Base>
    for EccChip<Fixed>
{
    type Var = AssignedCell<pallas::Base, pallas::Base>;
}

impl<FixedPoints: super::FixedPoints<pallas::Affine>> EccChip<FixedPoints> {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    /// # Side effects
    ///
    /// All columns in `advices` will be equality-enabled.
    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        lagrange_coeffs: [Column<Fixed>; 8],
        range_check: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    ) -> <Self as Chip<pallas::Base>>::Config {
        // Create witness point gate
        let witness_point = witness_point::Config::configure(meta, advices[0], advices[1]);
        // Create incomplete point addition gate
        let add_incomplete =
            add_incomplete::Config::configure(meta, advices[0], advices[1], advices[2], advices[3]);

        // Create complete point addition gate
        let add = add::Config::configure(
            meta, advices[0], advices[1], advices[2], advices[3], advices[4], advices[5],
            advices[6], advices[7], advices[8],
        );

        // Create variable-base scalar mul gates
        let mul = mul::Config::configure(meta, add, range_check, advices);

        // Create config that is shared across short, base-field, and full-width
        // fixed-base scalar mul.
        let mul_fixed = mul_fixed::Config::<FixedPoints>::configure(
            meta,
            lagrange_coeffs,
            advices[4],
            advices[0],
            advices[1],
            advices[5],
            add,
            add_incomplete,
        );

        // Create gate that is only used in full-width fixed-base scalar mul.
        let mul_fixed_full =
            mul_fixed::full_width::Config::<FixedPoints>::configure(meta, mul_fixed.clone());

        // Create gate that is only used in short fixed-base scalar mul.
        let mul_fixed_short =
            mul_fixed::short::Config::<FixedPoints>::configure(meta, mul_fixed.clone());

        // Create gate that is only used in fixed-base mul using a base field element.
        let mul_fixed_base_field = mul_fixed::base_field_elem::Config::<FixedPoints>::configure(
            meta,
            advices[6..9].try_into().unwrap(),
            range_check,
            mul_fixed,
        );

        EccConfig {
            advices,
            add_incomplete,
            add,
            mul,
            mul_fixed_full,
            mul_fixed_short,
            mul_fixed_base_field,
            witness_point,
            lookup_config: range_check,
        }
    }
}

/// A full-width scalar used for fixed-base scalar multiplication.
/// This is decomposed into 85 3-bit windows in little-endian order,
/// i.e. `windows` = [k_0, k_1, ..., k_84] (for a 255-bit scalar)
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_84 * (2^3)^84` and
/// each `k_i` is in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixed {
    value: Option<pallas::Scalar>,
    windows: ArrayVec<AssignedCell<pallas::Base, pallas::Base>, { NUM_WINDOWS }>,
}

// TODO: Make V a `u64`
type MagnitudeCell = AssignedCell<pallas::Base, pallas::Base>;
// TODO: Make V an enum Sign { Positive, Negative }
type SignCell = AssignedCell<pallas::Base, pallas::Base>;
type MagnitudeSign = (MagnitudeCell, SignCell);

/// A signed short scalar used for fixed-base scalar multiplication.
/// A short scalar must have magnitude in the range [0..2^64), with
/// a sign of either 1 or -1.
/// This is decomposed into 3-bit windows in little-endian order
/// using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
/// for element α = a_0 + (2^3) a_1 + ... + (2^{3(n-1)}) a_{n-1}.
/// Each `a_i` is in the range [0..2^3).
///
/// `windows` = [k_0, k_1, ..., k_21] (for a 64-bit magnitude)
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_84 * (2^3)^84` and
/// each `k_i` is in the range [0..2^3).
/// k_21 must be a single bit, i.e. 0 or 1.
#[derive(Clone, Debug)]
pub struct EccScalarFixedShort {
    magnitude: MagnitudeCell,
    sign: SignCell,
    running_sum: ArrayVec<AssignedCell<pallas::Base, pallas::Base>, { NUM_WINDOWS_SHORT + 1 }>,
}

/// A base field element used for fixed-base scalar multiplication.
/// This is decomposed into 3-bit windows in little-endian order
/// using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
/// for element α = a_0 + (2^3) a_1 + ... + (2^{3(n-1)}) a_{n-1}.
/// Each `a_i` is in the range [0..2^3).
///
/// `running_sum` = [z_0, ..., z_85], where we expect z_85 = 0.
/// Since z_0 is initialized as the scalar α, we store it as
/// `base_field_elem`.
#[derive(Clone, Debug)]
struct EccBaseFieldElemFixed {
    base_field_elem: AssignedCell<pallas::Base, pallas::Base>,
    running_sum: ArrayVec<AssignedCell<pallas::Base, pallas::Base>, { NUM_WINDOWS + 1 }>,
}

impl EccBaseFieldElemFixed {
    fn base_field_elem(&self) -> AssignedCell<pallas::Base, pallas::Base> {
        self.base_field_elem.clone()
    }
}

impl<Fixed: FixedPoints<pallas::Affine>> EccInstructions<pallas::Affine> for EccChip<Fixed>
where
    <Fixed as FixedPoints<pallas::Affine>>::Base:
        FixedPoint<pallas::Affine, ScalarKind = BaseFieldElem>,
    <Fixed as FixedPoints<pallas::Affine>>::FullScalar:
        FixedPoint<pallas::Affine, ScalarKind = FullScalar>,
    <Fixed as FixedPoints<pallas::Affine>>::ShortScalar:
        FixedPoint<pallas::Affine, ScalarKind = ShortScalar>,
{
    type ScalarFixed = EccScalarFixed;
    type ScalarFixedShort = EccScalarFixedShort;
    type ScalarVar = AssignedCell<pallas::Base, pallas::Base>;
    type Point = EccPoint;
    type NonIdentityPoint = NonIdentityEccPoint;
    type X = AssignedCell<pallas::Base, pallas::Base>;
    type FixedPoints = Fixed;

    fn constrain_equal(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "constrain equal",
            |mut region| {
                // Constrain x-coordinates
                region.constrain_equal(a.x().cell(), b.x().cell())?;
                // Constrain x-coordinates
                region.constrain_equal(a.y().cell(), b.y().cell())
            },
        )
    }

    fn witness_point(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Affine>,
    ) -> Result<Self::Point, Error> {
        let config = self.config().witness_point;
        layouter.assign_region(
            || "witness point",
            |mut region| config.point(value, 0, &mut region),
        )
    }

    fn witness_point_non_id(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Affine>,
    ) -> Result<Self::NonIdentityPoint, Error> {
        let config = self.config().witness_point;
        layouter.assign_region(
            || "witness non-identity point",
            |mut region| config.point_non_id(value, 0, &mut region),
        )
    }

    fn extract_p<Point: Into<Self::Point> + Clone>(point: &Point) -> Self::X {
        let point: EccPoint = (point.clone()).into();
        point.x()
    }

    fn add_incomplete(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        a: &Self::NonIdentityPoint,
        b: &Self::NonIdentityPoint,
    ) -> Result<Self::NonIdentityPoint, Error> {
        let config = self.config().add_incomplete;
        layouter.assign_region(
            || "incomplete point addition",
            |mut region| config.assign_region(a, b, 0, &mut region),
        )
    }

    fn add<A: Into<Self::Point> + Clone, B: Into<Self::Point> + Clone>(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        a: &A,
        b: &B,
    ) -> Result<Self::Point, Error> {
        let config = self.config().add;
        layouter.assign_region(
            || "complete point addition",
            |mut region| {
                config.assign_region(&(a.clone()).into(), &(b.clone()).into(), 0, &mut region)
            },
        )
    }

    fn mul(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        scalar: &Self::Var,
        base: &Self::NonIdentityPoint,
    ) -> Result<(Self::Point, Self::ScalarVar), Error> {
        let config = self.config().mul;
        config.assign(
            layouter.namespace(|| "variable-base scalar mul"),
            scalar.clone(),
            base,
        )
    }

    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        scalar: Option<pallas::Scalar>,
        base: &<Self::FixedPoints as FixedPoints<pallas::Affine>>::FullScalar,
    ) -> Result<(Self::Point, Self::ScalarFixed), Error> {
        let config = self.config().mul_fixed_full.clone();
        config.assign(
            layouter.namespace(|| format!("fixed-base mul of {:?}", base)),
            scalar,
            base,
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        magnitude_sign: MagnitudeSign,
        base: &<Self::FixedPoints as FixedPoints<pallas::Affine>>::ShortScalar,
    ) -> Result<(Self::Point, Self::ScalarFixedShort), Error> {
        let config = self.config().mul_fixed_short.clone();
        config.assign(
            layouter.namespace(|| format!("short fixed-base mul of {:?}", base)),
            magnitude_sign,
            base,
        )
    }

    fn mul_fixed_base_field_elem(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        base_field_elem: AssignedCell<pallas::Base, pallas::Base>,
        base: &<Self::FixedPoints as FixedPoints<pallas::Affine>>::Base,
    ) -> Result<Self::Point, Error> {
        let config = self.config().mul_fixed_base_field.clone();
        config.assign(
            layouter.namespace(|| format!("base-field elem fixed-base mul of {:?}", base)),
            base_field_elem,
            base,
        )
    }
}
