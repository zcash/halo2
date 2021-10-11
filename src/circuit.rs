//! Traits and structs for implementing circuit components.

use std::{convert::TryInto, fmt, marker::PhantomData};

use ff::Field;

use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Any, Assigned, Column, Error, Fixed, Instance, Selector, TableColumn},
};

pub mod floor_planner;
pub use floor_planner::single_pass::SimpleFloorPlanner;

pub mod layouter;

/// A chip implements a set of instructions that can be used by gadgets.
///
/// The chip stores state that is required at circuit synthesis time in
/// [`Chip::Config`], which can be fetched via [`Chip::config`].
///
/// The chip also loads any fixed configuration needed at synthesis time
/// using its own implementation of `load`, and stores it in [`Chip::Loaded`].
/// This can be accessed via [`Chip::loaded`].
pub trait Chip<F: FieldExt>: Sized {
    /// A type that holds the configuration for this chip, and any other state it may need
    /// during circuit synthesis, that can be derived during [`Circuit::configure`].
    ///
    /// [`Circuit::configure`]: crate::plonk::Circuit::configure
    type Config: fmt::Debug + Clone;

    /// A type that holds any general chip state that needs to be loaded at the start of
    /// [`Circuit::synthesize`]. This might simply be `()` for some chips.
    ///
    /// [`Circuit::synthesize`]: crate::plonk::Circuit::synthesize
    type Loaded: fmt::Debug + Clone;

    /// The chip holds its own configuration.
    fn config(&self) -> &Self::Config;

    /// Provides access to general chip state loaded at the beginning of circuit
    /// synthesis.
    ///
    /// Panics if called before `Chip::load`.
    fn loaded(&self) -> &Self::Loaded;
}

/// Index of a region in a layouter
#[derive(Clone, Copy, Debug)]
pub struct RegionIndex(usize);

impl From<usize> for RegionIndex {
    fn from(idx: usize) -> RegionIndex {
        RegionIndex(idx)
    }
}

impl std::ops::Deref for RegionIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Starting row of a region in a layouter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionStart(usize);

impl From<usize> for RegionStart {
    fn from(idx: usize) -> RegionStart {
        RegionStart(idx)
    }
}

impl std::ops::Deref for RegionStart {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A pointer to a cell within a circuit.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    /// Identifies the region in which this cell resides.
    region_index: RegionIndex,
    /// The relative offset of this cell within its region.
    row_offset: usize,
    /// The column of this cell.
    column: Column<Any>,
}

/// An allocated cell.
#[derive(Clone, Copy, Debug)]
pub struct AllocatedCell<F: FieldExt, T: Copy + Into<F>> {
    value: Option<T>,
    cell: Cell,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, T: Copy + Into<F>> AllocatedCell<F, T> {
    /// Instantiate a new AllocatedCell.
    pub(crate) fn new(value: Option<T>, cell: Cell) -> Self {
        Self {
            value,
            cell,
            _marker: PhantomData,
        }
    }

    /// Returns the value of the AllocatedCell.
    pub fn value(&self) -> Option<T> {
        self.value
    }

    /// Returns the value of the AllocatedCell as a field element.
    pub fn value_field(&self) -> Option<F> {
        self.value.map(|v| v.into())
    }

    /// Returns the cell.
    pub fn cell(&self) -> Cell {
        self.cell
    }

    /// Copies the value to a given advice cell and constrains them to be equal.
    pub(crate) fn copy_advice<A, AR>(
        &self,
        annotation: A,
        region: &mut Region<'_, F>,
        column: Column<Advice>,
        offset: usize,
    ) -> Result<Self, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let cell = region.assign_advice(annotation, column, offset, || {
            self.value_field().ok_or(Error::SynthesisError)
        })?;

        Ok(Self {
            value: self.value(),
            cell,
            _marker: PhantomData,
        })
    }
}

/// A region of the circuit in which a [`Chip`] can assign cells.
///
/// Inside a region, the chip may freely use relative offsets; the [`Layouter`] will
/// treat these assignments as a single "region" within the circuit.
///
/// The [`Layouter`] is allowed to optimise between regions as it sees fit. Chips must use
/// [`Region::constrain_equal`] to copy in variables assigned in other regions.
///
/// TODO: It would be great if we could constrain the columns in these types to be
/// "logical" columns that are guaranteed to correspond to the chip (and have come from
/// `Chip::Config`).
#[derive(Debug)]
pub struct Region<'r, F: Field> {
    region: &'r mut dyn layouter::RegionLayouter<F>,
}

impl<'r, F: Field> From<&'r mut dyn layouter::RegionLayouter<F>> for Region<'r, F> {
    fn from(region: &'r mut dyn layouter::RegionLayouter<F>) -> Self {
        Region { region }
    }
}

impl<'r, F: Field> Region<'r, F> {
    /// Enables a selector at the given offset.
    pub(crate) fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        offset: usize,
    ) -> Result<(), Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region
            .enable_selector(&|| annotation().into(), selector, offset)
    }

    /// Assign an advice column value (witness).
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    pub fn assign_advice<'v, V, VR, A, AR>(
        &'v mut self,
        annotation: A,
        column: Column<Advice>,
        offset: usize,
        mut to: V,
    ) -> Result<Cell, Error>
    where
        V: FnMut() -> Result<VR, Error> + 'v,
        VR: Into<Assigned<F>>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region
            .assign_advice(&|| annotation().into(), column, offset, &mut || {
                to().map(|v| v.into())
            })
    }

    /// Assigns a constant value to the column `advice` at `offset` within this region.
    ///
    /// The constant value will be assigned to a cell within one of the fixed columns
    /// configured via `ConstraintSystem::enable_constant`.
    ///
    /// Returns the advice cell.
    pub fn assign_advice_from_constant<VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        offset: usize,
        constant: VR,
    ) -> Result<Cell, Error>
    where
        VR: Into<Assigned<F>>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region.assign_advice_from_constant(
            &|| annotation().into(),
            column,
            offset,
            constant.into(),
        )
    }

    /// Assign the value of the instance column's cell at absolute location
    /// `row` to the column `advice` at `offset` within this region.
    ///
    /// Returns the advice cell, and its value if known.
    pub fn assign_advice_from_instance<A, AR>(
        &mut self,
        annotation: A,
        instance: Column<Instance>,
        row: usize,
        advice: Column<Advice>,
        offset: usize,
    ) -> Result<(Cell, Option<F>), Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region.assign_advice_from_instance(
            &|| annotation().into(),
            instance,
            row,
            advice,
            offset,
        )
    }

    /// Assign a fixed value.
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    pub fn assign_fixed<'v, V, VR, A, AR>(
        &'v mut self,
        annotation: A,
        column: Column<Fixed>,
        offset: usize,
        mut to: V,
    ) -> Result<Cell, Error>
    where
        V: FnMut() -> Result<VR, Error> + 'v,
        VR: Into<Assigned<F>>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region
            .assign_fixed(&|| annotation().into(), column, offset, &mut || {
                to().map(|v| v.into())
            })
    }

    /// Constrains a cell to have a constant value.
    ///
    /// Returns an error if the cell is in a column where equality has not been enabled.
    pub fn constrain_constant<VR>(&mut self, cell: Cell, constant: VR) -> Result<(), Error>
    where
        VR: Into<Assigned<F>>,
    {
        self.region.constrain_constant(cell, constant.into())
    }

    /// Constrains two cells to have the same value.
    ///
    /// Returns an error if either of the cells are in columns where equality
    /// has not been enabled.
    pub fn constrain_equal(&mut self, left: Cell, right: Cell) -> Result<(), Error> {
        self.region.constrain_equal(left, right)
    }
}

/// A lookup table in the circuit.
#[derive(Debug)]
pub struct Table<'r, F: Field> {
    table: &'r mut dyn layouter::TableLayouter<F>,
}

impl<'r, F: Field> From<&'r mut dyn layouter::TableLayouter<F>> for Table<'r, F> {
    fn from(table: &'r mut dyn layouter::TableLayouter<F>) -> Self {
        Table { table }
    }
}

impl<'r, F: Field> Table<'r, F> {
    /// Assigns a fixed value to a table cell.
    ///
    /// Returns an error if the table cell has already been assigned to.
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    pub fn assign_cell<'v, V, VR, A, AR>(
        &'v mut self,
        annotation: A,
        column: TableColumn,
        offset: usize,
        mut to: V,
    ) -> Result<(), Error>
    where
        V: FnMut() -> Result<VR, Error> + 'v,
        VR: Into<Assigned<F>>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.table
            .assign_cell(&|| annotation().into(), column, offset, &mut || {
                to().map(|v| v.into())
            })
    }
}

/// A layout strategy within a circuit. The layouter is chip-agnostic and applies its
/// strategy to the context and config it is given.
///
/// This abstracts over the circuit assignments, handling row indices etc.
///
pub trait Layouter<F: Field> {
    /// Represents the type of the "root" of this layouter, so that nested namespaces
    /// can minimize indirection.
    type Root: Layouter<F>;

    /// Assign a region of gates to an absolute row number.
    ///
    /// Inside the closure, the chip may freely use relative offsets; the `Layouter` will
    /// treat these assignments as a single "region" within the circuit. Outside this
    /// closure, the `Layouter` is allowed to optimise as it sees fit.
    ///
    /// ```ignore
    /// fn assign_region(&mut self, || "region name", |region| {
    ///     let config = chip.config();
    ///     region.assign_advice(config.a, offset, || { Some(value)});
    /// });
    /// ```
    fn assign_region<A, AR, N, NR>(&mut self, name: N, assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>;

    /// Assign a table region to an absolute row number.
    ///
    /// ```ignore
    /// fn assign_table(&mut self, || "table name", |table| {
    ///     let config = chip.config();
    ///     table.assign_fixed(config.a, offset, || { Some(value)});
    /// });
    /// ```
    fn assign_table<A, N, NR>(&mut self, name: N, assignment: A) -> Result<(), Error>
    where
        A: FnMut(Table<'_, F>) -> Result<(), Error>,
        N: Fn() -> NR,
        NR: Into<String>;

    /// Constrains a [`Cell`] to equal an instance column's row value at an
    /// absolute position.
    fn constrain_instance(
        &mut self,
        cell: Cell,
        column: Column<Instance>,
        row: usize,
    ) -> Result<(), Error>;

    /// Gets the "root" of this assignment, bypassing the namespacing.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn get_root(&mut self) -> &mut Self::Root;

    /// Creates a new (sub)namespace and enters into it.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR;

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, gadget_name: Option<String>);

    /// Enters into a namespace.
    fn namespace<NR, N>(&mut self, name_fn: N) -> NamespacedLayouter<'_, F, Self::Root>
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.get_root().push_namespace(name_fn);

        NamespacedLayouter(self.get_root(), PhantomData)
    }
}

/// This is a "namespaced" layouter which borrows a `Layouter` (pushing a namespace
/// context) and, when dropped, pops out of the namespace context.
#[derive(Debug)]
pub struct NamespacedLayouter<'a, F: Field, L: Layouter<F> + 'a>(&'a mut L, PhantomData<F>);

impl<'a, F: Field, L: Layouter<F> + 'a> Layouter<F> for NamespacedLayouter<'a, F, L> {
    type Root = L::Root;

    fn assign_region<A, AR, N, NR>(&mut self, name: N, assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.0.assign_region(name, assignment)
    }

    fn assign_table<A, N, NR>(&mut self, name: N, assignment: A) -> Result<(), Error>
    where
        A: FnMut(Table<'_, F>) -> Result<(), Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.0.assign_table(name, assignment)
    }

    fn constrain_instance(
        &mut self,
        cell: Cell,
        column: Column<Instance>,
        row: usize,
    ) -> Result<(), Error> {
        self.0.constrain_instance(cell, column, row)
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self.0.get_root()
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        panic!("Only the root's push_namespace should be called");
    }

    fn pop_namespace(&mut self, _gadget_name: Option<String>) {
        panic!("Only the root's pop_namespace should be called");
    }
}

impl<'a, F: Field, L: Layouter<F> + 'a> Drop for NamespacedLayouter<'a, F, L> {
    fn drop(&mut self) {
        let gadget_name = {
            #[cfg(feature = "gadget-traces")]
            {
                let mut gadget_name = None;
                let mut is_second_frame = false;
                backtrace::trace(|frame| {
                    if is_second_frame {
                        // Resolve this instruction pointer to a symbol name.
                        backtrace::resolve_frame(frame, |symbol| {
                            gadget_name = symbol.name().map(|name| format!("{:#}", name));
                        });

                        // We are done!
                        false
                    } else {
                        // We want the next frame.
                        is_second_frame = true;
                        true
                    }
                });
                gadget_name
            }

            #[cfg(not(feature = "gadget-traces"))]
            None
        };

        self.get_root().pop_namespace(gadget_name);
    }
}
