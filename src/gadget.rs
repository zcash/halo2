//! Self-contained circuit implementations of various primitives.

use std::fmt;

use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Any, Column, ConstraintSystem, Error, Fixed},
};

/// Handles the configuration of a chip
pub trait FloorPlanner {}

/// The configuration for a chip that is relevant to a [`Layouter`].
pub trait ChipConfig {
    // lookup columns
    // permutation columns
}

/// A chip implements a set of instructions that can be used by gadgets.
///
/// The chip itself should not store any state; instead, state that is required at circuit
/// synthesis time should be stored in [`Chip::Config`], which can then be fetched via
/// [`Layouter::config`].
pub trait Chip: Sized {
    /// A type that holds the configuration for this chip, and any other state it may need
    /// during circuit synthesis.
    type Config: ChipConfig;

    /// The field that the chip is defined over.
    ///
    /// This provides a type that the chip's configuration can reference if necessary.
    type Field: FieldExt;

    /// Load any fixed configuration for this chip into the circuit.
    fn load(layouter: &mut impl Layouter<Self>) -> Result<(), Error>;
}

/// A pointer to a cell within a circuit.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    /// Identifies the region in which this cell resides.
    region_index: usize,
    row_offset: usize,
    column: Column<Any>,
}

/// A permutation configured by a chip.
#[derive(Clone, Debug)]
pub struct Permutation {
    index: usize,
    mapping: Vec<Column<Any>>,
}

impl Permutation {
    /// Configures a new permutation for the given columns.
    pub fn new<F: FieldExt>(meta: &mut ConstraintSystem<F>, columns: &[Column<Advice>]) -> Self {
        let index = meta.permutation(columns);
        Permutation {
            index,
            mapping: columns.iter().map(|c| (*c).into()).collect(),
        }
    }
}

/// This trait allows a [`Chip`] to direct a [`Layouter`] to assign cells within a
/// region.
///
/// TODO: It would be great if we could constrain the columns in these types to be
/// "logical" columns that are guaranteed to correspond to the chip (and have come from
/// `Chip::Config`).
pub trait DynRegion<C: Chip>: fmt::Debug {
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

/// This struct allows a [`Chip`] to direct a [`Layouter`] to assign cells within a
/// region.
///
/// TODO: It would be great if we could constrain the columns in these types to be
/// "logical" columns that are guaranteed to correspond to the chip (and have come from
/// `Chip::Config`).
#[derive(Debug)]
pub struct Region<'r, C: Chip> {
    region: &'r mut dyn DynRegion<C>,
}

impl<'r, C: Chip> Region<'r, C> {
    /// Assign an advice column value (witness).
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    fn assign_advice<'v>(
        &'v mut self,
        column: Column<Advice>,
        offset: usize,
        mut to: impl FnMut() -> Result<C::Field, Error> + 'v,
    ) -> Result<Cell, Error> {
        self.region.assign_advice(column, offset, &mut to)
    }

    /// Assign a fixed value.
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    fn assign_fixed<'v>(
        &'v mut self,
        column: Column<Fixed>,
        offset: usize,
        mut to: impl FnMut() -> Result<C::Field, Error> + 'v,
    ) -> Result<Cell, Error> {
        self.region.assign_fixed(column, offset, &mut to)
    }

    /// Constraint two cells to have the same value.
    ///
    /// Returns an error if either of the cells is not within the given permutation.
    fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error> {
        self.region.constrain_equal(permutation, left, right)
    }
}

/// A layout strategy for a specific chip within a circuit.
///
/// This abstracts over the circuit assignments, handling row indices etc.
///
/// A particular concrete layout strategy will implement this trait for each chip it
/// supports.
pub trait Layouter<C: Chip> {
    /// Provides access to the chip configuration.
    fn config(&self) -> &C::Config;

    /// Assign a region of gates to an absolute row number.
    ///
    /// Inside the closure, the chip may freely use relative offsets; the `Layouter` will
    /// treat these assignments as a single "region" within the circuit. Outside this
    /// closure, the `Layouter` is allowed to optimise as it sees fit.
    ///
    /// ```ignore
    /// fn assign_region(&mut self, |region| {
    ///     region.assign_advice(self.config.a, offset, || { Some(value)});
    /// });
    /// ```
    fn assign_region(
        &mut self,
        assignment: impl FnOnce(Region<'_, C>) -> Result<(), Error>,
    ) -> Result<(), Error>;
}
