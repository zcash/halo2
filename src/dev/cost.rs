//! Developer tools for investigating the cost of a circuit.

use std::{
    collections::{HashMap, HashSet},
    iter,
    marker::PhantomData,
    ops::{Add, Mul},
};

use ff::{Field, PrimeField};
use group::prime::PrimeGroup;

use crate::{
    plonk::{
        Advice, Any, Assigned, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed,
        FloorPlanner, Instance, Selector,
    },
    poly::Rotation,
};

/// Measures a circuit to determine its costs, and explain what contributes to them.
#[derive(Debug)]
pub struct CircuitCost<G: PrimeGroup, ConcreteCircuit: Circuit<G::Scalar>> {
    /// Power-of-2 bound on the number of rows in the circuit.
    k: usize,
    /// Maximum degree of the circuit.
    max_deg: usize,
    /// Number of advice columns.
    advice_columns: usize,
    /// Number of direct queries for each column type.
    instance_queries: usize,
    advice_queries: usize,
    fixed_queries: usize,
    /// Number of lookup arguments.
    lookups: usize,
    /// Number of columns in the global permutation.
    permutation_cols: usize,
    /// Number of distinct sets of points in the multiopening argument.
    point_sets: usize,

    _marker: PhantomData<(G, ConcreteCircuit)>,
}

struct Assembly {
    selectors: Vec<Vec<bool>>,
}

impl<F: Field> Assignment<F> for Assembly {
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

    fn enable_selector<A, AR>(&mut self, _: A, selector: &Selector, row: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.selectors[selector.0][row] = true;

        Ok(())
    }

    fn query_instance(&self, _: Column<Instance>, _: usize) -> Result<Option<F>, Error> {
        Ok(None)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Advice>,
        _: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
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
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        Ok(())
    }

    fn copy(&mut self, _: Column<Any>, _: usize, _: Column<Any>, _: usize) -> Result<(), Error> {
        Ok(())
    }

    fn fill_from_row(
        &mut self,
        _: Column<Fixed>,
        _: usize,
        _: Option<Assigned<F>>,
    ) -> Result<(), Error> {
        Ok(())
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

impl<G: PrimeGroup, ConcreteCircuit: Circuit<G::Scalar>> CircuitCost<G, ConcreteCircuit> {
    /// Measures a circuit with parameter constant `k`.
    ///
    /// Panics if `k` is not large enough for the circuit.
    pub fn measure(k: usize, circuit: &ConcreteCircuit) -> Self {
        // Collect the layout details.
        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure(&mut cs);
        let mut assembly = Assembly {
            selectors: vec![vec![false; 1 << k]; cs.num_selectors],
        };
        ConcreteCircuit::FloorPlanner::synthesize(
            &mut assembly,
            circuit,
            config,
            cs.constants.clone(),
        )
        .unwrap();
        let (cs, _) = cs.compress_selectors(assembly.selectors);

        assert!((1 << k) >= cs.minimum_rows());

        // Figure out how many point sets we have due to queried cells.
        let mut column_queries: HashMap<Column<Any>, HashSet<i32>> = HashMap::new();
        for (c, r) in iter::empty()
            .chain(
                cs.advice_queries
                    .iter()
                    .map(|(c, r)| (Column::<Any>::from(*c), *r)),
            )
            .chain(cs.instance_queries.iter().map(|(c, r)| ((*c).into(), *r)))
            .chain(cs.fixed_queries.iter().map(|(c, r)| ((*c).into(), *r)))
            .chain(
                cs.permutation
                    .get_columns()
                    .into_iter()
                    .map(|c| (c, Rotation::cur())),
            )
        {
            column_queries.entry(c).or_default().insert(r.0);
        }
        let mut point_sets: HashSet<Vec<i32>> = HashSet::new();
        for (_, r) in column_queries {
            // Sort the query sets so we merge duplicates.
            let mut query_set: Vec<_> = r.into_iter().collect();
            query_set.sort_unstable();
            point_sets.insert(query_set);
        }

        // Include lookup polynomials in point sets:
        point_sets.insert(vec![0, 1]); // product_poly
        point_sets.insert(vec![-1, 0]); // permuted_input_poly
        point_sets.insert(vec![0]); // permuted_table_poly

        // Include permutation polynomials in point sets.
        point_sets.insert(vec![0, 1]); // permutation_product_poly
        let max_deg = cs.degree();
        let permutation_cols = cs.permutation.get_columns().len();
        if permutation_cols > max_deg - 2 {
            // permutation_product_poly for chaining chunks.
            point_sets.insert(vec![-((cs.blinding_factors() + 1) as i32), 0, 1]);
        }

        CircuitCost {
            k,
            max_deg,
            advice_columns: cs.num_advice_columns,
            instance_queries: cs.instance_queries.len(),
            advice_queries: cs.advice_queries.len(),
            fixed_queries: cs.fixed_queries.len(),
            lookups: cs.lookups.len(),
            permutation_cols,
            point_sets: point_sets.len(),
            _marker: PhantomData::default(),
        }
    }

    fn permutation_chunks(&self) -> usize {
        let chunk_size = self.max_deg - 2;
        (self.permutation_cols + chunk_size - 1) / chunk_size
    }

    /// Returns the marginal proof size per instance of this circuit.
    pub fn marginal_proof_size(&self) -> MarginalProofSize<G> {
        let chunks = self.permutation_chunks();

        MarginalProofSize {
            // Cells:
            // - 1 commitment per advice column per instance
            // - 1 eval per instance column query per instance
            // - 1 eval per advice column query per instance
            instance: ProofContribution::new(0, self.instance_queries),
            advice: ProofContribution::new(self.advice_columns, self.advice_queries),

            // Lookup arguments:
            // - 3 commitments per lookup argument per instance
            // - 5 evals per lookup argument per instance
            lookups: ProofContribution::new(3 * self.lookups, 5 * self.lookups),

            // Global permutation argument:
            // - chunks commitments per instance
            // - 2*chunks + (chunks - 1) evals per instance
            equality: ProofContribution::new(chunks, 3 * chunks - 1),

            _marker: PhantomData::default(),
        }
    }

    /// Returns the proof size for the given number of instances of this circuit.
    pub fn proof_size(&self, instances: usize) -> ProofSize<G> {
        let marginal = self.marginal_proof_size();

        ProofSize {
            // Cells:
            // - marginal cost per instance
            // - 1 eval per fixed column query
            instance: marginal.instance * instances,
            advice: marginal.advice * instances,
            fixed: ProofContribution::new(0, self.fixed_queries),

            // Lookup arguments:
            // - marginal cost per instance
            lookups: marginal.lookups * instances,

            // Global permutation argument:
            // - marginal cost per instance
            // - 1 eval per column
            equality: marginal.equality * instances
                + ProofContribution::new(0, self.permutation_cols),

            // Vanishing argument:
            // - 1 + (max_deg - 1) commitments
            // - 1 random_poly eval
            vanishing: ProofContribution::new(self.max_deg, 1),

            // Multiopening argument:
            // - f_commitment
            // - 1 eval per set of points in multiopen argument
            multiopen: ProofContribution::new(1, self.point_sets),

            // Polycommit:
            // - s_poly commitment
            // - inner product argument (2 * k round commitments)
            // - a
            // - xi
            polycomm: ProofContribution::new(1 + 2 * self.k, 2),

            _marker: PhantomData::default(),
        }
    }
}

/// (commitments, evaluations)
#[derive(Debug)]
struct ProofContribution {
    commitments: usize,
    evaluations: usize,
}

impl ProofContribution {
    fn new(commitments: usize, evaluations: usize) -> Self {
        ProofContribution {
            commitments,
            evaluations,
        }
    }

    fn len(&self, point: usize, scalar: usize) -> usize {
        self.commitments * point + self.evaluations * scalar
    }
}

impl Add for ProofContribution {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            commitments: self.commitments + rhs.commitments,
            evaluations: self.evaluations + rhs.evaluations,
        }
    }
}

impl Mul<usize> for ProofContribution {
    type Output = Self;

    fn mul(self, instances: usize) -> Self::Output {
        Self {
            commitments: self.commitments * instances,
            evaluations: self.evaluations * instances,
        }
    }
}

/// The marginal size of a Halo 2 proof, broken down into its contributing factors.
#[derive(Debug)]
pub struct MarginalProofSize<G: PrimeGroup> {
    instance: ProofContribution,
    advice: ProofContribution,
    lookups: ProofContribution,
    equality: ProofContribution,
    _marker: PhantomData<G>,
}

impl<G: PrimeGroup> From<MarginalProofSize<G>> for usize {
    fn from(proof: MarginalProofSize<G>) -> Self {
        let point = G::Repr::default().as_ref().len();
        let scalar = <G::Scalar as PrimeField>::Repr::default().as_ref().len();

        proof.instance.len(point, scalar)
            + proof.advice.len(point, scalar)
            + proof.lookups.len(point, scalar)
            + proof.equality.len(point, scalar)
    }
}

/// The size of a Halo 2 proof, broken down into its contributing factors.
#[derive(Debug)]
pub struct ProofSize<G: PrimeGroup> {
    instance: ProofContribution,
    advice: ProofContribution,
    fixed: ProofContribution,
    lookups: ProofContribution,
    equality: ProofContribution,
    vanishing: ProofContribution,
    multiopen: ProofContribution,
    polycomm: ProofContribution,
    _marker: PhantomData<G>,
}

impl<G: PrimeGroup> From<ProofSize<G>> for usize {
    fn from(proof: ProofSize<G>) -> Self {
        let point = G::Repr::default().as_ref().len();
        let scalar = <G::Scalar as PrimeField>::Repr::default().as_ref().len();

        proof.instance.len(point, scalar)
            + proof.advice.len(point, scalar)
            + proof.fixed.len(point, scalar)
            + proof.lookups.len(point, scalar)
            + proof.equality.len(point, scalar)
            + proof.vanishing.len(point, scalar)
            + proof.multiopen.len(point, scalar)
            + proof.polycomm.len(point, scalar)
    }
}
