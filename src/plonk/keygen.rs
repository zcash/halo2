use ff::Field;

use super::{
    circuit::{Advice, Assignment, Circuit, Column, ConstraintSystem, Fixed},
    permutation, Error, ProvingKey, VerifyingKey,
};
use crate::arithmetic::{Curve, CurveAffine};
use crate::poly::{
    commitment::{Blind, Params},
    EvaluationDomain, LagrangeCoeff, Polynomial, Rotation,
};

/// Generate a `ProvingKey` from an instance of `Circuit`.
pub fn keygen<C, ConcreteCircuit>(
    params: &Params<C>,
    circuit: &ConcreteCircuit,
) -> Result<ProvingKey<C>, Error>
where
    C: CurveAffine,
    ConcreteCircuit: Circuit<C::Scalar>,
{
    struct Assembly<F: Field> {
        fixed: Vec<Polynomial<F, LagrangeCoeff>>,
        permutations: Vec<permutation::keygen::Assembly>,
        _marker: std::marker::PhantomData<F>,
    }

    impl<F: Field> Assignment<F> for Assembly<F> {
        fn assign_advice(
            &mut self,
            _: Column<Advice>,
            _: usize,
            _: impl FnOnce() -> Result<F, Error>,
        ) -> Result<(), Error> {
            // We only care about fixed columns here
            Ok(())
        }

        fn assign_fixed(
            &mut self,
            column: Column<Fixed>,
            row: usize,
            to: impl FnOnce() -> Result<F, Error>,
        ) -> Result<(), Error> {
            *self
                .fixed
                .get_mut(column.index())
                .and_then(|v| v.get_mut(row))
                .ok_or(Error::BoundsFailure)? = to()?;

            Ok(())
        }

        fn copy(
            &mut self,
            permutation: usize,
            left_column: usize,
            left_row: usize,
            right_column: usize,
            right_row: usize,
        ) -> Result<(), Error> {
            // Check bounds first
            if permutation >= self.permutations.len() {
                return Err(Error::BoundsFailure);
            }

            self.permutations[permutation].copy(left_column, left_row, right_column, right_row)
        }
    }

    let mut cs = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut cs);

    // The permutation argument will serve alongside the gates, so must be
    // accounted for.
    let mut degree = cs
        .permutations
        .iter()
        .map(|p| p.required_degree())
        .max()
        .unwrap_or(1);

    // The lookup argument also serves alongside the gates and must be accounted
    // for.
    degree = std::cmp::max(
        degree,
        cs.lookups
            .iter()
            .map(|l| l.required_degree())
            .max()
            .unwrap_or(1),
    );

    // Account for each gate to ensure our quotient polynomial is the
    // correct degree and that our extended domain is the right size.
    for poly in cs.gates.iter() {
        degree = std::cmp::max(degree, poly.degree());
    }

    let domain = EvaluationDomain::new(degree as u32, params.k);

    let mut assembly: Assembly<C::Scalar> = Assembly {
        fixed: vec![domain.empty_lagrange(); cs.num_fixed_columns],
        permutations: cs
            .permutations
            .iter()
            .map(|p| permutation::keygen::Assembly::new(params.n as usize, p))
            .collect(),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain SRS
    circuit.synthesize(&mut assembly, config)?;

    let permutation_helper = permutation::keygen::Assembly::build_helper(params, &cs, &domain);

    let (permutation_pks, permutation_vks) = cs
        .permutations
        .iter()
        .zip(assembly.permutations.into_iter())
        .map(|(p, assembly)| assembly.build_keys(params, &domain, &permutation_helper, p))
        .unzip();

    let fixed_commitments = assembly
        .fixed
        .iter()
        .map(|poly| params.commit_lagrange(poly, Blind::default()).to_affine())
        .collect();

    let fixed_polys: Vec<_> = assembly
        .fixed
        .iter()
        .map(|poly| domain.lagrange_to_coeff(poly.clone()))
        .collect();

    let fixed_cosets = cs
        .fixed_queries
        .iter()
        .map(|&(column, at)| {
            let poly = fixed_polys[column.index()].clone();
            domain.coeff_to_extended(poly, at)
        })
        .collect();

    // Compute l_0(X)
    // TODO: this can be done more efficiently
    let mut l0 = domain.empty_lagrange();
    l0[0] = C::Scalar::one();
    let l0 = domain.lagrange_to_coeff(l0);
    let l0 = domain.coeff_to_extended(l0, Rotation::default());

    Ok(ProvingKey {
        vk: VerifyingKey {
            domain,
            fixed_commitments,
            permutations: permutation_vks,
            cs,
        },
        l0,
        fixed_values: assembly.fixed,
        fixed_polys,
        fixed_cosets,
        permutations: permutation_pks,
    })
}
