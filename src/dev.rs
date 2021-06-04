//! Tools for developing circuits.

use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::iter;

use ff::Field;

use crate::{
    arithmetic::{FieldExt, Group},
    plonk::{
        permutation, Advice, Any, Assignment, Circuit, Column, ColumnType, ConstraintSystem, Error,
        Expression, Fixed, Permutation, Selector,
    },
    poly::Rotation,
};

#[cfg(feature = "dev-graph")]
mod graph;

#[cfg(feature = "dev-graph")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-graph")))]
pub use graph::{circuit_dot_graph, layout::circuit_layout};

/// Cells that haven't been explicitly assigned to, default to zero.
fn cell_value<F: Field>(cell: Option<F>) -> F {
    cell.unwrap_or_else(F::zero)
}

/// The reasons why a particular circuit is not satisfied.
#[derive(Debug, PartialEq)]
pub enum VerifyFailure {
    /// A cell used in an active gate was not assigned to.
    Cell {
        /// The column in which this cell is located.
        column: Column<Any>,
        /// The row in which this cell is located.
        row: usize,
        /// The index of the active gate. These indices are assigned in the order in which
        /// `ConstraintSystem::create_gate` is called during `Circuit::configure`.
        gate_index: usize,
        /// The name of the active gate. These are specified by the gate creator (such as
        /// a chip implementation), and is not enforced to be unique.
        gate_name: &'static str,
    },
    /// A constraint was not satisfied for a particular row.
    Constraint {
        /// The index of the gate containing the unsatisfied constraint. These indices are
        /// assigned in the order in which `ConstraintSystem::create_gate` is called
        /// during `Circuit::configure`.
        gate_index: usize,
        /// The name of the gate containing the unsatisfied constraint. This is specified
        /// by the gate creator (such as a chip implementation), and is not enforced to be
        /// unique.
        gate_name: &'static str,
        /// The index of the polynomial constraint within the gate that is not satisfied.
        /// These indices correspond to the order in which the constraints are returned
        /// from the closure passed to `ConstraintSystem::create_gate` during
        /// `Circuit::configure`.
        constraint_index: usize,
        /// The row on which this constraint is not satisfied.
        row: usize,
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
        /// The index of the permutation that is not satisfied. These indices are assigned
        /// in the order in which `ConstraintSystem::lookup` is called during
        /// `Circuit::configure`.
        perm_index: usize,
        /// The column in which this permutation is not satisfied.
        column: usize,
        /// The row on which this permutation is not satisfied.
        row: usize,
    },
}

impl fmt::Display for VerifyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cell {
                column,
                row,
                gate_index,
                gate_name,
            } => {
                write!(
                    f,
                    "Cell ({:?}, {}) was not assigned to, but it is used by active gate {} ('{}').",
                    column, row, gate_index, gate_name
                )
            }
            Self::Constraint {
                gate_index,
                gate_name,
                constraint_index,
                row,
            } => {
                write!(
                    f,
                    "Constraint {} in gate {} ('{}') is not satisfied on row {}",
                    constraint_index, gate_index, gate_name, row
                )
            }
            Self::Lookup { lookup_index, row } => {
                write!(f, "Lookup {} is not satisfied on row {}", lookup_index, row)
            }
            Self::Permutation {
                perm_index,
                column,
                row,
            } => {
                write!(
                    f,
                    "Permutation {} is not satisfied by cell ({:?}, {})",
                    perm_index, column, row
                )
            }
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
///     dev::{MockProver, VerifyFailure},
///     pasta::Fp,
///     plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error},
///     poly::Rotation,
/// };
/// const K: u32 = 5;
///
/// #[derive(Copy, Clone)]
/// struct MyConfig {
///     a: Column<Advice>,
///     b: Column<Advice>,
///     c: Column<Advice>,
/// }
///
/// #[derive(Clone)]
/// struct MyCircuit {
///     a: Option<u64>,
///     b: Option<u64>,
/// }
///
/// impl<F: FieldExt> Circuit<F> for MyCircuit {
///     type Config = MyConfig;
///
///     fn configure(meta: &mut ConstraintSystem<F>) -> MyConfig {
///         let a = meta.advice_column();
///         let b = meta.advice_column();
///         let c = meta.advice_column();
///
///         meta.create_gate("R1CS constraint", |meta| {
///             let a = meta.query_advice(a, Rotation::cur());
///             let b = meta.query_advice(b, Rotation::cur());
///             let c = meta.query_advice(c, Rotation::cur());
///
///             // BUG: Should be a * b - c
///             vec![a * b + c]
///         });
///
///         MyConfig { a, b, c }
///     }
///
///     fn synthesize(&self, cs: &mut impl Assignment<F>, config: MyConfig) -> Result<(), Error> {
///         cs.assign_advice(|| "a", config.a, 0, || {
///             self.a.map(|v| F::from_u64(v)).ok_or(Error::SynthesisError)
///         })?;
///         cs.assign_advice(|| "b", config.b, 0, || {
///             self.b.map(|v| F::from_u64(v)).ok_or(Error::SynthesisError)
///         })?;
///         cs.assign_advice(|| "c", config.c, 0, || {
///             self.a
///                 .and_then(|a| self.b.map(|b| F::from_u64(a * b)))
///                 .ok_or(Error::SynthesisError)
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
///     Err(vec![VerifyFailure::Constraint {
///         gate_index: 0,
///         gate_name: "R1CS constraint",
///         constraint_index: 0,
///         row: 0
///     }])
/// );
/// ```
#[derive(Debug)]
pub struct MockProver<F: Group + Field> {
    n: u32,
    cs: ConstraintSystem<F>,

    /// The selectors that have been enabled in the circuit. All other selectors are by
    /// construction not enabled.
    enabled_selectors: HashMap<Selector, Vec<usize>>,

    // The fixed cells in the circuit, arranged as [column][row].
    fixed: Vec<Vec<Option<F>>>,
    // The advice cells in the circuit, arranged as [column][row].
    advice: Vec<Vec<Option<F>>>,
    // The instance cells in the circuit, arranged as [column][row].
    instance: Vec<Vec<F>>,

    permutations: Vec<permutation::keygen::Assembly>,
}

impl<F: Field + Group> Assignment<F> for MockProver<F> {
    fn enter_region<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn exit_region(&mut self) {}

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
        // Track that this selector was enabled.
        self.enabled_selectors
            .entry(*selector)
            .or_default()
            .push(row);

        // Selectors are just fixed columns.
        self.assign_fixed(annotation, selector.0, row, || Ok(F::one()))
    }

    fn assign_advice<V, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        *self
            .advice
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = Some(to()?);

        Ok(())
    }

    fn assign_fixed<V, A, AR>(
        &mut self,
        _: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        *self
            .fixed
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = Some(to()?);

        Ok(())
    }

    fn copy(
        &mut self,
        permutation: &Permutation,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), crate::plonk::Error> {
        // Check bounds first
        if permutation.index() >= self.permutations.len() {
            return Err(Error::BoundsFailure);
        }

        let left_column_index = permutation
            .mapping()
            .iter()
            .position(|c| c == &left_column)
            .ok_or(Error::SynthesisError)?;
        let right_column_index = permutation
            .mapping()
            .iter()
            .position(|c| c == &right_column)
            .ok_or(Error::SynthesisError)?;

        self.permutations[permutation.index()].copy(
            left_column_index,
            left_row,
            right_column_index,
            right_row,
        )
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

        let fixed = vec![vec![None; n as usize]; cs.num_fixed_columns];
        let advice = vec![vec![None; n as usize]; cs.num_advice_columns];
        let permutations = cs
            .permutations
            .iter()
            .map(|p| permutation::keygen::Assembly::new(n as usize, p))
            .collect();

        let mut prover = MockProver {
            n,
            cs,
            enabled_selectors: HashMap::default(),
            fixed,
            advice,
            instance,
            permutations,
        };

        circuit.synthesize(&mut prover, config)?;

        Ok(prover)
    }

    /// Returns `Ok(())` if this `MockProver` is satisfied, or a list of errors indicating
    /// the reasons that the circuit is not satisfied.
    pub fn verify(&self) -> Result<(), Vec<VerifyFailure>> {
        let n = self.n as i32;

        // Check that all cells used in instantiated gates have been assigned to.
        let selector_errors = self
            .cs
            .gates
            .iter()
            .enumerate()
            .flat_map(|(gate_index, gate)| {
                gate.queried_selectors()
                    .iter()
                    .map(|vc| (Selector(vc.column.try_into().unwrap()), vc.rotation))
                    // Assume that if a queried selector is enabled, the user wants to use the
                    // corresponding gate in some way.
                    //
                    // TODO: This will trip up on the reverse case, where leaving a selector
                    // un-enabled keeps a gate enabled. We could alternatively require that
                    // every selector is explicitly enabled or disabled on every row? But that
                    // seems messy and confusing.
                    .filter_map(|(s, rotation)| {
                        self.enabled_selectors.get(&s).map(|at| (at, rotation))
                    })
                    .flat_map(move |(at, rotation)| {
                        at.iter().flat_map(move |selector_row| {
                            // Determine the gate instance's logical row from the selector's
                            // concrete row and its rotation in the gate.
                            let gate_row = (*selector_row as i32 + n - rotation.0) % n;

                            gate.queried_cells().iter().filter_map(move |cell| {
                                // Determine where this cell should have been assigned.
                                let cell_row = ((gate_row + n + cell.rotation.0) % n) as usize;

                                // Check that it was assigned!
                                if match cell.column.column_type() {
                                    Any::Advice => {
                                        self.advice[cell.column.index()][cell_row].is_some()
                                    }
                                    Any::Fixed => {
                                        self.fixed[cell.column.index()][cell_row].is_some()
                                    }
                                    // Instance column cells are assigned outside the circuit.
                                    Any::Instance => true,
                                } {
                                    None
                                } else {
                                    Some(VerifyFailure::Cell {
                                        column: cell.column,
                                        row: cell_row,
                                        gate_index,
                                        gate_name: gate.name(),
                                    })
                                }
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
                        fn load_opt<'a, F: FieldExt, T: ColumnType>(
                            n: i32,
                            row: i32,
                            queries: &'a [(Column<T>, Rotation)],
                            cells: &'a [Vec<Option<F>>],
                        ) -> impl Fn(usize) -> F + 'a {
                            move |index| {
                                let (column, at) = &queries[index];
                                let resolved_row = (row + at.0) % n;
                                cell_value(cells[column.index()][resolved_row as usize])
                            }
                        }

                        fn load<'a, F: FieldExt, T: ColumnType>(
                            n: i32,
                            row: i32,
                            queries: &'a [(Column<T>, Rotation)],
                            cells: &'a [Vec<F>],
                        ) -> impl Fn(usize) -> F + 'a {
                            move |index| {
                                let (column, at) = &queries[index];
                                let resolved_row = (row + at.0) % n;
                                cells[column.index()][resolved_row as usize]
                            }
                        }

                        gate.polynomials().iter().enumerate().filter_map(
                            move |(poly_index, poly)| {
                                if poly.evaluate(
                                    &|scalar| scalar,
                                    &load_opt(n, row, &self.cs.fixed_queries, &self.fixed),
                                    &load_opt(n, row, &self.cs.advice_queries, &self.advice),
                                    &load(n, row, &self.cs.instance_queries, &self.instance),
                                    &|a, b| a + &b,
                                    &|a, b| a * &b,
                                    &|a, scalar| a * scalar,
                                ) == F::zero()
                                {
                                    None
                                } else {
                                    Some(VerifyFailure::Constraint {
                                        gate_index,
                                        gate_name: gate.name(),
                                        constraint_index: poly_index,
                                        row: (row - n) as usize,
                                    })
                                }
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
                    (0..n).filter_map(move |input_row| {
                        let load = |expression: &Expression<F>, row| {
                            expression.evaluate(
                                &|scalar| scalar,
                                &|index| {
                                    let query = self.cs.fixed_queries[index];
                                    let column_index = query.0.index();
                                    let rotation = query.1 .0;
                                    cell_value(
                                        self.fixed[column_index]
                                            [(row as i32 + n + rotation) as usize % n as usize],
                                    )
                                },
                                &|index| {
                                    let query = self.cs.advice_queries[index];
                                    let column_index = query.0.index();
                                    let rotation = query.1 .0;
                                    cell_value(
                                        self.advice[column_index]
                                            [(row as i32 + n + rotation) as usize % n as usize],
                                    )
                                },
                                &|index| {
                                    let query = self.cs.instance_queries[index];
                                    let column_index = query.0.index();
                                    let rotation = query.1 .0;
                                    self.instance[column_index]
                                        [(row as i32 + n + rotation) as usize % n as usize]
                                },
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                            )
                        };

                        let inputs: Vec<_> = lookup
                            .input_expressions
                            .iter()
                            .map(|c| load(c, input_row))
                            .collect();
                        let lookup_passes = (0..n)
                            .map(|table_row| {
                                lookup
                                    .table_expressions
                                    .iter()
                                    .map(move |c| load(c, table_row))
                            })
                            .any(|table_row| table_row.eq(inputs.iter().cloned()));
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
        let perm_errors =
            self.permutations
                .iter()
                .enumerate()
                .flat_map(|(perm_index, assembly)| {
                    // Original values of columns involved in the permutation
                    let original = |perm_index: usize, column, row| {
                        self.cs.permutations[perm_index]
                            .get_columns()
                            .get(column)
                            .map(|c: &Column<Any>| match c.column_type() {
                                Any::Advice => cell_value(self.advice[c.index()][row]),
                                Any::Fixed => cell_value(self.fixed[c.index()][row]),
                                Any::Instance => self.instance[c.index()][row],
                            })
                            .unwrap()
                    };

                    // Iterate over each column of the permutation
                    assembly
                        .mapping
                        .iter()
                        .enumerate()
                        .flat_map(move |(column, values)| {
                            // Iterate over each row of the column to check that the cell's
                            // value is preserved by the mapping.
                            values.iter().enumerate().filter_map(move |(row, cell)| {
                                let original_cell = original(perm_index, column, row);
                                let permuted_cell = original(perm_index, cell.0, cell.1);
                                if original_cell == permuted_cell {
                                    None
                                } else {
                                    Some(VerifyFailure::Permutation {
                                        perm_index,
                                        column,
                                        row,
                                    })
                                }
                            })
                        })
                });

        let errors: Vec<_> = iter::empty()
            .chain(selector_errors)
            .chain(gate_errors)
            .chain(lookup_errors)
            .chain(perm_errors)
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use pasta_curves::Fp;

    use super::{MockProver, VerifyFailure};
    use crate::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        plonk::{Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Error, Selector},
        poly::Rotation,
    };

    #[test]
    fn unassigned_cell() {
        const K: u32 = 4;
        const FAULTY_ROW: usize = 2;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            a: Column<Advice>,
            q: Selector,
        }

        struct FaultyCircuit {}

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let b = meta.advice_column();
                let q = meta.selector();

                meta.create_gate("Equality check", |cells| {
                    let a = cells.query_advice(a, Rotation::prev());
                    let b = cells.query_advice(b, Rotation::cur());
                    let q = cells.query_selector(q, Rotation::cur());

                    // If q is enabled, a and b must be assigned to.
                    vec![q * (a - b)]
                });

                FaultyCircuitConfig { a, q }
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<Fp>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;
                layouter.assign_region(
                    || "Faulty synthesis",
                    |mut region| {
                        // Enable the equality gate.
                        config.q.enable(&mut region, FAULTY_ROW)?;

                        // Assign a = 0.
                        region.assign_advice(
                            || "a",
                            config.a,
                            FAULTY_ROW - 1,
                            || Ok(Fp::zero()),
                        )?;

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
                column: Column::new(1, Any::Advice),
                row: FAULTY_ROW,
                gate_index: 0,
                gate_name: "Equality check"
            }])
        );
    }
}
