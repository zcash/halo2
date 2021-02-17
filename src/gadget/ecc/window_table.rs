use super::EccChip;
use crate::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    circuit::Layouter,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use group::Curve;

/// Table containing precomputed multiples of a base point
#[derive(Clone, Debug)]
pub(super) struct WindowTable {
    w: usize, // width of window
    table_idx: Column<Fixed>,
    table_x: Column<Fixed>,
    table_y: Column<Fixed>,
}

impl WindowTable {
    pub(super) fn configure<F: FieldExt, C: CurveAffine<Base = F>>(
        meta: &mut ConstraintSystem<F>,
        w: usize,
    ) -> Self {
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
    fn generate<F: FieldExt, C: CurveAffine<Base = F>>(&self) -> impl Iterator<Item = (F, F, F)> {
        let (init_x, init_y) = get_s_by_idx::<F, C>(0).to_affine().get_xy().unwrap();

        (1..=(1 << self.k)).scan((F::zero(), init_x, init_y), move |(idx, x, y), i| {
            // We computed this table row in the previous iteration.
            let res = (*idx, *x, *y);

            // i holds the zero-indexed row number for the next table row.
            *idx = F::from_u64(i as u64);

            let (new_x, new_y) = get_s_by_idx::<F, C>(i).to_affine().get_xy().unwrap();

            *x = new_x;
            *y = new_y;

            Some(res)
        })
    }


}