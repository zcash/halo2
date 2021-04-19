//! Traits and structs for implementing circuit components.

use std::{collections::BTreeMap, fmt, marker::PhantomData};

use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Any, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};

pub mod layouter;

/// A chip's configuration (i.e. columns, selectors, permutations).
pub trait Config: fmt::Debug + Clone {
    /// An empty config
    fn empty() -> Self;
}

/// A chip's loaded configuration (i.e. lookups, pre-computed fixed constants).
pub trait Loaded: fmt::Debug + Clone {
    /// An empty loaded configuration
    fn empty() -> Self;
}

impl Loaded for () {
    fn empty() -> Self {}
}

/// A chip implements a set of instructions that can be used by gadgets.
///
/// The chip stores state that is required at circuit synthesis time in
/// [`Chip::Config`], which can be fetched via [`Chip::config`].
///
/// The chip also loads any fixed configuration needed at synthesis time
/// using [`Chip::load`], and stores it in [`Chip::Loaded`]. This can be
/// accessed via [`Chip::loaded`].
pub trait Chip<F: FieldExt>: Sized {
    /// A type that holds the configuration for this chip, and any other state it may need
    /// during circuit synthesis, that can be derived during [`Circuit::configure`].
    ///
    /// [`Circuit::configure`]: crate::plonk::Circuit::configure
    type Config: Config;

    /// A type that holds any general chip state that needs to be loaded at the start of
    /// [`Circuit::synthesize`]. This might simply be `()` for some chips.
    ///
    /// [`Circuit::synthesize`]: crate::plonk::Circuit::synthesize
    type Loaded: Loaded;

    /// A new chip that is not yet configured or loaded.
    fn new() -> Self;

    /// A chip constructed from a given config and loaded state.
    fn construct(config: Self::Config, loaded: Self::Loaded) -> Self;

    /// The chip is responsible for mutating the constraint system to set up
    /// the columns, gates, lookups and permutations it needs.
    ///
    /// We allow for the chip to use pre-existing columns, selectors, and
    /// permutations in its configuration. Chips that are lower in a hierarchy
    /// depend on other chips to pass down these objects.
    fn configure(
        &mut self,
        meta: &mut ConstraintSystem<F>,
        selectors: BTreeMap<&str, Selector>,
        columns: BTreeMap<&str, Column<Any>>,
        perms: BTreeMap<&str, Permutation>,
    ) -> Self::Config;

    /// The chip holds its own configuration.
    fn config(&self) -> &Self::Config;

    /// Load any fixed configuration for this chip into the circuit, i.e.
    /// assigns cells in fixed columns.
    ///
    /// `self.loaded()` will panic if called inside this function.
    fn load(&mut self, layouter: &mut impl Layouter<F>) -> Result<Self::Loaded, Error>;

    /// Provides access to general chip state loaded at the beginning of circuit
    /// synthesis.
    ///
    /// Panics if called inside `Chip::load`.
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
#[derive(Clone, Copy, Debug)]
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

/// A structure containing a cell and its assigned value.
#[derive(Clone, Debug)]
pub struct CellValue<T> {
    /// The cell of this `CellValue`
    pub cell: Cell,
    /// The value assigned to this `CellValue`
    pub value: Option<T>,
}

impl<T> CellValue<T> {
    /// Construct a `CellValue`.
    pub fn new(cell: Cell, value: Option<T>) -> Self {
        CellValue { cell, value }
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
pub struct Region<'r, F: FieldExt> {
    region: &'r mut dyn layouter::RegionLayouter<F>,
}

impl<'r, F: FieldExt> From<&'r mut dyn layouter::RegionLayouter<F>> for Region<'r, F> {
    fn from(region: &'r mut dyn layouter::RegionLayouter<F>) -> Self {
        Region { region }
    }
}

impl<'r, F: FieldExt> Region<'r, F> {
    /// Assign an advice column value (witness).
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    pub fn assign_advice<'v, V, A, AR>(
        &'v mut self,
        annotation: A,
        column: Column<Advice>,
        offset: usize,
        mut to: V,
    ) -> Result<Cell, Error>
    where
        V: FnMut() -> Result<F, Error> + 'v,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region
            .assign_advice(&|| annotation().into(), column, offset, &mut to)
    }

    /// Assign a fixed value.
    ///
    /// Even though `to` has `FnMut` bounds, it is guaranteed to be called at most once.
    pub fn assign_fixed<'v, V, A, AR>(
        &'v mut self,
        annotation: A,
        column: Column<Fixed>,
        offset: usize,
        mut to: V,
    ) -> Result<Cell, Error>
    where
        V: FnMut() -> Result<F, Error> + 'v,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.region
            .assign_fixed(&|| annotation().into(), column, offset, &mut to)
    }

    /// Constraint two cells to have the same value.
    ///
    /// Returns an error if either of the cells is not within the given permutation.
    pub fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error> {
        self.region.constrain_equal(permutation, left, right)
    }
}

/// A layout strategy within a circuit. The layouter is chip-agnostic and applies its
/// strategy to the context and config it is given.
///
/// This abstracts over the circuit assignments, handling row indices etc.
///
pub trait Layouter<F: FieldExt> {
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
    ///     let config = chip.config().clone();
    ///     region.assign_advice(config.a, offset, || { Some(value)});
    /// });
    /// ```
    fn assign_region<A, AR, N, NR>(&mut self, name: N, assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>;

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
pub struct NamespacedLayouter<'a, F: FieldExt, L: Layouter<F> + 'a>(&'a mut L, PhantomData<F>);

impl<'a, F: FieldExt, L: Layouter<F> + 'a> Layouter<F> for NamespacedLayouter<'a, F, L> {
    type Root = L::Root;

    fn assign_region<A, AR, N, NR>(&mut self, name: N, assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.0.assign_region(name, assignment)
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

impl<'a, F: FieldExt, L: Layouter<F> + 'a> Drop for NamespacedLayouter<'a, F, L> {
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
