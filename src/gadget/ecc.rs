//! Gadgets and chips for elliptic curve operations.

use std::fmt;

use crate::{
    arithmetic::Curve,
    circuit::{Chip, Layouter},
    plonk::Error,
};

/// The set of circuit instructions required to use the ECC gadgets.
pub trait EccInstructions<C: Curve>: Chip<Field = C::Base> {
    /// Variable representing an elliptic curve point.
    type Point: Clone + fmt::Debug;
    /// Variable representing an element of the elliptic curve's scalar field.
    type Scalar: Clone + fmt::Debug;

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
        base: &Self::Point,
        scalar: &Self::Scalar,
    ) -> Result<Self::Point, Error>;
}

/// An element of the given elliptic curve's scalar field.
#[derive(Debug)]
pub struct Scalar<C: Curve, EccChip: EccInstructions<C>> {
    inner: EccChip::Scalar,
}

/// An elliptic curve point over the given curve.
#[derive(Debug)]
pub struct Point<C: Curve, EccChip: EccInstructions<C>> {
    inner: EccChip::Point,
}

impl<C: Curve, EccChip: EccInstructions<C>> Point<C, EccChip> {
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
        EccChip::mul(&mut layouter, &self.inner, &by.inner).map(|inner| Point { inner })
    }
}
