//! Implementations of common circuit layouters.

use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use super::{Cell, Config, Layouter, Region, RegionIndex, RegionStart};
use crate::arithmetic::FieldExt;
use crate::plonk::{Advice, Any, Assignment, Column, Error, Fixed, Permutation};

/// Helper trait for implementing a custom [`Layouter`].
///
/// This trait is used for implementing region assignments:
///
/// ```ignore
/// impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Layouter<C> for MyLayouter<'a, C, CS> {
///     fn assign_region(
///         &mut self,
///         assignment: impl FnOnce(Region<'_, C>) -> Result<(), Error>,
///     ) -> Result<(), Error> {
///         let region_index = self.region_starts.len();
///         self.region_starts.push(self.current_gate);
///
///         let mut region = MyRegion::new(self, region_index);
///         {
///             let region: &mut dyn RegionLayouter<C> = &mut region;
///             assignment(region.into())?;
///         }
///         self.current_gate += region.row_count;
///
///         Ok(())
///     }
/// }
/// ```
///
/// TODO: It would be great if we could constrain the columns in these types to be
/// "logical" columns that are guaranteed to correspond to the chip (and have come from
/// `Config::Configured`).
pub trait RegionLayouter<C: Config>: fmt::Debug {
    /// Assign an advice column value (witness)
    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error>;

    /// Assign a fixed value
    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error>;

    /// Constrain two cells to have the same value.
    ///
    /// Returns an error if either of the cells is not within the given permutation.
    fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error>;
}

/// A [`Layouter`] for a single-chip circuit.
pub struct SingleConfigLayouter<'a, F: FieldExt, CS: Assignment<F> + 'a> {
    /// Constraint system
    pub cs: &'a mut CS,
    /// Stores the starting row for each region.
    pub region_starts: Vec<RegionStart>,
    /// Stores the columns used by each region.
    region_columns: Vec<Vec<Column<Any>>>,
    /// Stores the first empty row for each column.
    columns: HashMap<Column<Any>, usize>,
    marker: PhantomData<F>,
}

impl<'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for SingleConfigLayouter<'a, F, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleConfigLayouter")
            .field("regions", &self.region_starts)
            .field("columns", &self.columns)
            .finish()
    }
}

impl<'a, F: FieldExt, CS: Assignment<F>> SingleConfigLayouter<'a, F, CS> {
    /// Creates a new single-core layouter.
    pub fn new(cs: &'a mut CS) -> Self {
        SingleConfigLayouter {
            cs,
            region_starts: vec![],
            region_columns: vec![],
            columns: HashMap::default(),
            marker: PhantomData,
        }
    }
}

impl<'a, F: FieldExt, CS: Assignment<F>> Layouter<F> for SingleConfigLayouter<'a, F, CS> {
    fn assign_new_region<A, AR, N, NR, C: Config<Field = F>>(
        &mut self,
        columns: &[Column<Any>],
        name: N,
        mut assignment: A,
    ) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, C>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        let region_index = self.region_starts.len().into();

        // Get shape of the region.
        let mut shape = RegionShape::new(region_index, columns);
        {
            let region: &mut dyn RegionLayouter<C> = &mut shape;
            assignment(region.into())?;
        }

        // Lay out this region. We implement the simplest approach here: position the
        // region starting at the earliest row for which none of the columns are in use.
        let mut region_start = 0;
        for column in &shape.columns {
            region_start = cmp::max(region_start, self.columns.get(column).cloned().unwrap_or(0));
        }
        self.region_starts.push(region_start.into());
        self.region_columns.push(columns.to_vec());

        // Update column usage information.
        for column in shape.columns {
            self.columns.insert(column, region_start + shape.row_count);
        }

        self.cs.enter_region(name);
        let mut region = SingleConfigLayouterRegion::new(self, region_index.into());
        let result = {
            let region: &mut dyn RegionLayouter<C> = &mut region;
            assignment(region.into())
        }?;
        self.cs.exit_region();

        Ok(result)
    }

    fn assign_existing_region<A, AR, N, NR, C: Config<Field = F>>(
        &mut self,
        region_index: super::RegionIndex,
        name: N,
        mut assignment: A,
    ) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, C>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        let mut shape = RegionShape::new(region_index, &self.region_columns[*region_index]);
        {
            let region: &mut dyn RegionLayouter<C> = &mut shape;
            assignment(region.into())?;
        }

        let region_start = self.region_starts[*region_index];

        // Update column usage information.
        for column in shape.columns {
            let new_row = *region_start + shape.row_count;
            let current_row = self.columns.get(&column).unwrap();

            if *current_row < new_row {
                self.columns.insert(column, new_row);
            } else {
            }
        }

        self.cs.enter_region(name);
        let mut region = SingleConfigLayouterRegion::new(self, region_index.into());
        let result = {
            let region: &mut dyn RegionLayouter<C> = &mut region;
            assignment(region.into())
        }?;
        self.cs.exit_region();

        Ok(result)
    }
}

/// The shape of a region. For a region at a certain index, we track
/// the set of columns it uses as well as the number of rows it uses.
#[derive(Debug)]
pub struct RegionShape {
    region_index: RegionIndex,
    columns: Vec<Column<Any>>,
    row_count: usize,
}

impl RegionShape {
    /// Create a new `RegionShape` for an assignment at `region_index`.
    pub fn new(region_index: RegionIndex, columns: &[Column<Any>]) -> Self {
        RegionShape {
            region_index,
            columns: columns.to_vec(),
            row_count: 0,
        }
    }

    /// Get the `region_index` of a `RegionShape`.
    pub fn region_index(&self) -> RegionIndex {
        self.region_index
    }

    /// Get a reference to the set of `columns` used in a `RegionShape`.
    pub fn columns(&self) -> &[Column<Any>] {
        &self.columns
    }

    /// Get the `row_count` of a `RegionShape`.
    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

impl<C: Config> RegionLayouter<C> for RegionShape {
    fn assign_advice<'v>(
        &'v mut self,
        _: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        _to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        // self.columns.insert(column.into());
        self.row_count = cmp::max(self.row_count, offset + 1);

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn assign_fixed<'v>(
        &'v mut self,
        _: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        _to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        // self.columns.insert(column.into());
        self.row_count = cmp::max(self.row_count, offset + 1);

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn constrain_equal(
        &mut self,
        _permutation: &Permutation,
        _left: Cell,
        _right: Cell,
    ) -> Result<(), Error> {
        // Equality constraints don't affect the region shape.
        Ok(())
    }
}

struct SingleConfigLayouterRegion<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> {
    layouter: &'r mut SingleConfigLayouter<'a, F, CS>,
    region_index: RegionIndex,
}

impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug
    for SingleConfigLayouterRegion<'r, 'a, F, CS>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleConfigLayouterRegion")
            .field("layouter", &self.layouter)
            .field("region_index", &self.region_index)
            .finish()
    }
}

impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> SingleConfigLayouterRegion<'r, 'a, F, CS> {
    fn new(layouter: &'r mut SingleConfigLayouter<'a, F, CS>, region_index: RegionIndex) -> Self {
        SingleConfigLayouterRegion {
            layouter,
            region_index,
        }
    }
}

impl<'r, 'a, F: FieldExt, C: Config<Field = F>, CS: Assignment<F> + 'a> RegionLayouter<C>
    for SingleConfigLayouterRegion<'r, 'a, F, CS>
{
    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_advice(
            annotation,
            column,
            *self.layouter.region_starts[*self.region_index] + offset,
            to,
        )?;

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_fixed(
            annotation,
            column,
            *self.layouter.region_starts[*self.region_index] + offset,
            to,
        )?;

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error> {
        self.layouter.cs.copy(
            permutation,
            left.column,
            *self.layouter.region_starts[*left.region_index] + left.row_offset,
            right.column,
            *self.layouter.region_starts[*right.region_index] + right.row_offset,
        )?;

        Ok(())
    }
}
