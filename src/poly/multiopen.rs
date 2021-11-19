//! This module contains an optimisation of the polynomial commitment opening
//! scheme described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    transcript::ChallengeScalar,
};

mod prover;
mod verifier;

pub use prover::create_proof;
pub use verifier::verify_proof;

#[derive(Clone, Copy, Debug)]
struct X1 {}
/// Challenge for compressing openings at the same point sets together.
type ChallengeX1<F> = ChallengeScalar<F, X1>;

#[derive(Clone, Copy, Debug)]
struct X2 {}
/// Challenge for keeping the multi-point quotient polynomial terms linearly independent.
type ChallengeX2<F> = ChallengeScalar<F, X2>;

#[derive(Clone, Copy, Debug)]
struct X3 {}
/// Challenge point at which the commitments are opened.
type ChallengeX3<F> = ChallengeScalar<F, X3>;

#[derive(Clone, Copy, Debug)]
struct X4 {}
/// Challenge for collapsing the openings of the various remaining polynomials at x_3
/// together.
type ChallengeX4<F> = ChallengeScalar<F, X4>;

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct ProverQuery<'a, C: CurveAffine> {
    /// rotation of challenge at which polynomial is queried
    rotation: Rotation,
    /// coefficients of polynomial
    poly: &'a Polynomial<C::Scalar, Coeff>,
    /// blinding factor of polynomial
    blind: commitment::Blind<C::Scalar>,
}

impl<'a, C: CurveAffine> ProverQuery<'a, C> {
    /// Create a new prover query based on a polynomial
    pub fn new(
        domain: &EvaluationDomain<C::Scalar>,
        poly: &'a Polynomial<C::Scalar, Coeff>,
        point: C::Scalar,
        x: C::Scalar,
        rotation: Rotation,
        blind: commitment::Blind<C::Scalar>,
    ) -> Self {
        assert_eq!(domain.rotate_omega(x, rotation), point);

        ProverQuery {
            poly,
            rotation,
            blind,
        }
    }
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct VerifierQuery<'r, 'params: 'r, C: CurveAffine> {
    /// rotation at which the challenge polynomial is queried
    rotation: Rotation,
    /// commitment to polynomial
    commitment: CommitmentReference<'r, 'params, C>,
    /// evaluation of polynomial at query point
    eval: C::Scalar,
}

impl<'r, 'params: 'r, C: CurveAffine> VerifierQuery<'r, 'params, C> {
    /// Create a new verifier query based on a commitment
    pub fn new_commitment(
        domain: &EvaluationDomain<C::Scalar>,
        commitment: &'r C,
        point: C::Scalar,
        x: C::Scalar,
        rotation: Rotation,
        eval: C::Scalar,
    ) -> Self {
        assert_eq!(domain.rotate_omega(x, rotation), point);

        VerifierQuery {
            rotation,
            eval,
            commitment: CommitmentReference::Commitment(commitment),
        }
    }

    /// Create a new verifier query based on a linear combination of commitments
    pub fn new_msm(
        domain: &EvaluationDomain<C::Scalar>,
        msm: &'r commitment::MSM<'params, C>,
        point: C::Scalar,
        x: C::Scalar,
        rotation: Rotation,
        eval: C::Scalar,
    ) -> Self {
        assert_eq!(domain.rotate_omega(x, rotation), point);

        VerifierQuery {
            rotation,
            eval,
            commitment: CommitmentReference::MSM(msm),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum CommitmentReference<'r, 'params: 'r, C: CurveAffine> {
    Commitment(&'r C),
    MSM(&'r commitment::MSM<'params, C>),
}

impl<'r, 'params: 'r, C: CurveAffine> PartialEq for CommitmentReference<'r, 'params, C> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&CommitmentReference::Commitment(a), &CommitmentReference::Commitment(b)) => {
                std::ptr::eq(a, b)
            }
            (&CommitmentReference::MSM(a), &CommitmentReference::MSM(b)) => std::ptr::eq(a, b),
            _ => false,
        }
    }
}

struct CommitmentData<F, T: PartialEq> {
    commitment: T,
    set_index: usize,
    point_indices: Vec<usize>,
    evals: Vec<F>,
}

impl<F, T: PartialEq> CommitmentData<F, T> {
    fn new(commitment: T) -> Self {
        CommitmentData {
            commitment,
            set_index: 0,
            point_indices: vec![],
            evals: vec![],
        }
    }
}

trait Query<F>: Sized {
    type Commitment: PartialEq + Copy;
    type Eval: Clone + Default;

    fn get_rotation(&self) -> Rotation;
    fn get_eval(&self) -> Self::Eval;
    fn get_commitment(&self) -> Self::Commitment;
}

type IntermediateSets<F, Q> = (
    Vec<CommitmentData<<Q as Query<F>>::Eval, <Q as Query<F>>::Commitment>>,
    Vec<Vec<Rotation>>,
);

fn construct_intermediate_sets<F: FieldExt, I, Q: Query<F>>(queries: I) -> IntermediateSets<F, Q>
where
    I: IntoIterator<Item = Q> + Clone,
{
    // Construct sets of unique commitments and corresponding information about
    // their queries.
    let mut commitment_map: Vec<CommitmentData<Q::Eval, Q::Commitment>> = vec![];

    // Also construct mapping from a unique point to a point_index. This defines
    // an ordering on the points.
    let mut point_index_map = BTreeMap::new();

    // Iterate over all of the queries, computing the ordering of the points
    // while also creating new commitment data.
    for query in queries.clone() {
        let num_points = point_index_map.len();
        let point_idx = point_index_map
            .entry(query.get_rotation())
            .or_insert(num_points);

        if let Some(pos) = commitment_map
            .iter()
            .position(|comm| comm.commitment == query.get_commitment())
        {
            commitment_map[pos].point_indices.push(*point_idx);
        } else {
            let mut tmp = CommitmentData::new(query.get_commitment());
            tmp.point_indices.push(*point_idx);
            commitment_map.push(tmp);
        }
    }

    // Also construct inverse mapping from point_index to the point
    let mut inverse_point_index_map = BTreeMap::new();
    for (&point, &point_index) in point_index_map.iter() {
        inverse_point_index_map.insert(point_index, point);
    }

    // Construct map of unique ordered point_idx_sets to their set_idx
    let mut point_idx_sets = BTreeMap::new();
    // Also construct mapping from commitment to point_idx_set
    let mut commitment_set_map = Vec::new();

    for commitment_data in commitment_map.iter() {
        let mut point_index_set = BTreeSet::new();
        // Note that point_index_set is ordered, unlike point_indices
        for &point_index in commitment_data.point_indices.iter() {
            point_index_set.insert(point_index);
        }

        // Push point_index_set to CommitmentData for the relevant commitment
        commitment_set_map.push((commitment_data.commitment, point_index_set.clone()));

        let num_sets = point_idx_sets.len();
        point_idx_sets.entry(point_index_set).or_insert(num_sets);
    }

    // Initialise empty evals vec for each unique commitment
    for commitment_data in commitment_map.iter_mut() {
        let len = commitment_data.point_indices.len();
        commitment_data.evals = vec![Q::Eval::default(); len];
    }

    // Populate set_index, evals and points for each commitment using point_idx_sets
    for query in queries {
        // The index of the point at which the commitment is queried
        let point_index = point_index_map.get(&query.get_rotation()).unwrap();

        // The point_index_set at which the commitment was queried
        let mut point_index_set = BTreeSet::new();
        for (commitment, point_idx_set) in commitment_set_map.iter() {
            if query.get_commitment() == *commitment {
                point_index_set = point_idx_set.clone();
            }
        }
        assert!(!point_index_set.is_empty());

        // The set_index of the point_index_set
        let set_index = point_idx_sets.get(&point_index_set).unwrap();
        for commitment_data in commitment_map.iter_mut() {
            if query.get_commitment() == commitment_data.commitment {
                commitment_data.set_index = *set_index;
            }
        }
        let point_index_set: Vec<usize> = point_index_set.iter().cloned().collect();

        // The offset of the point_index in the point_index_set
        let point_index_in_set = point_index_set
            .iter()
            .position(|i| i == point_index)
            .unwrap();

        for commitment_data in commitment_map.iter_mut() {
            if query.get_commitment() == commitment_data.commitment {
                // Insert the eval using the ordering of the point_index_set
                commitment_data.evals[point_index_in_set] = query.get_eval();
            }
        }
    }

    // Get actual points in each point set
    let mut point_sets: Vec<Vec<Rotation>> = vec![Vec::new(); point_idx_sets.len()];
    for (point_idx_set, &set_idx) in point_idx_sets.iter() {
        for &point_idx in point_idx_set.iter() {
            let point = inverse_point_index_map.get(&point_idx).unwrap();
            point_sets[set_idx].push(*point);
        }
    }

    (commitment_map, point_sets)
}

#[test]
fn test_roundtrip() {
    use group::Curve;

    use super::commitment::{Blind, Params};
    use crate::arithmetic::{eval_polynomial, FieldExt};
    use crate::pasta::{EqAffine, Fp};
    use crate::transcript::Challenge255;

    const K: u32 = 4;

    let params: Params<EqAffine> = Params::new(K);
    let domain = EvaluationDomain::new(1, K);

    let mut ax = domain.empty_coeff();
    for (i, a) in ax.iter_mut().enumerate() {
        *a = Fp::from(10 + i as u64);
    }

    let mut bx = domain.empty_coeff();
    for (i, a) in bx.iter_mut().enumerate() {
        *a = Fp::from(100 + i as u64);
    }

    let mut cx = domain.empty_coeff();
    for (i, a) in cx.iter_mut().enumerate() {
        *a = Fp::from(100 + i as u64);
    }

    let blind = Blind(Fp::rand());

    let a = params.commit(&ax, blind).to_affine();
    let b = params.commit(&bx, blind).to_affine();
    let c = params.commit(&cx, blind).to_affine();

    let x = Fp::rand();
    let y = domain.rotate_omega(x, Rotation::next());
    let avx = eval_polynomial(&ax, x);
    let bvx = eval_polynomial(&bx, x);
    let cvy = eval_polynomial(&cx, y);

    let mut transcript = crate::transcript::Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &domain,
        x,
        &mut transcript,
        std::iter::empty()
            .chain(Some(ProverQuery {
                rotation: Rotation::cur(),
                poly: &ax,
                blind,
            }))
            .chain(Some(ProverQuery {
                rotation: Rotation::cur(),
                poly: &bx,
                blind,
            }))
            .chain(Some(ProverQuery {
                rotation: Rotation::next(),
                poly: &cx,
                blind,
            })),
    )
    .unwrap();
    let proof = transcript.finalize();

    {
        let mut proof = &proof[..];
        let mut transcript =
            crate::transcript::Blake2bRead::<_, _, Challenge255<_>>::init(&mut proof);
        let msm = params.empty_msm();

        let guard = verify_proof(
            &params,
            &domain,
            x,
            &mut transcript,
            std::iter::empty()
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &a,
                    x,
                    x,
                    Rotation::cur(),
                    avx,
                )))
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &b,
                    x,
                    x,
                    Rotation::cur(),
                    avx,
                ))) // NB: wrong!
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &c,
                    y,
                    x,
                    Rotation::next(),
                    cvy,
                ))),
            msm,
        )
        .unwrap();

        // Should fail.
        assert!(!guard.use_challenges().eval());
    }

    {
        let mut proof = &proof[..];

        let mut transcript =
            crate::transcript::Blake2bRead::<_, _, Challenge255<_>>::init(&mut proof);
        let msm = params.empty_msm();

        let guard = verify_proof(
            &params,
            &domain,
            x,
            &mut transcript,
            std::iter::empty()
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &a,
                    x,
                    x,
                    Rotation::cur(),
                    avx,
                )))
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &b,
                    x,
                    x,
                    Rotation::cur(),
                    bvx,
                )))
                .chain(Some(VerifierQuery::new_commitment(
                    &domain,
                    &c,
                    y,
                    x,
                    Rotation::next(),
                    cvy,
                ))),
            msm,
        )
        .unwrap();

        // Should succeed.
        assert!(guard.use_challenges().eval());
    }
}
