//! Traits and structs for implementing circuit components.

use crate::plonk;
use crate::plonk::{
    permutation,
    sealed::{self, SealedPhase},
    Advice, Assignment, Circuit, ConstraintSystem, FirstPhase, Fixed, FloorPlanner, Instance,
    SecondPhase, ThirdPhase,
};
use halo2_middleware::circuit::{Any, CompiledCircuit, Preprocessing};
use halo2_middleware::ff::{BatchInvert, Field};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::RangeTo;

pub mod floor_planner;
mod table_layouter;

use std::{fmt, marker::PhantomData};

use crate::plonk::Assigned;
use crate::plonk::{Challenge, Column, Error, Selector, TableColumn};

mod value;
pub use value::Value;

pub use floor_planner::single_pass::SimpleFloorPlanner;

pub mod layouter;

pub use table_layouter::{SimpleTableLayouter, TableLayouter};

/// Compile a circuit.  Runs configure and synthesize on the circuit in order to materialize the
/// circuit into its columns and the column configuration; as well as doing the fixed column and
/// copy constraints assignments.  The output of this function can then be used for the key
/// generation, and proof generation.
/// If `compress_selectors` is true, multiple selector columns may be multiplexed.
#[allow(clippy::type_complexity)]
pub fn compile_circuit<F: Field, ConcreteCircuit: Circuit<F>>(
    k: u32,
    circuit: &ConcreteCircuit,
    compress_selectors: bool,
) -> Result<
    (
        CompiledCircuit<F>,
        ConcreteCircuit::Config,
        ConstraintSystem<F>,
    ),
    Error,
> {
    let n = 2usize.pow(k);

    let mut cs = ConstraintSystem::default();
    #[cfg(feature = "circuit-params")]
    let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
    #[cfg(not(feature = "circuit-params"))]
    let config = ConcreteCircuit::configure(&mut cs);
    let cs = cs;

    if n < cs.minimum_rows() {
        return Err(Error::not_enough_rows_available(k));
    }

    let mut assembly = plonk::keygen::Assembly {
        k,
        fixed: vec![vec![F::ZERO.into(); n]; cs.num_fixed_columns],
        permutation: permutation::Assembly::new(n, &cs.permutation),
        selectors: vec![vec![false; n]; cs.num_selectors],
        usable_rows: 0..n - (cs.blinding_factors() + 1),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    ConcreteCircuit::FloorPlanner::synthesize(
        &mut assembly,
        circuit,
        config.clone(),
        cs.constants.clone(),
    )?;

    let mut fixed = batch_invert_assigned(assembly.fixed);
    let (cs, selector_polys) = if compress_selectors {
        cs.compress_selectors(assembly.selectors)
    } else {
        // After this, the ConstraintSystem should not have any selectors: `verify` does not need them, and `keygen_pk` regenerates `cs` from scratch anyways.
        let selectors = std::mem::take(&mut assembly.selectors);
        cs.directly_convert_selectors_to_fixed(selectors)
    };

    fixed.extend(selector_polys);

    // sort the "copies" for deterministic ordering
    #[cfg(feature = "thread-safe-region")]
    assembly.permutation.copies.sort();

    let preprocessing = Preprocessing {
        permutation: halo2_middleware::permutation::AssemblyMid {
            copies: assembly.permutation.copies,
        },
        fixed,
    };

    Ok((
        CompiledCircuit {
            cs: cs.clone().into(),
            preprocessing,
        },
        config,
        cs,
    ))
}

struct WitnessCollection<'a, F: Field> {
    k: u32,
    current_phase: sealed::Phase,
    advice_column_phase: &'a Vec<sealed::Phase>,
    advice: Vec<Vec<Assigned<F>>>,
    challenges: &'a HashMap<usize, F>,
    instances: &'a [Vec<F>],
    usable_rows: RangeTo<usize>,
}

impl<'a, F: Field> Assignment<F> for WitnessCollection<'a, F> {
    fn enter_region<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about regions in this context.
    }

    fn exit_region(&mut self) {
        // Do nothing; we don't care about regions in this context.
    }

    fn enable_selector<A, AR>(&mut self, _: A, _: &Selector, _: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn annotate_column<A, AR>(&mut self, _annotation: A, _column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Do nothing
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Value<F>, Error> {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        self.instances
            .get(column.index())
            .and_then(|column| column.get(row))
            .map(|v| Value::known(*v))
            .ok_or(Error::BoundsFailure)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Ignore assignment of advice column in different phase than current one.
        let phase = self.advice_column_phase[column.index];
        if self.current_phase != phase {
            return Ok(());
        }

        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        *self
            .advice
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = to().into_field().assign()?;

        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Fixed>,
        _: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn copy(&mut self, _: Column<Any>, _: usize, _: Column<Any>, _: usize) -> Result<(), Error> {
        // We only care about advice columns here

        Ok(())
    }

    fn fill_from_row(
        &mut self,
        _: Column<Fixed>,
        _: usize,
        _: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_challenge(&self, challenge: Challenge) -> Value<F> {
        self.challenges
            .get(&challenge.index())
            .cloned()
            .map(Value::known)
            .unwrap_or_else(Value::unknown)
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about namespaces in this context.
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // Do nothing; we don't care about namespaces in this context.
    }
}

/// Witness calculator.  Frontend function
#[derive(Debug)]
pub struct WitnessCalculator<'a, F: Field, ConcreteCircuit: Circuit<F>> {
    k: u32,
    n: usize,
    unusable_rows_start: usize,
    circuit: &'a ConcreteCircuit,
    config: &'a ConcreteCircuit::Config,
    cs: &'a ConstraintSystem<F>,
    instances: &'a [Vec<F>],
    next_phase: u8,
}

impl<'a, F: Field, ConcreteCircuit: Circuit<F>> WitnessCalculator<'a, F, ConcreteCircuit> {
    /// Create a new WitnessCalculator
    pub fn new(
        k: u32,
        circuit: &'a ConcreteCircuit,
        config: &'a ConcreteCircuit::Config,
        cs: &'a ConstraintSystem<F>,
        instances: &'a [Vec<F>],
    ) -> Self {
        let n = 2usize.pow(k);
        let unusable_rows_start = n - (cs.blinding_factors() + 1);
        Self {
            k,
            n,
            unusable_rows_start,
            circuit,
            config,
            cs,
            instances,
            next_phase: 0,
        }
    }

    /// Calculate witness at phase
    pub fn calc(
        &mut self,
        phase: u8,
        challenges: &HashMap<usize, F>,
    ) -> Result<Vec<Option<Vec<F>>>, Error> {
        if phase != self.next_phase {
            return Err(Error::Other(format!(
                "Expected phase {}, got {}",
                self.next_phase, phase
            )));
        }
        let current_phase = match phase {
            0 => FirstPhase.to_sealed(),
            1 => SecondPhase.to_sealed(),
            2 => ThirdPhase.to_sealed(),
            _ => unreachable!("only phase [0,2] supported"),
        };

        let mut witness = WitnessCollection {
            k: self.k,
            current_phase,
            advice_column_phase: &self.cs.advice_column_phase,
            advice: vec![vec![Assigned::Zero; self.n]; self.cs.num_advice_columns],
            instances: self.instances,
            challenges,
            // The prover will not be allowed to assign values to advice
            // cells that exist within inactive rows, which include some
            // number of blinding factors and an extra row for use in the
            // permutation argument.
            usable_rows: ..self.unusable_rows_start,
        };

        // Synthesize the circuit to obtain the witness and other information.
        ConcreteCircuit::FloorPlanner::synthesize(
            &mut witness,
            self.circuit,
            self.config.clone(),
            self.cs.constants.clone(),
        )
        .expect("todo");

        let column_indices = self
            .cs
            .advice_column_phase
            .iter()
            .enumerate()
            .filter_map(|(column_index, phase)| {
                if current_phase == *phase {
                    Some(column_index)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        self.next_phase += 1;
        let advice_values = batch_invert_assigned(witness.advice);
        Ok(advice_values
            .into_iter()
            .enumerate()
            .map(|(column_index, advice)| {
                if column_indices.contains(&column_index) {
                    Some(advice)
                } else {
                    None
                }
            })
            .collect())
    }
}

// Turn vectors of `Assigned<F>` into vectors of `F` by evaluation the divisions in `Assigned<F>`
// using batched inversions.
fn batch_invert_assigned<F: Field>(assigned: Vec<Vec<Assigned<F>>>) -> Vec<Vec<F>> {
    let mut assigned_denominators: Vec<_> = assigned
        .iter()
        .map(|f| {
            f.iter()
                .map(|value| value.denominator())
                .collect::<Vec<_>>()
        })
        .collect();

    assigned_denominators
        .iter_mut()
        .flat_map(|f| {
            f.iter_mut()
                // If the denominator is trivial, we can skip it, reducing the
                // size of the batch inversion.
                .filter_map(|d| d.as_mut())
        })
        .batch_invert();

    assigned
        .iter()
        .zip(assigned_denominators)
        .map(|(poly, inv_denoms)| {
            poly_invert(poly, inv_denoms.into_iter().map(|d| d.unwrap_or(F::ONE)))
        })
        .collect()
}

// Turn a slice of `Assigned<F>` into a vector of F by multiplying each numerator with the elements
// from `inv_denoms`, assuming that `inv_denoms` are the inverted denominators of the
// `Assigned<F>`.
fn poly_invert<F: Field>(
    poly: &[Assigned<F>],
    inv_denoms: impl ExactSizeIterator<Item = F>,
) -> Vec<F> {
    assert_eq!(inv_denoms.len(), poly.len());
    poly.iter()
        .zip(inv_denoms)
        .map(|(a, inv_den)| a.numerator() * inv_den)
        .collect()
}

/// A chip implements a set of instructions that can be used by gadgets.
///
/// The chip stores state that is required at circuit synthesis time in
/// [`Chip::Config`], which can be fetched via [`Chip::config`].
///
/// The chip also loads any fixed configuration needed at synthesis time
/// using its own implementation of `load`, and stores it in [`Chip::Loaded`].
/// This can be accessed via [`Chip::loaded`].
pub trait Chip<F: Field>: Sized {
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
    pub region_index: RegionIndex,
    /// The relative offset of this cell within its region.
    pub row_offset: usize,
    /// The column of this cell.
    pub column: Column<Any>,
}

/// An assigned cell.
#[derive(Clone, Debug)]
pub struct AssignedCell<V, F: Field> {
    value: Value<V>,
    cell: Cell,
    _marker: PhantomData<F>,
}

impl<V, F: Field> AssignedCell<V, F> {
    /// Returns the value of the [`AssignedCell`].
    pub fn value(&self) -> Value<&V> {
        self.value.as_ref()
    }

    /// Returns the cell.
    pub fn cell(&self) -> Cell {
        self.cell
    }
}

impl<V, F: Field> AssignedCell<V, F>
where
    for<'v> Assigned<F>: From<&'v V>,
{
    /// Returns the field element value of the [`AssignedCell`].
    pub fn value_field(&self) -> Value<Assigned<F>> {
        self.value.to_field()
    }
}

impl<F: Field> AssignedCell<Assigned<F>, F> {
    /// Evaluates this assigned cell's value directly, performing an unbatched inversion
    /// if necessary.
    ///
    /// If the denominator is zero, the returned cell's value is zero.
    pub fn evaluate(self) -> AssignedCell<F, F> {
        AssignedCell {
            value: self.value.evaluate(),
            cell: self.cell,
            _marker: Default::default(),
        }
    }
}

impl<V: Clone, F: Field> AssignedCell<V, F>
where
    for<'v> Assigned<F>: From<&'v V>,
{
    /// Copies the value to a given advice cell and constrains them to be equal.
    ///
    /// Returns an error if either this cell or the given cell are in columns
    /// where equality has not been enabled.
    pub fn copy_advice<A, AR>(
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
        let assigned_cell =
            region.assign_advice(annotation, column, offset, || self.value.clone())?;
        region.constrain_equal(assigned_cell.cell(), self.cell())?;

        Ok(assigned_cell)
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
    pub fn enable_selector<A, AR>(
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

    /// Allows the circuit implementor to name/annotate a Column within a Region context.
    ///
    /// This is useful in order to improve the amount of information that `prover.verify()`
    /// and `prover.assert_satisfied()` can provide.
    pub fn name_column<A, AR, T>(&mut self, annotation: A, column: T)
    where
        A: Fn() -> AR,
        AR: Into<String>,
        T: Into<Column<Any>>,
    {
        self.region
            .name_column(&|| annotation().into(), column.into());
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
    ) -> Result<AssignedCell<VR, F>, Error>
    where
        V: FnMut() -> Value<VR> + 'v,
        for<'vr> Assigned<F>: From<&'vr VR>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let mut value = Value::unknown();
        let cell =
            self.region
                .assign_advice(&|| annotation().into(), column, offset, &mut || {
                    let v = to();
                    let value_f = v.to_field();
                    value = v;
                    value_f
                })?;

        Ok(AssignedCell {
            value,
            cell,
            _marker: PhantomData,
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
    ) -> Result<AssignedCell<VR, F>, Error>
    where
        for<'vr> Assigned<F>: From<&'vr VR>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let cell = self.region.assign_advice_from_constant(
            &|| annotation().into(),
            column,
            offset,
            (&constant).into(),
        )?;

        Ok(AssignedCell {
            value: Value::known(constant),
            cell,
            _marker: PhantomData,
        })
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
    ) -> Result<AssignedCell<F, F>, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let (cell, value) = self.region.assign_advice_from_instance(
            &|| annotation().into(),
            instance,
            row,
            advice,
            offset,
        )?;

        Ok(AssignedCell {
            value,
            cell,
            _marker: PhantomData,
        })
    }

    /// Returns the value of the instance column's cell at absolute location `row`.
    ///
    /// This method is only provided for convenience; it does not create any constraints.
    /// Callers still need to use [`Self::assign_advice_from_instance`] to constrain the
    /// instance values in their circuit.
    pub fn instance_value(
        &mut self,
        instance: Column<Instance>,
        row: usize,
    ) -> Result<Value<F>, Error> {
        self.region.instance_value(instance, row)
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
    ) -> Result<AssignedCell<VR, F>, Error>
    where
        V: FnMut() -> Value<VR> + 'v,
        for<'vr> Assigned<F>: From<&'vr VR>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let mut value = Value::unknown();
        let cell =
            self.region
                .assign_fixed(&|| annotation().into(), column, offset, &mut || {
                    let v = to();
                    let value_f = v.to_field();
                    value = v;
                    value_f
                })?;

        Ok(AssignedCell {
            value,
            cell,
            _marker: PhantomData,
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
    table: &'r mut dyn TableLayouter<F>,
}

impl<'r, F: Field> From<&'r mut dyn TableLayouter<F>> for Table<'r, F> {
    fn from(table: &'r mut dyn TableLayouter<F>) -> Self {
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
        V: FnMut() -> Value<VR> + 'v,
        VR: Into<Assigned<F>>,
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.table
            .assign_cell(&|| annotation().into(), column, offset, &mut || {
                to().into_field()
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

    /// Queries the value of the given challenge.
    ///
    /// Returns `Value::unknown()` if the current synthesis phase is before the challenge can be queried.
    fn get_challenge(&self, challenge: Challenge) -> Value<F>;

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

    fn get_challenge(&self, challenge: Challenge) -> Value<F> {
        self.0.get_challenge(challenge)
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
                            gadget_name = symbol.name().map(|name| format!("{name:#}"));
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
