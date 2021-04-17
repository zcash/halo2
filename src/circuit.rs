//! Traits and structs for implementing circuit components.

use std::{fmt, marker::PhantomData};

use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Any, Column, Error, Fixed, Permutation},
};

pub mod layouter;

/// TODO
pub trait Chip<F: FieldExt, Co: Core<F>> {
    /// Represents the type of the "root" of this chip, so that nested namespaces
    /// can minimize indirection.
    type Root: Chip<F, Co>;

    /// TODO
    type Config;

    /// TODO
    type Layouter;

    /// TODO
    fn new(config: Self::Config, layouter: Self::Layouter) -> Self::Root;

    /// Gets the "root" of this assignment, bypassing the namespacing.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn root(&mut self) -> &mut Self::Root;

    /// Access the core inside this chip.
    fn core(&mut self) -> &mut Co;

    /// Creates a new (sub)namespace and enters into it.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, _gadget_name: Option<String>) {}

    /// Enters into a namespace.
    fn namespace<NR, N>(&mut self, name_fn: N) -> NamespacedChip<'_, F, Co, Self::Root>
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.root().push_namespace(name_fn);

        NamespacedChip(self.root(), PhantomData, PhantomData)
    }
}

/// A chip implements a set of instructions that can be used by gadgets.
///
/// The chip itself should not store any state; instead, state that is required at circuit
/// synthesis time should be stored in [`Core::Config`], which can then be fetched via
/// [`Layouter::config`].
pub trait Core<F: FieldExt>: Sized {
    /// A type that holds the configuration for this chip, and any other state it may need
    /// during circuit synthesis, that can be derived during [`Circuit::configure`].
    ///
    /// [`Circuit::configure`]: crate::plonk::Circuit::configure
    type Config: fmt::Debug;

    /// A type that holds any general core state that needs to be loaded at the start of
    /// [`Circuit::synthesize`]. This might simply be `()` for some cores.
    ///
    /// [`Circuit::synthesize`]: crate::plonk::Circuit::synthesize
    type Loaded: fmt::Debug;

    /// Layouter type
    type Layouter: Layouter<F>;

    // /// Instantiate a new core given a configuration and layouter.
    // fn new(config: Self::Config, layouter: Self::Layouter) -> Self;

    /// Access `Config`
    fn config(&self) -> &Self::Config;

    /// Access `Loaded`
    fn loaded(&self) -> &Self::Loaded;

    /// Load any fixed configuration for this core into the circuit.
    ///
    /// `layouter.loaded()` will panic if called inside this function.
    fn load(&mut self) -> Result<Self::Loaded, Error>;

    /// The layouter for this core.
    fn layouter(&mut self) -> &mut Self::Layouter;
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

/// A region of the circuit in which a [`Core`] can assign cells.
///
/// Inside a region, the core may freely use relative offsets; the [`Layouter`] will
/// treat these assignments as a single "region" within the circuit.
///
/// The [`Layouter`] is allowed to optimise between regions as it sees fit. Cores must use
/// [`Region::constrain_equal`] to copy in variables assigned in other regions.
///
/// TODO: It would be great if we could constrain the columns in these types to be
/// "logical" columns that are guaranteed to correspond to the core (and have come from
/// `Core::Config`).
#[derive(Debug)]
pub struct Region<'r, F: FieldExt, C: Core<F>> {
    region: &'r mut dyn layouter::RegionLayouter<F, C>,
}

impl<'r, F: FieldExt, C: Core<F>> From<&'r mut dyn layouter::RegionLayouter<F, C>>
    for Region<'r, F, C>
{
    fn from(region: &'r mut dyn layouter::RegionLayouter<F, C>) -> Self {
        Region { region }
    }
}

impl<'r, F: FieldExt, C: Core<F>> Region<'r, F, C> {
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

/// A layout strategy for a specific core within a circuit.
///
/// This abstracts over the circuit assignments, handling row indices etc.
///
/// A particular concrete layout strategy will implement this trait for each core it
/// supports.
pub trait Layouter<F: FieldExt> {
    /// Assign a region of gates to an absolute row number.
    ///
    /// Inside the closure, the core may freely use relative offsets; the `Layouter` will
    /// treat these assignments as a single "region" within the circuit. Outside this
    /// closure, the `Layouter` is allowed to optimise as it sees fit.
    ///
    /// ```ignore
    /// fn assign_region(&mut self, || "region name", |region| {
    ///     region.assign_advice(self.config.a, offset, || { Some(value)});
    /// });
    /// ```
    fn assign_region<A, AR, N, NR, C: Core<F>>(
        &mut self,
        name: N,
        assignment: A,
    ) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F, C>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>;
}

impl<F: FieldExt> Layouter<F> for () {
    fn assign_region<A, AR, N, NR, C: Core<F>>(
        &mut self,
        _name: N,
        _assignment: A,
    ) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F, C>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        Err(Error::SynthesisError)
    }
}

/// This is a "namespaced" layouter which borrows a `Layouter` (pushing a namespace
/// context) and, when dropped, pops out of the namespace context.
#[derive(Debug)]
pub struct NamespacedChip<'a, F: FieldExt, Co: Core<F>, Ch: Chip<F, Co>>(
    &'a mut Ch,
    PhantomData<F>,
    PhantomData<Co>,
);

impl<'a, F: FieldExt, Co: Core<F>, Ch: Chip<F, Co> + 'a> Chip<F, Co>
    for NamespacedChip<'a, F, Co, Ch>
{
    type Root = Ch::Root;
    type Config = Ch::Config;
    type Layouter = Ch::Layouter;

    fn new(config: Self::Config, layouter: Self::Layouter) -> Self::Root {
        Ch::new(config, layouter)
    }

    fn root(&mut self) -> &mut Self::Root {
        self.0.root()
    }

    fn core(&mut self) -> &mut Co {
        self.0.core()
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

impl<'a, F: FieldExt, Co: Core<F>, Ch: Chip<F, Co>> Drop for NamespacedChip<'a, F, Co, Ch> {
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

        self.root().pop_namespace(gadget_name);
    }
}
