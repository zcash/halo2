//! This module contains an optimisation of the polynomial commitment opening
//! scheme described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use super::*;
use crate::{arithmetic::CurveAffine, transcript::ChallengeScalar};

#[cfg(test)]
mod differential_tests;
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

/// Returns `None` if `queries` contains two queries with the same point and commitment.
fn construct_intermediate_sets<F: Field + Ord, I, Q: Query<F>>(
    queries: I,
) -> Option<IntermediateSets<F, Q>>
where
    I: IntoIterator<Item = Q>,
{
    // Preserve first-seen commitment and point order: set indices feed the transcript.
    let mut commitments = Vec::new();
    let mut commitment_indices = HashMap::new();
    let mut points = Vec::new();

    for query in queries {
        let point = query.get_point();
        let point_index = match points.iter().position(|p| *p == point) {
            Some(index) => index,
            None => {
                points.push(point);
                points.len() - 1
            }
        };
        let commitment = query.get_commitment();
        let index = *commitment_indices.entry(commitment).or_insert_with(|| {
            commitments.push((commitment, CommitmentData::new(())));
            commitments.len() - 1
        });
        let data = &mut commitments[index].1;
        if data.point_indices.contains(&point_index) {
            return None;
        }
        data.point_indices.push(point_index);
        data.evals.push(query.get_eval());
    }

    let mut point_sets = BTreeMap::new();
    let commitments = commitments
        .into_iter()
        .map(|(commitment, data)| {
            let mut indexed_evals = data
                .point_indices
                .iter()
                .copied()
                .zip(data.evals)
                .collect::<Vec<_>>();
            indexed_evals.sort_unstable_by_key(|(index, _)| *index);
            let set = indexed_evals
                .iter()
                .map(|(index, _)| *index)
                .collect::<Vec<_>>();
            let next_index = point_sets.len();
            let set_index = *point_sets.entry(set).or_insert(next_index);
            CommitmentData {
                commitment,
                set_index,
                point_indices: data.point_indices,
                evals: indexed_evals.into_iter().map(|(_, eval)| eval).collect(),
            }
        })
        .collect();

    let mut ordered_sets = vec![Vec::new(); point_sets.len()];
    for (indices, set_index) in point_sets {
        ordered_sets[set_index] = indices.into_iter().map(|index| points[index]).collect();
    }
    Some((commitments, ordered_sets))
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
mod proptests {
    use group::ff::FromUniformBytes;
    use proptest::{
        collection::{hash_set, vec},
        prelude::*,
        sample::select,
    };

    use super::construct_intermediate_sets;
    use pasta_curves::Fp;

    use std::convert::TryFrom;

    #[derive(Debug, Clone)]
    struct MyQuery<F> {
        point: F,
        eval: F,
        commitment: usize,
    }

    impl super::Query<Fp> for MyQuery<Fp> {
        type Commitment = usize;
        type Eval = Fp;

        fn get_point(&self) -> Fp {
            self.point
        }

        fn get_eval(&self) -> Self::Eval {
            self.eval
        }

        fn get_commitment(&self) -> Self::Commitment {
            self.commitment
        }
    }

    prop_compose! {
        fn arb_point()(
            bytes in vec(any::<u8>(), 64)
        ) -> Fp {
            Fp::from_uniform_bytes(&<[u8; 64]>::try_from(bytes).unwrap())
        }
    }

    prop_compose! {
        fn arb_query(commitment: usize, point: Fp)(
            eval in arb_point()
        ) -> MyQuery<Fp> {
            MyQuery {
                point,
                eval,
                commitment
            }
        }
    }

    prop_compose! {
        // Mapping from column index to point index.
        fn arb_queries_inner(num_points: usize, num_cols: usize, num_queries: usize)(
            // Use a HashSet to ensure we sample distinct (column, point) queries.
            queries in hash_set(
                (
                    select((0..num_cols).collect::<Vec<_>>()),
                    select((0..num_points).collect::<Vec<_>>()),
                ),
                num_queries,
            )
        ) -> Vec<(usize, usize)> {
            queries.into_iter().collect()
        }
    }

    prop_compose! {
        fn compare_queries(
            num_points: usize,
            num_cols: usize,
            num_queries: usize,
        )(
            points_1 in vec(arb_point(), num_points),
            points_2 in vec(arb_point(), num_points),
            mapping in arb_queries_inner(num_points, num_cols, num_queries)
        )(
            queries_1 in mapping.iter().map(|(commitment, point_idx)| arb_query(*commitment, points_1[*point_idx])).collect::<Vec<_>>(),
            queries_2 in mapping.iter().map(|(commitment, point_idx)| arb_query(*commitment, points_2[*point_idx])).collect::<Vec<_>>(),
        ) -> (
            Vec<MyQuery<Fp>>,
            Vec<MyQuery<Fp>>
        ) {
            (
                queries_1,
                queries_2,
            )
        }
    }

    proptest! {
        #[test]
        fn test_intermediate_sets(
            (queries_1, queries_2) in compare_queries(8, 8, 16)
        ) {
            let (commitment_data, _point_sets) = construct_intermediate_sets(queries_1)
                .ok_or_else(|| TestCaseError::Fail("mismatched evals".into()))?;
            let set_indices = commitment_data.iter().map(|data| data.set_index).collect::<Vec<_>>();
            let point_indices = commitment_data.iter().map(|data| data.point_indices.clone()).collect::<Vec<_>>();

            // It shouldn't matter what the point or eval values are; we should get
            // the same exact point set indices and point indices again.
            let (new_commitment_data, _new_point_sets) = construct_intermediate_sets(queries_2)
                .ok_or_else(|| TestCaseError::Fail("mismatched evals".into()))?;
            let new_set_indices = new_commitment_data.iter().map(|data| data.set_index).collect::<Vec<_>>();
            let new_point_indices = new_commitment_data.iter().map(|data| data.point_indices.clone()).collect::<Vec<_>>();

            assert_eq!(set_indices, new_set_indices);
            assert_eq!(point_indices, new_point_indices);
        }
    }
}
