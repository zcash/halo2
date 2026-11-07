//! This module contains an optimisation of the polynomial commitment opening
//! scheme described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use super::*;
use crate::{arithmetic::CurveAffine, transcript::ChallengeScalar};

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
    /// point at which polynomial is queried
    pub point: C::Scalar,
    /// coefficients of polynomial
    pub poly: &'a Polynomial<C::Scalar, Coeff>,
    /// blinding factor of polynomial
    pub blind: commitment::Blind<C::Scalar>,
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct VerifierQuery<'r, 'params: 'r, C: CurveAffine> {
    /// point at which polynomial is queried
    point: C::Scalar,
    /// commitment to polynomial
    commitment: CommitmentReference<'r, 'params, C>,
    /// evaluation of polynomial at query point
    eval: C::Scalar,
}

impl<'r, 'params: 'r, C: CurveAffine> VerifierQuery<'r, 'params, C> {
    /// Create a new verifier query based on a commitment
    pub fn new_commitment(commitment: &'r C, point: C::Scalar, eval: C::Scalar) -> Self {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::Commitment(commitment),
        }
    }

    /// Create a new verifier query based on a linear combination of commitments
    pub fn new_msm(
        msm: &'r commitment::MSM<'params, C>,
        point: C::Scalar,
        eval: C::Scalar,
    ) -> Self {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::MSM(msm),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
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

impl<'r, 'params: 'r, C: CurveAffine> Eq for CommitmentReference<'r, 'params, C> {}

impl<'r, 'params: 'r, C: CurveAffine> Hash for CommitmentReference<'r, 'params, C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match *self {
            CommitmentReference::Commitment(a) => std::ptr::hash(a, state),
            CommitmentReference::MSM(a) => std::ptr::hash(a, state),
        }
    }
}

#[derive(Debug)]
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
    type Commitment: Eq + Hash + Copy;
    type Eval: Clone + Default;

    fn get_point(&self) -> F;
    fn get_eval(&self) -> Self::Eval;
    fn get_commitment(&self) -> Self::Commitment;
}

type IntermediateSets<F, Q> = (
    Vec<CommitmentData<<Q as Query<F>>::Eval, <Q as Query<F>>::Commitment>>,
    Vec<Vec<F>>,
);

/// A commitment with the evaluations queried for it, before its point set is known.
type PendingCommitment<F, Q> = (
    <Q as Query<F>>::Commitment,
    CommitmentData<<Q as Query<F>>::Eval, ()>,
);

/// Returns `None` if `queries` contains the same point and commitment more than once.
fn construct_intermediate_sets<F: Field + Ord, I, Q: Query<F>>(
    queries: I,
) -> Option<IntermediateSets<F, Q>>
where
    I: IntoIterator<Item = Q> + Clone,
{
    // First-seen commitment order feeds the transcript and must be deterministic.
    // Keep a side index so batched proofs do not require linear commitment lookups.
    let mut commitment_map: Vec<PendingCommitment<F, Q>> = Vec::new();
    let mut commitment_indices: HashMap<Q::Commitment, usize> = HashMap::new();

    // Assign point indices in first-seen order.
    let mut point_index_map = BTreeMap::new();
    let mut points = Vec::new();

    // Collect evaluations while traversing the query iterator only once.
    for query in queries {
        let point = query.get_point();
        let point_idx = *point_index_map.entry(point).or_insert_with(|| {
            let point_idx = points.len();
            points.push(point);
            point_idx
        });

        let commitment = query.get_commitment();
        let commitment_idx = *commitment_indices.entry(commitment).or_insert_with(|| {
            commitment_map.push((commitment, CommitmentData::new(())));
            commitment_map.len() - 1
        });
        let commitment_data = &mut commitment_map[commitment_idx].1;
        if commitment_data.point_indices.contains(&point_idx) {
            // Reject duplicate queries even when their evaluations agree.
            return None;
        }
        commitment_data.point_indices.push(point_idx);
        commitment_data.evals.push(query.get_eval());
    }

    // Number unique point-index sets in first-seen commitment order.
    let mut point_idx_sets = BTreeMap::new();

    let commitment_map = commitment_map
        .into_iter()
        .map(|(commitment, commitment_data)| {
            let mut indexed_evals = commitment_data
                .point_indices
                .iter()
                .copied()
                .zip(commitment_data.evals)
                .collect::<Vec<_>>();
            indexed_evals.sort_unstable_by_key(|(point_idx, _)| *point_idx);

            let point_idx_set = indexed_evals
                .iter()
                .map(|(point_idx, _)| *point_idx)
                .collect::<Vec<_>>();
            let num_sets = point_idx_sets.len();
            let set_index = *point_idx_sets.entry(point_idx_set).or_insert(num_sets);

            CommitmentData {
                commitment,
                set_index,
                point_indices: commitment_data.point_indices,
                evals: indexed_evals.into_iter().map(|(_, eval)| eval).collect(),
            }
        })
        .collect();

    // Get actual points in each point set, sorted by global point index.
    let mut point_sets: Vec<Vec<F>> = vec![Vec::new(); point_idx_sets.len()];
    for (point_idx_set, set_idx) in point_idx_sets {
        point_sets[set_idx] = point_idx_set
            .into_iter()
            .map(|point_idx| points[point_idx])
            .collect();
    }

    Some((commitment_map, point_sets))
}

#[test]
fn test_roundtrip() {
    use pasta_curves::arithmetic::CurveExt;
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    use super::commitment::{Blind, Params};
    use crate::arithmetic::eval_polynomial;
    use crate::pasta::{EqAffine, Fp};
    use crate::transcript::Challenge255;

    const K: u32 = 4;

    let params: Params<EqAffine> = Params::new(K);
    let domain = EvaluationDomain::new(1, K);
    let mut rng = UnwrapErr(SysRng);

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

    let blind = Blind(Fp::random(&mut rng));

    let a = params.commit(&ax, blind).to_affine_vartime();
    let b = params.commit(&bx, blind).to_affine_vartime();
    let c = params.commit(&cx, blind).to_affine_vartime();

    let x = Fp::random(&mut rng);
    let y = Fp::random(&mut rng);
    let avx = eval_polynomial(&ax, x);
    let bvx = eval_polynomial(&bx, x);
    let cvy = eval_polynomial(&cx, y);

    let mut transcript = crate::transcript::Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        rng,
        &mut transcript,
        std::iter::empty()
            .chain(Some(ProverQuery {
                point: x,
                poly: &ax,
                blind,
            }))
            .chain(Some(ProverQuery {
                point: x,
                poly: &bx,
                blind,
            }))
            .chain(Some(ProverQuery {
                point: y,
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
            &mut transcript,
            std::iter::empty()
                .chain(Some(VerifierQuery::new_commitment(&a, x, avx)))
                .chain(Some(VerifierQuery::new_commitment(&b, x, avx))) // NB: wrong!
                .chain(Some(VerifierQuery::new_commitment(&c, y, cvy))),
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
            &mut transcript,
            std::iter::empty()
                .chain(Some(VerifierQuery::new_commitment(&a, x, avx)))
                .chain(Some(VerifierQuery::new_commitment(&b, x, bvx)))
                .chain(Some(VerifierQuery::new_commitment(&c, y, cvy))),
            msm,
        )
        .unwrap();

        // Should succeed.
        assert!(guard.use_challenges().eval());
    }
}

#[test]
fn test_identical_queries() {
    use assert_matches::assert_matches;
    use pasta_curves::arithmetic::CurveExt;
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    use super::commitment::{Blind, Params};
    use crate::arithmetic::eval_polynomial;
    use crate::pasta::{EqAffine, Fp};
    use crate::transcript::Challenge255;

    const K: u32 = 4;

    let params: Params<EqAffine> = Params::new(K);
    let domain = EvaluationDomain::new(1, K);
    let mut rng = UnwrapErr(SysRng);

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

    let blind = Blind(Fp::random(&mut rng));

    let a = params.commit(&ax, blind).to_affine_vartime();
    let b = params.commit(&bx, blind).to_affine_vartime();
    let c = params.commit(&cx, blind).to_affine_vartime();

    let x = Fp::random(&mut rng);
    let y = Fp::random(&mut rng);
    let avx = eval_polynomial(&ax, x);
    let bvx = eval_polynomial(&bx, x);
    let bvx_bad = Fp::random(&mut rng);
    let cvy = eval_polynomial(&cx, y);

    let mut transcript = crate::transcript::Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        rng,
        &mut transcript,
        std::iter::empty()
            .chain(Some(ProverQuery {
                point: x,
                poly: &ax,
                blind,
            }))
            .chain(Some(ProverQuery {
                point: x,
                poly: &bx,
                blind,
            }))
            .chain(Some(ProverQuery {
                point: y,
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

        assert_matches!(
            verify_proof(
                &params,
                &mut transcript,
                std::iter::empty()
                    .chain(Some(VerifierQuery::new_commitment(&a, x, avx)))
                    .chain(Some(VerifierQuery::new_commitment(&b, x, bvx_bad))) // This is wrong.
                    .chain(Some(VerifierQuery::new_commitment(&b, x, bvx)))
                    .chain(Some(VerifierQuery::new_commitment(&c, y, cvy))),
                msm,
            ),
            Err(Error::OpeningError)
        );
    }
}

#[cfg(test)]
mod proptests;
