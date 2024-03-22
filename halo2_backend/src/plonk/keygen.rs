//! This module
//! - creates the proving and verifying keys for a circuit
//! - crates a domain, constraint system, and configuration for a circuit

#![allow(clippy::int_plus_one)]

use group::Curve;
use halo2_middleware::ff::{Field, FromUniformBytes};

use super::{evaluation::Evaluator, permutation, Polynomial, ProvingKey, VerifyingKey};
use crate::{
    arithmetic::{parallelize, CurveAffine},
    plonk::circuit::{
        ConstraintSystemBack, ExpressionBack, GateBack, LookupArgumentBack, QueryBack,
        ShuffleArgumentBack, VarBack,
    },
    plonk::Error,
    poly::{
        commitment::{Blind, Params},
        EvaluationDomain,
    },
};
use halo2_middleware::circuit::{
    Any, ColumnMid, CompiledCircuitV2, ConstraintSystemMid, ExpressionMid, VarMid,
};
use halo2_middleware::{lookup, poly::Rotation, shuffle};
use std::collections::HashMap;

/// Creates a domain, constraint system, and configuration for a circuit.
pub(crate) fn create_domain<C>(
    cs: &ConstraintSystemBack<C::Scalar>,
    k: u32,
) -> EvaluationDomain<C::Scalar>
where
    C: CurveAffine,
{
    let degree = cs.degree();
    EvaluationDomain::new(degree as u32, k)
}

/// Generate a `VerifyingKey` from an instance of `CompiledCircuit`.
pub fn keygen_vk_v2<'params, C, P>(
    params: &P,
    circuit: &CompiledCircuitV2<C::Scalar>,
) -> Result<VerifyingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<'params, C>,
    C::Scalar: FromUniformBytes<64>,
{
    let cs_mid = &circuit.cs;
    let cs: ConstraintSystemBack<C::Scalar> = cs_mid.clone().into();
    let domain = EvaluationDomain::new(cs.degree() as u32, params.k());

    if (params.n() as usize) < cs.minimum_rows() {
        return Err(Error::not_enough_rows_available(params.k()));
    }

    let permutation_vk = permutation::keygen::Assembly::new_from_assembly_mid(
        params.n() as usize,
        &cs_mid.permutation,
        &circuit.preprocessing.permutation,
    )?
    .build_vk(params, &domain, &cs.permutation);

    let fixed_commitments = circuit
        .preprocessing
        .fixed
        .iter()
        .map(|poly| {
            params
                .commit_lagrange(
                    &Polynomial::new_lagrange_from_vec(poly.clone()),
                    Blind::default(),
                )
                .to_affine()
        })
        .collect();

    Ok(VerifyingKey::from_parts(
        domain,
        fixed_commitments,
        permutation_vk,
        cs,
    ))
}

/// Generate a `ProvingKey` from a `VerifyingKey` and an instance of `CompiledCircuit`.
pub fn keygen_pk_v2<'params, C, P>(
    params: &P,
    vk: VerifyingKey<C>,
    circuit: &CompiledCircuitV2<C::Scalar>,
) -> Result<ProvingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<'params, C>,
{
    let cs = &circuit.cs;

    if (params.n() as usize) < vk.cs.minimum_rows() {
        return Err(Error::not_enough_rows_available(params.k()));
    }

    // Compute fixeds

    let fixed_polys: Vec<_> = circuit
        .preprocessing
        .fixed
        .iter()
        .map(|poly| {
            vk.domain
                .lagrange_to_coeff(Polynomial::new_lagrange_from_vec(poly.clone()))
        })
        .collect();

    let fixed_cosets = fixed_polys
        .iter()
        .map(|poly| vk.domain.coeff_to_extended(poly.clone()))
        .collect();

    let fixed_values = circuit
        .preprocessing
        .fixed
        .clone()
        .into_iter()
        .map(Polynomial::new_lagrange_from_vec)
        .collect();

    // Compute l_0(X)
    // TODO: this can be done more efficiently
    // https://github.com/privacy-scaling-explorations/halo2/issues/269
    let mut l0 = vk.domain.empty_lagrange();
    l0[0] = C::Scalar::ONE;
    let l0 = vk.domain.lagrange_to_coeff(l0);
    let l0 = vk.domain.coeff_to_extended(l0);

    // Compute l_blind(X) which evaluates to 1 for each blinding factor row
    // and 0 otherwise over the domain.
    let mut l_blind = vk.domain.empty_lagrange();
    for evaluation in l_blind[..].iter_mut().rev().take(vk.cs.blinding_factors()) {
        *evaluation = C::Scalar::ONE;
    }
    let l_blind = vk.domain.lagrange_to_coeff(l_blind);
    let l_blind = vk.domain.coeff_to_extended(l_blind);

    // Compute l_last(X) which evaluates to 1 on the first inactive row (just
    // before the blinding factors) and 0 otherwise over the domain
    let mut l_last = vk.domain.empty_lagrange();
    l_last[params.n() as usize - vk.cs.blinding_factors() - 1] = C::Scalar::ONE;
    let l_last = vk.domain.lagrange_to_coeff(l_last);
    let l_last = vk.domain.coeff_to_extended(l_last);

    // Compute l_active_row(X)
    let one = C::Scalar::ONE;
    let mut l_active_row = vk.domain.empty_extended();
    parallelize(&mut l_active_row, |values, start| {
        for (i, value) in values.iter_mut().enumerate() {
            let idx = i + start;
            *value = one - (l_last[idx] + l_blind[idx]);
        }
    });

    // Compute the optimized evaluation data structure
    let ev = Evaluator::new(&vk.cs);

    // Compute the permutation proving key
    let permutation_pk = permutation::keygen::Assembly::new_from_assembly_mid(
        params.n() as usize,
        &cs.permutation,
        &circuit.preprocessing.permutation,
    )?
    .build_pk(params, &vk.domain, &cs.permutation.clone());

    Ok(ProvingKey {
        vk,
        l0,
        l_last,
        l_active_row,
        fixed_values,
        fixed_polys,
        fixed_cosets,
        permutation: permutation_pk,
        ev,
    })
}

struct QueriesMap {
    map: HashMap<(ColumnMid, Rotation), usize>,
    advice: Vec<(ColumnMid, Rotation)>,
    instance: Vec<(ColumnMid, Rotation)>,
    fixed: Vec<(ColumnMid, Rotation)>,
}

impl QueriesMap {
    fn add(&mut self, col: ColumnMid, rot: Rotation) -> usize {
        *self
            .map
            .entry((col, rot))
            .or_insert_with(|| match col.column_type {
                Any::Advice => {
                    self.advice.push((col, rot));
                    self.advice.len() - 1
                }
                Any::Instance => {
                    self.instance.push((col, rot));
                    self.instance.len() - 1
                }
                Any::Fixed => {
                    self.fixed.push((col, rot));
                    self.fixed.len() - 1
                }
            })
    }
}

impl QueriesMap {
    fn as_expression<F: Field>(&mut self, expr: &ExpressionMid<F>) -> ExpressionBack<F> {
        match expr {
            ExpressionMid::Constant(c) => ExpressionBack::Constant(*c),
            ExpressionMid::Var(VarMid::Query(query)) => {
                let column = ColumnMid::new(query.column_type, query.column_index);
                let index = self.add(column, query.rotation);
                ExpressionBack::Var(VarBack::Query(QueryBack {
                    index,
                    column_index: query.column_index,
                    column_type: query.column_type,
                    rotation: query.rotation,
                }))
            }
            ExpressionMid::Var(VarMid::Challenge(c)) => ExpressionBack::Var(VarBack::Challenge(*c)),
            ExpressionMid::Negated(e) => ExpressionBack::Negated(Box::new(self.as_expression(e))),
            ExpressionMid::Sum(lhs, rhs) => ExpressionBack::Sum(
                Box::new(self.as_expression(lhs)),
                Box::new(self.as_expression(rhs)),
            ),
            ExpressionMid::Product(lhs, rhs) => ExpressionBack::Product(
                Box::new(self.as_expression(lhs)),
                Box::new(self.as_expression(rhs)),
            ),
            ExpressionMid::Scaled(e, c) => {
                ExpressionBack::Scaled(Box::new(self.as_expression(e)), *c)
            }
        }
    }
}

/// Collect queries used in gates while mapping those gates to equivalent ones with indexed
/// query references in the expressions.
fn cs_mid_collect_queries_gates<F: Field>(
    cs_mid: &ConstraintSystemMid<F>,
    queries: &mut QueriesMap,
) -> Vec<GateBack<F>> {
    cs_mid
        .gates
        .iter()
        .map(|gate| GateBack {
            name: gate.name.clone(),
            poly: queries.as_expression(&gate.poly),
        })
        .collect()
}

/// Collect queries used in lookups while mapping those lookups to equivalent ones with indexed
/// query references in the expressions.
fn cs_mid_collect_queries_lookups<F: Field>(
    cs_mid: &ConstraintSystemMid<F>,
    queries: &mut QueriesMap,
) -> Vec<LookupArgumentBack<F>> {
    cs_mid
        .lookups
        .iter()
        .map(|lookup| lookup::Argument {
            name: lookup.name.clone(),
            input_expressions: lookup
                .input_expressions
                .iter()
                .map(|e| queries.as_expression(e))
                .collect(),
            table_expressions: lookup
                .table_expressions
                .iter()
                .map(|e| queries.as_expression(e))
                .collect(),
        })
        .collect()
}

/// Collect queries used in shuffles while mapping those lookups to equivalent ones with indexed
/// query references in the expressions.
fn cs_mid_collect_queries_shuffles<F: Field>(
    cs_mid: &ConstraintSystemMid<F>,
    queries: &mut QueriesMap,
) -> Vec<ShuffleArgumentBack<F>> {
    cs_mid
        .shuffles
        .iter()
        .map(|shuffle| shuffle::Argument {
            name: shuffle.name.clone(),
            input_expressions: shuffle
                .input_expressions
                .iter()
                .map(|e| queries.as_expression(e))
                .collect(),
            shuffle_expressions: shuffle
                .shuffle_expressions
                .iter()
                .map(|e| queries.as_expression(e))
                .collect(),
        })
        .collect()
}

/// Collect all queries used in the expressions of gates, lookups and shuffles.  Map the
/// expressions of gates, lookups and shuffles into equivalent ones with indexed query
/// references.
#[allow(clippy::type_complexity)]
fn collect_queries<F: Field>(
    cs_mid: &ConstraintSystemMid<F>,
) -> (
    Queries,
    Vec<GateBack<F>>,
    Vec<LookupArgumentBack<F>>,
    Vec<ShuffleArgumentBack<F>>,
) {
    let mut queries = QueriesMap {
        map: HashMap::new(),
        advice: Vec::new(),
        instance: Vec::new(),
        fixed: Vec::new(),
    };

    let gates = cs_mid_collect_queries_gates(cs_mid, &mut queries);
    let lookups = cs_mid_collect_queries_lookups(cs_mid, &mut queries);
    let shuffles = cs_mid_collect_queries_shuffles(cs_mid, &mut queries);

    // Each column used in a copy constraint involves a query at rotation current.
    for column in &cs_mid.permutation.columns {
        queries.add(*column, Rotation::cur());
    }

    let mut num_advice_queries = vec![0; cs_mid.num_advice_columns];
    for (column, _) in queries.advice.iter() {
        num_advice_queries[column.index] += 1;
    }

    let queries = Queries {
        advice: queries.advice,
        instance: queries.instance,
        fixed: queries.fixed,
        num_advice_queries,
    };
    (queries, gates, lookups, shuffles)
}

impl<F: Field> From<ConstraintSystemMid<F>> for ConstraintSystemBack<F> {
    fn from(cs_mid: ConstraintSystemMid<F>) -> Self {
        let (queries, gates, lookups, shuffles) = collect_queries(&cs_mid);
        Self {
            num_fixed_columns: cs_mid.num_fixed_columns,
            num_advice_columns: cs_mid.num_advice_columns,
            num_instance_columns: cs_mid.num_instance_columns,
            num_challenges: cs_mid.num_challenges,
            unblinded_advice_columns: cs_mid.unblinded_advice_columns,
            advice_column_phase: cs_mid.advice_column_phase,
            challenge_phase: cs_mid.challenge_phase,
            gates,
            advice_queries: queries.advice,
            num_advice_queries: queries.num_advice_queries,
            instance_queries: queries.instance,
            fixed_queries: queries.fixed,
            permutation: cs_mid.permutation,
            lookups,
            shuffles,
            minimum_degree: cs_mid.minimum_degree,
        }
    }
}

/// List of queries (columns and rotations) used by a circuit
#[derive(Debug, Clone)]
pub struct Queries {
    /// List of unique advice queries
    pub advice: Vec<(ColumnMid, Rotation)>,
    /// List of unique instance queries
    pub instance: Vec<(ColumnMid, Rotation)>,
    /// List of unique fixed queries
    pub fixed: Vec<(ColumnMid, Rotation)>,
    /// Contains an integer for each advice column
    /// identifying how many distinct queries it has
    /// so far; should be same length as cs.num_advice_columns.
    pub num_advice_queries: Vec<usize>,
}

impl Queries {
    /// Returns the minimum necessary rows that need to exist in order to
    /// account for e.g. blinding factors.
    pub fn minimum_rows(&self) -> usize {
        self.blinding_factors() // m blinding factors
            + 1 // for l_{-(m + 1)} (l_last)
            + 1 // for l_0 (just for extra breathing room for the permutation
                // argument, to essentially force a separation in the
                // permutation polynomial between the roles of l_last, l_0
                // and the interstitial values.)
            + 1 // for at least one row
    }

    /// Compute the number of blinding factors necessary to perfectly blind
    /// each of the prover's witness polynomials.
    pub fn blinding_factors(&self) -> usize {
        // All of the prover's advice columns are evaluated at no more than
        let factors = *self.num_advice_queries.iter().max().unwrap_or(&1);
        // distinct points during gate checks.

        // - The permutation argument witness polynomials are evaluated at most 3 times.
        // - Each lookup argument has independent witness polynomials, and they are
        //   evaluated at most 2 times.
        let factors = std::cmp::max(3, factors);

        // Each polynomial is evaluated at most an additional time during
        // multiopen (at x_3 to produce q_evals):
        let factors = factors + 1;

        // h(x) is derived by the other evaluations so it does not reveal
        // anything; in fact it does not even appear in the proof.

        // h(x_3) is also not revealed; the verifier only learns a single
        // evaluation of a polynomial in x_1 which has h(x_3) and another random
        // polynomial evaluated at x_3 as coefficients -- this random polynomial
        // is "random_poly" in the vanishing argument.

        // Add an additional blinding factor as a slight defense against
        // off-by-one errors.
        factors + 1
    }
}
