use std::fmt::Debug;
use std::io::Read;
use std::marker::PhantomData;

use ff::Field;
use rand_core::RngCore;

use super::{
    construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4, Query,
};
use crate::arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt};
use crate::poly::commitment::{Params, Verifier, MSM};
use crate::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA, ParamsVerifierIPA};
use crate::poly::ipa::msm::MSMIPA;
use crate::poly::ipa::strategy::GuardIPA;
use crate::poly::query::{CommitmentReference, VerifierQuery};
use crate::poly::strategy::VerificationStrategy;
use crate::poly::Error;
use crate::transcript::{EncodedChallenge, TranscriptRead};

/// IPA multi-open verifier
#[derive(Debug)]
pub struct VerifierIPA<'params, C: CurveAffine> {
    params: &'params ParamsIPA<C>,
}

impl<'params, C: CurveAffine> Verifier<'params, IPACommitmentScheme<C>>
    for VerifierIPA<'params, C>
{
    type Guard = GuardIPA<'params, C>;
    type MSMAccumulator = MSMIPA<'params, C>;

    fn new(params: &'params ParamsVerifierIPA<C>) -> Self {
        Self { params }
    }

    fn verify_proof<'com, E: EncodedChallenge<C>, T: TranscriptRead<C, E>, I>(
        &self,
        transcript: &mut T,
        queries: I,
        mut msm: MSMIPA<'params, C>,
    ) -> Result<Self::Guard, Error>
    where
        'params: 'com,
        I: IntoIterator<Item = VerifierQuery<'com, C, MSMIPA<'params, C>>> + Clone,
    {
        // Sample x_1 for compressing openings at the same point sets together
        let x_1: ChallengeX1<_> = transcript.squeeze_challenge_scalar();

        // Sample a challenge x_2 for keeping the multi-point quotient
        // polynomial terms linearly independent.
        let x_2: ChallengeX2<_> = transcript.squeeze_challenge_scalar();

        let (commitment_map, point_sets) = construct_intermediate_sets(queries);

        // Compress the commitments and expected evaluations at x together.
        // using the challenge x_1
        let mut q_commitments: Vec<_> = vec![self.params.empty_msm(); point_sets.len()];

        // A vec of vecs of evals. The outer vec corresponds to the point set,
        // while the inner vec corresponds to the points in a particular set.
        let mut q_eval_sets = Vec::with_capacity(point_sets.len());
        for point_set in point_sets.iter() {
            q_eval_sets.push(vec![C::Scalar::zero(); point_set.len()]);
        }
        {
            let mut accumulate = |set_idx: usize,
                                  new_commitment: CommitmentReference<C, MSMIPA<'params, C>>,
                                  evals: Vec<C::Scalar>| {
                q_commitments[set_idx].scale(*x_1);
                match new_commitment {
                    CommitmentReference::Commitment(c) => {
                        q_commitments[set_idx].append_term(C::Scalar::one(), (*c).into());
                    }
                    CommitmentReference::MSM(msm) => {
                        q_commitments[set_idx].add_msm(msm);
                    }
                }
                for (eval, set_eval) in evals.iter().zip(q_eval_sets[set_idx].iter_mut()) {
                    *set_eval *= &(*x_1);
                    *set_eval += eval;
                }
            };

            // Each commitment corresponds to evaluations at a set of points.
            // For each set, we collapse each commitment's evals pointwise.
            for commitment_data in commitment_map.into_iter() {
                accumulate(
                    commitment_data.set_index,  // set_idx,
                    commitment_data.commitment, // commitment,
                    commitment_data.evals,      // evals
                );
            }
        }

        // Obtain the commitment to the multi-point quotient polynomial f(X).
        let q_prime_commitment = transcript.read_point().map_err(|_| Error::SamplingError)?;

        // Sample a challenge x_3 for checking that f(X) was committed to
        // correctly.
        let x_3: ChallengeX3<_> = transcript.squeeze_challenge_scalar();

        // u is a vector containing the evaluations of the Q polynomial
        // commitments at x_3
        let mut u = Vec::with_capacity(q_eval_sets.len());
        for _ in 0..q_eval_sets.len() {
            u.push(transcript.read_scalar().map_err(|_| Error::SamplingError)?);
        }

        // We can compute the expected msm_eval at x_3 using the u provided
        // by the prover and from x_2
        let msm_eval = point_sets
            .iter()
            .zip(q_eval_sets.iter())
            .zip(u.iter())
            .fold(
                C::Scalar::zero(),
                |msm_eval, ((points, evals), proof_eval)| {
                    let r_poly = lagrange_interpolate(points, evals);
                    let r_eval = eval_polynomial(&r_poly, *x_3);
                    let eval = points.iter().fold(*proof_eval - &r_eval, |eval, point| {
                        eval * &(*x_3 - point).invert().unwrap()
                    });
                    msm_eval * &(*x_2) + &eval
                },
            );

        // Sample a challenge x_4 that we will use to collapse the openings of
        // the various remaining polynomials at x_3 together.
        let x_4: ChallengeX4<_> = transcript.squeeze_challenge_scalar();

        // Compute the final commitment that has to be opened
        msm.append_term(C::Scalar::one(), q_prime_commitment.into());
        let (msm, v) = q_commitments.into_iter().zip(u.iter()).fold(
            (msm, msm_eval),
            |(mut msm, msm_eval), (q_commitment, q_eval)| {
                msm.scale(*x_4);
                msm.add_msm(&q_commitment);
                (msm, msm_eval * &(*x_4) + q_eval)
            },
        );

        // Verify the opening proof
        super::commitment::verify_proof(self.params, msm, transcript, *x_3, v)
    }
}
