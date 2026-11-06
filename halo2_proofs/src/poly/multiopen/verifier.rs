use ff::Field;
use pasta_curves::arithmetic::{VartimeBatchInvert, VartimeField};

use super::super::{
    commitment::{Guard, Params, MSM},
    Error,
};
use super::{
    construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4,
    CommitmentReference, Query, VerifierQuery,
};
use crate::arithmetic::CurveAffine;
use crate::transcript::{EncodedChallenge, TranscriptRead};

/// Evaluates the multi-point quotient with one inversion for all Lagrange
/// denominators and vanishing polynomials, without allocating interpolants.
fn compute_msm_eval<F: VartimeField>(
    point_sets: &[Vec<F>],
    q_eval_sets: &[Vec<F>],
    proof_evals: &[F],
    x_2: F,
    x_3: F,
) -> F {
    assert_eq!(point_sets.len(), q_eval_sets.len());
    assert_eq!(point_sets.len(), proof_evals.len());
    let count = point_sets.iter().map(|points| points.len() + 1).sum();
    let mut denominators = Vec::with_capacity(count);
    for points in point_sets {
        for (j, x_j) in points.iter().enumerate() {
            let denominator = points
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != j)
                .fold(F::ONE, |acc, (_, x_k)| acc * (*x_j - x_k));
            assert!(
                !denominator.is_zero_vartime(),
                "point sets must contain distinct points"
            );
            denominators.push(denominator);
        }
        let vanishing = points.iter().fold(F::ONE, |acc, point| acc * (x_3 - point));
        // Preserve the previous verifier's failure on a zero denominator.
        assert!(
            !vanishing.is_zero_vartime(),
            "verifier challenge must differ from query points"
        );
        denominators.push(vanishing);
    }
    denominators.iter_mut().batch_invert_vartime();
    let mut inverses = denominators.into_iter();
    point_sets.iter().zip(q_eval_sets).zip(proof_evals).fold(
        F::ZERO,
        |acc, ((points, evals), proof_eval)| {
            assert_eq!(points.len(), evals.len());
            let r_eval =
                points
                    .iter()
                    .enumerate()
                    .zip(evals)
                    .fold(F::ZERO, |r_eval, ((j, _), eval)| {
                        let numerator = points
                            .iter()
                            .enumerate()
                            .filter(|(k, _)| *k != j)
                            .fold(F::ONE, |acc, (_, point)| acc * (x_3 - point));
                        let inverse = inverses
                            .next()
                            .expect("one denominator per interpolation point");
                        r_eval + *eval * numerator * inverse
                    });
            let inverse = inverses
                .next()
                .expect("one vanishing denominator per point set");
            acc * x_2 + (*proof_eval - r_eval) * inverse
        },
    )
}

/// Verify a multi-opening proof
pub fn verify_proof<
    'r,
    'params: 'r,
    I,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    T: TranscriptRead<C, E>,
>(
    params: &'params Params<C>,
    transcript: &mut T,
    queries: I,
    mut msm: MSM<'params, C>,
) -> Result<Guard<'params, C, E>, Error>
where
    I: IntoIterator<Item = VerifierQuery<'r, 'params, C>> + Clone,
{
    // Sample x_1 for compressing openings at the same point sets together
    let x_1: ChallengeX1<_> = transcript.squeeze_challenge_scalar();

    // Sample a challenge x_2 for keeping the multi-point quotient
    // polynomial terms linearly independent.
    let x_2: ChallengeX2<_> = transcript.squeeze_challenge_scalar();

    let (commitment_map, point_sets) =
        construct_intermediate_sets(queries).ok_or(Error::OpeningError)?;

    // Compress the commitments and expected evaluations at x together.
    // using the challenge x_1
    let mut q_commitments: Vec<_> = vec![
        (params.empty_msm(), C::Scalar::ONE); // (accumulator, next x_1 power).
        point_sets.len()];

    // A vec of vecs of evals. The outer vec corresponds to the point set,
    // while the inner vec corresponds to the points in a particular set.
    let mut q_eval_sets = Vec::with_capacity(point_sets.len());
    for point_set in point_sets.iter() {
        q_eval_sets.push(vec![C::Scalar::ZERO; point_set.len()]);
    }

    {
        let mut accumulate = |set_idx: usize, new_commitment, evals: Vec<C::Scalar>| {
            let (q_commitment, x_1_power) = &mut q_commitments[set_idx];
            match new_commitment {
                CommitmentReference::Commitment(c) => {
                    q_commitment.append_term(*x_1_power, *c);
                }
                CommitmentReference::MSM(msm) => {
                    let mut msm = msm.clone();
                    msm.scale(*x_1_power);
                    q_commitment.add_msm(&msm);
                }
            }
            for (eval, set_eval) in evals.iter().zip(q_eval_sets[set_idx].iter_mut()) {
                *set_eval += (*eval) * (*x_1_power);
            }
            *x_1_power *= *x_1;
        };

        // Each commitment corresponds to evaluations at a set of points.
        // For each set, we collapse each commitment's evals pointwise.
        // Run in order of increasing x_1 powers.
        for commitment_data in commitment_map.into_iter().rev() {
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
    let msm_eval = compute_msm_eval(&point_sets, &q_eval_sets, &u, *x_2, *x_3);

    // Sample a challenge x_4 that we will use to collapse the openings of
    // the various remaining polynomials at x_3 together.
    let x_4: ChallengeX4<_> = transcript.squeeze_challenge_scalar();

    // Compute the final commitment that has to be opened
    msm.append_term(C::Scalar::ONE, q_prime_commitment);
    let (msm, v) = q_commitments.into_iter().zip(u.iter()).fold(
        (msm, msm_eval),
        |(mut msm, msm_eval), ((q_commitment, _), q_eval)| {
            msm.scale(*x_4);
            msm.add_msm(&q_commitment);
            (msm, msm_eval * &(*x_4) + q_eval)
        },
    );

    // Verify the opening proof
    super::commitment::verify_proof(params, msm, transcript, *x_3, v)
}

impl<'a, 'b, C: CurveAffine> Query<C::Scalar> for VerifierQuery<'a, 'b, C> {
    type Commitment = CommitmentReference<'a, 'b, C>;
    type Eval = C::Scalar;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        self.commitment
    }
}

#[cfg(test)]
mod tests;
