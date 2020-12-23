use ff::Field;
use std::io::Read;
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
use crate::transcript::{read_n_points, read_n_scalars, TranscriptRead};

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<'a, C: CurveAffine, R: Read, T: TranscriptRead<R, C>>(
    params: &'a Params<C>,
    vk: &VerifyingKey<C>,
    msm: MSM<'a, C>,
    aux_commitments: &[C],
    transcript: &mut T,
) -> Result<Guard<'a, C>, Error> {
    // Check that aux_commitments matches the expected number of aux columns
    if aux_commitments.len() != vk.cs.num_aux_columns {
        return Err(Error::IncompatibleParams);
    }

    // Hash the aux (external) commitments into the transcript
    for commitment in aux_commitments {
        transcript
            .common_point(*commitment)
            .map_err(|_| Error::TranscriptError)?
    }

    // Hash the prover's advice commitments into the transcript
    let advice_commitments =
        read_n_points(transcript, vk.cs.num_advice_columns).map_err(|_| Error::TranscriptError)?;

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta = ChallengeTheta::get(transcript);

    // Hash each lookup permuted commitment
    let lookups = vk
        .cs
        .lookups
        .iter()
        .map(|argument| argument.absorb_permuted_commitments(transcript))
        .collect::<Result<Vec<_>, _>>()?;

    // Sample beta challenge
    let beta = ChallengeBeta::get(transcript);

    // Sample gamma challenge
    let gamma = ChallengeGamma::get(transcript);

    // Hash each permutation product commitment
    let permutations = vk
        .cs
        .permutations
        .iter()
        .map(|argument| argument.absorb_product_commitment(transcript))
        .collect::<Result<Vec<_>, _>>()?;

    // Hash each lookup product commitment
    let lookups = lookups
        .into_iter()
        .map(|lookup| lookup.absorb_product_commitment(transcript))
        .collect::<Result<Vec<_>, _>>()?;

    // Sample y challenge, which keeps the gates linearly independent.
    let y = ChallengeY::get(transcript);

    let vanishing = vanishing::Argument::absorb_commitments(vk, transcript)?;

    // Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability.
    let x = ChallengeX::get(transcript);

    let advice_evals = read_n_scalars(transcript, vk.cs.advice_queries.len())
        .map_err(|_| Error::TranscriptError)?;
    let aux_evals =
        read_n_scalars(transcript, vk.cs.aux_queries.len()).map_err(|_| Error::TranscriptError)?;
    let fixed_evals = read_n_scalars(transcript, vk.cs.fixed_queries.len())
        .map_err(|_| Error::TranscriptError)?;

    let vanishing = vanishing.evaluate(transcript)?;

    let permutations = permutations
        .into_iter()
        .zip(vk.permutations.iter())
        .map(|(permutation, vkey)| permutation.evaluate(vkey, transcript))
        .collect::<Result<Vec<_>, _>>()?;

    let lookups = lookups
        .into_iter()
        .map(|lookup| lookup.evaluate(transcript))
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
        let expressions = std::iter::empty()
            // Evaluate the circuit using the custom gates provided
            .chain(vk.cs.gates.iter().map(|poly| {
                poly.evaluate(
                    &|index| fixed_evals[index],
                    &|index| advice_evals[index],
                    &|index| aux_evals[index],
                    &|a, b| a + &b,
                    &|a, b| a * &b,
                    &|a, scalar| a * &scalar,
                )
            }))
            .chain(
                permutations
                    .iter()
                    .zip(vk.cs.permutations.iter())
                    .map(|(p, argument)| {
                        p.expressions(vk, argument, &advice_evals, l_0, beta, gamma, x)
                    })
                    .into_iter()
                    .flatten(),
            )
            .chain(
                lookups
                    .iter()
                    .zip(vk.cs.lookups.iter())
                    .map(|(p, argument)| {
                        p.expressions(
                            vk,
                            l_0,
                            argument,
                            theta,
                            beta,
                            gamma,
                            &advice_evals,
                            &fixed_evals,
                            &aux_evals,
                        )
                    })
                    .into_iter()
                    .flatten(),
            );

        vanishing.verify(expressions, y, xn)?;
    }

    let queries = iter::empty()
        .chain(
            vk.cs
                .advice_queries
                .iter()
                .enumerate()
                .map(|(query_index, &(column, at))| VerifierQuery {
                    point: vk.domain.rotate_omega(*x, at),
                    commitment: &advice_commitments[column.index()],
                    eval: advice_evals[query_index],
                }),
        )
        .chain(
            vk.cs
                .aux_queries
                .iter()
                .enumerate()
                .map(|(query_index, &(column, at))| VerifierQuery {
                    point: vk.domain.rotate_omega(*x, at),
                    commitment: &aux_commitments[column.index()],
                    eval: aux_evals[query_index],
                }),
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
        .chain(vanishing.queries(x))
        .chain(
            permutations
                .iter()
                .zip(vk.permutations.iter())
                .map(|(p, vkey)| p.queries(vk, vkey, x))
                .into_iter()
                .flatten(),
        )
        .chain(
            lookups
                .iter()
                .map(|p| p.queries(vk, x))
                .into_iter()
                .flatten(),
        );

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.
    multiopen::verify_proof(params, transcript, queries, msm).map_err(|_| Error::OpeningError)
}
