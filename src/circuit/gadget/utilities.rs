use halo2::{
    circuit::{Cell, Layouter, Region},
    plonk::{Advice, Column, Error, Permutation},
};
use pasta_curves::arithmetic::FieldExt;
use std::array;

pub(crate) mod cond_swap;
pub(crate) mod enable_flag;
pub(crate) mod lookup_range_check;
pub(crate) mod plonk;

/// A variable representing a field element.
#[derive(Copy, Clone, Debug)]
pub struct CellValue<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

pub trait Var<F: FieldExt>: Copy + Clone + std::fmt::Debug {
    fn new(cell: Cell, value: Option<F>) -> Self;
    fn cell(&self) -> Cell;
    fn value(&self) -> Option<F>;
}

impl<F: FieldExt> Var<F> for CellValue<F> {
    fn new(cell: Cell, value: Option<F>) -> Self {
        Self { cell, value }
    }

    fn cell(&self) -> Cell {
        self.cell
    }

    fn value(&self) -> Option<F> {
        self.value
    }
}

pub trait UtilitiesInstructions<F: FieldExt> {
    type Var: Var<F>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        column: Column<Advice>,
        value: Option<F>,
    ) -> Result<Self::Var, Error> {
        layouter.assign_region(
            || "load private",
            |mut region| {
                let cell = region.assign_advice(
                    || "load private",
                    column,
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                Ok(Var::new(cell, value))
            },
        )
    }
}

/// Assigns a cell at a specific offset within the given region, constraining it
/// to the same value as another cell (which may be in any region).
///
/// Returns an error if either `column` or `copy` is not within `perm`.
pub fn copy<A, AR, F: FieldExt>(
    region: &mut Region<'_, F>,
    annotation: A,
    column: Column<Advice>,
    offset: usize,
    copy: &CellValue<F>,
    perm: &Permutation,
) -> Result<CellValue<F>, Error>
where
    A: Fn() -> AR,
    AR: Into<String>,
{
    let cell = region.assign_advice(annotation, column, offset, || {
        copy.value.ok_or(Error::SynthesisError)
    })?;

    region.constrain_equal(perm, cell, copy.cell)?;

    Ok(CellValue::new(cell, copy.value))
}

pub fn transpose_option_array<T: Copy + std::fmt::Debug, const LEN: usize>(
    option_array: Option<[T; LEN]>,
) -> [Option<T>; LEN] {
    let mut ret = [None; LEN];
    if let Some(arr) = option_array {
        for (entry, value) in ret.iter_mut().zip(array::IntoIter::new(arr)) {
            *entry = Some(value);
        }
    }
    ret
}
