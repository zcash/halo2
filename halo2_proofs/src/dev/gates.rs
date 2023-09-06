use std::{
    collections::BTreeSet,
    fmt::{self, Write},
};

use ff::PrimeField;

use crate::{
    dev::util,
    plonk::{sealed::SealedPhase, Circuit, ConstraintSystem, FirstPhase},
};

#[derive(Debug)]
struct Constraint {
    name: String,
    expression: String,
    queries: BTreeSet<String>,
}

#[derive(Debug)]
struct Gate {
    name: String,
    constraints: Vec<Constraint>,
}

/// A struct for collecting and displaying the gates within a circuit.
///
/// # Examples
///
/// ```
/// use ff::Field;
/// use halo2_proofs::{
///     circuit::{Layouter, SimpleFloorPlanner},
///     dev::CircuitGates,
///     plonk::{Circuit, ConstraintSystem, Error},
///     poly::Rotation,
/// };
/// use halo2curves::pasta::pallas;
///
/// #[derive(Copy, Clone)]
/// struct MyConfig {}
///
/// #[derive(Clone, Default)]
/// struct MyCircuit {}
///
/// impl<F: Field> Circuit<F> for MyCircuit {
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
///             Some(("R1CS", s * (a * b - c)))
///         });
///
///         // We aren't using this circuit for anything in this example.
///         MyConfig {}
///     }
///
///     fn synthesize(&self, _: MyConfig, _: impl Layouter<F>) -> Result<(), Error> {
///         // Gates are known at configure time; it doesn't matter how we use them.
///         Ok(())
///     }
/// }
///
/// #[cfg(feature = "circuit-params")]
/// let gates = CircuitGates::collect::<pallas::Base, MyCircuit>(());
/// #[cfg(not(feature = "circuit-params"))]
/// let gates = CircuitGates::collect::<pallas::Base, MyCircuit>();
/// assert_eq!(
///     format!("{}", gates),
///     r#####"R1CS constraint:
/// - R1CS:
///   S0 * (A0@0 * A1@0 - A2@0)
/// Total gates: 1
/// Total custom constraint polynomials: 1
/// Total negations: 1
/// Total additions: 1
/// Total multiplications: 2
/// "#####,
/// );
/// ```
#[derive(Debug)]
pub struct CircuitGates {
    gates: Vec<Gate>,
    total_negations: usize,
    total_additions: usize,
    total_multiplications: usize,
}

impl CircuitGates {
    /// Collects the gates from within the circuit.
    pub fn collect<F: PrimeField, C: Circuit<F>>(
        #[cfg(feature = "circuit-params")] params: C::Params,
    ) -> Self {
        // Collect the graph details.
        let mut cs = ConstraintSystem::default();
        #[cfg(feature = "circuit-params")]
        let _ = C::configure_with_params(&mut cs, params);
        #[cfg(not(feature = "circuit-params"))]
        let _ = C::configure(&mut cs);

        let gates = cs
            .gates
            .iter()
            .map(|gate| Gate {
                name: gate.name().to_string(),
                constraints: gate
                    .polynomials()
                    .iter()
                    .enumerate()
                    .map(|(i, constraint)| Constraint {
                        name: gate.constraint_name(i).to_string(),
                        expression: constraint.evaluate(
                            &util::format_value,
                            &|selector| format!("S{}", selector.0),
                            &|query| format!("F{}@{}", query.column_index, query.rotation.0),
                            &|query| {
                                if query.phase == FirstPhase.to_sealed() {
                                    format!("A{}@{}", query.column_index, query.rotation.0)
                                } else {
                                    format!(
                                        "A{}({})@{}",
                                        query.column_index,
                                        query.phase(),
                                        query.rotation.0
                                    )
                                }
                            },
                            &|query| format!("I{}@{}", query.column_index, query.rotation.0),
                            &|challenge| format!("C{}({})", challenge.index(), challenge.phase()),
                            &|a| {
                                if a.contains(' ') {
                                    format!("-({})", a)
                                } else {
                                    format!("-{}", a)
                                }
                            },
                            &|a, b| {
                                if let Some(b) = b.strip_prefix('-') {
                                    format!("{} - {}", a, b)
                                } else {
                                    format!("{} + {}", a, b)
                                }
                            },
                            &|a, b| match (a.contains(' '), b.contains(' ')) {
                                (false, false) => format!("{} * {}", a, b),
                                (false, true) => format!("{} * ({})", a, b),
                                (true, false) => format!("({}) * {}", a, b),
                                (true, true) => format!("({}) * ({})", a, b),
                            },
                            &|a, s| {
                                if a.contains(' ') {
                                    format!("({}) * {}", a, util::format_value(s))
                                } else {
                                    format!("{} * {}", a, util::format_value(s))
                                }
                            },
                        ),
                        queries: constraint.evaluate(
                            &|_| BTreeSet::default(),
                            &|selector| vec![format!("S{}", selector.0)].into_iter().collect(),
                            &|query| {
                                vec![format!("F{}@{}", query.column_index, query.rotation.0)]
                                    .into_iter()
                                    .collect()
                            },
                            &|query| {
                                let query = if query.phase == FirstPhase.to_sealed() {
                                    format!("A{}@{}", query.column_index, query.rotation.0)
                                } else {
                                    format!(
                                        "A{}({})@{}",
                                        query.column_index,
                                        query.phase(),
                                        query.rotation.0
                                    )
                                };
                                vec![query].into_iter().collect()
                            },
                            &|query| {
                                vec![format!("I{}@{}", query.column_index, query.rotation.0)]
                                    .into_iter()
                                    .collect()
                            },
                            &|challenge| {
                                vec![format!("C{}({})", challenge.index(), challenge.phase())]
                                    .into_iter()
                                    .collect()
                            },
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
                        ),
                    })
                    .collect(),
            })
            .collect();

        let (total_negations, total_additions, total_multiplications) = cs
            .gates
            .iter()
            .flat_map(|gate| {
                gate.polynomials().iter().map(|poly| {
                    poly.evaluate(
                        &|_| (0, 0, 0),
                        &|_| (0, 0, 0),
                        &|_| (0, 0, 0),
                        &|_| (0, 0, 0),
                        &|_| (0, 0, 0),
                        &|_| (0, 0, 0),
                        &|(a_n, a_a, a_m)| (a_n + 1, a_a, a_m),
                        &|(a_n, a_a, a_m), (b_n, b_a, b_m)| (a_n + b_n, a_a + b_a + 1, a_m + b_m),
                        &|(a_n, a_a, a_m), (b_n, b_a, b_m)| (a_n + b_n, a_a + b_a, a_m + b_m + 1),
                        &|(a_n, a_a, a_m), _| (a_n, a_a, a_m + 1),
                    )
                })
            })
            .fold((0, 0, 0), |(acc_n, acc_a, acc_m), (n, a, m)| {
                (acc_n + n, acc_a + a, acc_m + m)
            });

        CircuitGates {
            gates,
            total_negations,
            total_additions,
            total_multiplications,
        }
    }

    /// Prints the queries in this circuit to a CSV grid.
    pub fn queries_to_csv(&self) -> String {
        let mut queries = BTreeSet::new();
        for gate in &self.gates {
            for constraint in &gate.constraints {
                for query in &constraint.queries {
                    queries.insert(query);
                }
            }
        }

        let mut ret = String::new();
        let w = &mut ret;
        for query in &queries {
            write!(w, "{},", query).unwrap();
        }
        writeln!(w, "Name").unwrap();

        for gate in &self.gates {
            for constraint in &gate.constraints {
                for query in &queries {
                    if constraint.queries.contains(*query) {
                        write!(w, "1").unwrap();
                    } else {
                        write!(w, "0").unwrap();
                    }
                    write!(w, ",").unwrap();
                }
                writeln!(w, "{}/{}", gate.name, constraint.name).unwrap();
            }
        }
        ret
    }
}

impl fmt::Display for CircuitGates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for gate in &self.gates {
            writeln!(f, "{}:", gate.name)?;
            for constraint in &gate.constraints {
                if constraint.name.is_empty() {
                    writeln!(f, "- {}", constraint.expression)?;
                } else {
                    writeln!(f, "- {}:", constraint.name)?;
                    writeln!(f, "  {}", constraint.expression)?;
                }
            }
        }
        writeln!(f, "Total gates: {}", self.gates.len())?;
        writeln!(
            f,
            "Total custom constraint polynomials: {}",
            self.gates
                .iter()
                .map(|gate| gate.constraints.len())
                .sum::<usize>()
        )?;
        writeln!(f, "Total negations: {}", self.total_negations)?;
        writeln!(f, "Total additions: {}", self.total_additions)?;
        writeln!(f, "Total multiplications: {}", self.total_multiplications)
    }
}
