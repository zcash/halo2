use halo2::{
    circuit::{Cell, Chip, Layouter, Region},
    plonk::{Advice, Column, Error, Permutation},
};
use pasta_curves::arithmetic::FieldExt;

mod cond_swap;
mod enable_flag;
mod plonk;

/// A variable representing a number.
#[derive(Copy, Clone, Debug)]
pub struct CellValue<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

pub trait Var<F: FieldExt> {
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

pub trait UtilitiesInstructions<F: FieldExt>: Chip<F> {
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

/// Assign a cell the same value as another cell and set up a copy constraint between them.
pub fn copy<A, AR, F: FieldExt>(
    region: &mut Region<'_, F>,
    annotation: A,
    column: Column<Advice>,
    row: usize,
    copy: &CellValue<F>,
    perm: &Permutation,
) -> Result<CellValue<F>, Error>
where
    A: Fn() -> AR,
    AR: Into<String>,
{
    let cell = region.assign_advice(annotation, column, row, || {
        copy.value.ok_or(Error::SynthesisError)
    })?;

    region.constrain_equal(perm, cell, copy.cell)?;

    Ok(CellValue::new(cell, copy.value))
}
