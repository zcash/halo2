use ff::{Field, PrimeField};
use group::Curve;

use super::{Argument, ProvingKey, VerifyingKey};
use crate::{
    arithmetic::{parallelize, CurveAffine},
    plonk::{Any, Column, Error},
    poly::{
        commitment::{Blind, Params},
        EvaluationDomain,
    },
};

#[cfg(feature = "multicore")]
use crate::multicore::{IndexedParallelIterator, ParallelIterator};

#[cfg(feature = "thread-safe-region")]
use std::collections::{BTreeSet, HashMap};

#[cfg(not(feature = "thread-safe-region"))]
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

#[cfg(not(feature = "thread-safe-region"))]
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
        build_vk(params, domain, p, |i, j| self.mapping[i][j])
    }

    pub(crate) fn build_pk<'params, C: CurveAffine, P: Params<'params, C>>(
        self,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        p: &Argument,
    ) -> ProvingKey<C> {
        build_pk(params, domain, p, |i, j| self.mapping[i][j])
    }

    /// Returns columns that participate in the permutation argument.
    pub fn columns(&self) -> &[Column<Any>] {
        &self.columns
    }

    #[cfg(feature = "multicore")]
    /// Returns mappings of the copies.
    pub fn mapping(
        &self,
    ) -> impl Iterator<Item = impl IndexedParallelIterator<Item = (usize, usize)> + '_> {
        use crate::multicore::IntoParallelRefIterator;

        self.mapping.iter().map(|c| c.par_iter().copied())
    }

    #[cfg(not(feature = "multicore"))]
    /// Returns mappings of the copies.
    pub fn mapping(&self) -> impl Iterator<Item = impl Iterator<Item = (usize, usize)> + '_> {
        self.mapping.iter().map(|c| c.iter().copied())
    }
}

#[cfg(feature = "thread-safe-region")]
/// Struct that accumulates all the necessary data in order to construct the permutation argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assembly {
    /// Columns that participate on the copy permutation argument.
    columns: Vec<Column<Any>>,
    /// Mapping of the actual copies done.
    cycles: Vec<Vec<(usize, usize)>>,
    /// Mapping of the actual copies done.
    ordered_cycles: Vec<BTreeSet<(usize, usize)>>,
    /// Mapping of the actual copies done.
    aux: HashMap<(usize, usize), usize>,
    /// total length of a column
    col_len: usize,
    /// number of columns
    num_cols: usize,
}

#[cfg(feature = "thread-safe-region")]
impl Assembly {
    pub(crate) fn new(n: usize, p: &Argument) -> Self {
        Assembly {
            columns: p.columns.clone(),
            cycles: Vec::with_capacity(n),
            ordered_cycles: Vec::with_capacity(n),
            aux: HashMap::new(),
            col_len: n,
            num_cols: p.columns.len(),
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
        if left_row >= self.col_len || right_row >= self.col_len {
            return Err(Error::BoundsFailure);
        }

        let left_cycle = self.aux.get(&(left_column, left_row));
        let right_cycle = self.aux.get(&(right_column, right_row));

        // extract cycle elements
        let right_cycle_elems = match right_cycle {
            Some(i) => {
                let entry = self.cycles[*i].clone();
                self.cycles[*i] = vec![];
                entry
            }
            None => [(right_column, right_row)].into(),
        };

        assert!(right_cycle_elems.contains(&(right_column, right_row)));

        // merge cycles
        let cycle_idx = match left_cycle {
            Some(i) => {
                let entry = &mut self.cycles[*i];
                entry.extend(right_cycle_elems.clone());
                *i
            }
            // if they were singletons -- create a new cycle entry
            None => {
                let mut set: Vec<(usize, usize)> = right_cycle_elems.clone();
                set.push((left_column, left_row));
                self.cycles.push(set);
                let cycle_idx = self.cycles.len() - 1;
                self.aux.insert((left_column, left_row), cycle_idx);
                cycle_idx
            }
        };

        let index_updates = vec![cycle_idx; right_cycle_elems.len()].into_iter();
        let updates = right_cycle_elems.into_iter().zip(index_updates);

        self.aux.extend(updates);

        Ok(())
    }

    /// Builds the ordered mapping of the cycles.
    /// This will only get executed once.
    pub fn build_ordered_mapping(&mut self) {
        use crate::multicore::IntoParallelRefMutIterator;

        // will only get called once
        if self.ordered_cycles.is_empty() && !self.cycles.is_empty() {
            self.ordered_cycles = self
                .cycles
                .par_iter_mut()
                .map(|col| {
                    let mut set = BTreeSet::new();
                    set.extend(col.clone());
                    // free up memory
                    *col = vec![];
                    set
                })
                .collect();
        }
    }

    fn mapping_at_idx(&self, col: usize, row: usize) -> (usize, usize) {
        assert!(
            !self.ordered_cycles.is_empty() || self.cycles.is_empty(),
            "cycles have not been ordered"
        );

        if let Some(cycle_idx) = self.aux.get(&(col, row)) {
            let cycle = &self.ordered_cycles[*cycle_idx];
            let mut cycle_iter = cycle.range((
                std::ops::Bound::Excluded((col, row)),
                std::ops::Bound::Unbounded,
            ));
            // point to the next node in the cycle
            match cycle_iter.next() {
                Some((i, j)) => (*i, *j),
                // wrap back around to the first element which SHOULD exist
                None => *(cycle.iter().next().unwrap()),
            }
        // is a singleton
        } else {
            (col, row)
        }
    }

    pub(crate) fn build_vk<'params, C: CurveAffine, P: Params<'params, C>>(
        &mut self,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        p: &Argument,
    ) -> VerifyingKey<C> {
        self.build_ordered_mapping();
        build_vk(params, domain, p, |i, j| self.mapping_at_idx(i, j))
    }

    pub(crate) fn build_pk<'params, C: CurveAffine, P: Params<'params, C>>(
        &mut self,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        p: &Argument,
    ) -> ProvingKey<C> {
        self.build_ordered_mapping();
        build_pk(params, domain, p, |i, j| self.mapping_at_idx(i, j))
    }

    /// Returns columns that participate in the permutation argument.
    pub fn columns(&self) -> &[Column<Any>] {
        &self.columns
    }

    #[cfg(feature = "multicore")]
    /// Returns mappings of the copies.
    pub fn mapping(
        &self,
    ) -> impl Iterator<Item = impl IndexedParallelIterator<Item = (usize, usize)> + '_> {
        use crate::multicore::IntoParallelIterator;

        (0..self.num_cols).map(move |i| {
            (0..self.col_len)
                .into_par_iter()
                .map(move |j| self.mapping_at_idx(i, j))
        })
    }

    #[cfg(not(feature = "multicore"))]
    /// Returns mappings of the copies.
    pub fn mapping(&self) -> impl Iterator<Item = impl Iterator<Item = (usize, usize)> + '_> {
        (0..self.num_cols).map(move |i| (0..self.col_len).map(move |j| self.mapping_at_idx(i, j)))
    }
}

pub(crate) fn build_pk<'params, C: CurveAffine, P: Params<'params, C>>(
    params: &P,
    domain: &EvaluationDomain<C::Scalar>,
    p: &Argument,
    mapping: impl Fn(usize, usize) -> (usize, usize) + Sync,
) -> ProvingKey<C> {
    // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
    let mut omega_powers = vec![C::Scalar::ZERO; params.n() as usize];
    {
        let omega = domain.get_omega();
        parallelize(&mut omega_powers, |o, start| {
            let mut cur = omega.pow_vartime([start as u64]);
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
            let mut cur = C::Scalar::DELTA.pow_vartime([start as u64]);
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
                    let (permuted_i, permuted_j) = mapping(i, j);
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

pub(crate) fn build_vk<'params, C: CurveAffine, P: Params<'params, C>>(
    params: &P,
    domain: &EvaluationDomain<C::Scalar>,
    p: &Argument,
    mapping: impl Fn(usize, usize) -> (usize, usize) + Sync,
) -> VerifyingKey<C> {
    // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
    let mut omega_powers = vec![C::Scalar::ZERO; params.n() as usize];
    {
        let omega = domain.get_omega();
        parallelize(&mut omega_powers, |o, start| {
            let mut cur = omega.pow_vartime([start as u64]);
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
            let mut cur = C::Scalar::DELTA.pow_vartime([start as u64]);
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
                    let (permuted_i, permuted_j) = mapping(i, j);
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
