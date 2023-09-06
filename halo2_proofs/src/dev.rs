//! Tools for developing circuits.

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use std::ops::{Add, Mul, Neg, Range};

use blake2b_simd::blake2b;
use ff::Field;
use ff::FromUniformBytes;

use crate::plonk::permutation::keygen::Assembly;
use crate::{
    circuit,
    plonk::{
        permutation,
        sealed::{self, SealedPhase},
        Advice, Any, Assigned, Assignment, Challenge, Circuit, Column, ConstraintSystem, Error,
        Expression, FirstPhase, Fixed, FloorPlanner, Instance, Phase, Selector,
    },
};

#[cfg(feature = "multicore")]
use crate::multicore::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
    ParallelSliceMut,
};

pub mod metadata;
use metadata::Column as ColumnMetadata;
mod util;

mod failure;
pub use failure::{FailureLocation, VerifyFailure};

pub mod cost;
pub use cost::CircuitCost;

mod gates;
pub use gates::CircuitGates;

mod tfp;
pub use tfp::TracingFloorPlanner;

#[cfg(feature = "dev-graph")]
mod graph;

#[cfg(feature = "dev-graph")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-graph")))]
pub use graph::{circuit_dot_graph, layout::CircuitLayout};

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
    /// Annotations given to Advice, Fixed or Instance columns within a region context.
    annotations: HashMap<ColumnMetadata, String>,
    /// The cells assigned in this region. We store this as a `Vec` so that if any cells
    /// are double-assigned, they will be visibly darker.
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
}

/// The value of a particular cell within the circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellValue<F: Field> {
    /// An unassigned cell.
    Unassigned,
    /// A cell that has been assigned a value.
    Assigned(F),
    /// A unique poisoned cell.
    Poison(usize),
}

/// A value within an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
enum Value<F: Field> {
    Real(F),
    Poison,
}

impl<F: Field> From<CellValue<F>> for Value<F> {
    fn from(value: CellValue<F>) -> Self {
        match value {
            // Cells that haven't been explicitly assigned to, default to zero.
            CellValue::Unassigned => Value::Real(F::ZERO),
            CellValue::Assigned(v) => Value::Real(v),
            CellValue::Poison(_) => Value::Poison,
        }
    }
}

impl<F: Field> Neg for Value<F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Real(a) => Value::Real(-a),
            _ => Value::Poison,
        }
    }
}

impl<F: Field> Add for Value<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Real(a), Value::Real(b)) => Value::Real(a + b),
            _ => Value::Poison,
        }
    }
}

impl<F: Field> Mul for Value<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Real(a), Value::Real(b)) => Value::Real(a * b),
            // If poison is multiplied by zero, then we treat the poison as unconstrained
            // and we don't propagate it.
            (Value::Real(x), Value::Poison) | (Value::Poison, Value::Real(x))
                if x.is_zero_vartime() =>
            {
                Value::Real(F::ZERO)
            }
            _ => Value::Poison,
        }
    }
}

impl<F: Field> Mul<F> for Value<F> {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        match self {
            Value::Real(lhs) => Value::Real(lhs * rhs),
            // If poison is multiplied by zero, then we treat the poison as unconstrained
            // and we don't propagate it.
            Value::Poison if rhs.is_zero_vartime() => Value::Real(F::ZERO),
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
///     circuit::{Layouter, SimpleFloorPlanner, Value},
///     dev::{FailureLocation, MockProver, VerifyFailure},
///     plonk::{Advice, Any, Circuit, Column, ConstraintSystem, Error, Selector},
///     poly::Rotation,
/// };
/// use ff::PrimeField;
/// use halo2curves::pasta::Fp;
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
///     a: Value<u64>,
///     b: Value<u64>,
/// }
///
/// impl<F: PrimeField> Circuit<F> for MyCircuit {
///     type Config = MyConfig;
///     type FloorPlanner = SimpleFloorPlanner;
///     #[cfg(feature = "circuit-params")]
///     type Params = ();
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
///                 self.a.map(F::from)
///             })?;
///             region.assign_advice(|| "b", config.b, 0, || {
///                 self.b.map(F::from)
///             })?;
///             region.assign_advice(|| "c", config.c, 0, || {
///                 (self.a * self.b).map(F::from)
///             })?;
///             Ok(())
///         })
///     }
/// }
///
/// // Assemble the private inputs to the circuit.
/// let circuit = MyCircuit {
///     a: Value::known(2),
///     b: Value::known(4),
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
///             (((Any::advice(), 0).into(), 0).into(), "0x2".to_string()),
///             (((Any::advice(), 1).into(), 0).into(), "0x4".to_string()),
///             (((Any::advice(), 2).into(), 0).into(), "0x8".to_string()),
///         ],
///     }])
/// );
///
/// // If we provide a too-small K, we get a panic.
/// use std::panic;
/// let result = panic::catch_unwind(|| {
///     MockProver::<Fp>::run(2, &circuit, vec![]).unwrap_err()
/// });
/// assert_eq!(
///     result.unwrap_err().downcast_ref::<String>().unwrap(),
///     "n=4, minimum_rows=8, k=2"
/// );
/// ```
#[derive(Debug)]
pub struct MockProver<F: Field> {
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
    instance: Vec<Vec<InstanceValue<F>>>,

    selectors: Vec<Vec<bool>>,

    challenges: Vec<F>,

    permutation: permutation::keygen::Assembly,

    // A range of available rows for assignment and copies.
    usable_rows: Range<usize>,

    current_phase: sealed::Phase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InstanceValue<F: Field> {
    Assigned(F),
    Padding,
}

impl<F: Field> InstanceValue<F> {
    fn value(&self) -> F {
        match self {
            InstanceValue::Assigned(v) => *v,
            InstanceValue::Padding => F::ZERO,
        }
    }
}

impl<F: Field> MockProver<F> {
    fn in_phase<P: Phase>(&self, phase: P) -> bool {
        self.current_phase == phase.to_sealed()
    }
}

impl<F: Field> Assignment<F> for MockProver<F> {
    fn enter_region<NR, N>(&mut self, name: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        if !self.in_phase(FirstPhase) {
            return;
        }

        assert!(self.current_region.is_none());
        self.current_region = Some(Region {
            name: name().into(),
            columns: HashSet::default(),
            rows: None,
            annotations: HashMap::default(),
            enabled_selectors: HashMap::default(),
            cells: HashMap::default(),
        });
    }

    fn exit_region(&mut self) {
        if !self.in_phase(FirstPhase) {
            return;
        }

        self.regions.push(self.current_region.take().unwrap());
    }

    fn annotate_column<A, AR>(&mut self, annotation: A, column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.in_phase(FirstPhase) {
            return;
        }

        if let Some(region) = self.current_region.as_mut() {
            region
                .annotations
                .insert(ColumnMetadata::from(column), annotation().into());
        }
    }

    fn enable_selector<A, AR>(&mut self, _: A, selector: &Selector, row: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        assert!(
            self.usable_rows.contains(&row),
            "row={} not in usable_rows={:?}, k={}",
            row,
            self.usable_rows,
            self.k,
        );

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

    fn query_instance(
        &self,
        column: Column<Instance>,
        row: usize,
    ) -> Result<circuit::Value<F>, Error> {
        assert!(
            self.usable_rows.contains(&row),
            "row={}, usable_rows={:?}, k={}",
            row,
            self.usable_rows,
            self.k,
        );

        Ok(self
            .instance
            .get(column.index())
            .and_then(|column| column.get(row))
            .map(|v| circuit::Value::known(v.value()))
            .expect("bound failure"))
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> circuit::Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if self.in_phase(FirstPhase) {
            assert!(
                self.usable_rows.contains(&row),
                "row={}, usable_rows={:?}, k={}",
                row,
                self.usable_rows,
                self.k,
            );

            if let Some(region) = self.current_region.as_mut() {
                region.update_extent(column.into(), row);
                region
                    .cells
                    .entry((column.into(), row))
                    .and_modify(|count| *count += 1)
                    .or_default();
            }
        }

        match to().into_field().evaluate().assign() {
            Ok(to) => {
                let value = self
                    .advice
                    .get_mut(column.index())
                    .and_then(|v| v.get_mut(row))
                    .expect("bounds failure");
                *value = CellValue::Assigned(to);
            }
            Err(err) => {
                // Propagate `assign` error if the column is in current phase.
                if self.in_phase(column.column_type().phase) {
                    return Err(err);
                }
            }
        }

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
        V: FnOnce() -> circuit::Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        assert!(
            self.usable_rows.contains(&row),
            "row={}, usable_rows={:?}, k={}",
            row,
            self.usable_rows,
            self.k,
        );

        if let Some(region) = self.current_region.as_mut() {
            region.update_extent(column.into(), row);
            region
                .cells
                .entry((column.into(), row))
                .and_modify(|count| *count += 1)
                .or_default();
        }

        *self
            .fixed
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .expect("bounds failure") = CellValue::Assigned(to().into_field().evaluate().assign()?);

        Ok(())
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), crate::plonk::Error> {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        assert!(
            self.usable_rows.contains(&left_row) && self.usable_rows.contains(&right_row),
            "left_row={}, right_row={}, usable_rows={:?}, k={}",
            left_row,
            right_row,
            self.usable_rows,
            self.k,
        );

        self.permutation
            .copy(left_column, left_row, right_column, right_row)
    }

    fn fill_from_row(
        &mut self,
        col: Column<Fixed>,
        from_row: usize,
        to: circuit::Value<Assigned<F>>,
    ) -> Result<(), Error> {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        assert!(
            self.usable_rows.contains(&from_row),
            "row={}, usable_rows={:?}, k={}",
            from_row,
            self.usable_rows,
            self.k,
        );

        for row in self.usable_rows.clone().skip(from_row) {
            self.assign_fixed(|| "", col, row, || to)?;
        }

        Ok(())
    }

    fn get_challenge(&self, challenge: Challenge) -> circuit::Value<F> {
        if self.current_phase <= challenge.phase {
            return circuit::Value::unknown();
        }

        circuit::Value::known(self.challenges[challenge.index()])
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

impl<F: FromUniformBytes<64> + Ord> MockProver<F> {
    /// Runs a synthetic keygen-and-prove operation on the given circuit, collecting data
    /// about the constraints and their assignments.
    pub fn run<ConcreteCircuit: Circuit<F>>(
        k: u32,
        circuit: &ConcreteCircuit,
        instance: Vec<Vec<F>>,
    ) -> Result<Self, Error> {
        let n = 1 << k;

        let mut cs = ConstraintSystem::default();
        #[cfg(feature = "circuit-params")]
        let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
        #[cfg(not(feature = "circuit-params"))]
        let config = ConcreteCircuit::configure(&mut cs);
        let cs = cs;

        assert!(
            n >= cs.minimum_rows(),
            "n={}, minimum_rows={}, k={}",
            n,
            cs.minimum_rows(),
            k,
        );

        assert_eq!(instance.len(), cs.num_instance_columns);

        let instance = instance
            .into_iter()
            .map(|instance| {
                assert!(
                    instance.len() <= n - (cs.blinding_factors() + 1),
                    "instance.len={}, n={}, cs.blinding_factors={}",
                    instance.len(),
                    n,
                    cs.blinding_factors()
                );

                let mut instance_values = vec![InstanceValue::Padding; n];
                for (idx, value) in instance.into_iter().enumerate() {
                    instance_values[idx] = InstanceValue::Assigned(value);
                }

                instance_values
            })
            .collect::<Vec<_>>();

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

        // Use hash chain to derive deterministic challenges for testing
        let challenges = {
            let mut hash: [u8; 64] = blake2b(b"Halo2-MockProver").as_bytes().try_into().unwrap();
            iter::repeat_with(|| {
                hash = blake2b(&hash).as_bytes().try_into().unwrap();
                F::from_uniform_bytes(&hash)
            })
            .take(cs.num_challenges)
            .collect()
        };

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
            challenges,
            permutation,
            usable_rows: 0..usable_rows,
            current_phase: FirstPhase.to_sealed(),
        };

        for current_phase in prover.cs.phases() {
            prover.current_phase = current_phase;
            ConcreteCircuit::FloorPlanner::synthesize(
                &mut prover,
                circuit,
                config.clone(),
                constants.clone(),
            )?;
        }

        let (cs, selector_polys) = prover.cs.compress_selectors(prover.selectors.clone());
        prover.cs = cs;
        prover.fixed.extend(selector_polys.into_iter().map(|poly| {
            let mut v = vec![CellValue::Unassigned; n];
            for (v, p) in v.iter_mut().zip(&poly[..]) {
                *v = CellValue::Assigned(*p);
            }
            v
        }));

        #[cfg(feature = "thread-safe-region")]
        prover.permutation.build_ordered_mapping();

        Ok(prover)
    }

    /// Return the content of an advice column as assigned by the circuit.
    pub fn advice_values(&self, column: Column<Advice>) -> &[CellValue<F>] {
        &self.advice[column.index()]
    }

    /// Return the content of a fixed column as assigned by the circuit.
    pub fn fixed_values(&self, column: Column<Fixed>) -> &[CellValue<F>] {
        &self.fixed[column.index()]
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

                                match cell.column.column_type() {
                                    Any::Instance => {
                                        // Handle instance cells, which are not in the region.
                                        let instance_value =
                                            &self.instance[cell.column.index()][cell_row];
                                        match instance_value {
                                            InstanceValue::Assigned(_) => None,
                                            _ => Some(VerifyFailure::InstanceCellNotAssigned {
                                                gate: (gate_index, gate.name()).into(),
                                                region: (r_i, r.name.clone()).into(),
                                                gate_offset: *selector_row,
                                                column: cell.column.try_into().unwrap(),
                                                row: cell_row,
                                            }),
                                        }
                                    }
                                    _ => {
                                        // Check that it was assigned!
                                        if r.cells.contains_key(&(cell.column, cell_row)) {
                                            None
                                        } else {
                                            Some(VerifyFailure::CellNotAssigned {
                                                gate: (gate_index, gate.name()).into(),
                                                region: (
                                                    r_i,
                                                    r.name.clone(),
                                                    r.annotations.clone(),
                                                )
                                                    .into(),
                                                gate_offset: *selector_row,
                                                column: cell.column,
                                                offset: cell_row as isize
                                                    - r.rows.unwrap().0 as isize,
                                            })
                                        }
                                    }
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
                    (gate_row_ids.clone().chain(blinding_rows.into_iter())).flat_map(move |row| {
                        let row = row as i32 + n;
                        gate.polynomials().iter().enumerate().filter_map(
                            move |(poly_index, poly)| match poly.evaluate_lazy(
                                &|scalar| Value::Real(scalar),
                                &|_| panic!("virtual selectors are removed during optimization"),
                                &util::load(n, row, &self.cs.fixed_queries, &self.fixed),
                                &util::load(n, row, &self.cs.advice_queries, &self.advice),
                                &util::load_instance(
                                    n,
                                    row,
                                    &self.cs.instance_queries,
                                    &self.instance,
                                ),
                                &|challenge| Value::Real(self.challenges[challenge.index()]),
                                &|a| -a,
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                                &Value::Real(F::ZERO),
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
                                        (row - n) as usize,
                                        Some(poly).into_iter(),
                                    ),
                                    cell_values: util::cell_values(
                                        gate,
                                        poly,
                                        &util::load(n, row, &self.cs.fixed_queries, &self.fixed),
                                        &util::load(n, row, &self.cs.advice_queries, &self.advice),
                                        &util::load_instance(
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

        let load = |expression: &Expression<F>, row| {
            expression.evaluate_lazy(
                &|scalar| Value::Real(scalar),
                &|_| panic!("virtual selectors are removed during optimization"),
                &|query| {
                    let query = self.cs.fixed_queries[query.index.unwrap()];
                    let column_index = query.0.index();
                    let rotation = query.1 .0;
                    self.fixed[column_index][(row as i32 + n + rotation) as usize % n as usize]
                        .into()
                },
                &|query| {
                    let query = self.cs.advice_queries[query.index.unwrap()];
                    let column_index = query.0.index();
                    let rotation = query.1 .0;
                    self.advice[column_index][(row as i32 + n + rotation) as usize % n as usize]
                        .into()
                },
                &|query| {
                    let query = self.cs.instance_queries[query.index.unwrap()];
                    let column_index = query.0.index();
                    let rotation = query.1 .0;
                    Value::Real(
                        self.instance[column_index]
                            [(row as i32 + n + rotation) as usize % n as usize]
                            .value(),
                    )
                },
                &|challenge| Value::Real(self.challenges[challenge.index()]),
                &|a| -a,
                &|a, b| a + b,
                &|a, b| a * b,
                &|a, scalar| a * scalar,
                &Value::Real(F::ZERO),
            )
        };

        let mut cached_table = Vec::new();
        let mut cached_table_identifier = Vec::new();
        // Check that all lookups exist in their respective tables.
        let lookup_errors =
            self.cs
                .lookups
                .iter()
                .enumerate()
                .flat_map(|(lookup_index, lookup)| {
                    assert!(lookup.table_expressions.len() == lookup.input_expressions.len());
                    assert!(self.usable_rows.end > 0);

                    // We optimize on the basis that the table might have been filled so that the last
                    // usable row now has the fill contents (it doesn't matter if there was no filling).
                    // Note that this "fill row" necessarily exists in the table, and we use that fact to
                    // slightly simplify the optimization: we're only trying to check that all input rows
                    // are contained in the table, and so we can safely just drop input rows that
                    // match the fill row.
                    let fill_row: Vec<_> = lookup
                        .table_expressions
                        .iter()
                        .map(move |c| load(c, self.usable_rows.end - 1))
                        .collect();

                    let table_identifier = lookup
                        .table_expressions
                        .iter()
                        .map(Expression::identifier)
                        .collect::<Vec<_>>();
                    if table_identifier != cached_table_identifier {
                        cached_table_identifier = table_identifier;

                        // In the real prover, the lookup expressions are never enforced on
                        // unusable rows, due to the (1 - (l_last(X) + l_blind(X))) term.
                        cached_table = self
                            .usable_rows
                            .clone()
                            .filter_map(|table_row| {
                                let t = lookup
                                    .table_expressions
                                    .iter()
                                    .map(move |c| load(c, table_row))
                                    .collect();

                                if t != fill_row {
                                    Some(t)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        cached_table.sort_unstable();
                    }
                    let table = &cached_table;

                    let mut inputs: Vec<(Vec<_>, usize)> = lookup_input_row_ids
                        .clone()
                        .filter_map(|input_row| {
                            let t = lookup
                                .input_expressions
                                .iter()
                                .map(move |c| load(c, input_row))
                                .collect();

                            if t != fill_row {
                                // Also keep track of the original input row, since we're going to sort.
                                Some((t, input_row))
                            } else {
                                None
                            }
                        })
                        .collect();
                    inputs.sort_unstable();

                    let mut i = 0;
                    inputs
                        .iter()
                        .filter_map(move |(input, input_row)| {
                            while i < table.len() && &table[i] < input {
                                i += 1;
                            }
                            if i == table.len() || &table[i] > input {
                                assert!(table.binary_search(input).is_err());

                                Some(VerifyFailure::Lookup {
                                    name: lookup.name.clone(),
                                    lookup_index,
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        *input_row,
                                        lookup.input_expressions.iter(),
                                    ),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                });

        let shuffle_errors =
            self.cs
                .shuffles
                .iter()
                .enumerate()
                .flat_map(|(shuffle_index, shuffle)| {
                    assert!(shuffle.shuffle_expressions.len() == shuffle.input_expressions.len());
                    assert!(self.usable_rows.end > 0);

                    let mut shuffle_rows: Vec<Vec<Value<F>>> = self
                        .usable_rows
                        .clone()
                        .map(|row| {
                            let t = shuffle
                                .shuffle_expressions
                                .iter()
                                .map(move |c| load(c, row))
                                .collect();
                            t
                        })
                        .collect();
                    shuffle_rows.sort();

                    let mut input_rows: Vec<(Vec<Value<F>>, usize)> = self
                        .usable_rows
                        .clone()
                        .map(|input_row| {
                            let t = shuffle
                                .input_expressions
                                .iter()
                                .map(move |c| load(c, input_row))
                                .collect();

                            (t, input_row)
                        })
                        .collect();
                    input_rows.sort();

                    input_rows
                        .iter()
                        .zip(shuffle_rows.iter())
                        .filter_map(|((input_value, row), shuffle_value)| {
                            if shuffle_value != input_value {
                                Some(VerifyFailure::Shuffle {
                                    name: shuffle.name.clone(),
                                    shuffle_index,
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        *row,
                                        shuffle.input_expressions.iter(),
                                    ),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                });

        let mapping = self.permutation.mapping();
        // Check that permutations preserve the original values of the cells.
        let perm_errors = {
            // Original values of columns involved in the permutation.
            let original = |column, row| {
                self.cs
                    .permutation
                    .get_columns()
                    .get(column)
                    .map(|c: &Column<Any>| match c.column_type() {
                        Any::Advice(_) => self.advice[c.index()][row],
                        Any::Fixed => self.fixed[c.index()][row],
                        Any::Instance => {
                            let cell: &InstanceValue<F> = &self.instance[c.index()][row];
                            CellValue::Assigned(cell.value())
                        }
                    })
                    .unwrap()
            };

            // Iterate over each column of the permutation
            mapping.enumerate().flat_map(move |(column, values)| {
                // Iterate over each row of the column to check that the cell's
                // value is preserved by the mapping.
                values
                    .enumerate()
                    .filter_map(move |(row, cell)| {
                        let original_cell = original(column, row);
                        let permuted_cell = original(cell.0, cell.1);
                        if original_cell == permuted_cell {
                            None
                        } else {
                            let columns = self.cs.permutation.get_columns();
                            let column = columns.get(column).unwrap();
                            Some(VerifyFailure::Permutation {
                                column: (*column).into(),
                                location: FailureLocation::find(
                                    &self.regions,
                                    row,
                                    Some(column).into_iter().cloned().collect(),
                                ),
                            })
                        }
                    })
                    .collect::<Vec<_>>()
            })
        };

        let mut errors: Vec<_> = iter::empty()
            .chain(selector_errors)
            .chain(gate_errors)
            .chain(lookup_errors)
            .chain(perm_errors)
            .chain(shuffle_errors)
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

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    /// Constraints and lookup are checked at `usable_rows`, parallelly.
    #[cfg(feature = "multicore")]
    pub fn verify_par(&self) -> Result<(), Vec<VerifyFailure>> {
        self.verify_at_rows_par(self.usable_rows.clone(), self.usable_rows.clone())
    }

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    /// Constraints are only checked at `gate_row_ids`, and lookup inputs are only checked at `lookup_input_row_ids`, parallelly.
    #[cfg(feature = "multicore")]
    pub fn verify_at_rows_par<I: Clone + Iterator<Item = usize>>(
        &self,
        gate_row_ids: I,
        lookup_input_row_ids: I,
    ) -> Result<(), Vec<VerifyFailure>> {
        let n = self.n as i32;

        let gate_row_ids = gate_row_ids.collect::<Vec<_>>();
        let lookup_input_row_ids = lookup_input_row_ids.collect::<Vec<_>>();

        // check all the row ids are valid
        gate_row_ids.par_iter().for_each(|row_id| {
            if !self.usable_rows.contains(row_id) {
                panic!("invalid gate row id {}", row_id);
            }
        });
        lookup_input_row_ids.par_iter().for_each(|row_id| {
            if !self.usable_rows.contains(row_id) {
                panic!("invalid gate row id {}", row_id);
            }
        });

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
                        at.par_iter()
                            .flat_map(move |selector_row| {
                                // Selectors are queried with no rotation.
                                let gate_row = *selector_row as i32;

                                gate.queried_cells()
                                    .iter()
                                    .filter_map(move |cell| {
                                        // Determine where this cell should have been assigned.
                                        let cell_row =
                                            ((gate_row + n + cell.rotation.0) % n) as usize;

                                        match cell.column.column_type() {
                                            Any::Instance => {
                                                // Handle instance cells, which are not in the region.
                                                let instance_value =
                                                    &self.instance[cell.column.index()][cell_row];
                                                match instance_value {
                                                    InstanceValue::Assigned(_) => None,
                                                    _ => Some(
                                                        VerifyFailure::InstanceCellNotAssigned {
                                                            gate: (gate_index, gate.name()).into(),
                                                            region: (r_i, r.name.clone()).into(),
                                                            gate_offset: *selector_row,
                                                            column: cell.column.try_into().unwrap(),
                                                            row: cell_row,
                                                        },
                                                    ),
                                                }
                                            }
                                            _ => {
                                                // Check that it was assigned!
                                                if r.cells.contains_key(&(cell.column, cell_row)) {
                                                    None
                                                } else {
                                                    Some(VerifyFailure::CellNotAssigned {
                                                        gate: (gate_index, gate.name()).into(),
                                                        region: (
                                                            r_i,
                                                            r.name.clone(),
                                                            r.annotations.clone(),
                                                        )
                                                            .into(),
                                                        gate_offset: *selector_row,
                                                        column: cell.column,
                                                        offset: cell_row as isize
                                                            - r.rows.unwrap().0 as isize,
                                                    })
                                                }
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>()
                    })
            })
        });

        // Check that all gates are satisfied for all rows.
        let gate_errors = self
            .cs
            .gates
            .iter()
            .enumerate()
            .flat_map(|(gate_index, gate)| {
                let blinding_rows =
                    (self.n as usize - (self.cs.blinding_factors() + 1))..(self.n as usize);
                (gate_row_ids
                    .clone()
                    .into_par_iter()
                    .chain(blinding_rows.into_par_iter()))
                .flat_map(move |row| {
                    let row = row as i32 + n;
                    gate.polynomials()
                        .iter()
                        .enumerate()
                        .filter_map(move |(poly_index, poly)| {
                            match poly.evaluate_lazy(
                                &|scalar| Value::Real(scalar),
                                &|_| panic!("virtual selectors are removed during optimization"),
                                &util::load(n, row, &self.cs.fixed_queries, &self.fixed),
                                &util::load(n, row, &self.cs.advice_queries, &self.advice),
                                &util::load_instance(
                                    n,
                                    row,
                                    &self.cs.instance_queries,
                                    &self.instance,
                                ),
                                &|challenge| Value::Real(self.challenges[challenge.index()]),
                                &|a| -a,
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                                &Value::Real(F::ZERO),
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
                                        (row - n) as usize,
                                        Some(poly).into_iter(),
                                    ),
                                    cell_values: util::cell_values(
                                        gate,
                                        poly,
                                        &util::load(n, row, &self.cs.fixed_queries, &self.fixed),
                                        &util::load(n, row, &self.cs.advice_queries, &self.advice),
                                        &util::load_instance(
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
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
            });

        let load = |expression: &Expression<F>, row| {
            expression.evaluate_lazy(
                &|scalar| Value::Real(scalar),
                &|_| panic!("virtual selectors are removed during optimization"),
                &|query| {
                    self.fixed[query.column_index]
                        [(row as i32 + n + query.rotation.0) as usize % n as usize]
                        .into()
                },
                &|query| {
                    self.advice[query.column_index]
                        [(row as i32 + n + query.rotation.0) as usize % n as usize]
                        .into()
                },
                &|query| {
                    Value::Real(
                        self.instance[query.column_index]
                            [(row as i32 + n + query.rotation.0) as usize % n as usize]
                            .value(),
                    )
                },
                &|challenge| Value::Real(self.challenges[challenge.index()]),
                &|a| -a,
                &|a, b| a + b,
                &|a, b| a * b,
                &|a, scalar| a * scalar,
                &Value::Real(F::ZERO),
            )
        };

        let mut cached_table = Vec::new();
        let mut cached_table_identifier = Vec::new();
        // Check that all lookups exist in their respective tables.
        let lookup_errors =
            self.cs
                .lookups
                .iter()
                .enumerate()
                .flat_map(|(lookup_index, lookup)| {
                    assert!(lookup.table_expressions.len() == lookup.input_expressions.len());
                    assert!(self.usable_rows.end > 0);

                    // We optimize on the basis that the table might have been filled so that the last
                    // usable row now has the fill contents (it doesn't matter if there was no filling).
                    // Note that this "fill row" necessarily exists in the table, and we use that fact to
                    // slightly simplify the optimization: we're only trying to check that all input rows
                    // are contained in the table, and so we can safely just drop input rows that
                    // match the fill row.
                    let fill_row: Vec<_> = lookup
                        .table_expressions
                        .iter()
                        .map(move |c| load(c, self.usable_rows.end - 1))
                        .collect();

                    let table_identifier = lookup
                        .table_expressions
                        .iter()
                        .map(Expression::identifier)
                        .collect::<Vec<_>>();
                    if table_identifier != cached_table_identifier {
                        cached_table_identifier = table_identifier;

                        // In the real prover, the lookup expressions are never enforced on
                        // unusable rows, due to the (1 - (l_last(X) + l_blind(X))) term.
                        cached_table = self
                            .usable_rows
                            .clone()
                            .into_par_iter()
                            .filter_map(|table_row| {
                                let t = lookup
                                    .table_expressions
                                    .iter()
                                    .map(move |c| load(c, table_row))
                                    .collect();

                                if t != fill_row {
                                    Some(t)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        cached_table.par_sort_unstable();
                    }
                    let table = &cached_table;

                    let mut inputs: Vec<(Vec<_>, usize)> = lookup_input_row_ids
                        .clone()
                        .into_par_iter()
                        .filter_map(|input_row| {
                            let t = lookup
                                .input_expressions
                                .iter()
                                .map(move |c| load(c, input_row))
                                .collect();

                            if t != fill_row {
                                // Also keep track of the original input row, since we're going to sort.
                                Some((t, input_row))
                            } else {
                                None
                            }
                        })
                        .collect();
                    inputs.par_sort_unstable();

                    inputs
                        .par_iter()
                        .filter_map(move |(input, input_row)| {
                            if table.binary_search(input).is_err() {
                                Some(VerifyFailure::Lookup {
                                    name: lookup.name.clone(),
                                    lookup_index,
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        *input_row,
                                        lookup.input_expressions.iter(),
                                    ),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                });

        let shuffle_errors =
            self.cs
                .shuffles
                .iter()
                .enumerate()
                .flat_map(|(shuffle_index, shuffle)| {
                    assert!(shuffle.shuffle_expressions.len() == shuffle.input_expressions.len());
                    assert!(self.usable_rows.end > 0);

                    let mut shuffle_rows: Vec<Vec<Value<F>>> = self
                        .usable_rows
                        .clone()
                        .map(|row| {
                            let t = shuffle
                                .shuffle_expressions
                                .iter()
                                .map(move |c| load(c, row))
                                .collect();
                            t
                        })
                        .collect();
                    shuffle_rows.sort();

                    let mut input_rows: Vec<(Vec<Value<F>>, usize)> = self
                        .usable_rows
                        .clone()
                        .map(|input_row| {
                            let t = shuffle
                                .input_expressions
                                .iter()
                                .map(move |c| load(c, input_row))
                                .collect();

                            (t, input_row)
                        })
                        .collect();
                    input_rows.sort();

                    input_rows
                        .iter()
                        .zip(shuffle_rows.iter())
                        .filter_map(|((input_value, row), shuffle_value)| {
                            if shuffle_value != input_value {
                                Some(VerifyFailure::Shuffle {
                                    name: shuffle.name.clone(),
                                    shuffle_index,
                                    location: FailureLocation::find_expressions(
                                        &self.cs,
                                        &self.regions,
                                        *row,
                                        shuffle.input_expressions.iter(),
                                    ),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                });

        let mapping = self.permutation.mapping();
        // Check that permutations preserve the original values of the cells.
        let perm_errors = {
            // Original values of columns involved in the permutation.
            let original = |column, row| {
                self.cs
                    .permutation
                    .get_columns()
                    .get(column)
                    .map(|c: &Column<Any>| match c.column_type() {
                        Any::Advice(_) => self.advice[c.index()][row],
                        Any::Fixed => self.fixed[c.index()][row],
                        Any::Instance => {
                            let cell: &InstanceValue<F> = &self.instance[c.index()][row];
                            CellValue::Assigned(cell.value())
                        }
                    })
                    .unwrap()
            };

            // Iterate over each column of the permutation
            mapping.enumerate().flat_map(move |(column, values)| {
                // Iterate over each row of the column to check that the cell's
                // value is preserved by the mapping.
                values
                    .enumerate()
                    .filter_map(move |(row, cell)| {
                        let original_cell = original(column, row);
                        let permuted_cell = original(cell.0, cell.1);
                        if original_cell == permuted_cell {
                            None
                        } else {
                            let columns = self.cs.permutation.get_columns();
                            let column = columns.get(column).unwrap();
                            Some(VerifyFailure::Permutation {
                                column: (*column).into(),
                                location: FailureLocation::find(
                                    &self.regions,
                                    row,
                                    Some(column).into_iter().cloned().collect(),
                                ),
                            })
                        }
                    })
                    .collect::<Vec<_>>()
            })
        };

        let mut errors: Vec<_> = iter::empty()
            .chain(selector_errors)
            .chain(gate_errors)
            .chain(lookup_errors)
            .chain(perm_errors)
            .chain(shuffle_errors)
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

    /// Panics if the circuit being checked by this `MockProver` is not satisfied.
    ///
    /// Any verification failures will be pretty-printed to stderr before the function
    /// panics.
    ///
    /// Apart from the stderr output, this method is equivalent to:
    /// ```ignore
    /// assert_eq!(prover.verify(), Ok(()));
    /// ```
    pub fn assert_satisfied(&self) {
        if let Err(errs) = self.verify() {
            for err in errs {
                err.emit(self);
                eprintln!();
            }
            panic!("circuit was not satisfied");
        }
    }

    /// Panics if the circuit being checked by this `MockProver` is not satisfied.
    ///
    /// Any verification failures will be pretty-printed to stderr before the function
    /// panics.
    ///
    /// Internally, this function uses a parallel aproach in order to verify the `MockProver` contents.
    ///
    /// Apart from the stderr output, this method is equivalent to:
    /// ```ignore
    /// assert_eq!(prover.verify_par(), Ok(()));
    /// ```
    #[cfg(feature = "multicore")]
    pub fn assert_satisfied_par(&self) {
        if let Err(errs) = self.verify_par() {
            for err in errs {
                err.emit(self);
                eprintln!();
            }
            panic!("circuit was not satisfied");
        }
    }

    /// Panics if the circuit being checked by this `MockProver` is not satisfied.
    ///
    /// Any verification failures will be pretty-printed to stderr before the function
    /// panics.
    ///
    /// Constraints are only checked at `gate_row_ids`, and lookup inputs are only checked at `lookup_input_row_ids`, parallelly.
    ///
    /// Apart from the stderr output, this method is equivalent to:
    /// ```ignore
    /// assert_eq!(prover.verify_at_rows_par(), Ok(()));
    /// ```
    #[cfg(feature = "multicore")]
    pub fn assert_satisfied_at_rows_par<I: Clone + Iterator<Item = usize>>(
        &self,
        gate_row_ids: I,
        lookup_input_row_ids: I,
    ) {
        if let Err(errs) = self.verify_at_rows_par(gate_row_ids, lookup_input_row_ids) {
            for err in errs {
                err.emit(self);
                eprintln!();
            }
            panic!("circuit was not satisfied");
        }
    }

    /// Returns the list of Fixed Columns used within a MockProver instance and the associated values contained on each Cell.
    pub fn fixed(&self) -> &Vec<Vec<CellValue<F>>> {
        &self.fixed
    }

    /// Returns the permutation argument (`Assembly`) used within a MockProver instance.
    pub fn permutation(&self) -> &Assembly {
        &self.permutation
    }
}

#[cfg(test)]
mod tests {
    use halo2curves::pasta::Fp;

    use super::{FailureLocation, MockProver, VerifyFailure};
    use crate::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        plonk::{
            sealed::SealedPhase, Advice, Any, Circuit, Column, ConstraintSystem, Error, Expression,
            FirstPhase, Fixed, Instance, Selector, TableColumn,
        },
        poly::Rotation,
    };

    #[test]
    fn unassigned_cell() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            b: Column<Advice>,
            q: Selector,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

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

                FaultyCircuitConfig { a, b, q }
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
                        region.assign_advice(|| "a", config.a, 0, || Value::known(Fp::zero()))?;

                        // Name Column a
                        region.name_column(|| "This is annotated!", config.a);

                        // Name Column b
                        region.name_column(|| "This is also annotated!", config.b);

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
                gate_offset: 1,
                column: Column::new(
                    1,
                    Any::Advice(Advice {
                        phase: FirstPhase.to_sealed()
                    })
                ),
                offset: 1,
            }])
        );
    }

    #[test]
    fn bad_lookup_any() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            table: Column<Instance>,
            advice_table: Column<Advice>,
            q: Selector,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let q = meta.complex_selector();
                let table = meta.instance_column();
                let advice_table = meta.advice_column();

                meta.annotate_lookup_any_column(table, || "Inst-Table");
                meta.enable_equality(table);
                meta.annotate_lookup_any_column(advice_table, || "Adv-Table");
                meta.enable_equality(advice_table);

                meta.lookup_any("lookup", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    let q = cells.query_selector(q);
                    let advice_table = cells.query_advice(advice_table, Rotation::cur());
                    let table = cells.query_instance(table, Rotation::cur());

                    // If q is enabled, a must be in the table.
                    // When q is not enabled, lookup the default value instead.
                    let not_q = Expression::Constant(Fp::one()) - q.clone();
                    let default = Expression::Constant(Fp::from(2));
                    vec![
                        (
                            q.clone() * a.clone() + not_q.clone() * default.clone(),
                            table,
                        ),
                        (q * a + not_q * default, advice_table),
                    ]
                });

                FaultyCircuitConfig {
                    a,
                    q,
                    table,
                    advice_table,
                }
            }

            fn without_witnesses(&self) -> Self {
                Self {}
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                // No assignment needed for the table as is an Instance Column.

                layouter.assign_region(
                    || "Good synthesis",
                    |mut region| {
                        // Enable the lookup on rows 0 and 1.
                        config.q.enable(&mut region, 0)?;
                        config.q.enable(&mut region, 1)?;

                        for i in 0..4 {
                            // Load Advice lookup table with Instance lookup table values.
                            region.assign_advice_from_instance(
                                || "Advice from instance tables",
                                config.table,
                                i,
                                config.advice_table,
                                i,
                            )?;
                        }

                        // Assign a = 2 and a = 6.
                        region.assign_advice(
                            || "a = 2",
                            config.a,
                            0,
                            || Value::known(Fp::from(2)),
                        )?;
                        region.assign_advice(
                            || "a = 6",
                            config.a,
                            1,
                            || Value::known(Fp::from(6)),
                        )?;

                        Ok(())
                    },
                )?;

                layouter.assign_region(
                    || "Faulty synthesis",
                    |mut region| {
                        // Enable the lookup on rows 0 and 1.
                        config.q.enable(&mut region, 0)?;
                        config.q.enable(&mut region, 1)?;

                        for i in 0..4 {
                            // Load Advice lookup table with Instance lookup table values.
                            region.assign_advice_from_instance(
                                || "Advice from instance tables",
                                config.table,
                                i,
                                config.advice_table,
                                i,
                            )?;
                        }

                        // Assign a = 4.
                        region.assign_advice(
                            || "a = 4",
                            config.a,
                            0,
                            || Value::known(Fp::from(4)),
                        )?;

                        // BUG: Assign a = 5, which doesn't exist in the table!
                        region.assign_advice(
                            || "a = 5",
                            config.a,
                            1,
                            || Value::known(Fp::from(5)),
                        )?;

                        region.name_column(|| "Witness example", config.a);

                        Ok(())
                    },
                )
            }
        }

        let prover = MockProver::run(
            K,
            &FaultyCircuit {},
            // This is our "lookup table".
            vec![vec![
                Fp::from(1u64),
                Fp::from(2u64),
                Fp::from(4u64),
                Fp::from(6u64),
            ]],
        )
        .unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::Lookup {
                name: "lookup".to_string(),
                lookup_index: 0,
                location: FailureLocation::InRegion {
                    region: (1, "Faulty synthesis").into(),
                    offset: 1,
                }
            }])
        );
    }

    #[test]
    fn bad_fixed_lookup() {
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
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let q = meta.complex_selector();
                let table = meta.lookup_table_column();
                meta.annotate_lookup_column(table, || "Table1");

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
                                    || Value::known(Fp::from(2 * i as u64)),
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
                        region.assign_advice(
                            || "a = 2",
                            config.a,
                            0,
                            || Value::known(Fp::from(2)),
                        )?;
                        region.assign_advice(
                            || "a = 6",
                            config.a,
                            1,
                            || Value::known(Fp::from(6)),
                        )?;

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
                        region.assign_advice(
                            || "a = 4",
                            config.a,
                            0,
                            || Value::known(Fp::from(4)),
                        )?;

                        // BUG: Assign a = 5, which doesn't exist in the table!
                        region.assign_advice(
                            || "a = 5",
                            config.a,
                            1,
                            || Value::known(Fp::from(5)),
                        )?;

                        region.name_column(|| "Witness example", config.a);

                        Ok(())
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit {}, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::Lookup {
                name: "lookup".to_string(),
                lookup_index: 0,
                location: FailureLocation::InRegion {
                    region: (2, "Faulty synthesis").into(),
                    offset: 1,
                }
            }])
        );
    }

    #[test]
    fn contraint_unsatisfied() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            b: Column<Advice>,
            c: Column<Advice>,
            d: Column<Fixed>,
            q: Selector,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let b = meta.advice_column();
                let c = meta.advice_column();
                let d = meta.fixed_column();
                let q = meta.selector();

                meta.create_gate("Equality check", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    let b = cells.query_advice(b, Rotation::cur());
                    let c = cells.query_advice(c, Rotation::cur());
                    let d = cells.query_fixed(d, Rotation::cur());
                    let q = cells.query_selector(q);

                    // If q is enabled, a and b must be assigned to.
                    vec![q * (a - b) * (c - d)]
                });

                FaultyCircuitConfig { a, b, c, d, q }
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
                    || "Correct synthesis",
                    |mut region| {
                        // Enable the equality gate.
                        config.q.enable(&mut region, 0)?;

                        // Assign a = 1.
                        region.assign_advice(|| "a", config.a, 0, || Value::known(Fp::one()))?;

                        // Assign b = 1.
                        region.assign_advice(|| "b", config.b, 0, || Value::known(Fp::one()))?;

                        // Assign c = 5.
                        region.assign_advice(
                            || "c",
                            config.c,
                            0,
                            || Value::known(Fp::from(5u64)),
                        )?;
                        // Assign d = 7.
                        region.assign_fixed(
                            || "d",
                            config.d,
                            0,
                            || Value::known(Fp::from(7u64)),
                        )?;
                        Ok(())
                    },
                )?;
                layouter.assign_region(
                    || "Wrong synthesis",
                    |mut region| {
                        // Enable the equality gate.
                        config.q.enable(&mut region, 0)?;

                        // Assign a = 1.
                        region.assign_advice(|| "a", config.a, 0, || Value::known(Fp::one()))?;

                        // Assign b = 0.
                        region.assign_advice(|| "b", config.b, 0, || Value::known(Fp::zero()))?;

                        // Name Column a
                        region.name_column(|| "This is Advice!", config.a);
                        // Name Column b
                        region.name_column(|| "This is Advice too!", config.b);

                        // Assign c = 5.
                        region.assign_advice(
                            || "c",
                            config.c,
                            0,
                            || Value::known(Fp::from(5u64)),
                        )?;
                        // Assign d = 7.
                        region.assign_fixed(
                            || "d",
                            config.d,
                            0,
                            || Value::known(Fp::from(7u64)),
                        )?;

                        // Name Column c
                        region.name_column(|| "Another one!", config.c);
                        // Name Column d
                        region.name_column(|| "This is a Fixed!", config.d);

                        // Note that none of the terms cancel eachother. Therefore we will have a constraint that is non satisfied for
                        // the `Equalty check` gate.
                        Ok(())
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit {}, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::ConstraintNotSatisfied {
                constraint: ((0, "Equality check").into(), 0, "").into(),
                location: FailureLocation::InRegion {
                    region: (1, "Wrong synthesis").into(),
                    offset: 0,
                },
                cell_values: vec![
                    (
                        (
                            (
                                Any::Advice(Advice {
                                    phase: FirstPhase.to_sealed()
                                }),
                                0
                            )
                                .into(),
                            0
                        )
                            .into(),
                        "1".to_string()
                    ),
                    (
                        (
                            (
                                Any::Advice(Advice {
                                    phase: FirstPhase.to_sealed()
                                }),
                                1
                            )
                                .into(),
                            0
                        )
                            .into(),
                        "0".to_string()
                    ),
                    (
                        (
                            (
                                Any::Advice(Advice {
                                    phase: FirstPhase.to_sealed()
                                }),
                                2
                            )
                                .into(),
                            0
                        )
                            .into(),
                        "0x5".to_string()
                    ),
                    (((Any::Fixed, 0).into(), 0).into(), "0x7".to_string()),
                ],
            },])
        )
    }
}
