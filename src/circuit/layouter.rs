//! Implementations of common circuit layouters.

use std::cmp;
use std::fmt;
use std::marker::PhantomData;

use super::{Cell, Chip, Layouter, Permutation, Region};
use crate::plonk::{Advice, Assignment, Column, Error, Fixed};

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
    current_gate: usize,
    _marker: PhantomData<C>,
}

impl<'a, C: Chip, CS: Assignment<C::Field> + 'a> fmt::Debug for SingleChip<'a, C, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleChip")
            .field("config", &self.config)
            .field("regions", &self.regions)
            .field("current_gate", &self.current_gate)
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
            current_gate: 0,
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
        assignment: impl FnOnce(Region<'_, C>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let region_index = self.regions.len();
        self.regions.push(self.current_gate);

        let mut region = SingleChipRegion::new(self, region_index);
        {
            let region: &mut dyn RegionLayouter<C> = &mut region;
            assignment(region.into())?;
        }
        self.current_gate += region.row_count;

        Ok(())
    }
}

struct SingleChipRegion<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> {
    layouter: &'r mut SingleChip<'a, C, CS>,
    region_index: usize,
    row_count: usize,
}

impl<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> fmt::Debug
    for SingleChipRegion<'r, 'a, C, CS>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleChipRegion")
            .field("layouter", &self.layouter)
            .field("region_index", &self.region_index)
            .field("row_count", &self.row_count)
            .finish()
    }
}

impl<'r, 'a, C: Chip, CS: Assignment<C::Field> + 'a> SingleChipRegion<'r, 'a, C, CS> {
    fn new(layouter: &'r mut SingleChip<'a, C, CS>, region_index: usize) -> Self {
        SingleChipRegion {
            layouter,
            region_index,
            row_count: 0,
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
        self.row_count = cmp::max(self.row_count, offset);

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
        self.row_count = cmp::max(self.row_count, offset);
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
