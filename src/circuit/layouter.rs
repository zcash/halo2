//! Implementations of common circuit layouters.

use std::cmp;
use std::collections::HashSet;
use std::fmt;

use super::{Cell, RegionIndex};
use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Any, Column, Error, Fixed, Permutation, Selector},
};

mod single_pass;
pub use single_pass::SingleChipLayouter;

/// Helper trait for implementing a custom [`Layouter`].
///
/// This trait is used for implementing region assignments:
///
/// ```ignore
/// impl<'a, F: FieldExt, C: Chip<F>, CS: Assignment<F> + 'a> Layouter<C> for MyLayouter<'a, C, CS> {
///     fn assign_region(
///         &mut self,
///         assignment: impl FnOnce(Region<'_, F, C>) -> Result<(), Error>,
///     ) -> Result<(), Error> {
///         let region_index = self.regions.len();
///         self.regions.push(self.current_gate);
///
///         let mut region = MyRegion::new(self, region_index);
///         {
///             let region: &mut dyn RegionLayouter<F> = &mut region;
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
/// `Chip::Config`).
///
/// [`Layouter`]: super::Layouter
pub trait RegionLayouter<F: FieldExt>: fmt::Debug {
    /// Enables a selector at the given offset.
    fn enable_selector<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        selector: &Selector,
        offset: usize,
    ) -> Result<(), Error>;

    /// Assign an advice column value (witness)
    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
    ) -> Result<Cell, Error>;

    /// Assign a fixed value
    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
    ) -> Result<Cell, Error>;

    /// Constraint two cells to have the same value.
    ///
    /// Returns an error if either of the cells is not within the given permutation.
    fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error>;
}

/// The shape of a region. For a region at a certain index, we track
/// the set of columns it uses as well as the number of rows it uses.
#[derive(Debug)]
pub struct RegionShape {
    region_index: RegionIndex,
    columns: HashSet<Column<Any>>,
    row_count: usize,
}

impl RegionShape {
    /// Create a new `RegionShape` for a region at `region_index`.
    pub fn new(region_index: RegionIndex) -> Self {
        RegionShape {
            region_index,
            columns: HashSet::default(),
            row_count: 0,
        }
    }

    /// Get the `region_index` of a `RegionShape`.
    pub fn region_index(&self) -> RegionIndex {
        self.region_index
    }

    /// Get a reference to the set of `columns` used in a `RegionShape`.
    pub fn columns(&self) -> &HashSet<Column<Any>> {
        &self.columns
    }

    /// Get the `row_count` of a `RegionShape`.
    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

impl<F: FieldExt> RegionLayouter<F> for RegionShape {
    fn enable_selector<'v>(
        &'v mut self,
        _: &'v (dyn Fn() -> String + 'v),
        selector: &Selector,
        offset: usize,
    ) -> Result<(), Error> {
        // Track the selector's fixed column as part of the region's shape.
        // TODO: Avoid exposing selector internals?
        self.columns.insert(selector.0.into());
        self.row_count = cmp::max(self.row_count, offset + 1);
        Ok(())
    }

    fn assign_advice<'v>(
        &'v mut self,
        _: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        _to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.columns.insert(column.into());
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
        _to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.columns.insert(column.into());
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
