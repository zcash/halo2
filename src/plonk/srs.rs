use super::{
    circuit::{AdviceWire, Circuit, ConstraintSystem, FixedWire, MetaCircuit},
    domain::{EvaluationDomain, Rotation},
    Error, SRS,
};
use crate::arithmetic::{Curve, CurveAffine, Field};
use crate::polycommit::Params;

impl<C: CurveAffine> SRS<C> {
    /// This generates a structured reference string for the provided `circuit`
    /// and `params`.
    pub fn generate<ConcreteCircuit: Circuit<C::Scalar>>(
        params: &Params<C>,
        circuit: &ConcreteCircuit,
    ) -> Result<Self, Error> {
        struct Assembly<F: Field> {
            fixed: Vec<Vec<F>>,
            copy: Vec<Vec<Vec<(usize, usize)>>>,
        }

        impl<F: Field> ConstraintSystem<F> for Assembly<F> {
            fn assign_advice(
                &mut self,
                _: AdviceWire,
                _: usize,
                _: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                // We only care about fixed wires here
                Ok(())
            }

            fn assign_fixed(
                &mut self,
                wire: FixedWire,
                row: usize,
                to: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                *self
                    .fixed
                    .get_mut(wire.0)
                    .and_then(|v| v.get_mut(row))
                    .ok_or(Error::BoundsFailure)? = to()?;

                Ok(())
            }

            fn copy(
                &mut self,
                permutation: usize,
                left_wire: usize,
                left_row: usize,
                right_wire: usize,
                right_row: usize,
            ) -> Result<(), Error> {
                let left: (usize, usize) = *self.copy[permutation]
                    .get_mut(left_wire)
                    .and_then(|wire| wire.get_mut(left_row))
                    .ok_or(Error::BoundsFailure)?;

                let right: (usize, usize) = *self.copy[permutation]
                    .get_mut(right_wire)
                    .and_then(|wire| wire.get_mut(right_row))
                    .ok_or(Error::BoundsFailure)?;

                if left == (left_wire, left_row) || right == (right_wire, right_row) {
                    // Don't perform the copy constraint because it will undo
                    // the effect of the permutation.
                } else {
                    *self.copy[permutation]
                        .get_mut(left_wire)
                        .and_then(|wire| wire.get_mut(left_row))
                        .ok_or(Error::BoundsFailure)? = right;

                    *self.copy[permutation]
                        .get_mut(right_wire)
                        .and_then(|wire| wire.get_mut(right_row))
                        .ok_or(Error::BoundsFailure)? = left;
                }

                Ok(())
            }
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

        let mut degree = 1;
        for poly in meta.gates.iter() {
            degree = std::cmp::max(degree, poly.degree());
        }
        for permutation in &meta.permutations {
            degree = std::cmp::max(degree, permutation.len() + 1);
        }

        let domain = EvaluationDomain::new(degree as u32, params.k);

        let mut largest_permutation_length = 0;
        for permutation in &meta.permutations {
            largest_permutation_length =
                std::cmp::max(permutation.len(), largest_permutation_length);
        }

        let mut omega_powers = Vec::with_capacity(params.n as usize);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..params.n {
                omega_powers.push(cur);
                cur *= &domain.get_omega();
            }
        }

        let mut deltaomega = Vec::with_capacity(largest_permutation_length);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..largest_permutation_length {
                let mut omega_powers = omega_powers.clone();
                for o in &mut omega_powers {
                    *o *= &cur;
                }

                deltaomega.push(omega_powers);

                cur *= &C::Scalar::DELTA;
            }
        }

        let mut assembly: Assembly<C::Scalar> = Assembly {
            fixed: vec![vec![C::Scalar::zero(); params.n as usize]; meta.num_fixed_wires],
            copy: vec![],
        };

        for permutation in &meta.permutations {
            let mut wires = vec![];
            for (i, _) in permutation.iter().enumerate() {
                wires.push((0..params.n).map(|j| (i, j as usize)).collect());
            }
            assembly.copy.push(wires);
        }

        // Synthesize the circuit to obtain SRS
        circuit.synthesize(&mut assembly, config)?;

        // Compute permutation polynomials
        let mut permutation_commitments = vec![];
        let mut permutation_polys = vec![];
        let mut permutation_cosets = vec![];
        for (permutation_index, permutation) in meta.permutations.iter().enumerate() {
            let mut commitments = vec![];
            let mut polys = vec![];
            let mut cosets = vec![];
            for (i, _) in permutation.iter().enumerate() {
                let permutation_poly: Vec<_> = (0..params.n as usize)
                    .map(|j| {
                        let (permuted_i, permuted_j) = assembly.copy[permutation_index][i][j];
                        deltaomega[permuted_i][permuted_j]
                    })
                    .collect();
                commitments.push(
                    params
                        .commit_lagrange(&permutation_poly, C::Scalar::one())
                        .to_affine(),
                );
                polys.push(permutation_poly.clone());
                cosets.push(domain.obtain_coset(permutation_poly, Rotation::default()));
            }
            permutation_commitments.push(commitments);
            permutation_polys.push(polys);
            permutation_cosets.push(cosets);
        }

        let fixed_commitments = assembly
            .fixed
            .iter()
            .map(|poly| params.commit_lagrange(poly, C::Scalar::one()).to_affine())
            .collect();

        let fixed_polys: Vec<_> = assembly
            .fixed
            .into_iter()
            .map(|poly| domain.obtain_poly(poly))
            .collect();

        let fixed_cosets = meta
            .fixed_queries
            .iter()
            .map(|&(wire, at)| {
                let poly = fixed_polys[wire.0].clone();
                domain.obtain_coset(poly, at)
            })
            .collect();

        Ok(SRS {
            domain,
            fixed_commitments,
            fixed_polys,
            fixed_cosets,
            permutation_commitments,
            permutation_polys,
            permutation_cosets,
            meta,
        })
    }
}
