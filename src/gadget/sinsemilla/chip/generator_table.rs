use super::super::S_DOMAIN_PREFIX;
use super::SinsemillaChip;
use crate::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    circuit::Layouter,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use group::Curve;

/// Table containing independent generators P[0..2^k]
#[derive(Clone, Debug)]
pub(super) struct GeneratorTable {
    k: usize,
    table_idx: Column<Fixed>,
    table_x: Column<Fixed>,
    table_y: Column<Fixed>,
}

impl GeneratorTable {
    pub(super) fn configure<F: FieldExt, C: CurveAffine<Base = F>>(
        meta: &mut ConstraintSystem<F>,
        k: usize,
        m: Expression<F>,
        x_p: Expression<F>,
        y_p: Expression<F>,
    ) -> Self {
        let table_idx = meta.fixed_column();
        let table_idx_cur = meta.query_fixed(table_idx, Rotation::cur());
        let table_x = meta.fixed_column();
        let table_x_cur = meta.query_fixed(table_x, Rotation::cur());
        let table_y = meta.fixed_column();
        let table_y_cur = meta.query_fixed(table_y, Rotation::cur());

        meta.lookup(&[m, x_p, y_p], &[table_idx_cur, table_x_cur, table_y_cur]);

        GeneratorTable {
            k,
            table_idx,
            table_x,
            table_y,
        }
    }

    // Generates P[0..2^k] as 2^k independent, verifiably random generators of the group.
    // Loads these generators into a lookup table along with their indices.
    // Uses SWU hash-to-curve.
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

    pub(super) fn load<F: FieldExt, C: CurveAffine<Base = F>>(
        &self,
        layouter: &mut impl Layouter<SinsemillaChip<C>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "generator_table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = self.generate::<F, C>();

                for index in 0..(1 << self.k) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "table_idx",
                        self.table_idx,
                        index,
                        || {
                            row = rows.next();
                            row.map(|(idx, _, _)| idx).ok_or(Error::SynthesisError)
                        },
                    )?;
                    gate.assign_fixed(
                        || "table_x",
                        self.table_x,
                        index,
                        || row.map(|(_, x, _)| x).ok_or(Error::SynthesisError),
                    )?;
                    gate.assign_fixed(
                        || "table_y",
                        self.table_y,
                        index,
                        || row.map(|(_, _, y)| y).ok_or(Error::SynthesisError),
                    )?;
                }
                Ok(())
            },
        )
    }
}

/// Get generator S by index
pub fn get_s_by_idx<F: FieldExt, C: CurveAffine<Base = F>>(idx: u64) -> C::Curve {
    let hash = C::CurveExt::hash_to_curve(S_DOMAIN_PREFIX);
    hash(&idx.to_le_bytes())
}
