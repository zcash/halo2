use ff::{Field, PrimeField};
use group::Curve;

use super::{Argument, ProvingKey, VerifyingKey};
use crate::{
    arithmetic::{parallelize, CurveAffine},
    plonk::{Any, Column, Error},
    poly::{
        commitment::{Blind, CommitmentScheme, Params},
        EvaluationDomain,
    },
};

/// Struct that accumulates all the necessary data in order to construct the permutation argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assembly {
    /// Columns that participate on the copy permutation argument.
    columns: Vec<Column<Any>>,
    /// Mapping of the actual copies done.
    mapping: Vec<Vec<(usize, usize)>>,
    /// Some aux data used to swap positions directly when sorting.
    aux: Vec<Vec<(usize, usize)>>,
    /// More aux data
    sizes: Vec<Vec<usize>>,
}

impl Assembly {
    pub(crate) fn new(n: usize, p: &Argument) -> Self {
        // Initialize the copy vector to keep track of copy constraints in all
        // the permutation arguments.
        let mut columns = vec![];
        for i in 0..p.columns.len() {
            // Computes [(i, 0), (i, 1), ..., (i, n - 1)]
            columns.push((0..n).map(|j| (i, j)).collect());
        }

        // Before any equality constraints are applied, every cell in the permutation is
        // in a 1-cycle; therefore mapping and aux are identical, because every cell is
        // its own distinguished element.
        Assembly {
            columns: p.columns.clone(),
            mapping: columns.clone(),
            aux: columns,
            sizes: vec![vec![1usize; n]; p.columns.len()],
        }
    }

    pub(crate) fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error> {
        let left_column = self
            .columns
            .iter()
            .position(|c| c == &left_column)
            .ok_or(Error::ColumnNotInPermutation(left_column))?;
        let right_column = self
            .columns
            .iter()
            .position(|c| c == &right_column)
            .ok_or(Error::ColumnNotInPermutation(right_column))?;

        // Check bounds
        if left_row >= self.mapping[left_column].len()
            || right_row >= self.mapping[right_column].len()
        {
            return Err(Error::BoundsFailure);
        }

        // See book/src/design/permutation.md for a description of this algorithm.

        let mut left_cycle = self.aux[left_column][left_row];
        let mut right_cycle = self.aux[right_column][right_row];

        // If left and right are in the same cycle, do nothing.
        if left_cycle == right_cycle {
            return Ok(());
        }

        if self.sizes[left_cycle.0][left_cycle.1] < self.sizes[right_cycle.0][right_cycle.1] {
            std::mem::swap(&mut left_cycle, &mut right_cycle);
        }

        // Merge the right cycle into the left one.
        self.sizes[left_cycle.0][left_cycle.1] += self.sizes[right_cycle.0][right_cycle.1];
        let mut i = right_cycle;
        loop {
            self.aux[i.0][i.1] = left_cycle;
            i = self.mapping[i.0][i.1];
            if i == right_cycle {
                break;
            }
        }

        let tmp = self.mapping[left_column][left_row];
        self.mapping[left_column][left_row] = self.mapping[right_column][right_row];
        self.mapping[right_column][right_row] = tmp;

        Ok(())
    }

    pub(crate) fn build_vk<'params, C: CurveAffine, P: Params<'params, C>>(
        self,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        p: &Argument,
    ) -> VerifyingKey<C> {
        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = vec![C::Scalar::ZERO; params.n() as usize];
        {
            let omega = domain.get_omega();
            parallelize(&mut omega_powers, |o, start| {
                let mut cur = omega.pow_vartime(&[start as u64]);
                for v in o.iter_mut() {
                    *v = cur;
                    cur *= &omega;
                }
            })
        }

        // Compute [omega_powers * \delta^0, omega_powers * \delta^1, ..., omega_powers * \delta^m]
        let mut deltaomega = vec![omega_powers; p.columns.len()];
        {
            parallelize(&mut deltaomega, |o, start| {
                let mut cur = C::Scalar::DELTA.pow_vartime(&[start as u64]);
                for omega_powers in o.iter_mut() {
                    for v in omega_powers {
                        *v *= &cur;
                    }
                    cur *= &<C::Scalar as PrimeField>::DELTA;
                }
            });
        }

        // Computes the permutation polynomial based on the permutation
        // description in the assembly.
        let mut permutations = vec![domain.empty_lagrange(); p.columns.len()];
        {
            parallelize(&mut permutations, |o, start| {
                for (x, permutation_poly) in o.iter_mut().enumerate() {
                    let i = start + x;
                    for (j, p) in permutation_poly.iter_mut().enumerate() {
                        let (permuted_i, permuted_j) = self.mapping[i][j];
                        *p = deltaomega[permuted_i][permuted_j];
                    }
                }
            });
        }

        // Pre-compute commitments for the URS.
        let mut commitments = Vec::with_capacity(p.columns.len());
        for permutation in &permutations {
            // Compute commitment to permutation polynomial
            commitments.push(
                params
                    .commit_lagrange(permutation, Blind::default())
                    .to_affine(),
            );
        }

        VerifyingKey { commitments }
    }

    pub(crate) fn build_pk<'params, C: CurveAffine, P: Params<'params, C>>(
        self,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        p: &Argument,
    ) -> ProvingKey<C> {
        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = vec![C::Scalar::ZERO; params.n() as usize];
        {
            let omega = domain.get_omega();
            parallelize(&mut omega_powers, |o, start| {
                let mut cur = omega.pow_vartime(&[start as u64]);
                for v in o.iter_mut() {
                    *v = cur;
                    cur *= &omega;
                }
            })
        }

        // Compute [omega_powers * \delta^0, omega_powers * \delta^1, ..., omega_powers * \delta^m]
        let mut deltaomega = vec![omega_powers; p.columns.len()];
        {
            parallelize(&mut deltaomega, |o, start| {
                let mut cur = C::Scalar::DELTA.pow_vartime(&[start as u64]);
                for omega_powers in o.iter_mut() {
                    for v in omega_powers {
                        *v *= &cur;
                    }
                    cur *= &C::Scalar::DELTA;
                }
            });
        }

        // Compute permutation polynomials, convert to coset form.
        let mut permutations = vec![domain.empty_lagrange(); p.columns.len()];
        {
            parallelize(&mut permutations, |o, start| {
                for (x, permutation_poly) in o.iter_mut().enumerate() {
                    let i = start + x;
                    for (j, p) in permutation_poly.iter_mut().enumerate() {
                        let (permuted_i, permuted_j) = self.mapping[i][j];
                        *p = deltaomega[permuted_i][permuted_j];
                    }
                }
            });
        }

        let mut polys = vec![domain.empty_coeff(); p.columns.len()];
        {
            parallelize(&mut polys, |o, start| {
                for (x, poly) in o.iter_mut().enumerate() {
                    let i = start + x;
                    let permutation_poly = permutations[i].clone();
                    *poly = domain.lagrange_to_coeff(permutation_poly);
                }
            });
        }

        let mut cosets = vec![domain.empty_extended(); p.columns.len()];
        {
            parallelize(&mut cosets, |o, start| {
                for (x, coset) in o.iter_mut().enumerate() {
                    let i = start + x;
                    let poly = polys[i].clone();
                    *coset = domain.coeff_to_extended(poly);
                }
            });
        }

        ProvingKey {
            permutations,
            polys,
            cosets,
        }
    }

    /// Returns columns that participate on the permutation argument.
    pub fn columns(&self) -> &[Column<Any>] {
        &self.columns
    }

    /// Returns mappings of the copies.
    pub fn mapping(&self) -> &[Vec<(usize, usize)>] {
        &self.mapping
    }
}
