#![allow(clippy::int_plus_one)]

use ff::{Field, FromUniformBytes};
use group::Curve;

use super::{
    circuit::ConstraintSystem, evaluation::Evaluator, permutation, Error, Polynomial, ProvingKey,
    VerifyingKey,
};
use crate::{
    arithmetic::{parallelize, CurveAffine},
    poly::{
        commitment::{Blind, Params},
        EvaluationDomain,
    },
};
use halo2_middleware::circuit::CompiledCircuitV2;

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
    let cs2 = &circuit.cs;
    let cs: ConstraintSystem<C::Scalar> = cs2.clone().into();
    let domain = EvaluationDomain::new(cs.degree() as u32, params.k());

    if (params.n() as usize) < cs.minimum_rows() {
        return Err(Error::not_enough_rows_available(params.k()));
    }

    let permutation_vk = permutation::keygen::Assembly::new_from_assembly_mid(
        params.n() as usize,
        &cs2.permutation,
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
        Vec::new(),
        false,
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

    let permutation_pk = permutation::keygen::Assembly::new_from_assembly_mid(
        params.n() as usize,
        &cs.permutation,
        &circuit.preprocessing.permutation,
    )?
    .build_pk(params, &vk.domain, &cs.permutation.clone().into());

    // Compute l_0(X)
    // TODO: this can be done more efficiently
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

    Ok(ProvingKey {
        vk,
        l0,
        l_last,
        l_active_row,
        fixed_values: circuit
            .preprocessing
            .fixed
            .clone()
            .into_iter()
            .map(Polynomial::new_lagrange_from_vec)
            .collect(),
        fixed_polys,
        fixed_cosets,
        permutation: permutation_pk,
        ev,
    })
}
