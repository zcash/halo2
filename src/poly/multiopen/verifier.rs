use ff::Field;

use super::super::{
    commitment::{Guard, Params, MSM},
    Error,
};
use super::{
    construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4, Query,
    VerifierQuery,
};
use crate::arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt};
use crate::transcript::TranscriptRead;

use std::io::Read;

#[derive(Debug, Clone)]
struct CommitmentData<C: CurveAffine> {
    set_index: usize,
    point_indices: Vec<usize>,
    evals: Vec<C::Scalar>,
}

/// Verify a multi-opening proof
pub fn verify_proof<'b, 'a: 'b, I, C: CurveAffine, R: Read, T: TranscriptRead<R, C>>(
    params: &'a Params<C>,
    transcript: &mut T,
    queries: I,
    mut msm: MSM<'a, C>,
) -> Result<Guard<'a, C>, Error>
where
    I: IntoIterator<Item = VerifierQuery<'b, C>> + Clone,
{
    // Scale the MSM by a random factor to ensure that if the existing MSM
    // has is_zero() == false then this argument won't be able to interfere
    // with it to make it true, with high probability.
    msm.scale(C::Scalar::rand());

    // Sample x_1 for compressing openings at the same point sets together
    let x_1 = ChallengeX1::get(transcript);

    // Sample a challenge x_2 for keeping the multi-point quotient
    // polynomial terms linearly independent.
    let x_2 = ChallengeX2::get(transcript);

    let (commitment_map, point_sets) = construct_intermediate_sets(queries);

    // Compress the commitments and expected evaluations at x together.
    // using the challenge x_1
    let mut q_commitments: Vec<_> = vec![params.empty_msm(); point_sets.len()];

    // A vec of vecs of evals. The outer vec corresponds to the point set,
    // while the inner vec corresponds to the points in a particular set.
    let mut q_eval_sets = Vec::with_capacity(point_sets.len());
    for point_set in point_sets.iter() {
        q_eval_sets.push(vec![C::Scalar::zero(); point_set.len()]);
    }
    {
        let mut accumulate = |set_idx: usize, new_commitment, evals: Vec<C::Scalar>| {
            q_commitments[set_idx].scale(*x_1);
            q_commitments[set_idx].append_term(C::Scalar::one(), new_commitment);
            for (eval, set_eval) in evals.iter().zip(q_eval_sets[set_idx].iter_mut()) {
                *set_eval *= &x_1;
                *set_eval += eval;
            }
        };

        // Each commitment corresponds to evaluations at a set of points.
        // For each set, we collapse each commitment's evals pointwise.
        for commitment_data in commitment_map.into_iter() {
            accumulate(
                commitment_data.set_index,     // set_idx,
                *commitment_data.commitment.0, // commitment,
                commitment_data.evals,         // evals
            );
        }
    }

    // Obtain the commitment to the multi-point quotient polynomial f(X).
    let f_commitment = transcript.read_point().map_err(|_| Error::SamplingError)?;

    // Sample a challenge x_3 for checking that f(X) was committed to
    // correctly.
    let x_3 = ChallengeX3::get(transcript);

    let mut q_evals = Vec::with_capacity(q_eval_sets.len());
    for _ in 0..q_eval_sets.len() {
        q_evals.push(transcript.read_scalar().map_err(|_| Error::SamplingError)?);
    }

    // We can compute the expected msm_eval at x_3 using the q_evals provided
    // by the prover and from x_2
    let msm_eval = point_sets
        .iter()
        .zip(q_eval_sets.iter())
        .zip(q_evals.iter())
        .fold(
            C::Scalar::zero(),
            |msm_eval, ((points, evals), proof_eval)| {
                let r_poly = lagrange_interpolate(points, evals);
                let r_eval = eval_polynomial(&r_poly, *x_3);
                let eval = points.iter().fold(*proof_eval - &r_eval, |eval, point| {
                    eval * &(*x_3 - point).invert().unwrap()
                });
                msm_eval * &x_2 + &eval
            },
        );

    // Sample a challenge x_4 that we will use to collapse the openings of
    // the various remaining polynomials at x_3 together.
    let x_4 = ChallengeX4::get(transcript);

    // Compute the final commitment that has to be opened
    let mut commitment_msm = params.empty_msm();
    commitment_msm.append_term(C::Scalar::one(), f_commitment);
    let (commitment_msm, msm_eval) = q_commitments.into_iter().zip(q_evals.iter()).fold(
        (commitment_msm, msm_eval),
        |(mut commitment_msm, msm_eval), (q_commitment, q_eval)| {
            commitment_msm.scale(*x_4);
            commitment_msm.add_msm(&q_commitment);
            (commitment_msm, msm_eval * &x_4 + q_eval)
        },
    );

    // Verify the opening proof
    super::commitment::verify_proof(params, msm, transcript, *x_3, commitment_msm, msm_eval)
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct CommitmentPointer<'a, C>(&'a C);

impl<'a, C> PartialEq for CommitmentPointer<'a, C> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a, C: CurveAffine> Query<C::Scalar> for VerifierQuery<'a, C> {
    type Commitment = CommitmentPointer<'a, C>;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        CommitmentPointer(self.commitment)
    }
}
