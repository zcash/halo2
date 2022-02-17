//! This module contains an optimisation of the polynomial commitment opening
//! scheme described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use super::{commitment::ParamsVerifier, PairMSM};
use crate::{
    arithmetic::{eval_polynomial, CurveAffine, FieldExt},
    pairing::arithmetic::{MillerLoopResult, MultiMillerLoop},
    poly::{msm::MSM, Coeff, Error, Polynomial},
};

use crate::poly::Rotation;
use ff::Field;
use group::Group;
use rand::RngCore;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    thread::AccessError,
};
use subtle::Choice;

cfg_if::cfg_if! {
    if #[cfg(feature = "shplonk")] {
        mod shplonk;
        pub use shplonk::*;
    } else {
        mod gwc;
        pub use gwc::*;
    }
}

/// Decider performs final pairing check with given verifier params and two channel linear combination
#[derive(Debug)]
pub struct Decider<E: MultiMillerLoop> {
    _marker: PhantomData<E>,
}

impl<E: MultiMillerLoop> Decider<E> {
    fn prepare(params: &ParamsVerifier<E>) -> (E::G2Prepared, E::G2Prepared) {
        let s_g2_prepared = E::G2Prepared::from(params.s_g2);
        let n_g2_prepared = E::G2Prepared::from(-params.g2);
        (s_g2_prepared, n_g2_prepared)
    }

    fn pairing_check(terms: &[(&E::G1Affine, &E::G2Prepared); 2]) -> bool {
        bool::from(
            E::multi_miller_loop(&terms[..])
                .final_exponentiation()
                .is_identity(),
        )
    }

    /// Performs final pairing check with given verifier params and two channel linear combination
    pub fn verify(params: &ParamsVerifier<E>, msm: PairMSM<E::G1Affine>) -> bool {
        let (s_g2, n_g2) = Self::prepare(params);
        let (left, right) = msm.eval();
        let (term_1, term_2) = ((&left, &s_g2), (&right, &n_g2));
        Self::pairing_check(&[term_1, term_2])
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct ProverQuery<'a, C: CurveAffine> {
    /// point at which polynomial is queried
    pub point: C::Scalar,
    pub rotation: Rotation,
    /// coefficients of polynomial
    pub poly: &'a Polynomial<C::Scalar, Coeff>,
}

/// A polynomial query at a point
#[derive(Debug, Clone, Copy)]
pub struct VerifierQuery<'r, C: CurveAffine> {
    /// point at which polynomial is queried
    point: C::Scalar,
    /// rotation at which polynomial is queried
    rotation: Rotation,
    /// commitment to polynomial
    commitment: CommitmentReference<'r, C>,
    /// evaluation of polynomial at query point
    eval: C::Scalar,
}

impl<'r, 'params: 'r, C: CurveAffine> VerifierQuery<'r, C> {
    /// Create a new verifier query based on a commitment
    pub fn new_commitment(
        commitment: &'r C,
        point: C::Scalar,
        rotation: Rotation,
        eval: C::Scalar,
    ) -> Self {
        VerifierQuery {
            point,
            rotation,
            eval,
            commitment: CommitmentReference::Commitment(commitment),
        }
    }

    /// Create a new verifier query based on a linear combination of commitments
    pub fn new_msm(msm: &'r MSM<C>, point: C::Scalar, rotation: Rotation, eval: C::Scalar) -> Self {
        VerifierQuery {
            point,
            rotation,
            eval,
            commitment: CommitmentReference::MSM(msm),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum CommitmentReference<'r, C: CurveAffine> {
    Commitment(&'r C),
    MSM(&'r MSM<C>),
}

impl<'r, 'params: 'r, C: CurveAffine> PartialEq for CommitmentReference<'r, C> {
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

trait Query<F: FieldExt>: Sized + Clone {
    type Commitment: PartialEq + Clone;

    fn get_rotation(&self) -> Rotation;
    fn get_point(&self) -> F;
    fn get_eval(&self) -> F;
    fn get_commitment(&self) -> Self::Commitment;
}

#[cfg(test)]
mod tests {

    use crate::arithmetic::{eval_polynomial, FieldExt};
    use crate::pairing::bn256::{Bn256, Fr, G1Affine};
    use crate::poly::{
        commitment::{Params, ParamsVerifier},
        multiopen::{create_proof, verify_proof, Decider, ProverQuery, Query, VerifierQuery},
        Coeff, Polynomial, Rotation,
    };
    use crate::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, ChallengeScalar, Transcript, TranscriptRead,
        TranscriptWrite,
    };

    use ff::Field;
    use rand::RngCore;
    use rand_core::OsRng;
    use std::collections::BTreeSet;
    use std::marker::PhantomData;

    fn rand_poly(n: usize, mut rng: impl RngCore) -> Polynomial<Fr, Coeff> {
        Polynomial {
            values: (0..n).into_iter().map(|_| Fr::random(&mut rng)).collect(),
            _marker: PhantomData,
        }
    }

    #[test]
    fn test_roundtrip() {
        use ff::Field;
        use group::Curve;
        use rand_core::OsRng;

        use super::*;
        use crate::arithmetic::{eval_polynomial, FieldExt};
        use crate::poly::{commitment::Params, EvaluationDomain};
        use crate::transcript::Challenge255;

        use pairing::bn256::{Bn256, Fr as Fp, G1Affine};

        const K: u32 = 4;

        let params: Params<G1Affine> = Params::<G1Affine>::unsafe_setup::<Bn256>(K);
        let params_verifier: ParamsVerifier<Bn256> = params.verifier(0).unwrap();

        let domain = EvaluationDomain::new(1, K);
        let rng = OsRng;

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

        let a = params.commit(&ax).to_affine();
        let b = params.commit(&bx).to_affine();
        let c = params.commit(&cx).to_affine();

        let cur = Rotation::cur();
        let next = Rotation::next();
        let x = Fp::random(rng);
        let y = domain.rotate_omega(x, next);
        let avx = eval_polynomial(&ax, x);
        let bvx = eval_polynomial(&bx, x);
        let cvy = eval_polynomial(&cx, y);

        let mut transcript = crate::transcript::Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof(
            &params,
            &mut transcript,
            std::iter::empty()
                .chain(Some(ProverQuery {
                    point: x,
                    rotation: cur,
                    poly: &ax,
                }))
                .chain(Some(ProverQuery {
                    point: x,
                    rotation: cur,
                    poly: &bx,
                }))
                .chain(Some(ProverQuery {
                    point: y,
                    rotation: Rotation::next(),
                    poly: &cx,
                })),
        )
        .unwrap();
        let proof = transcript.finalize();

        {
            let mut proof = &proof[..];
            let mut transcript =
                crate::transcript::Blake2bRead::<_, _, Challenge255<_>>::init(&mut proof);

            let pair = verify_proof(
                &params_verifier,
                &mut transcript,
                std::iter::empty()
                    .chain(Some(VerifierQuery::new_commitment(&a, x, cur, avx)))
                    .chain(Some(VerifierQuery::new_commitment(&b, x, cur, avx))) // NB: wrong!
                    .chain(Some(VerifierQuery::new_commitment(&c, y, next, cvy))),
            )
            .unwrap();

            // Should fail.
            assert!(!Decider::verify(&params_verifier, pair));
        }

        {
            let mut proof = &proof[..];

            let mut transcript =
                crate::transcript::Blake2bRead::<_, _, Challenge255<_>>::init(&mut proof);

            let guard = verify_proof(
                &params_verifier,
                &mut transcript,
                std::iter::empty()
                    .chain(Some(VerifierQuery::new_commitment(&a, x, cur, avx)))
                    .chain(Some(VerifierQuery::new_commitment(&b, x, cur, bvx)))
                    .chain(Some(VerifierQuery::new_commitment(&c, y, next, cvy))),
            )
            .unwrap();

            // Should succeed.
            assert!(Decider::verify(&params_verifier, guard));
        }
    }

    #[test]
    fn test_multiopen() {
        const K: u32 = 3;

        let params = Params::<G1Affine>::unsafe_setup::<Bn256>(K);
        let params_verifier: ParamsVerifier<Bn256> = params.verifier(0).unwrap();

        let rotation_sets_init = vec![
            vec![1i32, 2, 3],
            vec![2, 3, 4],
            vec![2, 3, 4],
            vec![4, 5, 6, 7],
            vec![8],
            vec![9],
            vec![10, 11],
            vec![10, 11],
            vec![10, 11],
        ];
        let commitment_per_set: Vec<usize> = rotation_sets_init
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .collect();
        let rotation_sets: Vec<Vec<(Rotation, Fr)>> = rotation_sets_init
            .into_iter()
            .map(|rot_set| {
                rot_set
                    .into_iter()
                    .map(|i| (Rotation(i), Fr::from(i as u64)))
                    .collect()
            })
            .collect();

        let mut prover_queries = vec![];
        let mut verifier_queries = vec![];

        #[allow(clippy::type_complexity)]
        let (polynomials, commitments): (
            Vec<Vec<Polynomial<Fr, Coeff>>>,
            Vec<Vec<G1Affine>>,
        ) = rotation_sets
            .iter()
            .enumerate()
            .map(|(i, _)| {
                (0..commitment_per_set[i])
                    .map(|_| {
                        let poly = rand_poly(params.n as usize, OsRng);
                        let commitment: G1Affine = params.commit(&poly).into();
                        (poly, commitment)
                    })
                    .unzip()
            })
            .unzip();

        for (i, rotation_set) in rotation_sets.iter().enumerate() {
            for (rot, point) in rotation_set.iter() {
                for j in 0..commitment_per_set[i] {
                    {
                        let query: ProverQuery<G1Affine> = ProverQuery {
                            poly: &polynomials[i][j],
                            point: *point,
                            rotation: *rot,
                        };
                        prover_queries.push(query);
                    }

                    {
                        let poly = &polynomials[i][j];
                        let commitment: &G1Affine = &commitments[i][j];
                        let eval = eval_polynomial(poly, *point);
                        let query = VerifierQuery::new_commitment(commitment, *point, *rot, eval);
                        verifier_queries.push(query);
                    }
                }
            }
        }

        // prover
        let proof = {
            let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
            create_proof(&params, &mut transcript, prover_queries).unwrap();
            transcript.finalize()
        };

        // verifier
        {
            let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&proof[..]);
            let pair = verify_proof(&params_verifier, &mut transcript, verifier_queries).unwrap();
            assert!(Decider::verify(&params_verifier, pair));
        }
    }
}
