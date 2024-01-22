//! The cost estimator takes high-level parameters for a circuit design, and estimates the
//! verification cost, as well as resulting proof size.

use std::collections::HashSet;
use std::{iter, num::ParseIntError, str::FromStr};

use crate::plonk::Circuit;
use ff::{Field, FromUniformBytes};
use serde::Deserialize;
use serde_derive::Serialize;

use super::MockProver;

/// Supported commitment schemes
#[derive(Debug, Eq, PartialEq)]
pub enum CommitmentScheme {
    /// Inner Product Argument commitment scheme
    IPA,
    /// KZG with GWC19 mutli-open strategy
    KZGGWC,
    /// KZG with BDFG20 mutli-open strategy
    KZGSHPLONK,
}

/// Options to build a circuit specification to measure the cost model of.
#[derive(Debug)]
pub struct CostOptions {
    /// An advice column with the given rotations. May be repeated.
    pub advice: Vec<Poly>,

    /// An instance column with the given rotations. May be repeated.
    pub instance: Vec<Poly>,

    /// A fixed column with the given rotations. May be repeated.
    pub fixed: Vec<Poly>,

    /// Maximum degree of the custom gates.
    pub gate_degree: usize,

    /// Maximum degree of the constraint system.
    pub max_degree: usize,

    /// A lookup over N columns with max input degree I and max table degree T. May be repeated.
    pub lookup: Vec<Lookup>,

    /// A permutation over N columns. May be repeated.
    pub permutation: Permutation,

    /// A shuffle over N columns with max input degree I and max shuffle degree T. May be repeated.
    pub shuffle: Vec<Shuffle>,

    /// 2^K bound on the number of rows.
    pub k: usize,
}

/// Structure holding polynomial related data for benchmarks
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Poly {
    /// Rotations for the given polynomial
    pub rotations: Vec<isize>,
}

impl FromStr for Poly {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rotations: Vec<isize> =
            s.split(',').map(|r| r.parse()).collect::<Result<_, _>>()?;
        rotations.sort_unstable();
        Ok(Poly { rotations })
    }
}

/// Structure holding the Lookup related data for circuit benchmarks.
#[derive(Debug, Clone)]
pub struct Lookup;

impl Lookup {
    fn queries(&self) -> impl Iterator<Item = Poly> {
        // - product commitments at x and \omega x
        // - input commitments at x and x_inv
        // - table commitments at x
        let product = "0,1".parse().unwrap();
        let input = "0,-1".parse().unwrap();
        let table = "0".parse().unwrap();

        iter::empty()
            .chain(Some(product))
            .chain(Some(input))
            .chain(Some(table))
    }
}

/// Number of permutation enabled columns
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Permutation {
    columns: usize,
}

impl Permutation {
    fn queries(&self) -> impl Iterator<Item = Poly> {
        // - product commitments at x and x_inv
        // - polynomial commitments at x
        let product = "0,-1".parse().unwrap();
        let poly = "0".parse().unwrap();

        iter::empty()
            .chain(Some(product))
            .chain(iter::repeat(poly).take(self.columns))
    }
}

/// Structure holding the [Shuffle] related data for circuit benchmarks.
#[derive(Debug, Clone)]
pub struct Shuffle;

impl Shuffle {
    fn queries(&self) -> impl Iterator<Item = Poly> {
        // Open shuffle product commitment at x and \omega x
        let shuffle = "0, 1".parse().unwrap();

        iter::empty().chain(Some(shuffle))
    }
}

/// High-level specifications of an abstract circuit.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCircuit {
    /// Power-of-2 bound on the number of rows in the circuit.
    pub k: usize,
    /// Maximum degree of the circuit.
    pub max_deg: usize,
    /// Number of advice columns.
    pub advice_columns: usize,
    /// Number of lookup arguments.
    pub lookups: usize,
    /// Equality constraint enabled columns.
    pub permutations: usize,
    /// Number of shuffle arguments
    pub shuffles: usize,
    /// Number of distinct column queries across all gates.
    pub column_queries: usize,
    /// Number of distinct sets of points in the multiopening argument.
    pub point_sets: usize,
    /// Size of the proof for the circuit
    pub size: usize,
}

impl CostOptions {
    /// Convert [CostOptions] to [ModelCircuit]. The proof sizè is computed depending on the base
    /// and scalar field size of the curve used, together with the [CommitmentScheme].
    pub fn into_model_circuit<const COMM: usize, const SCALAR: usize>(
        &self,
        comm_scheme: CommitmentScheme,
    ) -> ModelCircuit {
        let mut queries: Vec<_> = iter::empty()
            .chain(self.advice.iter())
            .chain(self.instance.iter())
            .chain(self.fixed.iter())
            .cloned()
            .chain(self.lookup.iter().flat_map(|l| l.queries()))
            .chain(self.permutation.queries())
            .chain(self.shuffle.iter().flat_map(|s| s.queries()))
            .chain(iter::repeat("0".parse().unwrap()).take(self.max_degree - 1))
            .collect();

        let column_queries = queries.len();
        queries.sort_unstable();
        queries.dedup();
        let point_sets = queries.len();

        let comp_bytes = |points: usize, scalars: usize| points * COMM + scalars * SCALAR;

        // PLONK:
        // - COMM bytes (commitment) per advice column
        // - 3 * COMM bytes (commitments) + 5 * SCALAR bytes (evals) per lookup column
        // - COMM bytes (commitment) + 2 * SCALAR bytes (evals) per permutation argument
        // - COMM bytes (eval) per column per permutation argument
        let plonk = comp_bytes(1, 0) * self.advice.len()
            + comp_bytes(3, 5) * self.lookup.len()
            + comp_bytes(1, 2 + self.permutation.columns);

        // Vanishing argument:
        // - (max_deg - 1) * COMM bytes (commitments) + (max_deg - 1) * SCALAR bytes (h_evals)
        //   for quotient polynomial
        // - SCALAR bytes (eval) per column query
        let vanishing =
            comp_bytes(self.max_degree - 1, self.max_degree - 1) + comp_bytes(0, column_queries);

        // Multiopening argument:
        // - f_commitment (COMM bytes)
        // - SCALAR bytes (evals) per set of points in multiopen argument
        let multiopen = comp_bytes(1, point_sets);

        let polycomm = match comm_scheme {
            CommitmentScheme::IPA => {
                // Polycommit IPA:
                // - s_poly commitment (COMM bytes)
                // - inner product argument (k rounds * 2 * COMM bytes)
                // - a (SCALAR bytes)
                // - xi (SCALAR bytes)
                comp_bytes(1 + 2 * self.k, 2)
            }
            CommitmentScheme::KZGGWC => {
                let mut nr_rotations = HashSet::new();
                for poly in self.advice.iter() {
                    nr_rotations.extend(poly.rotations.clone());
                }
                for poly in self.fixed.iter() {
                    nr_rotations.extend(poly.rotations.clone());
                }
                for poly in self.instance.iter() {
                    nr_rotations.extend(poly.rotations.clone());
                }

                // Polycommit GWC:
                // - number_rotations * COMM bytes
                comp_bytes(nr_rotations.len(), 0)
            }
            CommitmentScheme::KZGSHPLONK => {
                // Polycommit SHPLONK:
                // - quotient polynomial commitment (COMM bytes)
                comp_bytes(1, 0)
            }
        };

        let size = plonk + vanishing + multiopen + polycomm;

        ModelCircuit {
            k: self.k,
            max_deg: self.max_degree,
            advice_columns: self.advice.len(),
            lookups: self.lookup.len(),
            permutations: self.permutation.columns,
            shuffles: self.shuffle.len(),
            column_queries,
            point_sets,
            size,
        }
    }
}

/// Given a Plonk circuit, this function returns a [ModelCircuit]
pub fn from_circuit_to_model_circuit<
    F: Ord + Field + FromUniformBytes<64>,
    C: Circuit<F>,
    const COMM: usize,
    const SCALAR: usize,
>(
    k: u32,
    circuit: &C,
    instances: Vec<Vec<F>>,
    comm_scheme: CommitmentScheme,
) -> ModelCircuit {
    let options = from_circuit_to_cost_model_options(k, circuit, instances);
    options.into_model_circuit::<COMM, SCALAR>(comm_scheme)
}

/// Given a Plonk circuit, this function returns [CostOptions]
pub fn from_circuit_to_cost_model_options<F: Ord + Field + FromUniformBytes<64>, C: Circuit<F>>(
    k: u32,
    circuit: &C,
    instances: Vec<Vec<F>>,
) -> CostOptions {
    let prover = MockProver::run(k, circuit, instances).unwrap();
    let cs = prover.cs;

    let fixed = {
        // init the fixed polynomials with no rotations
        let mut fixed = vec![Poly { rotations: vec![] }; cs.num_fixed_columns()];
        for (col, rot) in cs.fixed_queries() {
            fixed[col.index()].rotations.push(rot.0 as isize);
        }
        fixed
    };

    let advice = {
        // init the advice polynomials with no rotations
        let mut advice = vec![Poly { rotations: vec![] }; cs.num_advice_columns()];
        for (col, rot) in cs.advice_queries() {
            advice[col.index()].rotations.push(rot.0 as isize);
        }
        advice
    };

    let instance = {
        // init the instance polynomials with no rotations
        let mut instance = vec![Poly { rotations: vec![] }; cs.num_instance_columns()];
        for (col, rot) in cs.instance_queries() {
            instance[col.index()].rotations.push(rot.0 as isize);
        }
        instance
    };

    let lookup = { cs.lookups().iter().map(|_| Lookup).collect::<Vec<_>>() };

    let permutation = Permutation {
        columns: cs.permutation().get_columns().len(),
    };

    let shuffle = { cs.shuffles.iter().map(|_| Shuffle).collect::<Vec<_>>() };

    let gate_degree = cs
        .gates
        .iter()
        .flat_map(|gate| gate.polynomials().iter().map(|poly| poly.degree()))
        .max()
        .unwrap_or(0);

    let k = prover.k.try_into().unwrap();

    CostOptions {
        advice,
        instance,
        fixed,
        gate_degree,
        max_degree: cs.degree(),
        lookup,
        permutation,
        shuffle,
        k,
    }
}
