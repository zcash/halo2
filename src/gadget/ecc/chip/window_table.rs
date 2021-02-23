use super::EccChip;
use crate::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    circuit::Layouter,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use std::ops::{Mul, MulAssign};

use ff::{Field, PrimeField};
use group::Curve;

/// Table containing precomputed multiples of a base point
#[derive(Clone, Debug)]
pub(super) struct WindowTable {
    w: u32, // width of window
    table_idx: Column<Fixed>,
    table_x: Column<Fixed>,
    table_y: Column<Fixed>,
}

impl WindowTable {
    pub(super) fn configure<F: FieldExt>(meta: &mut ConstraintSystem<F>, w: u32) -> Self {
        let table_idx = meta.fixed_column();
        let table_x = meta.fixed_column();
        let table_y = meta.fixed_column();

        WindowTable {
            w,
            table_idx,
            table_x,
            table_y,
        }
    }

    // Generates $[(1 << w)^i]B$ multiples of the fixed base in $w-$bit windows,
    // where $i \in [0..(NUM_BITS / w)]$. (`NUM_BITS` is the number of bits needed
    // to represent an element in the scalar field.)
    // Loads these generators into fixed columns along with their indices.
    fn generate<F: FieldExt, C>(&self, base: C::Curve) -> impl Iterator<Item = (C::Base, C::Curve)>
    where
        C: CurveAffine<ScalarExt = F>,
        C::Curve: Mul<F, Output = C::Curve> + MulAssign<F>,
    {
        let w = self.w;
        let num_windows = C::Base::NUM_BITS / w;

        (1..=num_windows).scan((C::Base::one(), base), move |(idx, point), _| {
            // We computed this table row in the previous iteration.
            let res = (*idx, *point);

            // Obtain the next idx as the current idx * 2^w
            *idx = *idx * C::Base::from_u64(1 << w);

            // Obtain the next point as the current point + 2^w
            *point = point.clone() * C::Scalar::from_u64(1 << w);

            Some(res)
        })
    }

    pub(super) fn load<F: FieldExt, C: CurveAffine<ScalarExt = F>>(
        &self,
        layouter: &mut impl Layouter<EccChip<C>>,
        base: C::Curve,
    ) -> Result<C::Curve, Error> {
        layouter.assign_region(
            || "generator_table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = self.generate::<F, C>(base);

                let num_windows = F::NUM_BITS / self.w;

                for index in 0..(1 << num_windows) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "table_idx",
                        self.table_idx,
                        index,
                        || {
                            row = rows.next();
                            row.map(|(idx, _)| idx).ok_or(Error::SynthesisError)
                        },
                    )?;
                    let (x, y) =
                        Option::from(row.map(|(_, point)| point.to_affine().get_xy().unwrap()))
                            .ok_or(Error::SynthesisError)?;
                    gate.assign_fixed(|| "table_x", self.table_x, index, || Ok(x))?;
                    gate.assign_fixed(|| "table_y", self.table_y, index, || Ok(y))?;
                }
                Ok(())
            },
        )?;

        Ok(base)
    }
}
