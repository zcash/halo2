#![allow(clippy::int_plus_one)]

use ff::Field;
use group::Curve;

use super::{
    circuit::{
        Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Fixed, FloorPlanner, Selector,
    },
    permutation, Assigned, Error, LagrangeCoeff, Polynomial, ProvingKey, VerifyingKey,
};
use crate::poly::{
    commitment::{Blind, Params},
    EvaluationDomain, Rotation,
};
use crate::{arithmetic::CurveAffine, poly::batch_invert_assigned};

pub(crate) fn create_domain<C, ConcreteCircuit>(
    params: &Params<C>,
) -> (
    EvaluationDomain<C::Scalar>,
    ConstraintSystem<C::Scalar>,
    ConcreteCircuit::Config,
)
where
    C: CurveAffine,
    ConcreteCircuit: Circuit<C::Scalar>,
{
    let mut cs = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut cs);

    let cs = cs;
    // There needs to be enough room for at least one row.
    assert!(
        cs.blinding_factors() // m blinding factors
        + 1 // for l_{-(m + 1)}
        + 1 // for l_0
        + 1 // for at least one row
        <= (params.n as usize)
    );

    let degree = cs.degree();

    let domain = EvaluationDomain::new(degree as u32, params.k);

    (domain, cs, config)
}

/// Assembly to be used in circuit synthesis.
#[derive(Debug)]
struct Assembly<F: Field> {
    fixed: Vec<Polynomial<Assigned<F>, LagrangeCoeff>>,
    permutation: permutation::keygen::Assembly,
    // All rows including and above this one are off
    // limits due to blinding factors.
    upper_bound_cell_index: usize,
    _marker: std::marker::PhantomData<F>,
}

impl<F: Field> Assignment<F> for Assembly<F> {
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
        if row >= self.upper_bound_cell_index {
            return Err(Error::BoundsFailure);
        }
        // Selectors are just fixed columns.
        // TODO: Ensure that the default for a selector's cells is always zero, if we
        // alter the proving system to change the global default.
        // TODO: Implement selector combining optimization
        // https://github.com/zcash/halo2/issues/116
        self.assign_fixed(annotation, selector.0, row, || Ok(F::one()))
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
        // We only care about fixed columns here
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
        if row >= self.upper_bound_cell_index {
            return Err(Error::BoundsFailure);
        }

        *self
            .fixed
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = to()?.into();

        Ok(())
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error> {
        // Check bounds first
        if left_row >= self.upper_bound_cell_index || right_row >= self.upper_bound_cell_index {
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
        // Do nothing; we don't care about namespaces in this context.
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // Do nothing; we don't care about namespaces in this context.
    }
}

/// Generate a `VerifyingKey` from an instance of `Circuit`.
pub fn keygen_vk<C, ConcreteCircuit>(
    params: &Params<C>,
    circuit: &ConcreteCircuit,
) -> Result<VerifyingKey<C>, Error>
where
    C: CurveAffine,
    ConcreteCircuit: Circuit<C::Scalar>,
{
    let (domain, cs, config) = create_domain::<C, ConcreteCircuit>(params);

    let mut assembly: Assembly<C::Scalar> = Assembly {
        fixed: vec![domain.empty_lagrange_assigned(); cs.num_fixed_columns],
        permutation: permutation::keygen::Assembly::new(params.n as usize, &cs.permutation),
        upper_bound_cell_index: params.n as usize - (cs.blinding_factors() + 1),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    ConcreteCircuit::FloorPlanner::synthesize(&mut assembly, circuit, config)?;

    let fixed = batch_invert_assigned(assembly.fixed);

    let permutation_vk = assembly
        .permutation
        .build_vk(params, &domain, &cs.permutation);

    let fixed_commitments = fixed
        .iter()
        .map(|poly| params.commit_lagrange(poly, Blind::default()).to_affine())
        .collect();

    Ok(VerifyingKey {
        domain,
        fixed_commitments,
        permutation: permutation_vk,
        cs,
    })
}

/// Generate a `ProvingKey` from a `VerifyingKey` and an instance of `Circuit`.
pub fn keygen_pk<C, ConcreteCircuit>(
    params: &Params<C>,
    vk: VerifyingKey<C>,
    circuit: &ConcreteCircuit,
) -> Result<ProvingKey<C>, Error>
where
    C: CurveAffine,
    ConcreteCircuit: Circuit<C::Scalar>,
{
    let mut cs = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut cs);

    let cs = cs;
    // There needs to be enough room for at least one row.
    assert!(
        cs.blinding_factors() // m blinding factors
        + 1 // for l_{-(m + 1)}
        + 1 // for l_0
        + 1 // for at least one row
        <= (params.n as usize)
    );

    let mut assembly: Assembly<C::Scalar> = Assembly {
        fixed: vec![vk.domain.empty_lagrange_assigned(); vk.cs.num_fixed_columns],
        permutation: permutation::keygen::Assembly::new(params.n as usize, &vk.cs.permutation),
        upper_bound_cell_index: params.n as usize - (vk.cs.blinding_factors() + 1),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    ConcreteCircuit::FloorPlanner::synthesize(&mut assembly, circuit, config)?;

    let fixed = batch_invert_assigned(assembly.fixed);

    let fixed_polys: Vec<_> = fixed
        .iter()
        .map(|poly| vk.domain.lagrange_to_coeff(poly.clone()))
        .collect();

    let fixed_cosets = vk
        .cs
        .fixed_queries
        .iter()
        .map(|&(column, at)| {
            let poly = fixed_polys[column.index()].clone();
            vk.domain.coeff_to_extended(poly, at)
        })
        .collect();

    let permutation_pk = assembly
        .permutation
        .build_pk(params, &vk.domain, &vk.cs.permutation);

    // Compute l_0(X)
    // TODO: this can be done more efficiently
    let mut l0 = vk.domain.empty_lagrange();
    l0[0] = C::Scalar::one();
    let l0 = vk.domain.lagrange_to_coeff(l0);
    let l0 = vk.domain.coeff_to_extended(l0, Rotation::cur());

    // Compute l_cover(X) which evaluates to 1 for each blinding factor row
    // and 0 otherwise over the domain.
    let mut l_cover = vk.domain.empty_lagrange();
    for evaluation in l_cover[..].iter_mut().rev().take(cs.blinding_factors()) {
        *evaluation = C::Scalar::one();
    }
    let l_cover = vk.domain.lagrange_to_coeff(l_cover);
    let l_cover = vk.domain.coeff_to_extended(l_cover, Rotation::cur());

    // Compute l_last(X) which evaluates to 1 on the first inactive row (just
    // before the blinding factors) and 0 otherwise over the domain
    let mut l_last = vk.domain.empty_lagrange();
    *(l_last[..]
        .iter_mut()
        .rev()
        .nth(cs.blinding_factors())
        .unwrap()) = C::Scalar::one();
    let l_last = vk.domain.lagrange_to_coeff(l_last);
    let l_last = vk.domain.coeff_to_extended(l_last, Rotation::cur());

    Ok(ProvingKey {
        vk,
        l0,
        l_cover,
        l_last,
        fixed_values: fixed,
        fixed_polys,
        fixed_cosets,
        permutation: permutation_pk,
    })
}
