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
pub trait EccInstructions<C: CurveAffine>: Chip<Field = C::Base> {
    /// Variable representing an element of the elliptic curve's scalar field.
    type Scalar: Clone + fmt::Debug;
    /// Variable representing an elliptic curve point.
    type Point: Clone + fmt::Debug;
    /// Variable representing the set of fixed bases in the circuit.
    type FixedPoints: FixedPoints<C>;
    /// Variable representing a fixed elliptic curve point (constant in the circuit).
    type FixedPoint: Clone + fmt::Debug;

    /// Witnesses the given scalar as a private input to the circuit.
    fn witness_scalar(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::Scalar, Error>;

    /// Witnesses the given point as a private input to the circuit.
    fn witness_point(
        layouter: &mut impl Layouter<Self>,
        value: Option<C>,
    ) -> Result<Self::Point, Error>;

    /// Gets a fixed point into the circuit.
    fn get_fixed(
        layouter: &mut impl Layouter<Self>,
        fixed_points: Self::FixedPoints,
    ) -> Result<Self::FixedPoint, Error>;

    /// Performs point addition, returning `a + b`.
    fn add(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs point doubling, returning `[2] a`.
    fn double(layouter: &mut impl Layouter<Self>, a: &Self::Point) -> Result<Self::Point, Error>;

    /// Performs variable-base scalar multiplication, returning `[scalar] base`.
    fn mul(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::Scalar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error>;

    /// Performs fixed-base scalar multiplication, returning `[scalar] base`.
    fn mul_fixed(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::Scalar,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error>;
}

/// An element of the given elliptic curve's scalar field.
#[derive(Debug)]
pub struct Scalar<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::Scalar,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> Scalar<C, EccChip> {
    /// Constructs a new point with the given value.
    pub fn new(
        mut layouter: impl Layouter<EccChip>,
        value: Option<C::Scalar>,
    ) -> Result<Self, Error> {
        EccChip::witness_scalar(&mut layouter, value).map(|inner| Scalar { inner })
    }
}

/// An elliptic curve point over the given curve.
#[derive(Debug)]
pub struct Point<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::Point,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> Point<C, EccChip> {
    /// Constructs a new point with the given value.
    pub fn new(mut layouter: impl Layouter<EccChip>, value: Option<C>) -> Result<Self, Error> {
        EccChip::witness_point(&mut layouter, value).map(|inner| Point { inner })
    }

    /// Returns `self + other`.
    pub fn add(&self, mut layouter: impl Layouter<EccChip>, other: &Self) -> Result<Self, Error> {
        EccChip::add(&mut layouter, &self.inner, &other.inner).map(|inner| Point { inner })
    }

    /// Returns `[2] self`.
    pub fn double(&self, mut layouter: impl Layouter<EccChip>) -> Result<Self, Error> {
        EccChip::double(&mut layouter, &self.inner).map(|inner| Point { inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<EccChip>,
        by: &Scalar<C, EccChip>,
    ) -> Result<Self, Error> {
        EccChip::mul(&mut layouter, &by.inner, &self.inner).map(|inner| Point { inner })
    }
}

/// A constant elliptic curve point over the given curve, for which scalar multiplication
/// is more efficient.
#[derive(Debug)]
pub struct FixedPoint<C: CurveAffine, EccChip: EccInstructions<C>> {
    inner: EccChip::FixedPoint,
}

impl<C: CurveAffine, EccChip: EccInstructions<C>> FixedPoint<C, EccChip> {
    /// Gets a reference to the specified fixed point in the circuit.
    pub fn get(
        mut layouter: impl Layouter<EccChip>,
        point: EccChip::FixedPoints,
    ) -> Result<Self, Error> {
        EccChip::get_fixed(&mut layouter, point).map(|inner| FixedPoint { inner })
    }

    /// Returns `[by] self`.
    pub fn mul(
        &self,
        mut layouter: impl Layouter<EccChip>,
        by: &Scalar<C, EccChip>,
    ) -> Result<Point<C, EccChip>, Error> {
        EccChip::mul_fixed(&mut layouter, &by.inner, &self.inner).map(|inner| Point { inner })
    }
}
