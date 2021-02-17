use std::marker::PhantomData;

use super::EccInstructions;
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

/// Configuration for the ECC chip
#[derive(Clone, Debug)]
pub struct EccConfig {
    num_bases: usize, // number of fixed-base points
    w: u32,           // width of windows for fixed-base window tables
    x_a: Column<Advice>,
    y_a: Column<Advice>,
    x_p: Column<Advice>,
    y_p: Column<Advice>,
    lambda1: Column<Advice>,
    lambda2: Column<Advice>,
    q_add: Column<Fixed>,
    q_double: Column<Fixed>,
    q_mul: Column<Fixed>,
    q_mul_fixed: Column<Fixed>,
}

/// A chip implementing EccInstructions
#[derive(Debug)]
pub struct EccChip<C: CurveAffine> {
    _marker_c: PhantomData<C>,
}

impl<C: CurveAffine> EccChip<C> {
    fn configure(meta: &mut ConstraintSystem<C::Base>, num_bases: usize, w: u32) -> EccConfig {
        let x_a = meta.advice_column();
        let y_a = meta.advice_column();
        let x_p = meta.advice_column();
        let y_p = meta.advice_column();
        let lambda1 = meta.advice_column();
        let lambda2 = meta.advice_column();
        let q_add = meta.fixed_column();
        let q_double = meta.fixed_column();
        let q_mul = meta.fixed_column();
        let q_mul_fixed = meta.fixed_column();

        // TODO: Create gates

        EccConfig {
            num_bases,
            w,
            x_a,
            y_a,
            x_p,
            y_p,
            lambda1,
            lambda2,
            q_add,
            q_double,
            q_mul,
            q_mul_fixed,
        }
    }
}

impl<C: CurveAffine> Chip for EccChip<C> {
    type Config = EccConfig;
    type Field = C::Base;

    fn load(layouter: &mut impl Layouter<Self>) -> Result<(), Error> {
        todo!()
    }
}

impl<C: CurveAffine> EccInstructions<C> for EccChip<C> {
    type Scalar = C::Scalar;
    type Point = C;
    type FixedPoint = C;

    fn witness_scalar(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::Scalar, Error> {
        todo!()
    }

    fn witness_point(
        layouter: &mut impl Layouter<Self>,
        value: Option<C>,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn load_fixed(
        layouter: &mut impl Layouter<Self>,
        value: Option<C>,
    ) -> Result<Self::FixedPoint, Error> {
        todo!()
    }

    fn add(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn double(layouter: &mut impl Layouter<Self>, a: &Self::Point) -> Result<Self::Point, Error> {
        todo!()
    }

    fn mul(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::Scalar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn mul_fixed(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::Scalar,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        todo!()
    }
}
