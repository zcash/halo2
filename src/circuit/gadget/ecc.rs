//! Gadgets for elliptic curve operations.

use std::fmt;

use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::Error,
};

/// Trait allowing circuit's fixed points to be enumerated.
pub trait FixedPoints<C: CurveAffine>: Clone + fmt::Debug {}

/// The set of circuit instructions required to use the ECC gadgets.
pub trait EccInstructions<C: CurveAffine>: Chip<C::Base> {
    /// Variable representing a full-width element of the elliptic curve's scalar field, to be used for fixed-base scalar mul.
    type ScalarFixed: Clone + fmt::Debug;
    /// Variable representing a signed short element of the elliptic curve's scalar field, to be used for fixed-base scalar mul.
    type ScalarFixedShort: Clone + fmt::Debug;
    /// Variable representing an elliptic curve point.
    type Point: Clone + fmt::Debug;
    /// Variable representing the x-coordinate of an elliptic curve point.
    type X: Clone + fmt::Debug;
    /// Variable representing the set of fixed bases in the circuit.
    type FixedPoints: FixedPoints<C>;
    /// Variable representing a fixed elliptic curve point (constant in the circuit).
    type FixedPoint: Clone + fmt::Debug;

    /// Witnesses the given full-width scalar as a private input to the circuit for fixed-based scalar mul.
    fn witness_scalar_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error>;

    /// Witnesses the given signed short scalar as a private input to the circuit for fixed-based scalar mul.
    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error>;

    /// Witnesses the given point as a private input to the circuit.
    fn witness_point(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self::Point, Error>;

    /// Extracts the x-coordinate of a point.
    fn extract_p(point: &Self::Point) -> &Self::X;

    /// Gets a fixed point into the circuit.
    fn get_fixed(&self, fixed_points: Self::FixedPoints) -> Result<Self::FixedPoint, Error>;

    /// Performs point addition, returning `a + b`.
    fn add(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs complete point addition, returning `a + b`.
    fn add_complete(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs variable-base scalar multiplication, returning `[scalar] base`.
    fn mul(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: C::Scalar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs fixed-base scalar multiplication using a full-width scalar, returning `[scalar] base`.
    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error>;

    /// Performs fixed-base scalar multiplication using a short signed scalar, returning `[scalar] base`.
    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error>;
}

/// A full-width element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixed<C: CurveAffine, EccChip: EccInstructions<C> + Clone> {
    chip: EccChip,
    inner: EccChip::ScalarFixed,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone> ScalarFixed<C, EccChip> {
    /// Constructs a new ScalarFixed with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_fixed(&mut layouter, value)
            .map(|inner| ScalarFixed {
                chip: chip.clone(),
                inner,
            })
    }
}

/// A signed short element of the given elliptic curve's scalar field, to be used for fixed-base scalar mul.
#[derive(Debug)]
pub struct ScalarFixedShort<C: CurveAffine, EccChip: EccInstructions<C> + Clone> {
    chip: EccChip,
    inner: EccChip::ScalarFixedShort,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone> ScalarFixedShort<C, EccChip> {
    /// Constructs a new ScalarFixedShort with the given value.
    pub fn new(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        chip.witness_scalar_fixed_short(&mut layouter, value)
            .map(|inner| ScalarFixedShort {
                chip: chip.clone(),
                inner,
            })
    }
}

/// An elliptic curve point over the given curve.
#[derive(Debug)]
pub struct Point<C: CurveAffine, EccChip: EccInstructions<C> + Clone> {
    chip: EccChip,
    inner: EccChip::Point,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone> Point<C, EccChip> {
    /// Constructs a new point with the given value.
    pub fn new(
        &self,
        mut layouter: impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self, Error> {
        self.chip
            .witness_point(&mut layouter, value)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }

    /// Extracts the x-coordinate of a point.
    pub fn extract_p(&self) -> X<C, EccChip> {
        X::from_inner(self.chip.clone(), EccChip::extract_p(&self.inner).clone())
    }

    /// Wraps the given point (obtained directly from an instruction) in a gadget.
    pub fn from_inner(chip: EccChip, inner: EccChip::Point) -> Self {
        Point { chip, inner }
    }

    /// Returns `self + other`.
    pub fn add(&self, mut layouter: impl Layouter<C::Base>, other: &Self) -> Result<Self, Error> {
        self.chip
            .add(&mut layouter, &self.inner, &other.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }

    /// Returns `[by] self`.
    pub fn mul(&self, mut layouter: impl Layouter<C::Base>, by: C::Scalar) -> Result<Self, Error> {
        self.chip
            .mul(&mut layouter, by, &self.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }
}

/// The x-coordinate of an elliptic curve point over the given curve.
#[derive(Debug)]
pub struct X<C: CurveAffine, EccChip: EccInstructions<C> + Clone> {
    chip: EccChip,
    inner: EccChip::X,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone> X<C, EccChip> {
    /// Wraps the given x-coordinate (obtained directly from an instruction) in a gadget.
    pub fn from_inner(chip: EccChip, inner: EccChip::X) -> Self {
        X { chip, inner }
    }
}

/// A constant elliptic curve point over the given curve, for which scalar multiplication
/// is more efficient.
#[derive(Clone, Debug)]
pub struct FixedPoint<C: CurveAffine, EccChip: EccInstructions<C> + Clone> {
    chip: EccChip,
    inner: EccChip::FixedPoint,
}

impl<C: CurveAffine, EccChip: EccInstructions<C> + Clone> FixedPoint<C, EccChip> {
    /// Gets a reference to the specified fixed point in the circuit.
    pub fn get(chip: EccChip, point: EccChip::FixedPoints) -> Result<Self, Error> {
        chip.get_fixed(point).map(|inner| FixedPoint {
            chip: chip.clone(),
            inner,
        })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<C::Base>,
        by: &ScalarFixed<C, EccChip>,
    ) -> Result<Point<C, EccChip>, Error> {
        self.chip
            .mul_fixed(&mut layouter, &by.inner, &self.inner)
            .map(|inner| Point {
                chip: self.chip.clone(),
                inner,
            })
    }
}
