//! Tools for developing circuits.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::iter;
use std::ops::{Add, Mul, Neg, Range};

use ff::Field;

use crate::plonk::Assigned;
use crate::{
    arithmetic::{FieldExt, Group},
    plonk::{
        permutation, Advice, Any, Assignment, Circuit, Column, ColumnType, ConstraintSystem, Error,
        Expression, Fixed, FloorPlanner, Instance, Selector, VirtualCell,
    },
    poly::Rotation,
};

pub mod metadata;
mod util;

pub mod cost;
pub use cost::CircuitCost;

mod gates;
pub use gates::CircuitGates;

#[cfg(feature = "dev-graph")]
mod graph;

#[cfg(feature = "dev-graph")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-graph")))]
pub use graph::{circuit_dot_graph, layout::CircuitLayout};

/// The location within the circuit at which a particular [`VerifyFailure`] occurred.
#[derive(Debug, PartialEq)]
pub enum FailureLocation {
    /// A location inside a region.
    InRegion {
        /// The region in which the failure occurred.
        region: metadata::Region,
        /// The offset (relative to the start of the region) at which the failure
        /// occurred.
        offset: usize,
    },
    /// A location outside of a region.
    OutsideRegion {
        /// The circuit row on which the failure occurred.
        row: usize,
    },
}

impl fmt::Display for FailureLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InRegion { region, offset } => write!(f, "in {} at offset {}", region, offset),
            Self::OutsideRegion { row } => {
                write!(f, "on row {}", row)
            }
        }
    }
}

impl FailureLocation {
    fn find_expressions<'a, F: Field>(
        cs: &ConstraintSystem<F>,
        regions: &[Region],
        failure_row: usize,
        failure_expressions: impl Iterator<Item = &'a Expression<F>>,
    ) -> Self {
        let failure_columns: HashSet<Column<Any>> = failure_expressions
            .flat_map(|expression| {
                expression.evaluate(
                    &|_| vec![],
                    &|_| panic!("virtual selectors are removed during optimization"),
                    &|index, _, _| vec![cs.fixed_queries[index].0.into()],
                    &|index, _, _| vec![cs.advice_queries[index].0.into()],
                    &|index, _, _| vec![cs.instance_queries[index].0.into()],
                    &|a| a,
                    &|mut a, mut b| {
                        a.append(&mut b);
                        a
                    },
                    &|mut a, mut b| {
                        a.append(&mut b);
                        a
                    },
                    &|a, _| a,
                )
            })
            .collect();

        Self::find(regions, failure_row, failure_columns)
    }

    /// Figures out whether the given row and columns overlap an assigned region.
    fn find(regions: &[Region], failure_row: usize, failure_columns: HashSet<Column<Any>>) -> Self {
        regions
            .iter()
            .enumerate()
            .find(|(_, r)| {
                let (start, end) = r.rows.unwrap();
                // We match the region if any input columns overlap, rather than all of
                // them, because matching complex selector columns is hard. As long as
                // regions are rectangles, and failures occur due to assignments entirely
                // within single regions, "any" will be equivalent to "all". If these
                // assumptions change, we'll start getting bug reports from users :)
                (start..=end).contains(&failure_row) && !failure_columns.is_disjoint(&r.columns)
            })
            .map(|(r_i, r)| FailureLocation::InRegion {
                region: (r_i, r.name.clone()).into(),
                offset: failure_row as usize - r.rows.unwrap().0 as usize,
            })
            .unwrap_or_else(|| FailureLocation::OutsideRegion {
                row: failure_row as usize,
            })
    }
}

/// The reasons why a particular circuit is not satisfied.
#[derive(Debug, PartialEq)]
pub enum VerifyFailure {
    /// A cell used in an active gate was not assigned to.
    CellNotAssigned {
        /// The index of the active gate.
        gate: metadata::Gate,
        /// The region in which this cell should be assigned.
        region: metadata::Region,
        /// The column in which this cell should be assigned.
        column: Column<Any>,
        /// The offset (relative to the start of the region) at which this cell should be
        /// assigned. This may be negative (for example, if a selector enables a gate at
        /// offset 0, but the gate uses `Rotation::prev()`).
        offset: isize,
    },
    /// A constraint was not satisfied for a particular row.
    ConstraintNotSatisfied {
        /// The polynomial constraint that is not satisfied.
        constraint: metadata::Constraint,
        /// The location at which this constraint is not satisfied.
        ///
        /// `FailureLocation::OutsideRegion` is usually caused by a constraint that does
        /// not contain a selector, and as a result is active on every row.
        location: FailureLocation,
        /// The values of the virtual cells used by this constraint.
        cell_values: Vec<(metadata::VirtualCell, String)>,
    },
    /// A constraint was active on an unusable row, and is likely missing a selector.
    ConstraintPoisoned {
        /// The polynomial constraint that is not satisfied.
        constraint: metadata::Constraint,
    },
    /// A lookup input did not exist in its corresponding table.
    Lookup {
        /// The name of the lookup that is not satisfied.
        name: &'static str,
        /// The index of the lookup that is not satisfied. These indices are assigned in
        /// the order in which `ConstraintSystem::lookup` is called during
        /// `Circuit::configure`.
        lookup_index: usize,
        /// The location at which the lookup is not satisfied.
        ///
        /// `FailureLocation::InRegion` is most common, and may be due to the intentional
        /// use of a lookup (if its inputs are conditional on a complex selector), or an
        /// unintentional lookup constraint that overlaps the region (indicating that the
        /// lookup's inputs should be made conditional).
        ///
        /// `FailureLocation::OutsideRegion` is uncommon, and could mean that:
        /// - The input expressions do not correctly constrain a default value that exists
        ///   in the table when the lookup is not being used.
        /// - The input expressions use a column queried at a non-zero `Rotation`, and the
        ///   lookup is active on a row adjacent to an unrelated region.
        location: FailureLocation,
    },
    /// A permutation did not preserve the original value of a cell.
    Permutation {
        /// The column in which this permutation is not satisfied.
        column: metadata::Column,
        /// The row on which this permutation is not satisfied.
        row: usize,
    },
}

impl fmt::Display for VerifyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellNotAssigned {
                gate,
                region,
                column,
                offset,
            } => {
                write!(
                    f,
                    "{} uses {}, which requires cell in column {:?} at offset {} to be assigned.",
                    region, gate, column, offset
                )
            }
            Self::ConstraintNotSatisfied {
                constraint,
                location,
                cell_values,
            } => {
                writeln!(f, "{} is not satisfied {}", constraint, location)?;
                for (name, value) in cell_values {
                    writeln!(f, "- {} = {}", name, value)?;
                }
                Ok(())
            }
            Self::ConstraintPoisoned { constraint } => {
                write!(
                    f,
                    "{} is active on an unusable row - missing selector?",
                    constraint
                )
            }
            Self::Lookup {
                name,
                lookup_index,
                location,
            } => {
                write!(
                    f,
                    "Lookup {}(index: {}) is not satisfied {}",
                    name, lookup_index, location
                )
            }
            Self::Permutation { column, row } => {
                write!(
                    f,
                    "Equality constraint not satisfied by cell ({:?}, {})",
                    column, row
                )
            }
        }
    }
}

#[derive(Debug)]
struct Region {
    /// The name of the region. Not required to be unique.
    name: String,
    /// The columns involved in this region.
    columns: HashSet<Column<Any>>,
    /// The rows that this region starts and ends on, if known.
    rows: Option<(usize, usize)>,
    /// The selectors that have been enabled in this region. All other selectors are by
    /// construction not enabled.
    enabled_selectors: HashMap<Selector, Vec<usize>>,
    /// The cells assigned in this region. We store this as a `HashMap` with count
    /// so that if any cells are double-assigned, they will be visibly darker.
    cells: HashMap<(Column<Any>, usize), usize>,
}

impl Region {
    fn update_extent(&mut self, column: Column<Any>, row: usize) {
        self.columns.insert(column);

        // The region start is the earliest row assigned to.
        // The region end is the latest row assigned to.
        let (mut start, mut end) = self.rows.unwrap_or((row, row));
        if row < start {
            // The first row assigned was not at start 0 within the region.
            start = row;
        }
        if row > end {
            end = row;
        }
        self.rows = Some((start, end));
    }

    fn track_cell(&mut self, column: Column<Any>, row: usize) {
        // Keep track of how many times this cell has been assigned to.
        let count = *self.cells.get(&(column, row)).unwrap_or(&0);
        self.cells.insert((column, row), count + 1);
    }

    fn is_assigned(&self, column: Column<Any>, row: usize) -> bool {
        self.cells.contains_key(&(column, row))
    }
}

/// The value of a particular cell within the circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellValue<F: Group + Field> {
    // An unassigned cell.
    Unassigned,
    // A cell that has been assigned a value.
    Assigned(F),
    // A unique poisoned cell.
    Poison(usize),
}

/// A value within an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
enum Value<F: Group + Field> {
    Real(F),
    Poison,
}

impl<F: Group + Field> From<CellValue<F>> for Value<F> {
    fn from(value: CellValue<F>) -> Self {
        match value {
            // Cells that haven't been explicitly assigned to, default to zero.
            CellValue::Unassigned => Value::Real(F::zero()),
            CellValue::Assigned(v) => Value::Real(v),
            CellValue::Poison(_) => Value::Poison,
        }
    }
}

impl<F: Group + Field> Neg for Value<F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Real(a) => Value::Real(-a),
            _ => Value::Poison,
        }
    }
}

impl<F: Group + Field> Add for Value<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Real(a), Value::Real(b)) => Value::Real(a + b),
            _ => Value::Poison,
        }
    }
}

impl<F: Group + Field> Mul for Value<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Real(a), Value::Real(b)) => Value::Real(a * b),
            // If poison is multiplied by zero, then we treat the poison as unconstrained
            // and we don't propagate it.
            (Value::Real(x), Value::Poison) | (Value::Poison, Value::Real(x))
                if x.is_zero_vartime() =>
            {
                Value::Real(F::zero())
            }
            _ => Value::Poison,
        }
    }
}

impl<F: Group + Field> Mul<F> for Value<F> {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        match self {
            Value::Real(lhs) => Value::Real(lhs * rhs),
            // If poison is multiplied by zero, then we treat the poison as unconstrained
            // and we don't propagate it.
            Value::Poison if rhs.is_zero_vartime() => Value::Real(F::zero()),
            _ => Value::Poison,
        }
    }
}

/// A test prover for debugging circuits.
///
/// The normal proving process, when applied to a buggy circuit implementation, might
/// return proofs that do not validate when they should, but it can't indicate anything
/// other than "something is invalid". `MockProver` can be used to figure out _why_ these
/// are invalid: it stores all the private inputs along with the circuit internals, and
/// then checks every constraint manually.
///
/// # Examples
///
/// ```
/// use halo2_proofs::{
///     arithmetic::FieldExt,
///     circuit::{Layouter, SimpleFloorPlanner},
///     dev::{FailureLocation, MockProver, VerifyFailure},
///     pairing::bn256::Fr as Fp,
///     plonk::{Advice, Any, Circuit, Column, ConstraintSystem, Error, Selector},
///     poly::Rotation,
/// };
/// const K: u32 = 5;
///
/// #[derive(Copy, Clone)]
/// struct MyConfig {
///     a: Column<Advice>,
///     b: Column<Advice>,
///     c: Column<Advice>,
///     s: Selector,
/// }
///
/// #[derive(Clone, Default)]
/// struct MyCircuit {
///     a: Option<u64>,
///     b: Option<u64>,
/// }
///
/// impl<F: FieldExt> Circuit<F> for MyCircuit {
///     type Config = MyConfig;
///     type FloorPlanner = SimpleFloorPlanner;
///
///     fn without_witnesses(&self) -> Self {
///         Self::default()
///     }
///
///     fn configure(meta: &mut ConstraintSystem<F>) -> MyConfig {
///         let a = meta.advice_column();
///         let b = meta.advice_column();
///         let c = meta.advice_column();
///         let s = meta.selector();
///
///         meta.create_gate("R1CS constraint", |meta| {
///             let a = meta.query_advice(a, Rotation::cur());
///             let b = meta.query_advice(b, Rotation::cur());
///             let c = meta.query_advice(c, Rotation::cur());
///             let s = meta.query_selector(s);
///
///             // BUG: Should be a * b - c
///             Some(("buggy R1CS", s * (a * b + c)))
///         });
///
///         MyConfig { a, b, c, s }
///     }
///
///     fn synthesize(&self, config: MyConfig, mut layouter: impl Layouter<F>) -> Result<(), Error> {
///         layouter.assign_region(|| "Example region", |mut region| {
///             config.s.enable(&mut region, 0)?;
///             region.assign_advice(|| "a", config.a, 0, || {
///                 self.a.map(|v| F::from(v)).ok_or(Error::Synthesis)
///             })?;
///             region.assign_advice(|| "b", config.b, 0, || {
///                 self.b.map(|v| F::from(v)).ok_or(Error::Synthesis)
///             })?;
///             region.assign_advice(|| "c", config.c, 0, || {
///                 self.a
///                     .and_then(|a| self.b.map(|b| F::from(a * b)))
///                     .ok_or(Error::Synthesis)
///             })?;
///             Ok(())
///         })
///     }
/// }
///
/// // Assemble the private inputs to the circuit.
/// let circuit = MyCircuit {
///     a: Some(2),
///     b: Some(4),
/// };
///
/// // This circuit has no public inputs.
/// let instance = vec![];
///
/// let prover = MockProver::<Fp>::run(K, &circuit, instance).unwrap();
/// assert_eq!(
///     prover.verify(),
///     Err(vec![VerifyFailure::ConstraintNotSatisfied {
///         constraint: ((0, "R1CS constraint").into(), 0, "buggy R1CS").into(),
///         location: FailureLocation::InRegion {
///             region: (0, "Example region").into(),
///             offset: 0,
///         },
///         cell_values: vec![
///             (((Any::Advice, 0).into(), 0).into(), "0x2".to_string()),
///             (((Any::Advice, 1).into(), 0).into(), "0x4".to_string()),
///             (((Any::Advice, 2).into(), 0).into(), "0x8".to_string()),
///         ],
///     }])
/// );
///
/// // If we provide a too-small K, we get an error.
/// assert!(matches!(
///     MockProver::<Fp>::run(2, &circuit, vec![]).unwrap_err(),
///     Error::NotEnoughRowsAvailable {
///         current_k,
///     } if current_k == 2,
/// ));
/// ```
#[derive(Debug)]
pub struct MockProver<F: Group + Field> {
    k: u32,
    n: u32,
    cs: ConstraintSystem<F>,

    /// The regions in the circuit.
    regions: Vec<Region>,
    /// The current region being assigned to. Will be `None` after the circuit has been
    /// synthesized.
    current_region: Option<Region>,

    // The fixed cells in the circuit, arranged as [column][row].
    fixed: Vec<Vec<CellValue<F>>>,
    // The advice cells in the circuit, arranged as [column][row].
    advice: Vec<Vec<CellValue<F>>>,
    // The instance cells in the circuit, arranged as [column][row].
    instance: Vec<Vec<F>>,

    selectors: Vec<Vec<bool>>,

    permutation: permutation::keygen::Assembly,

    // A range of available rows for assignment and copies.
    usable_rows: Range<usize>,
}

impl<F: Field + Group> Assignment<F> for MockProver<F> {
    fn enter_region<NR, N>(&mut self, name: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        assert!(self.current_region.is_none());
        self.current_region = Some(Region {
            name: name().into(),
            columns: HashSet::default(),
            rows: None,
            enabled_selectors: HashMap::default(),
            cells: HashMap::default(),
        });
    }

    fn exit_region(&mut self) {
        self.regions.push(self.current_region.take().unwrap());
    }

    fn enable_selector<A, AR>(&mut self, _: A, selector: &Selector, row: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        // Track that this selector was enabled. We require that all selectors are enabled
        // inside some region (i.e. no floating selectors).
        self.current_region
            .as_mut()
            .unwrap()
            .enabled_selectors
            .entry(*selector)
            .or_default()
            .push(row);

        self.selectors[selector.0][row] = true;

        Ok(())
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Option<F>, Error> {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        self.instance
            .get(column.index())
            .and_then(|column| column.get(row))
            .map(|v| Some(*v))
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
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        if let Some(region) = self.current_region.as_mut() {
            region.update_extent(column.into(), row);
            region.track_cell(column.into(), row);
        }

        *self
            .advice
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = CellValue::Assigned(to()?.into().evaluate());

        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        if let Some(region) = self.current_region.as_mut() {
            region.update_extent(column.into(), row);
            region.track_cell(column.into(), row);
        }

        *self
            .fixed
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = CellValue::Assigned(to()?.into().evaluate());

        Ok(())
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), crate::plonk::Error> {
        if !self.usable_rows.contains(&left_row) || !self.usable_rows.contains(&right_row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        self.permutation
            .copy(left_column, left_row, right_column, right_row)
    }

    fn fill_from_row(
        &mut self,
        col: Column<Fixed>,
        from_row: usize,
        to: Option<Assigned<F>>,
    ) -> Result<(), Error> {
        if !self.usable_rows.contains(&from_row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        for row in self.usable_rows.clone().skip(from_row) {
            self.assign_fixed(|| "", col, row, || to.ok_or(Error::Synthesis))?;
        }

        Ok(())
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // TODO: Do something with namespaces :)
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // TODO: Do something with namespaces :)
    }
}

impl<F: FieldExt> MockProver<F> {
    /// Runs a synthetic keygen-and-prove operation on the given circuit, collecting data
    /// about the constraints and their assignments.
    pub fn run<ConcreteCircuit: Circuit<F>>(
        k: u32,
        circuit: &ConcreteCircuit,
        instance: Vec<Vec<F>>,
    ) -> Result<Self, Error> {
        let n = 1 << k;

        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure(&mut cs);
        let cs = cs;

        if n < cs.minimum_rows() {
            return Err(Error::not_enough_rows_available(k));
        }

        if instance.len() != cs.num_instance_columns {
            return Err(Error::InvalidInstances);
        }

        let instance = instance
            .into_iter()
            .map(|mut instance| {
                if instance.len() > n - (cs.blinding_factors() + 1) {
                    return Err(Error::InstanceTooLarge);
                }

                instance.resize(n, F::zero());
                Ok(instance)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Fixed columns contain no blinding factors.
        let fixed = vec![vec![CellValue::Unassigned; n]; cs.num_fixed_columns];
        let selectors = vec![vec![false; n]; cs.num_selectors];
        // Advice columns contain blinding factors.
        let blinding_factors = cs.blinding_factors();
        let usable_rows = n - (blinding_factors + 1);
        let advice = vec![
            {
                let mut column = vec![CellValue::Unassigned; n];
                // Poison unusable rows.
                for (i, cell) in column.iter_mut().enumerate().skip(usable_rows) {
                    *cell = CellValue::Poison(i);
                }
                column
            };
            cs.num_advice_columns
        ];
        let permutation = permutation::keygen::Assembly::new(n, &cs.permutation);
        let constants = cs.constants.clone();

        let mut prover = MockProver {
            k,
            n: n as u32,
            cs,
            regions: vec![],
            current_region: None,
            fixed,
            advice,
            instance,
            selectors,
            permutation,
            usable_rows: 0..usable_rows,
        };

        ConcreteCircuit::FloorPlanner::synthesize(&mut prover, circuit, config, constants)?;

        let (cs, selector_polys) = prover.cs.compress_selectors(prover.selectors.clone());
        prover.cs = cs;
        prover.fixed.extend(selector_polys.into_iter().map(|poly| {
            let mut v = vec![CellValue::Unassigned; n];
            for (v, p) in v.iter_mut().zip(&poly[..]) {
                *v = CellValue::Assigned(*p);
            }
            v
        }));

        Ok(prover)
    }

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    pub fn verify(&self) -> Result<(), Vec<VerifyFailure>> {
        self.verify_at_rows(self.usable_rows.clone(), self.usable_rows.clone())
    }

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    /// Constraints are only checked at `gate_row_ids`,
    /// and lookup inputs are only checked at `lookup_input_row_ids`
    pub fn verify_at_rows<I: Clone + Iterator<Item = usize>>(
        &self,
        gate_row_ids: I,
        lookup_input_row_ids: I,
    ) -> Result<(), Vec<VerifyFailure>> {
        let n = self.n as i32;

        // check all the row ids are valid
        for row_id in gate_row_ids.clone() {
            if !self.usable_rows.contains(&row_id) {
                panic!("invalid gate row id {}", row_id)
            }
        }
        for row_id in lookup_input_row_ids.clone() {
            if !self.usable_rows.contains(&row_id) {
                panic!("invalid lookup row id {}", row_id)
            }
        }

        // Check that within each region, all cells used in instantiated gates have been
        // assigned to.
        let selector_errors = self.regions.iter().enumerate().flat_map(|(r_i, r)| {
            r.enabled_selectors.iter().flat_map(move |(selector, at)| {
                // Find the gates enabled by this selector
                self.cs
                    .gates
                    .iter()
                    // Assume that if a queried selector is enabled, the user wants to use the
                    // corresponding gate in some way.
                    //
                    // TODO: This will trip up on the reverse case, where leaving a selector
                    // un-enabled keeps a gate enabled. We could alternatively require that
                    // every selector is explicitly enabled or disabled on every row? But that
                    // seems messy and confusing.
                    .enumerate()
                    .filter(move |(_, g)| g.queried_selectors().contains(selector))
                    .flat_map(move |(gate_index, gate)| {
                        at.iter().flat_map(move |selector_row| {
                            // Selectors are queried with no rotation.
                            let gate_row = *selector_row as i32;

                            gate.queried_cells().iter().filter_map(move |cell| {
                                // Determine where this cell should have been assigned.
                                let cell_row = ((gate_row + n + cell.rotation.0) % n) as usize;

                                // Check that it was assigned!
                                if r.is_assigned(cell.column, cell_row) {
                                    None
                                } else {
                                    Some(VerifyFailure::CellNotAssigned {
                                        gate: (gate_index, gate.name()).into(),
                                        region: (r_i, r.name.clone()).into(),
                                        column: cell.column,
                                        offset: cell_row as isize - r.rows.unwrap().0 as isize,
                                    })
                                }
                            })
                        })
                    })
            })
        });

        // Check that all gates are satisfied for all rows.
        let gate_errors =
            self.cs
                .gates
                .iter()
                .enumerate()
                .flat_map(|(gate_index, gate)| {
                    let blinding_rows =
                        (self.n as usize - (self.cs.blinding_factors() + 1))..(self.n as usize);
                    (gate_row_ids
                        .clone()
                        .into_iter()
                        .chain(blinding_rows.into_iter()))
                    .flat_map(move |row| {
                        fn load_instance<'a, F: FieldExt, T: ColumnType>(
                            n: i32,
                            row: i32,
                            queries: &'a [(Column<T>, Rotation)],
                            cells: &'a [Vec<F>],
                        ) -> impl Fn(usize, usize, Rotation) -> Value<F> + 'a
                        {
                            move |index, _, _| {
                                let (column, at) = &queries[index];
                                let resolved_row = (row + n + at.0) % n;
                                Value::Real(cells[column.index()][resolved_row as usize])
                            }
                        }

                        fn load<'a, F: FieldExt, T: ColumnType>(
                            n: i32,
                            row: i32,
                            queries: &'a [(Column<T>, Rotation)],
                            cells: &'a [Vec<CellValue<F>>],
                        ) -> impl Fn(usize, usize, Rotation) -> Value<F> + 'a
                        {
                            move |index, _, _| {
                                let (column, at) = &queries[index];
                                let resolved_row = (row + n + at.0) % n;
                                cells[column.index()][resolved_row as usize].into()
                            }
                        }
                        let row = row as i32;
                        gate.polynomials().iter().enumerate().filter_map(
                            move |(poly_index, poly)| match poly.evaluate_lazy(
                                &|scalar| Value::Real(scalar),
                                &|_| panic!("virtual selectors are removed during optimization"),
                                &load(n, row, &self.cs.fixed_queries, &self.fixed),
                                &load(n, row, &self.cs.advice_queries, &self.advice),
                                &load_instance(n, row, &self.cs.instance_queries, &self.instance),
                                &|a| -a,
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                                &Value::Real(F::zero()),
                            ) {
                                Value::Real(x) if x.is_zero_vartime() => None,
                                Value::Real(_) => Some(VerifyFailure::ConstraintNotSatisfied {
                                    constraint: (
                                        (gate_index, gate.name()).into(),
                                        poly_index,
                                        gate.constraint_name(poly_index),
                                    )
                                        .into(),
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        row as usize,
                                        Some(poly).into_iter(),
                                    ),
                                    cell_values: util::cell_values(
                                        gate,
                                        poly,
                                        &load(n, row, &self.cs.fixed_queries, &self.fixed),
                                        &load(n, row, &self.cs.advice_queries, &self.advice),
                                        &load_instance(
                                            n,
                                            row,
                                            &self.cs.instance_queries,
                                            &self.instance,
                                        ),
                                    ),
                                }),
                                Value::Poison => Some(VerifyFailure::ConstraintPoisoned {
                                    constraint: (
                                        (gate_index, gate.name()).into(),
                                        poly_index,
                                        gate.constraint_name(poly_index),
                                    )
                                        .into(),
                                }),
                            },
                        )
                    })
                });

        // Check that all lookups exist in their respective tables.
        let lookup_errors =
            self.cs
                .lookups
                .iter()
                .enumerate()
                .flat_map(|(lookup_index, lookup)| {
                    let load = |expression: &Expression<F>, row| {
                        expression.evaluate_lazy(
                            &|scalar| Value::Real(scalar),
                            &|_| panic!("virtual selectors are removed during optimization"),
                            &|index, _, _| {
                                let query = self.cs.fixed_queries[index];
                                let column_index = query.0.index();
                                let rotation = query.1 .0;
                                self.fixed[column_index]
                                    [(row as i32 + n + rotation) as usize % n as usize]
                                    .into()
                            },
                            &|index, _, _| {
                                let query = self.cs.advice_queries[index];
                                let column_index = query.0.index();
                                let rotation = query.1 .0;
                                self.advice[column_index]
                                    [(row as i32 + n + rotation) as usize % n as usize]
                                    .into()
                            },
                            &|index, _, _| {
                                let query = self.cs.instance_queries[index];
                                let column_index = query.0.index();
                                let rotation = query.1 .0;
                                Value::Real(
                                    self.instance[column_index]
                                        [(row as i32 + n + rotation) as usize % n as usize],
                                )
                            },
                            &|a| -a,
                            &|a, b| a + b,
                            &|a, b| a * b,
                            &|a, scalar| a * scalar,
                            &Value::Real(F::zero()),
                        )
                    };

                    // In the real prover, the lookup expressions are never enforced on
                    // unusable rows, due to the (1 - (l_last(X) + l_blind(X))) term.
                    let table: std::collections::BTreeSet<Vec<_>> = self
                        .usable_rows
                        .clone()
                        .map(|table_row| {
                            lookup
                                .table_expressions
                                .iter()
                                .map(move |c| load(c, table_row))
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    lookup_input_row_ids
                        .clone()
                        .into_iter()
                        .filter_map(move |input_row| {
                            let inputs: Vec<_> = lookup
                                .input_expressions
                                .iter()
                                .map(|c| load(c, input_row))
                                .collect();
                            let lookup_passes = table.contains(&inputs);
                            if lookup_passes {
                                None
                            } else {
                                Some(VerifyFailure::Lookup {
                                    name: lookup.name,
                                    lookup_index,
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        input_row,
                                        lookup.input_expressions.iter(),
                                    ),
                                })
                            }
                        })
                });

        // Check that permutations preserve the original values of the cells.
        let perm_errors = {
            // Original values of columns involved in the permutation.
            let original = |column, row| {
                self.cs
                    .permutation
                    .get_columns()
                    .get(column)
                    .map(|c: &Column<Any>| match c.column_type() {
                        Any::Advice => self.advice[c.index()][row],
                        Any::Fixed => self.fixed[c.index()][row],
                        Any::Instance => CellValue::Assigned(self.instance[c.index()][row]),
                    })
                    .unwrap()
            };

            // Iterate over each column of the permutation
            self.permutation
                .mapping
                .iter()
                .enumerate()
                .flat_map(move |(column, values)| {
                    // Iterate over each row of the column to check that the cell's
                    // value is preserved by the mapping.
                    values.iter().enumerate().filter_map(move |(row, cell)| {
                        let original_cell = original(column, row);
                        let permuted_cell = original(cell.0, cell.1);
                        if original_cell == permuted_cell {
                            None
                        } else {
                            Some(VerifyFailure::Permutation {
                                column: (*self.cs.permutation.get_columns().get(column).unwrap())
                                    .into(),
                                row,
                            })
                        }
                    })
                })
        };

        let mut errors: Vec<_> = iter::empty()
            .chain(selector_errors)
            .chain(gate_errors)
            .chain(lookup_errors)
            .chain(perm_errors)
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            // Remove any duplicate `ConstraintPoisoned` errors (we check all unavailable
            // rows in case the trigger is row-specific, but the error message only points
            // at the constraint).
            errors.dedup_by(|a, b| match (a, b) {
                (
                    a @ VerifyFailure::ConstraintPoisoned { .. },
                    b @ VerifyFailure::ConstraintPoisoned { .. },
                ) => a == b,
                _ => false,
            });
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use pairing::bn256::Fr as Fp;

    use super::{FailureLocation, MockProver, VerifyFailure};
    use crate::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{
            Advice, Any, Circuit, Column, ConstraintSystem, Error, Expression, Selector,
            TableColumn,
        },
        poly::Rotation,
    };

    #[test]
    fn unassigned_cell() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            q: Selector,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let b = meta.advice_column();
                let q = meta.selector();

                meta.create_gate("Equality check", |cells| {
                    let a = cells.query_advice(a, Rotation::prev());
                    let b = cells.query_advice(b, Rotation::cur());
                    let q = cells.query_selector(q);

                    // If q is enabled, a and b must be assigned to.
                    vec![q * (a - b)]
                });

                FaultyCircuitConfig { a, q }
            }

            fn without_witnesses(&self) -> Self {
                Self {}
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_region(
                    || "Faulty synthesis",
                    |mut region| {
                        // Enable the equality gate.
                        config.q.enable(&mut region, 1)?;

                        // Assign a = 0.
                        region.assign_advice(|| "a", config.a, 0, || Ok(Fp::zero()))?;

                        // BUG: Forget to assign b = 0! This could go unnoticed during
                        // development, because cell values default to zero, which in this
                        // case is fine, but for other assignments would be broken.
                        Ok(())
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit {}, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::CellNotAssigned {
                gate: (0, "Equality check").into(),
                region: (0, "Faulty synthesis".to_owned()).into(),
                column: Column::new(1, Any::Advice),
                offset: 1,
            }])
        );
    }

    #[test]
    fn bad_lookup() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            q: Selector,
            table: TableColumn,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let q = meta.complex_selector();
                let table = meta.lookup_table_column();

                meta.lookup("lookup", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    let q = cells.query_selector(q);

                    // If q is enabled, a must be in the table.
                    // When q is not enabled, lookup the default value instead.
                    let not_q = Expression::Constant(Fp::one()) - q.clone();
                    let default = Expression::Constant(Fp::from(2));
                    vec![(q * a + not_q * default, table)]
                });

                FaultyCircuitConfig { a, q, table }
            }

            fn without_witnesses(&self) -> Self {
                Self {}
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_table(
                    || "Doubling table",
                    |mut table| {
                        (1..(1 << (K - 1)))
                            .map(|i| {
                                table.assign_cell(
                                    || format!("table[{}] = {}", i, 2 * i),
                                    config.table,
                                    i - 1,
                                    || Ok(Fp::from(2 * i as u64)),
                                )
                            })
                            .fold(Ok(()), |acc, res| acc.and(res))
                    },
                )?;

                layouter.assign_region(
                    || "Good synthesis",
                    |mut region| {
                        // Enable the lookup on rows 0 and 1.
                        config.q.enable(&mut region, 0)?;
                        config.q.enable(&mut region, 1)?;

                        // Assign a = 2 and a = 6.
                        region.assign_advice(|| "a = 2", config.a, 0, || Ok(Fp::from(2)))?;
                        region.assign_advice(|| "a = 6", config.a, 1, || Ok(Fp::from(6)))?;

                        Ok(())
                    },
                )?;

                layouter.assign_region(
                    || "Faulty synthesis",
                    |mut region| {
                        // Enable the lookup on rows 0 and 1.
                        config.q.enable(&mut region, 0)?;
                        config.q.enable(&mut region, 1)?;

                        // Assign a = 4.
                        region.assign_advice(|| "a = 4", config.a, 0, || Ok(Fp::from(4)))?;

                        // BUG: Assign a = 5, which doesn't exist in the table!
                        region.assign_advice(|| "a = 5", config.a, 1, || Ok(Fp::from(5)))?;

                        Ok(())
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit {}, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::Lookup {
                name: "lookup",
                lookup_index: 0,
                location: FailureLocation::InRegion {
                    region: (2, "Faulty synthesis").into(),
                    offset: 1,
                }
            }])
        );
    }
}
