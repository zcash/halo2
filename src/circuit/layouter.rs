//! Implementations of common circuit layouters.

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use super::{Cell, Chip, Layouter, Permutation, Region};
use crate::plonk::{Advice, Any, Assignment, Column, Error, Fixed};

/// Helper trait for implementing a custom [`Layouter`].
///
/// This trait is used for implementing region assignments:
///
/// ```ignore
/// impl<'a, C: Chip, CS: Assignment<C::Field> + 'a> Layouter<C> for MyLayouter<'a, C, CS> {
///     fn assign_region(
///         &mut self,
///         assignment: impl FnOnce(Region<'_, C>) -> Result<(), Error>,
///     ) -> Result<(), Error> {
///         let region_index = self.regions.len();
///         self.regions.push(self.current_gate);
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
/// `Chip::Config`).
pub trait RegionLayouter<C: Chip>: fmt::Debug {
    /// Assign an advice column value (witness)
    fn assign_advice<'v>(
        &'v mut self,
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error>;

    /// Assign a fixed value
    fn assign_fixed<'v>(
        &'v mut self,
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
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

/// A [`Layouter`] for a single-chip circuit.
pub struct SingleChip<'a, C: Chip, CS: Assignment<C::Field> + 'a> {
    cs: &'a mut CS,
    config: C::Config,
    regions: Vec<usize>,
    /// Stores the first empty row for each column.
    columns: HashMap<Column<Any>, usize>,
    _marker: PhantomData<C>,
}

impl<'a, C: Chip, CS: Assignment<C::Field> + 'a> fmt::Debug for SingleChip<'a, C, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleChip")
            .field("config", &self.config)
            .field("regions", &self.regions)
            .field("columns", &self.columns)
            .finish()
    }
}

impl<'a, C: Chip, CS: Assignment<C::Field>> SingleChip<'a, C, CS> {
    /// Creates a new single-chip layouter.
    pub fn new(cs: &'a mut CS, config: C::Config) -> Self {
        SingleChip {
            cs,
            config,
            regions: vec![],
            columns: HashMap::default(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: Chip, CS: Assignment<C::Field> + 'a> Layouter<C> for SingleChip<'a, C, CS> {
    fn config(&self) -> &C::Config {
        &self.config
    }

    fn assign_region(
        &mut self,
        mut assignment: impl FnMut(Region<'_, C>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let region_index = self.regions.len();

        // Get shape of the region.
        let mut shape = RegionShape::new(region_index);
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
        self.regions.push(region_start);

        // Update column usage information.
        for column in shape.columns {
            self.columns.insert(column, region_start + shape.row_count);
        }

        let mut region = SingleChipRegion::new(self, region_index);
        {
            let region: &mut dyn RegionLayouter<C> = &mut region;
            assignment(region.into())?;
        }

        Ok(())
    }
}

/// The shape of a region. For a region at a certain index, we track
/// the set of columns it uses as well as the number of rows it uses.
#[derive(Debug)]
pub struct RegionShape {
    region_index: usize,
    columns: HashSet<Column<Any>>,
    row_count: usize,
}

impl RegionShape {
    /// Create a new `RegionShape` for a region at `region_index`.
    pub fn new(region_index: usize) -> Self {
        RegionShape {
            region_index,
            columns: HashSet::default(),
            row_count: 0,
        }
    }

    /// Get the `region_index` of a `RegionShape`.
    pub fn region_index(&self) -> usize {
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

impl<C: Chip> RegionLayouter<C> for RegionShape {
    fn assign_advice<'v>(
        &'v mut self,
        column: Column<Advice>,
        offset: usize,
        _to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
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
        column: Column<Fixed>,
        offset: usize,
        _to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
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

struct SingleChipRegion<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> {
    layouter: &'r mut SingleChip<'a, C, CS>,
    region_index: usize,
}

impl<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> fmt::Debug
    for SingleChipRegion<'r, 'a, C, CS>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleChipRegion")
            .field("layouter", &self.layouter)
            .field("region_index", &self.region_index)
            .finish()
    }
}

impl<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> SingleChipRegion<'r, 'a, C, CS> {
    fn new(layouter: &'r mut SingleChip<'a, C, CS>, region_index: usize) -> Self {
        SingleChipRegion {
            layouter,
            region_index,
        }
    }
}

impl<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> RegionLayouter<C>
    for SingleChipRegion<'r, 'a, C, CS>
{
    fn assign_advice<'v>(
        &'v mut self,
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_advice(
            column,
            self.layouter.regions[self.region_index] + offset,
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
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_fixed(
            column,
            self.layouter.regions[self.region_index] + offset,
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
        let left_column = permutation
            .mapping
            .iter()
            .position(|c| c == &left.column)
            .ok_or(Error::SynthesisError)?;
        let right_column = permutation
            .mapping
            .iter()
            .position(|c| c == &right.column)
            .ok_or(Error::SynthesisError)?;

        self.layouter.cs.copy(
            permutation.index,
            left_column,
            self.layouter.regions[left.region_index] + left.row_offset,
            right_column,
            self.layouter.regions[right.region_index] + right.row_offset,
        )?;

        Ok(())
    }
}
