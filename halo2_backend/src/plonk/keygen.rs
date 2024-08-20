//! This module
//! - creates the proving and verifying keys for a circuit
//! - crates a domain, constraint system, and configuration for a circuit

#![allow(clippy::int_plus_one)]

use ff::{BatchInvert, WithSmallOrderMulGroup};
use group::Curve;
use halo2_middleware::ff::{Field, FromUniformBytes};
use halo2_middleware::zal::impls::H2cEngine;

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
    Any, ColumnMid, CompiledCircuit, ConstraintSystemMid, ExpressionMid, VarMid,
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
pub fn keygen_vk<C, P>(
    params: &P,
    circuit: &CompiledCircuit<C::Scalar>,
) -> Result<VerifyingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<C>,
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

    let fixed_commitments = {
        let fixed_commitments_projective: Vec<C::CurveExt> = circuit
            .preprocessing
            .fixed
            .iter()
            .map(|poly| {
                params.commit_lagrange(
                    &H2cEngine::new(),
                    &Polynomial::new_lagrange_from_vec(poly.clone()),
                    Blind::default(),
                )
            })
            .collect();
        let mut fixed_commitments = vec![C::identity(); fixed_commitments_projective.len()];
        C::CurveExt::batch_normalize(&fixed_commitments_projective, &mut fixed_commitments);
        fixed_commitments
    };

    Ok(VerifyingKey::from_parts(
        domain,
        fixed_commitments,
        permutation_vk,
        cs,
    ))
}

/// Generate a `ProvingKey` from a `VerifyingKey` and an instance of `CompiledCircuit`.
pub fn keygen_pk<C, P>(
    params: &P,
    vk: VerifyingKey<C>,
    circuit: &CompiledCircuit<C::Scalar>,
) -> Result<ProvingKey<C>, Error>
where
    C: CurveAffine,
    P: Params<C>,
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

    // Compute L_0(X) in the extended co-domain.
    // L_0(X) the 0th Lagrange polynomial in the original domain.
    // Its representation in the original domain H = {1, g, g^2, ..., g^(n-1)}
    // is [1, 0, ..., 0].
    // We compute its represenation in the extended co-domain
    // zH = {z, z*w, z*w^2, ... , z*w^(n*k - 1)}, where k is the extension factor
    // of the domain, and z is the extended root such that w^k = g.
    // We assume z = F::ZETA, a cubic root the field. This simplifies the computation.
    //
    // The computation uses the fomula:
    // L_i(X) = g^i/n * (X^n -1)/(X-g^i)
    // L_0(X) = 1/n * (X^n -1)/(X-1)
    let l0 = {
        let one = C::ScalarExt::ONE;
        let zeta = <C::ScalarExt as WithSmallOrderMulGroup<3>>::ZETA;

        let n: u64 = 1 << vk.domain.k();
        let c = (C::ScalarExt::from(n)).invert().unwrap();
        let mut l0 = vec![C::ScalarExt::ZERO; vk.domain.extended_len()];

        let w = vk.domain.get_extended_omega();
        let wn = w.pow_vartime(&[n]);
        let zeta_n = match n % 3 {
            1 => zeta,
            2 => zeta * zeta,
            _ => one,
        };

        // Compute denominators.
        let mut acc = zeta;
        l0.iter_mut().for_each(|e| {
            *e = acc - one;
            acc *= w;
        });
        l0.batch_invert();

        // Compute numinators.
        //  C * (zeta * w^i)^n = (C * zeta^n) * w^(i*n)
        // We use w^k = g and g^n = 1 to save multiplications.
        let k = 1 << (vk.domain.extended_k() - vk.domain.k());
        let mut wn_powers = vec![zeta_n * c; k];
        for i in 1..k {
            wn_powers[i] = wn_powers[i - 1] * wn
        }

        parallelize(&mut l0, |e, mut index| {
            for e in e {
                *e *= wn_powers[index % k] - c;
                index += 1;
            }
        });

        Polynomial {
            values: l0,
            _marker: std::marker::PhantomData,
        }
    };

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
    // TODO L_0 method could be used here too.
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
pub(crate) struct Queries {
    /// List of unique advice queries
    pub(crate) advice: Vec<(ColumnMid, Rotation)>,
    /// List of unique instance queries
    pub(crate) instance: Vec<(ColumnMid, Rotation)>,
    /// List of unique fixed queries
    pub(crate) fixed: Vec<(ColumnMid, Rotation)>,
    /// Contains an integer for each advice column
    /// identifying how many distinct queries it has
    /// so far; should be same length as cs.num_advice_columns.
    pub(crate) num_advice_queries: Vec<usize>,
}
