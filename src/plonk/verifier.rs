use ff::Field;
use std::iter;

use super::{
    vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, ChallengeY, Error,
    VerifyingKey,
};
use crate::arithmetic::{CurveAffine, FieldExt};
use crate::poly::{
    commitment::{Guard, Params, MSM},
    multiopen::{self, VerifierQuery},
};
use crate::transcript::{read_n_points, read_n_scalars, EncodedChallenge, TranscriptRead};

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<
    'a,
    C: CurveAffine,
    E: EncodedChallenge<C, [u8; 64]>,
    T: TranscriptRead<C, [u8; 64], E>,
>(
    params: &'a Params<C>,
    vk: &VerifyingKey<C>,
    msm: MSM<'a, C>,
    instance_commitments: &[&[C]],
    transcript: &mut T,
) -> Result<Guard<'a, C, [u8; 64], E>, Error> {
    // Check that instance_commitments matches the expected number of instance columns
    for instance_commitments in instance_commitments.iter() {
        if instance_commitments.len() != vk.cs.num_instance_columns {
            return Err(Error::IncompatibleParams);
        }
    }

    let num_proofs = instance_commitments.len();

    // Hash verification key into transcript
    vk.hash_into(transcript)
        .map_err(|_| Error::TranscriptError)?;

    for instance_commitments in instance_commitments.iter() {
        // Hash the instance (external) commitments into the transcript
        for commitment in *instance_commitments {
            transcript
                .common_point(*commitment)
                .map_err(|_| Error::TranscriptError)?
        }
    }

    let advice_commitments = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash the prover's advice commitments into the transcript
            read_n_points(transcript, vk.cs.num_advice_columns).map_err(|_| Error::TranscriptError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta: ChallengeTheta<_> = transcript.squeeze_challenge_scalar();

    let lookups_permuted = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash each lookup permuted commitment
            vk.cs
                .lookups
                .iter()
                .map(|argument| argument.read_permuted_commitments(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample beta challenge
    let beta: ChallengeBeta<_> = transcript.squeeze_challenge_scalar();

    // Sample gamma challenge
    let gamma: ChallengeGamma<_> = transcript.squeeze_challenge_scalar();

    let permutations_committed = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash each permutation product commitment
            vk.cs
                .permutations
                .iter()
                .map(|argument| argument.read_product_commitment(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let lookups_committed = lookups_permuted
        .into_iter()
        .map(|lookups| {
            // Hash each lookup product commitment
            lookups
                .into_iter()
                .map(|lookup| lookup.read_product_commitment(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample y challenge, which keeps the gates linearly independent.
    let y: ChallengeY<_> = transcript.squeeze_challenge_scalar();
    let vanishing = vanishing::Argument::read_commitments(vk, transcript)?;

    // Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability.
    let x: ChallengeX<_> = transcript.squeeze_challenge_scalar();
    let instance_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            read_n_scalars(transcript, vk.cs.instance_queries.len())
                .map_err(|_| Error::TranscriptError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let advice_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            read_n_scalars(transcript, vk.cs.advice_queries.len())
                .map_err(|_| Error::TranscriptError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let fixed_evals = read_n_scalars(transcript, vk.cs.fixed_queries.len())
        .map_err(|_| Error::TranscriptError)?;

    let vanishing = vanishing.evaluate(transcript)?;

    let permutations_evaluated = permutations_committed
        .into_iter()
        .map(|permutations| -> Result<Vec<_>, _> {
            permutations
                .into_iter()
                .zip(vk.permutations.iter())
                .map(|(permutation, vkey)| permutation.evaluate(vkey, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let lookups_evaluated = lookups_committed
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            lookups
                .into_iter()
                .map(|lookup| lookup.evaluate(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // This check ensures the circuit is satisfied so long as the polynomial
    // commitments open to the correct values.
    {
        // x^n
        let xn = x.pow(&[params.n as u64, 0, 0, 0]);

        // TODO: bubble this error up
        // l_0(x)
        let l_0 = (*x - &C::Scalar::one()).invert().unwrap() // 1 / (x - 1)
            * &(xn - &C::Scalar::one()) // (x^n - 1) / (x - 1)
            * &vk.domain.get_barycentric_weight(); // l_0(x)

        // Compute the expected value of h(x)
        let expressions = advice_evals
            .iter()
            .zip(instance_evals.iter())
            .zip(permutations_evaluated.iter())
            .zip(lookups_evaluated.iter())
            .flat_map(
                |(((advice_evals, instance_evals), permutations), lookups)| {
                    let fixed_evals = fixed_evals.clone();
                    let fixed_evals_copy = fixed_evals.clone();
                    let fixed_evals_copy_copy = fixed_evals.clone();

                    std::iter::empty()
                        // Evaluate the circuit using the custom gates provided
                        .chain(vk.cs.gates.iter().map(move |(_, poly)| {
                            poly.evaluate(
                                &|scalar| scalar,
                                &|index| fixed_evals[index],
                                &|index| advice_evals[index],
                                &|index| instance_evals[index],
                                &|a, b| a + &b,
                                &|a, b| a * &b,
                                &|a, scalar| a * &scalar,
                            )
                        }))
                        .chain(
                            permutations
                                .iter()
                                .zip(vk.cs.permutations.iter())
                                .flat_map(move |(p, argument)| {
                                    p.expressions(
                                        vk,
                                        argument,
                                        &advice_evals,
                                        &fixed_evals_copy,
                                        &instance_evals,
                                        l_0,
                                        beta,
                                        gamma,
                                        x,
                                    )
                                })
                                .into_iter(),
                        )
                        .chain(
                            lookups
                                .iter()
                                .zip(vk.cs.lookups.iter())
                                .flat_map(move |(p, argument)| {
                                    p.expressions(
                                        l_0,
                                        argument,
                                        theta,
                                        beta,
                                        gamma,
                                        &advice_evals,
                                        &fixed_evals_copy_copy,
                                        &instance_evals,
                                    )
                                })
                                .into_iter(),
                        )
                },
            );

        vanishing.verify(expressions, y, xn)?;
    }

    let queries = instance_commitments
        .iter()
        .zip(instance_evals.iter())
        .zip(advice_commitments.iter())
        .zip(advice_evals.iter())
        .zip(permutations_evaluated.iter())
        .zip(lookups_evaluated.iter())
        .flat_map(
            |(
                (
                    (((instance_commitments, instance_evals), advice_commitments), advice_evals),
                    permutations,
                ),
                lookups,
            )| {
                iter::empty()
                    .chain(vk.cs.instance_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| VerifierQuery {
                            point: vk.domain.rotate_omega(*x, at),
                            commitment: &instance_commitments[column.index()],
                            eval: instance_evals[query_index],
                        },
                    ))
                    .chain(vk.cs.advice_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| VerifierQuery {
                            point: vk.domain.rotate_omega(*x, at),
                            commitment: &advice_commitments[column.index()],
                            eval: advice_evals[query_index],
                        },
                    ))
                    .chain(
                        permutations
                            .iter()
                            .zip(vk.permutations.iter())
                            .flat_map(move |(p, vkey)| p.queries(vk, vkey, x))
                            .into_iter(),
                    )
                    .chain(
                        lookups
                            .iter()
                            .flat_map(move |p| p.queries(vk, x))
                            .into_iter(),
                    )
            },
        )
        .chain(
            vk.cs
                .fixed_queries
                .iter()
                .enumerate()
                .map(|(query_index, &(column, at))| VerifierQuery {
                    point: vk.domain.rotate_omega(*x, at),
                    commitment: &vk.fixed_commitments[column.index()],
                    eval: fixed_evals[query_index],
                }),
        )
        .chain(vanishing.queries(x));

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.
    multiopen::verify_proof(params, transcript, queries, msm).map_err(|_| Error::OpeningError)
}
