use std::marker::PhantomData;

use super::EccInstructions;
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use ff::Field;

mod window_table;
use window_table::WindowTable;

/// Configuration for the ECC chip
#[derive(Clone, Debug)]
pub struct EccConfig {
    num_bases: usize,        // number of fixed bases
    num_loaded_bases: usize, // number of fixed bases that have been loaded
    w: u32,                  // width of windows for fixed-base window tables
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
    fixed_bases: Vec<WindowTable>,
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

        let fixed_bases: Vec<_> = (0..num_bases)
            .map(|_| WindowTable::configure(meta, w))
            .collect();

        // Create point doubling gate
        meta.create_gate("point doubling", |meta| {
            let q_double = meta.query_fixed(q_double, Rotation::cur());
            let x_a = meta.query_advice(x_a, Rotation::cur());
            let y_a = meta.query_advice(y_a, Rotation::cur());
            let x_p = meta.query_advice(x_p, Rotation::cur());
            let y_p = meta.query_advice(y_p, Rotation::cur());

            let x_p_4 = x_p.clone() * x_p.clone() * x_p.clone() * x_p.clone();
            let expr1 = y_p.clone()
                * y_p.clone()
                * (x_a.clone() + x_p.clone() * C::Base::from_u64(2))
                * C::Base::from_u64(2)
                - x_p_4 * C::Base::from_u64(9);
            let expr2 = y_p.clone() * (y_a + y_p) * C::Base::from_u64(2)
                - x_p.clone() * x_p.clone() * (x_p - x_a) * C::Base::from_u64(3);

            q_double * (expr1 + expr2)
        });

        // Create point addition gate
        meta.create_gate("point addition", |meta| {
            let q_add = meta.query_fixed(q_add, Rotation::cur());
            let x_q = meta.query_advice(x_a, Rotation::cur());
            let y_q = meta.query_advice(y_a, Rotation::cur());
            let x_a = meta.query_advice(x_a, Rotation::next());
            let y_a = meta.query_advice(y_a, Rotation::next());
            let x_p = meta.query_advice(x_p, Rotation::cur());
            let y_p = meta.query_advice(y_p, Rotation::cur());

            let expr1 = x_a.clone() + x_q.clone() + x_p.clone()
                - (y_p.clone() - y_q.clone()) * (y_p.clone() - y_q.clone());
            let expr2 = (y_a + y_q.clone()) * (x_p - x_q.clone()) - (y_p - y_q) * (x_q - x_a);

            q_add * (expr1 + expr2)
        });

        EccConfig {
            num_bases,
            num_loaded_bases: 0,
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
            fixed_bases,
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
    type FixedPoint = C::Curve;

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
        value: Option<C::Curve>,
    ) -> Result<Self::FixedPoint, Error> {
        let config = layouter.config().clone();

        // Check that we do not exceed the number of bases in the config
        if config.num_loaded_bases >= config.num_bases {
            return Err(Error::ConstraintSystemFailure);
        }

        let table = config.fixed_bases[config.num_loaded_bases].clone();

        // TODO: update num_loaded_bases
        // layouter.config().num_loaded_bases += 1;
        table.load(layouter, value.unwrap())
    }

    fn add(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();
        let (x_q, y_q) = a.get_xy().unwrap();
        let (x_p, y_p) = b.get_xy().unwrap();
        let mut x_a = C::Base::zero();
        let mut y_a = C::Base::zero();

        layouter.assign_region(
            || "point addition",
            |mut region| {
                region.assign_fixed(|| "q_add", config.q_add, 0, || Ok(C::Base::one()))?;

                region.assign_advice(|| "x_a", config.x_a, 0, || Ok(x_q))?;
                region.assign_advice(|| "y_a", config.y_a, 0, || Ok(y_q))?;
                region.assign_advice(|| "x_p", config.x_p, 0, || Ok(x_p))?;
                region.assign_advice(|| "y_p", config.y_p, 0, || Ok(y_p))?;

                let lambda1 = (y_p - y_q) * (x_p - x_q).invert().unwrap();
                x_a = lambda1 * lambda1 - x_q - x_p;
                region.assign_advice(|| "x_a", config.x_a, 1, || Ok(x_a))?;

                y_a = lambda1 * (x_q - x_a) - y_q;
                region.assign_advice(|| "y_a", config.y_a, 1, || Ok(y_a))?;

                Ok(())
            },
        )?;

        Ok(C::from_xy(x_a, y_a).unwrap())
    }

    fn double(layouter: &mut impl Layouter<Self>, a: &Self::Point) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();
        let (x, y) = a.get_xy().unwrap();
        let mut x_a = C::Base::zero();
        let mut y_a = C::Base::zero();
        layouter.assign_region(
            || "point doubling",
            |mut region| {
                region.assign_fixed(|| "q_double", config.q_double, 0, || Ok(C::Base::one()))?;

                region.assign_advice(|| "x_p", config.x_p, 0, || Ok(x))?;
                region.assign_advice(|| "y_p", config.y_p, 0, || Ok(y))?;

                let lambda1 =
                    C::Base::from_u64(3) * x * x * (C::Base::from_u64(2) * y).invert().unwrap();

                x_a = lambda1 * lambda1 * C::Base::from_u64(2) * x;
                region.assign_advice(|| "x_a", config.x_a, 0, || Ok(x_a))?;

                y_a = lambda1 * (x - x_a) - y;
                region.assign_advice(|| "y_a", config.y_a, 0, || Ok(y_a))?;

                Ok(())
            },
        )?;

        Ok(C::from_xy(x_a, y_a).unwrap())
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
