use super::super::P_DOMAIN_PREFIX;
use super::SinsemillaChip;
use crate::{
    arithmetic::{
        Curve, CurveAffine, FieldExt, HashToCurve, Shake128, SimplifiedSWUWithDegree3Isogeny,
    },
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};
use subtle::CtOption;

/// Inputs to a lookp argument involving GeneratorTable
#[derive(Clone, Debug)]
pub(super) struct GeneratorInputs {
    m: Column<Advice>,
    x_p: Column<Advice>,
    y_p: Column<Advice>,
}

/// Table containing independent generators P[0..2^k]
#[derive(Clone, Debug)]
pub(super) struct GeneratorTable {
    k: usize,
    table_idx: Column<Fixed>,
    table_x: Column<Fixed>,
    table_y: Column<Fixed>,
}

impl GeneratorTable {
    pub(super) fn configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        k: usize,
        m: Column<Advice>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
    ) -> (GeneratorInputs, Self) {
        let table_idx = meta.fixed_column();
        let table_x = meta.fixed_column();
        let table_y = meta.fixed_column();

        meta.lookup(
            &[m.into(), x_p.into(), y_p.into()],
            &[table_idx.into(), table_x.into(), table_y.into()],
        );

        (
            GeneratorInputs { m, x_p, y_p },
            GeneratorTable {
                k,
                table_idx,
                table_x,
                table_y,
            },
        )
    }

    // Generates P[0..2^k] as 2^k independent, verifiably random generators of the group.
    // Loads these generators into a lookup table along with their indices.
    // Uses SWU hash-to-curve.
    fn generate<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>>(
        &self,
        map: &'static SimplifiedSWUWithDegree3Isogeny<F, I, C>,
    ) -> impl Iterator<Item = (F, F, F)> {
        let (init_x, init_y) = get_p_by_idx(map, 0u32).unwrap();

        (1..=(1 << self.k)).scan((F::zero(), init_x, init_y), move |(idx, x, y), i| {
            // We computed this table row in the previous iteration.
            let res = (*idx, *x, *y);

            // i holds the zero-indexed row number for the next table row.
            *idx = F::from_u64(i as u64);

            let (new_x, new_y) = get_p_by_idx(map, i as u32).unwrap();

            *x = new_x;
            *y = new_y;

            Some(res)
        })
    }

    pub(super) fn load<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>>(
        &self,
        layouter: &mut impl Layouter<SinsemillaChip<F>>,
        map: &'static SimplifiedSWUWithDegree3Isogeny<F, I, C>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "generator_table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = self.generate(&map);

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

pub fn get_p_by_idx<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>>(
    map: &SimplifiedSWUWithDegree3Isogeny<F, I, C>,
    idx: u32,
) -> CtOption<(F, F)> {
    let hash = map.hash_to_curve(P_DOMAIN_PREFIX, Shake128::default());
    let p: C = hash(&idx.to_le_bytes()).to_affine();
    p.get_xy()
}
