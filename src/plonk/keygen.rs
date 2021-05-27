use ff::Field;
use group::Curve;

use super::{
    circuit::{Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Fixed, Selector},
    permutation, Error, LagrangeCoeff, Permutation, Polynomial, ProvingKey, VerifyingKey,
};
use crate::arithmetic::CurveAffine;
use crate::poly::{
    commitment::{Blind, Params},
    EvaluationDomain, Rotation,
};

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

    let degree = cs.degree();

    let domain = EvaluationDomain::new(degree as u32, params.k);

    (domain, cs, config)
}

/// Assembly to be used in circuit synthesis.
#[derive(Debug)]
struct Assembly<F: Field> {
    fixed: Vec<Polynomial<F, LagrangeCoeff>>,
    permutations: Vec<permutation::keygen::Assembly>,
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
        // Selectors are just fixed columns.
        // TODO: Ensure that the default for a selector's cells is always zero, if we
        // alter the proving system to change the global default.
        // TODO: Implement selector combining optimization
        // https://github.com/zcash/halo2/issues/116
        self.assign_fixed(annotation, selector.0, row, || Ok(F::one()))
    }

    fn assign_advice<V, A, AR>(
        &mut self,
        _: A,
        _: Column<Advice>,
        _: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about fixed columns here
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
    ) -> Result<(), Error> {
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
        fixed: vec![domain.empty_lagrange(); cs.num_fixed_columns],
        permutations: cs
            .permutations
            .iter()
            .map(|p| permutation::keygen::Assembly::new(params.n as usize, p))
            .collect(),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    circuit.synthesize(&mut assembly, config)?;

    let permutation_helper = permutation::keygen::Assembly::build_helper(params, &cs, &domain);

    let permutation_vks = cs
        .permutations
        .iter()
        .zip(assembly.permutations.into_iter())
        .map(|(p, assembly)| assembly.build_vk(params, &domain, &permutation_helper, p))
        .collect();

    let fixed_commitments = assembly
        .fixed
        .iter()
        .map(|poly| params.commit_lagrange(poly, Blind::default()).to_affine())
        .collect();

    Ok(VerifyingKey {
        domain,
        fixed_commitments,
        permutations: permutation_vks,
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

    let mut assembly: Assembly<C::Scalar> = Assembly {
        fixed: vec![vk.domain.empty_lagrange(); vk.cs.num_fixed_columns],
        permutations: vk
            .cs
            .permutations
            .iter()
            .map(|p| permutation::keygen::Assembly::new(params.n as usize, p))
            .collect(),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    circuit.synthesize(&mut assembly, config)?;

    let fixed_polys: Vec<_> = assembly
        .fixed
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

    let permutation_helper =
        permutation::keygen::Assembly::build_helper(params, &vk.cs, &vk.domain);

    let permutation_pks = vk
        .cs
        .permutations
        .iter()
        .zip(assembly.permutations.into_iter())
        .map(|(p, assembly)| assembly.build_pk(&vk.domain, &permutation_helper, p))
        .collect();

    // Compute l_0(X)
    // TODO: this can be done more efficiently
    let mut l0 = vk.domain.empty_lagrange();
    l0[0] = C::Scalar::one();
    let l0 = vk.domain.lagrange_to_coeff(l0);
    let l0 = vk.domain.coeff_to_extended(l0, Rotation::cur());

    Ok(ProvingKey {
        vk,
        l0,
        fixed_values: assembly.fixed,
        fixed_polys,
        fixed_cosets,
        permutations: permutation_pks,
    })
}
