//! Tools for developing circuits.

use ff::Field;

use crate::{
    arithmetic::{FieldExt, Group},
    plonk::{
        permutation, Advice, Any, Assignment, Circuit, Column, ColumnType, ConstraintSystem, Error,
        Expression, Fixed, Permutation,
    },
    poly::Rotation,
};

#[cfg(feature = "dev-graph")]
mod graph;

#[cfg(feature = "dev-graph")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-graph")))]
pub use graph::{circuit_dot_graph, layout::circuit_layout};

/// The reasons why a particular circuit is not satisfied.
#[derive(Debug, PartialEq)]
pub enum VerifyFailure {
    /// A gate was not satisfied for a particular row.
    Gate {
        /// The index of the gate that is not satisfied. These indices are assigned in the
        /// order in which `ConstraintSystem::create_gate` is called during
        /// `Circuit::configure`.
        gate_index: usize,
        /// The name of the gate that is not satisfied. These are specified by the gate
        /// creator (such as a chip implementation), and may not be unique.
        gate_name: &'static str,
        /// The row on which this gate is not satisfied.
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
///             a * b + c
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
///     Err(vec![VerifyFailure::Gate {
///         gate_index: 0,
///         gate_name: "R1CS constraint",
///         row: 0
///     }])
/// );
/// ```
#[derive(Debug)]
pub struct MockProver<F: Group + Field> {
    n: u32,
    cs: ConstraintSystem<F>,

    // The fixed cells in the circuit, arranged as [column][row].
    fixed: Vec<Vec<F>>,
    // The advice cells in the circuit, arranged as [column][row].
    advice: Vec<Vec<F>>,
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
            .ok_or(Error::BoundsFailure)? = to()?;

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
            .ok_or(Error::BoundsFailure)? = to()?;

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

        let fixed = vec![vec![F::zero(); n as usize]; cs.num_fixed_columns];
        let advice = vec![vec![F::zero(); n as usize]; cs.num_advice_columns];
        let permutations = cs
            .permutations
            .iter()
            .map(|p| permutation::keygen::Assembly::new(n as usize, p))
            .collect();

        let mut prover = MockProver {
            n,
            cs,
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

        // Check that all gates are satisfied for all rows.
        let gate_errors =
            self.cs
                .gates
                .iter()
                .enumerate()
                .flat_map(|(gate_index, (gate_name, gate))| {
                    // We iterate from n..2n so we can just reduce to handle wrapping.
                    (n..(2 * n)).filter_map(move |row| {
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

                        if gate.evaluate(
                            &|scalar| scalar,
                            &load(n, row, &self.cs.fixed_queries, &self.fixed),
                            &load(n, row, &self.cs.advice_queries, &self.advice),
                            &load(n, row, &self.cs.instance_queries, &self.instance),
                            &|a, b| a + &b,
                            &|a, b| a * &b,
                            &|a, scalar| a * scalar,
                        ) == F::zero()
                        {
                            None
                        } else {
                            Some(VerifyFailure::Gate {
                                gate_index,
                                gate_name,
                                row: (row - n) as usize,
                            })
                        }
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
                                    self.fixed[column_index]
                                        [(row as i32 + n + rotation) as usize % n as usize]
                                },
                                &|index| {
                                    let query = self.cs.advice_queries[index];
                                    let column_index = query.0.index();
                                    let rotation = query.1 .0;
                                    self.advice[column_index]
                                        [(row as i32 + n + rotation) as usize % n as usize]
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
                            .iter()
                            .map(|c| match c.column_type() {
                                Any::Advice => &self.advice[c.index()][..],
                                Any::Fixed => &self.fixed[c.index()][..],
                                Any::Instance => &self.instance[c.index()][..],
                            })
                            .nth(column)
                            .unwrap()[row]
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

        let errors: Vec<_> = gate_errors
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
