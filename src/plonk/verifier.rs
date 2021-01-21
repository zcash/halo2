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
use crate::transcript::{read_n_points, read_n_scalars, TranscriptRead};

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<'a, C: CurveAffine, T: TranscriptRead<C>>(
    params: &'a Params<C>,
    vk: &VerifyingKey<C>,
    msm: MSM<'a, C>,
    aux_commitments_vec: &[&[C]],
    transcript: &mut T,
) -> Result<Guard<'a, C>, Error> {
    // Check that aux_commitments matches the expected number of aux columns
    for aux_commitments in aux_commitments_vec.iter() {
        if aux_commitments.len() != vk.cs.num_aux_columns {
            return Err(Error::IncompatibleParams);
        }
    }

    let num_proofs = aux_commitments_vec.len();

    for aux_commitments in aux_commitments_vec.iter() {
        // Hash the aux (external) commitments into the transcript
        for commitment in *aux_commitments {
            transcript
                .common_point(*commitment)
                .map_err(|_| Error::TranscriptError)?
        }
    }

    let mut advice_commitments_vec = Vec::with_capacity(num_proofs);
    for _ in 0..num_proofs {
        // Hash the prover's advice commitments into the transcript
        let advice_commitments = read_n_points(transcript, vk.cs.num_advice_columns)
            .map_err(|_| Error::TranscriptError)?;
        advice_commitments_vec.push(advice_commitments);
    }

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta = ChallengeTheta::get(transcript);

    let mut lookups_permuted_vec = Vec::with_capacity(num_proofs);
    for _ in 0..num_proofs {
        // Hash each lookup permuted commitment
        let lookups = vk
            .cs
            .lookups
            .iter()
            .map(|argument| argument.read_permuted_commitments(transcript))
            .collect::<Result<Vec<_>, _>>()?;
        lookups_permuted_vec.push(lookups);
    }

    // Sample beta challenge
    let beta = ChallengeBeta::get(transcript);

    // Sample gamma challenge
    let gamma = ChallengeGamma::get(transcript);

    let mut permutations_committed_vec = Vec::with_capacity(num_proofs);
    for _ in 0..num_proofs {
        // Hash each permutation product commitment
        let permutations = vk
            .cs
            .permutations
            .iter()
            .map(|argument| argument.read_product_commitment(transcript))
            .collect::<Result<Vec<_>, _>>()?;
        permutations_committed_vec.push(permutations);
    }

    let mut lookups_committed_vec = Vec::with_capacity(num_proofs);
    for lookups in lookups_permuted_vec.into_iter() {
        // Hash each lookup product commitment
        let lookups = lookups
            .into_iter()
            .map(|lookup| lookup.read_product_commitment(transcript))
            .collect::<Result<Vec<_>, _>>()?;
        lookups_committed_vec.push(lookups);
    }

    // Sample y challenge, which keeps the gates linearly independent.
    let y = ChallengeY::get(transcript);

    let vanishing = vanishing::Argument::read_commitments(vk, transcript)?;

    // Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability.
    let x = ChallengeX::get(transcript);

    let mut aux_evals_vec = Vec::with_capacity(num_proofs);
    for _ in 0..num_proofs {
        let aux_evals = read_n_scalars(transcript, vk.cs.aux_queries.len())
            .map_err(|_| Error::TranscriptError)?;
        aux_evals_vec.push(aux_evals);
    }

    let mut advice_evals_vec = Vec::with_capacity(num_proofs);
    for _ in 0..num_proofs {
        let advice_evals = read_n_scalars(transcript, vk.cs.advice_queries.len())
            .map_err(|_| Error::TranscriptError)?;
        advice_evals_vec.push(advice_evals);
    }

    let fixed_evals = read_n_scalars(transcript, vk.cs.fixed_queries.len())
        .map_err(|_| Error::TranscriptError)?;

    let vanishing = vanishing.evaluate(transcript)?;

    let mut permutations_evaluated_vec = Vec::with_capacity(num_proofs);
    for permutations in permutations_committed_vec.into_iter() {
        let permutations = permutations
            .into_iter()
            .zip(vk.permutations.iter())
            .map(|(permutation, vkey)| permutation.evaluate(vkey, transcript))
            .collect::<Result<Vec<_>, _>>()?;
        permutations_evaluated_vec.push(permutations);
    }

    let mut lookups_evaluated_vec = Vec::with_capacity(num_proofs);
    for lookups in lookups_committed_vec.into_iter() {
        let lookups = lookups
            .into_iter()
            .map(|lookup| lookup.evaluate(transcript))
            .collect::<Result<Vec<_>, _>>()?;
        lookups_evaluated_vec.push(lookups);
    }

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
        let expressions = advice_evals_vec
            .iter()
            .zip(aux_evals_vec.iter())
            .zip(permutations_evaluated_vec.iter())
            .zip(lookups_evaluated_vec.iter())
            .flat_map(|(((advice_evals, aux_evals), permutations), lookups)| {
                let fixed_evals = fixed_evals.clone();
                let fixed_evals_copy = fixed_evals.clone();

                std::iter::empty()
                    // Evaluate the circuit using the custom gates provided
                    .chain(vk.cs.gates.iter().map(move |poly| {
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
                            .map(move |(p, argument)| {
                                p.expressions(vk, argument, &advice_evals, l_0, beta, gamma, x)
                            })
                            .into_iter()
                            .flatten(),
                    )
                    .chain(
                        lookups
                            .iter()
                            .zip(vk.cs.lookups.iter())
                            .map(move |(p, argument)| {
                                p.expressions(
                                    vk,
                                    l_0,
                                    argument,
                                    theta,
                                    beta,
                                    gamma,
                                    &advice_evals,
                                    &fixed_evals_copy,
                                    &aux_evals,
                                )
                            })
                            .into_iter()
                            .flatten(),
                    )
            })
            .collect::<Vec<_>>()
            .into_iter();

        vanishing.verify(expressions, y, xn)?;
    }

    let queries = aux_commitments_vec
        .iter()
        .zip(aux_evals_vec.iter())
        .zip(advice_commitments_vec.iter())
        .zip(advice_evals_vec.iter())
        .zip(permutations_evaluated_vec.iter())
        .zip(lookups_evaluated_vec.iter())
        .flat_map(
            |(
                ((((aux_commitments, aux_evals), advice_commitments), advice_evals), permutations),
                lookups,
            )| {
                iter::empty()
                    .chain(vk.cs.aux_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| VerifierQuery {
                            point: vk.domain.rotate_omega(*x, at),
                            commitment: &aux_commitments[column.index()],
                            eval: aux_evals[query_index],
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
                            .map(move |(p, vkey)| p.queries(vk, vkey, x))
                            .into_iter()
                            .flatten(),
                    )
                    .chain(
                        lookups
                            .iter()
                            .map(move |p| p.queries(vk, x))
                            .into_iter()
                            .flatten(),
                    )
            },
        )
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            vk.cs
                .fixed_queries
                .iter()
                .enumerate()
                .map(move |(query_index, &(column, at))| VerifierQuery {
                    point: vk.domain.rotate_omega(*x, at),
                    commitment: &vk.fixed_commitments[column.index()],
                    eval: fixed_evals.clone()[query_index],
                }),
        )
        .chain(vanishing.queries(x));

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.
    multiopen::verify_proof(params, transcript, queries, msm).map_err(|_| Error::OpeningError)
}
