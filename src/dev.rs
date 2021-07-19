//! Tools for developing circuits.

use std::collections::HashMap;
use std::fmt;
use std::iter;
use std::ops::{Add, Mul, Range};

use ff::Field;

use crate::plonk::Assigned;
use crate::{
    arithmetic::{FieldExt, Group},
    plonk::{
        permutation, Advice, Any, Assignment, Circuit, Column, ColumnType, ConstraintSystem, Error,
        Expression, Fixed, FloorPlanner, Instance, Selector,
    },
    poly::Rotation,
};

pub mod metadata;

#[cfg(feature = "dev-graph")]
mod graph;

#[cfg(feature = "dev-graph")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-graph")))]
pub use graph::{circuit_dot_graph, layout::CircuitLayout};

/// The reasons why a particular circuit is not satisfied.
#[derive(Debug, PartialEq)]
pub enum VerifyFailure {
    /// A cell used in an active gate was not assigned to.
    Cell {
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
        /// The row on which this constraint is not satisfied.
        row: usize,
    },
    /// A constraint was active on an unusable row, and is likely missing a selector.
    ConstraintPoisoned {
        /// The polynomial constraint that is not satisfied.
        constraint: metadata::Constraint,
    },
    /// A lookup input did not exist in its corresponding table.
    Lookup {
        /// The index of the lookup that is not satisfied. These indices are assigned in
        /// the order in which `ConstraintSystem::lookup` is called during
        /// `Circuit::configure`.
        lookup_index: usize,
        /// The row on which this lookup is not satisfied.
        row: usize,
    },
    /// A permutation did not preserve the original value of a cell.
    Permutation {
        /// The column in which this permutation is not satisfied.
        column: Column<Any>,
        /// The row on which this permutation is not satisfied.
        row: usize,
    },
}

impl fmt::Display for VerifyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cell {
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
            Self::ConstraintNotSatisfied { constraint, row } => {
                write!(f, "{} is not satisfied on row {}", constraint, row)
            }
            Self::ConstraintPoisoned { constraint } => {
                write!(
                    f,
                    "{} is active on an unusable row - missing selector?",
                    constraint
                )
            }
            Self::Lookup { lookup_index, row } => {
                write!(f, "Lookup {} is not satisfied on row {}", lookup_index, row)
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
    /// The row that this region starts on, if known.
    start: Option<usize>,
    /// The selectors that have been enabled in this region. All other selectors are by
    /// construction not enabled.
    enabled_selectors: HashMap<Selector, Vec<usize>>,
    /// The cells assigned in this region. We store this as a `Vec` so that if any cells
    /// are double-assigned, they will be visibly darker.
    cells: Vec<(Column<Any>, usize)>,
}

impl Region {
    fn update_start(&mut self, row: usize) {
        // The region start is the earliest row assigned to.
        let mut start = self.start.unwrap_or(row);
        if row < start {
            // The first row assigned was not at start 0 within the region.
            start = row;
        }
        self.start = Some(start);
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            (Value::Real(x), Value::Poison) | (Value::Poison, Value::Real(x)) if x.is_zero() => {
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
            Value::Poison if rhs.is_zero() => Value::Real(F::zero()),
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
/// use halo2::{
///     arithmetic::FieldExt,
///     circuit::{Layouter, SimpleFloorPlanner},
///     dev::{MockProver, VerifyFailure},
///     pasta::Fp,
///     plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector},
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
///                 self.a.map(|v| F::from_u64(v)).ok_or(Error::SynthesisError)
///             })?;
///             region.assign_advice(|| "b", config.b, 0, || {
///                 self.b.map(|v| F::from_u64(v)).ok_or(Error::SynthesisError)
///             })?;
///             region.assign_advice(|| "c", config.c, 0, || {
///                 self.a
///                     .and_then(|a| self.b.map(|b| F::from_u64(a * b)))
///                     .ok_or(Error::SynthesisError)
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
///         row: 0
///     }])
/// );
/// ```
#[derive(Debug)]
pub struct MockProver<F: Group + Field> {
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
            start: None,
            enabled_selectors: HashMap::default(),
            cells: vec![],
        });
    }

    fn exit_region(&mut self) {
        self.regions.push(self.current_region.take().unwrap());
    }

    fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.usable_rows.contains(&row) {
            return Err(Error::BoundsFailure);
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

        // Selectors are just fixed columns.
        self.assign_fixed(annotation, selector.0, row, || Ok(F::one()))
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Option<F>, Error> {
        if !self.usable_rows.contains(&row) {
            return Err(Error::BoundsFailure);
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
            return Err(Error::BoundsFailure);
        }

        if let Some(region) = self.current_region.as_mut() {
            region.update_start(row);
            region.cells.push((column.into(), row));
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
            return Err(Error::BoundsFailure);
        }

        if let Some(region) = self.current_region.as_mut() {
            region.update_start(row);
            region.cells.push((column.into(), row));
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
            return Err(Error::BoundsFailure);
        }

        self.permutation
            .copy(left_column, left_row, right_column, right_row)
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
            return Err(Error::NotEnoughRowsAvailable);
        }

        if instance.len() != cs.num_instance_columns {
            return Err(Error::IncompatibleParams);
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

        let mut prover = MockProver {
            n: n as u32,
            cs,
            regions: vec![],
            current_region: None,
            fixed,
            advice,
            instance,
            permutation,
            usable_rows: 0..usable_rows,
        };

        ConcreteCircuit::FloorPlanner::synthesize(&mut prover, circuit, config)?;

        Ok(prover)
    }

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    pub fn verify(&self) -> Result<(), Vec<VerifyFailure>> {
        let n = self.n as i32;

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
                                if r.cells.contains(&(cell.column, cell_row)) {
                                    None
                                } else {
                                    Some(VerifyFailure::Cell {
                                        gate: (gate_index, gate.name()).into(),
                                        region: (r_i, r.name.clone()).into(),
                                        column: cell.column,
                                        offset: cell_row as isize - r.start.unwrap() as isize,
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
                    // We iterate from n..2n so we can just reduce to handle wrapping.
                    (n..(2 * n)).flat_map(move |row| {
                        fn load_instance<'a, F: FieldExt, T: ColumnType>(
                            n: i32,
                            row: i32,
                            queries: &'a [(Column<T>, Rotation)],
                            cells: &'a [Vec<F>],
                        ) -> impl Fn(usize, usize, Rotation) -> Value<F> + 'a
                        {
                            move |index, _, _| {
                                let (column, at) = &queries[index];
                                let resolved_row = (row + at.0) % n;
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
                                let resolved_row = (row + at.0) % n;
                                cells[column.index()][resolved_row as usize].into()
                            }
                        }

                        gate.polynomials().iter().enumerate().filter_map(
                            move |(poly_index, poly)| match poly.evaluate(
                                &|scalar| Value::Real(scalar),
                                &load(n, row, &self.cs.fixed_queries, &self.fixed),
                                &load(n, row, &self.cs.advice_queries, &self.advice),
                                &load_instance(n, row, &self.cs.instance_queries, &self.instance),
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                            ) {
                                Value::Real(x) if x.is_zero() => None,
                                Value::Real(_) => Some(VerifyFailure::ConstraintNotSatisfied {
                                    constraint: (
                                        (gate_index, gate.name()).into(),
                                        poly_index,
                                        gate.constraint_name(poly_index),
                                    )
                                        .into(),
                                    row: (row - n) as usize,
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
                        expression.evaluate(
                            &|scalar| Value::Real(scalar),
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
                            &|a, b| a + b,
                            &|a, b| a * b,
                            &|a, scalar| a * scalar,
                        )
                    };

                    // In the real prover, the lookup expressions are never enforced on
                    // unusable rows, due to the (1 - (l_last(X) + l_blind(X))) term.
                    let table: Vec<_> = self
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
                    self.usable_rows.clone().filter_map(move |input_row| {
                        let inputs: Vec<_> = lookup
                            .input_expressions
                            .iter()
                            .map(|c| load(c, input_row))
                            .collect();
                        let lookup_passes = table
                            .iter()
                            .any(|table_row| table_row.iter().cloned().eq(inputs.iter().cloned()));
                        if lookup_passes {
                            None
                        } else {
                            Some(VerifyFailure::Lookup {
                                lookup_index,
                                row: input_row as usize,
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
                                column: *self.cs.permutation.get_columns().get(column).unwrap(),
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
    use pasta_curves::Fp;

    use super::{MockProver, VerifyFailure};
    use crate::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Advice, Any, Circuit, Column, ConstraintSystem, Error, Selector},
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
            Err(vec![VerifyFailure::Cell {
                gate: (0, "Equality check").into(),
                region: (0, "Faulty synthesis".to_owned()).into(),
                column: Column::new(1, Any::Advice),
                offset: 1,
            }])
        );
    }
}
