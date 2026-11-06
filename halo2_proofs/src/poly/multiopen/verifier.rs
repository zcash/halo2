use ff::{BatchInvert, Field};

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

/// Computes the expected evaluation of the multi-point quotient at `x_3`.
///
/// Each point set contributes one Lagrange denominator per point and one
/// vanishing denominator. Inverting all of them together reduces this phase to
/// a single field inversion.
fn compute_msm_eval<F: Field>(
    point_sets: &[Vec<F>],
    q_eval_sets: &[Vec<F>],
    proof_evals: &[F],
    x_2: F,
    x_3: F,
) -> Result<F, Error> {
    assert_eq!(point_sets.len(), q_eval_sets.len());
    assert_eq!(point_sets.len(), proof_evals.len());

    let denominator_count = point_sets.iter().map(|points| points.len() + 1).sum();
    let mut denominators = Vec::with_capacity(denominator_count);

    for points in point_sets {
        for (j, x_j) in points.iter().enumerate() {
            let denominator = points
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != j)
                .fold(F::ONE, |denominator, (_, x_k)| denominator * (*x_j - x_k));
            assert!(
                !bool::from(denominator.is_zero()),
                "point sets must contain distinct points"
            );
            denominators.push(denominator);
        }

        let vanishing_denominator = points
            .iter()
            .fold(F::ONE, |denominator, point| denominator * (x_3 - point));
        if bool::from(vanishing_denominator.is_zero()) {
            return Err(Error::OpeningError);
        }
        denominators.push(vanishing_denominator);
    }

    if denominators.is_empty() {
        return Ok(F::ZERO);
    }

    denominators.iter_mut().batch_invert();
    let mut inverse_denominators = denominators.into_iter();

    let msm_eval = point_sets.iter().zip(q_eval_sets).zip(proof_evals).fold(
        F::ZERO,
        |msm_eval, ((points, evals), proof_eval)| {
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
                            .fold(F::ONE, |numerator, (_, point)| numerator * (x_3 - point));
                        let denominator_inv = inverse_denominators
                            .next()
                            .expect("one inverse denominator per interpolation point");

                        r_eval + *eval * numerator * denominator_inv
                    });
            let vanishing_denominator_inv = inverse_denominators
                .next()
                .expect("one inverse vanishing denominator per point set");
            let eval = (*proof_eval - r_eval) * vanishing_denominator_inv;

            msm_eval * x_2 + eval
        },
    );

    debug_assert!(inverse_denominators.next().is_none());
    Ok(msm_eval)
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
    let msm_eval = compute_msm_eval(&point_sets, &q_eval_sets, &u, *x_2, *x_3)?;

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
mod tests {
    use assert_matches::assert_matches;
    use ff::Field;
    use group::CurveAffine;

    use super::{
        ChallengeX1, ChallengeX2, ChallengeX3, Error, Params, VerifierQuery, compute_msm_eval,
        verify_proof,
    };
    use crate::{
        arithmetic::{eval_polynomial, lagrange_interpolate},
        pasta::{EqAffine, Fp},
        transcript::{Blake2bRead, Blake2bWrite, Challenge255, Transcript, TranscriptWrite},
    };

    fn reference_msm_eval(
        point_sets: &[Vec<Fp>],
        q_eval_sets: &[Vec<Fp>],
        proof_evals: &[Fp],
        x_2: Fp,
        x_3: Fp,
    ) -> Fp {
        point_sets.iter().zip(q_eval_sets).zip(proof_evals).fold(
            Fp::ZERO,
            |msm_eval, ((points, evals), proof_eval)| {
                let r_poly = lagrange_interpolate(points, evals);
                let r_eval = eval_polynomial(&r_poly, x_3);
                let eval = points.iter().fold(*proof_eval - r_eval, |eval, point| {
                    eval * (x_3 - point).invert().unwrap()
                });

                msm_eval * x_2 + eval
            },
        )
    }

    #[test]
    fn batched_msm_eval_matches_reference_for_orchard_shapes() {
        assert_eq!(
            compute_msm_eval(&[], &[], &[], Fp::from(2), Fp::from(3))
                .expect("empty input is supported"),
            reference_msm_eval(&[], &[], &[], Fp::from(2), Fp::from(3)),
        );

        for offset in 0_u64..16 {
            let points = [
                Fp::from(11 + offset),
                Fp::from(37 + 2 * offset),
                Fp::from(83 + 3 * offset),
                Fp::from(151 + 5 * offset),
            ];
            // These are the point-set sizes used by the Orchard verifier.
            let point_sets = vec![
                vec![points[0]],
                vec![points[0], points[1]],
                vec![points[0], points[2]],
                vec![points[0], points[1], points[2]],
                vec![points[0], points[1], points[3]],
            ];
            let q_eval_sets: Vec<Vec<_>> = point_sets
                .iter()
                .enumerate()
                .map(|(set, points)| {
                    points
                        .iter()
                        .enumerate()
                        .map(|(point, _)| Fp::from(100 + offset * 20 + set as u64 + point as u64))
                        .collect()
                })
                .collect();
            let proof_evals: Vec<_> = (0..point_sets.len())
                .map(|i| Fp::from(1_000 + offset * 10 + i as u64))
                .collect();
            let x_2 = Fp::from(2_000 + offset);
            let x_3 = Fp::from(3_000 + offset);

            assert_eq!(
                compute_msm_eval(&point_sets, &q_eval_sets, &proof_evals, x_2, x_3)
                    .expect("challenge is distinct from all query points"),
                reference_msm_eval(&point_sets, &q_eval_sets, &proof_evals, x_2, x_3),
            );
        }
    }

    #[test]
    fn verifier_rejects_challenge_in_point_set() {
        let params = Params::<EqAffine>::new(1);
        let commitment = EqAffine::generator();

        // Reproduce the verifier transcript prefix to obtain x_3, then use it
        // as the query point in a fresh verification call.
        let mut transcript =
            Blake2bWrite::<Vec<u8>, EqAffine, Challenge255<EqAffine>>::init(vec![]);
        let _: ChallengeX1<_> = transcript.squeeze_challenge_scalar();
        let _: ChallengeX2<_> = transcript.squeeze_challenge_scalar();
        transcript.write_point(commitment).unwrap();
        let x_3: ChallengeX3<_> = transcript.squeeze_challenge_scalar();
        transcript.write_scalar(Fp::ONE).unwrap();
        let proof = transcript.finalize();

        let mut transcript = Blake2bRead::<&[u8], EqAffine, Challenge255<EqAffine>>::init(&proof);
        let queries = [VerifierQuery::new_commitment(&commitment, *x_3, Fp::ONE)];
        assert_matches!(
            verify_proof(&params, &mut transcript, queries, params.empty_msm(),),
            Err(Error::OpeningError)
        );
    }
}
