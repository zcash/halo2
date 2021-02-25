//! Implementations of common circuit layouters.

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use super::{Cell, Chip, Layouter, Region, RegionIndex, RegionStart};
use crate::plonk::{Advice, Any, Assignment, Column, Error, Fixed, Permutation};

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
    loaded: Option<C::Loaded>,
    /// Stores the starting row for each region.
    regions: Vec<RegionStart>,
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
    pub fn new(cs: &'a mut CS, config: C::Config) -> Result<Self, Error> {
        let mut ret = SingleChip {
            cs,
            config,
            loaded: None,
            regions: vec![],
            columns: HashMap::default(),
            _marker: PhantomData,
        };
        let loaded = C::load(&mut ret)?;
        ret.loaded = Some(loaded);
        Ok(ret)
    }
}

impl<'a, C: Chip, CS: Assignment<C::Field> + 'a> Layouter<C> for SingleChip<'a, C, CS> {
    type Root = Self;

    fn config(&self) -> &C::Config {
        &self.config
    }

    fn loaded(&self) -> &C::Loaded {
        self.loaded.as_ref().expect("We called C::load")
    }

    fn assign_region<A, AR, N, NR>(&mut self, name: N, mut assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, C>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        let region_index = self.regions.len();

        // Get shape of the region.
        let mut shape = RegionShape::new(region_index.into());
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
        self.regions.push(region_start.into());

        // Update column usage information.
        for column in shape.columns {
            self.columns.insert(column, region_start + shape.row_count);
        }

        self.cs.enter_region(name);
        let mut region = SingleChipRegion::new(self, region_index.into());
        let result = {
            let region: &mut dyn RegionLayouter<C> = &mut region;
            assignment(region.into())
        }?;
        self.cs.exit_region();

        Ok(result)
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.push_namespace(name_fn)
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        self.cs.pop_namespace(gadget_name)
    }
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

impl<C: Chip> RegionLayouter<C> for RegionShape {
    fn assign_advice<'v>(
        &'v mut self,
        _: &'v (dyn Fn() -> String + 'v),
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
        _: &'v (dyn Fn() -> String + 'v),
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
    region_index: RegionIndex,
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
    fn new(layouter: &'r mut SingleChip<'a, C, CS>, region_index: RegionIndex) -> Self {
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
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<C::Field, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_advice(
            annotation,
            column,
            *self.layouter.regions[*self.region_index] + offset,
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
            *self.layouter.regions[*self.region_index] + offset,
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
            *self.layouter.regions[*left.region_index] + left.row_offset,
            right.column,
            *self.layouter.regions[*right.region_index] + right.row_offset,
        )?;

        Ok(())
    }
}
