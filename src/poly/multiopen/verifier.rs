use ff::Field;

use super::super::{
    commitment::{ChallengeScalar, ChallengeZ, Guard, Params, MSM},
    Error,
};
use super::{construct_intermediate_sets, ChallengeX1, ChallengeX2, Proof, Query, VerifierQuery};
use crate::arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt};
use crate::transcript::{Hasher, Transcript};

#[derive(Debug, Clone)]
struct CommitmentData<C: CurveAffine> {
    set_index: usize,
    point_indices: Vec<usize>,
    evals: Vec<C::Scalar>,
}

impl<C: CurveAffine> Proof<C> {
    /// Verify a multi-opening proof
    pub fn verify<'a, I, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        params: &'a Params<C>,
        transcript: &mut Transcript<C, HBase, HScalar>,
        queries: I,
        mut msm: MSM<'a, C>,
    ) -> Result<Guard<'a, C>, Error>
    where
        I: IntoIterator<Item = VerifierQuery<'a, C>> + Clone,
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
        transcript
            .absorb_point(&self.f_commitment)
            .map_err(|_| Error::SamplingError)?;

        // Sample a challenge z for checking that f(X) was committed to
        // correctly.
        let z = ChallengeZ::get(transcript);

        for eval in self.q_evals.iter() {
            transcript.absorb_scalar(*eval);
        }

        // We can compute the expected msm_eval at z using the q_evals provided
        // by the prover and from x_2
        let msm_eval = point_sets
            .iter()
            .zip(q_eval_sets.iter())
            .zip(self.q_evals.iter())
            .fold(
                C::Scalar::zero(),
                |msm_eval, ((points, evals), proof_eval)| {
                    let r_poly = lagrange_interpolate(points, evals);
                    let r_eval = eval_polynomial(&r_poly, *z);
                    let eval = points.iter().fold(*proof_eval - &r_eval, |eval, point| {
                        eval * &(*z - point).invert().unwrap()
                    });
                    msm_eval * &x_2 + &eval
                },
            );

        // Sample a challenge x_7 that we will use to collapse the openings of
        // the various remaining polynomials at z together.
        let x_7 = ChallengeScalar::<_, ()>::get(transcript);

        // Compute the final commitment that has to be opened
        let mut commitment_msm = params.empty_msm();
        commitment_msm.append_term(C::Scalar::one(), self.f_commitment);
        let (commitment_msm, msm_eval) = q_commitments.into_iter().zip(self.q_evals.iter()).fold(
            (commitment_msm, msm_eval),
            |(mut commitment_msm, msm_eval), (q_commitment, q_eval)| {
                commitment_msm.scale(*x_7);
                commitment_msm.add_msm(&q_commitment);
                (commitment_msm, msm_eval * &x_7 + q_eval)
            },
        );

        // Verify the opening proof
        self.opening
            .verify(params, msm, transcript, z, commitment_msm, msm_eval)
    }
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
