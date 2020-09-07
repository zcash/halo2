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
            mapping: Vec<Vec<Vec<(usize, usize)>>>,
            aux: Vec<Vec<Vec<(usize, usize)>>>,
            sizes: Vec<Vec<Vec<usize>>>,
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
                // Check bounds first
                if permutation >= self.mapping.len()
                    || left_wire >= self.mapping[permutation].len()
                    || left_row >= self.mapping[permutation][left_wire].len()
                    || right_wire >= self.mapping[permutation].len()
                    || right_row >= self.mapping[permutation][right_wire].len()
                {
                    return Err(Error::BoundsFailure);
                }

                let mut left_cycle = self.aux[permutation][left_wire][left_row];
                let mut right_cycle = self.aux[permutation][right_wire][right_row];

                if left_cycle == right_cycle {
                    return Ok(());
                }

                if self.sizes[permutation][left_cycle.0][left_cycle.1]
                    < self.sizes[permutation][right_cycle.0][right_cycle.1]
                {
                    std::mem::swap(&mut left_cycle, &mut right_cycle);
                }

                self.sizes[permutation][left_cycle.0][left_cycle.1] +=
                    self.sizes[permutation][right_cycle.0][right_cycle.1];
                let mut i = right_cycle;
                loop {
                    self.aux[permutation][i.0][i.1] = left_cycle;
                    i = self.mapping[permutation][i.0][i.1];
                    if i == right_cycle {
                        break;
                    }
                }

                let tmp = self.mapping[permutation][left_wire][left_row];
                self.mapping[permutation][left_wire][left_row] =
                    self.mapping[permutation][right_wire][right_row];
                self.mapping[permutation][right_wire][right_row] = tmp;

                Ok(())
            }
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

        // Get the largest permutation argument length in terms of the number of
        // advice wires involved.
        let mut largest_permutation_length = 0;
        for permutation in &meta.permutations {
            largest_permutation_length =
                std::cmp::max(permutation.len(), largest_permutation_length);
        }

        // The permutation argument will serve alongside the gates, so must be
        // accounted for.
        let mut degree = largest_permutation_length + 1;

        // Account for each gate to ensure our quotient polynomial is the
        // correct degree and that our extended domain is the right size.
        for poly in meta.gates.iter() {
            degree = std::cmp::max(degree, poly.degree());
        }

        let domain = EvaluationDomain::new(degree as u32, params.k);

        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = Vec::with_capacity(params.n as usize);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..params.n {
                omega_powers.push(cur);
                cur *= &domain.get_omega();
            }
        }

        // Compute [omega_powers * \delta^0, omega_powers * \delta^1, ..., omega_powers * \delta^m]
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
            mapping: vec![],
            aux: vec![],
            sizes: vec![],
        };

        // Initialize the copy vector to keep track of copy constraints in all
        // the permutation arguments.
        for permutation in &meta.permutations {
            let mut wires = vec![];
            for i in 0..permutation.len() {
                // Computes [(i, 0), (i, 1), ..., (i, n - 1)]
                wires.push((0..params.n).map(|j| (i, j as usize)).collect());
            }
            assembly.mapping.push(wires.clone());
            assembly.aux.push(wires);
            assembly
                .sizes
                .push(vec![vec![1usize; params.n as usize]; permutation.len()]);
        }

        // Synthesize the circuit to obtain SRS
        circuit.synthesize(&mut assembly, config)?;

        // Compute permutation polynomials, convert to coset form and
        // pre-compute commitments for the SRS.
        let mut permutation_commitments = vec![];
        let mut permutations = vec![];
        let mut permutation_polys = vec![];
        let mut permutation_cosets = vec![];
        for (permutation_index, permutation) in meta.permutations.iter().enumerate() {
            let mut commitments = vec![];
            let mut inner_permutations = vec![];
            let mut polys = vec![];
            let mut cosets = vec![];
            for i in 0..permutation.len() {
                // Computes the permutation polynomial based on the permutation
                // description in the assembly.
                let permutation_poly: Vec<_> = (0..params.n as usize)
                    .map(|j| {
                        // assembly.copy[permutation_index] is indexed by wire
                        // i, and then indexed by row j, obtaining the index of
                        // the permuted value in deltaomega.
                        let (permuted_i, permuted_j) = assembly.mapping[permutation_index][i][j];
                        deltaomega[permuted_i][permuted_j]
                    })
                    .collect();

                // Compute commitment to permutation polynomial
                commitments.push(
                    params
                        .commit_lagrange(&permutation_poly, C::Scalar::one())
                        .to_affine(),
                );
                // Store permutation polynomial and precompute its coset evaluation
                inner_permutations.push(permutation_poly.clone());
                let poly = domain.obtain_poly(permutation_poly);
                polys.push(poly.clone());
                cosets.push(domain.obtain_coset(poly, Rotation::default()));
            }
            permutation_commitments.push(commitments);
            permutations.push(inner_permutations);
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

        // Compute l_0(X)
        // TODO: this can be done more efficiently
        let mut l0 = vec![C::Scalar::zero(); params.n as usize];
        l0[0] = C::Scalar::one();
        let l0 = domain.obtain_poly(l0);
        let l0 = domain.obtain_coset(l0, Rotation::default());

        Ok(SRS {
            domain,
            l0,
            fixed_commitments,
            fixed_polys,
            fixed_cosets,
            permutation_commitments,
            permutations,
            permutation_polys,
            permutation_cosets,
            meta,
        })
    }
}
