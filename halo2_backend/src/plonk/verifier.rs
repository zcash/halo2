//! Verify a plonk proof

use group::prime::PrimeCurveAffine;
use group::Curve;
use halo2_middleware::circuit::Any;
use halo2_middleware::ff::{Field, FromUniformBytes, WithSmallOrderMulGroup};
use halo2_middleware::zal::impls::H2cEngine;
use halo2curves::CurveAffine;
use std::iter;

use super::{vanishing, VerifyingKey};
use crate::arithmetic::compute_inner_product;
use crate::plonk::{
    circuit::VarBack, lookup::verifier::lookup_read_permuted_commitments,
    permutation::verifier::permutation_read_product_commitments,
    shuffle::verifier::shuffle_read_product_commitment, ChallengeBeta, ChallengeGamma,
    ChallengeTheta, ChallengeX, ChallengeY, Error,
};
use crate::poly::commitment::ParamsVerifier;
use crate::poly::{
    commitment::{Blind, CommitmentScheme, Params, Verifier},
    VerificationStrategy, VerifierQuery,
};
use crate::transcript::{read_n_scalars, EncodedChallenge, TranscriptRead};

#[cfg(feature = "batch")]
mod batch;
#[cfg(feature = "batch")]
pub use batch::BatchVerifier;

/// Returns a boolean indicating whether or not the proof is valid.  Verifies a single proof (not
/// batched).
pub fn verify_proof_single<'params, Scheme, V, E, T, Strategy>(
    params: &'params Scheme::ParamsVerifier,
    vk: &VerifyingKey<Scheme::Curve>,
    strategy: Strategy,
    instance: Vec<Vec<Scheme::Scalar>>,
    transcript: &mut T,
) -> Result<Strategy::Output, Error>
where
    Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    Scheme: CommitmentScheme,
    V: Verifier<'params, Scheme>,
    E: EncodedChallenge<Scheme::Curve>,
    T: TranscriptRead<Scheme::Curve, E>,
    Strategy: VerificationStrategy<'params, Scheme, V>,
{
    verify_proof(params, vk, strategy, &[instance], transcript)
}

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<
    'params,
    Scheme: CommitmentScheme,
    V: Verifier<'params, Scheme>,
    E: EncodedChallenge<Scheme::Curve>,
    T: TranscriptRead<Scheme::Curve, E>,
    Strategy: VerificationStrategy<'params, Scheme, V>,
>(
    params: &'params Scheme::ParamsVerifier,
    vk: &VerifyingKey<Scheme::Curve>,
    strategy: Strategy,
    instances: &[Vec<Vec<Scheme::Scalar>>],
    transcript: &mut T,
) -> Result<Strategy::Output, Error>
where
    Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
{
    // ZAL: Verification is (supposedly) cheap, hence we don't use an accelerator engine
    let default_engine = H2cEngine::new();

    // Check that instances matches the expected number of instance columns
    for instances in instances.iter() {
        if instances.len() != vk.cs.num_instance_columns {
            return Err(Error::InvalidInstances);
        }
    }

    // Check that the Scheme parameters support commitment to instance
    // if it is required by the verifier.
    assert!(
        !V::QUERY_INSTANCE
            || <Scheme::ParamsVerifier as ParamsVerifier<Scheme::Curve>>::COMMIT_INSTANCE
    );

    // 1. Get the commitments of the instance polynomials. ----------------------------------------

    let instance_commitments = if V::QUERY_INSTANCE {
        let mut instance_commitments = Vec::with_capacity(instances.len());

        let instances_projective = instances
            .iter()
            .map(|instance| {
                instance
                    .iter()
                    .map(|instance| {
                        if instance.len() > params.n() as usize - (vk.cs.blinding_factors() + 1) {
                            return Err(Error::InstanceTooLarge);
                        }
                        let mut poly = instance.to_vec();
                        poly.resize(params.n() as usize, Scheme::Scalar::ZERO);
                        let poly = vk.domain.lagrange_from_vec(poly);

                        Ok(params.commit_lagrange(&default_engine, &poly, Blind::default()))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        for instance_projective in instances_projective {
            let mut affines =
                vec![<Scheme as CommitmentScheme>::Curve::identity(); instance_projective.len()];
            <<Scheme as CommitmentScheme>::Curve as CurveAffine>::CurveExt::batch_normalize(
                &instance_projective,
                &mut affines,
            );
            instance_commitments.push(affines);
        }
        instance_commitments
    } else {
        vec![vec![]; instances.len()]
    };

    let num_proofs = instance_commitments.len();

    // 2. Add hash of verification key and instances into transcript. -----------------------------
    // [TRANSCRIPT-1]

    vk.hash_into(transcript)?;

    // 3. Add instance commitments into the transcript. --------------------------------------------
    // [TRANSCRIPT-2]

    if V::QUERY_INSTANCE {
        for instance_commitments in instance_commitments.iter() {
            // Hash the instance (external) commitments into the transcript
            for commitment in instance_commitments {
                transcript.common_point(*commitment)?
            }
        }
    } else {
        for instance in instances.iter() {
            for instance in instance.iter() {
                for value in instance.iter() {
                    transcript.common_scalar(*value)?;
                }
            }
        }
    }

    // 3. Hash the prover's advice commitments into the transcript and squeeze challenges ---------

    let (advice_commitments, challenges) = {
        let mut advice_commitments =
            vec![vec![Scheme::Curve::default(); vk.cs.num_advice_columns]; num_proofs];
        let mut challenges = vec![Scheme::Scalar::ZERO; vk.cs.num_challenges];

        for current_phase in vk.cs.phases() {
            // [TRANSCRIPT-3]
            for advice_commitments in advice_commitments.iter_mut() {
                for (phase, commitment) in vk
                    .cs
                    .advice_column_phase
                    .iter()
                    .zip(advice_commitments.iter_mut())
                {
                    if current_phase == *phase {
                        *commitment = transcript.read_point()?;
                    }
                }
            }

            // [TRANSCRIPT-4]
            for (phase, challenge) in vk.cs.challenge_phase.iter().zip(challenges.iter_mut()) {
                if current_phase == *phase {
                    *challenge = *transcript.squeeze_challenge_scalar::<()>();
                }
            }
        }

        (advice_commitments, challenges)
    };

    // 4. Sample theta challenge for keeping lookup columns linearly independent ------------------
    // [TRANSCRIPT-5]

    let theta: ChallengeTheta<_> = transcript.squeeze_challenge_scalar();

    // 5. Read lookup permuted commitments
    // [TRANSCRIPT-6]

    let lookups_permuted = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash each lookup permuted commitment
            vk.cs
                .lookups
                .iter()
                .map(|_argument| lookup_read_permuted_commitments(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // 6. Sample beta and gamma challenges --------------------------------------------------------

    // Sample beta challenge
    // [TRANSCRIPT-7]
    let beta: ChallengeBeta<_> = transcript.squeeze_challenge_scalar();

    // Sample gamma challenge
    // [TRANSCRIPT-8]
    let gamma: ChallengeGamma<_> = transcript.squeeze_challenge_scalar();

    // 7. Read commitments for permutation, lookups, and shuffles ---------------------------------

    // [TRANSCRIPT-9]
    let permutations_committed = (0..num_proofs)
        .map(|_| {
            // Hash each permutation product commitment
            permutation_read_product_commitments(&vk.cs.permutation, vk, transcript)
        })
        .collect::<Result<Vec<_>, _>>()?;

    // [TRANSCRIPT-10]
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

    // [TRANSCRIPT-11]
    let shuffles_committed = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash each shuffle product commitment
            vk.cs
                .shuffles
                .iter()
                .map(|_argument| shuffle_read_product_commitment(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // 8. Read vanishing argument (before y) ------------------------------------------------------
    // [TRANSCRIPT-12]
    let vanishing = vanishing::Argument::read_commitments_before_y(transcript)?;

    // 9. Sample y challenge, which keeps the gates linearly independent. -------------------------
    // [TRANSCRIPT-13]
    let y: ChallengeY<_> = transcript.squeeze_challenge_scalar();

    // 10. Read vanishing argument (after y) ------------------------------------------------------
    // [TRANSCRIPT-14]
    let vanishing = vanishing.read_commitments_after_y(vk, transcript)?;

    // 11. Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability. -----------------------------------------------------------
    // [TRANSCRIPT-15]
    let x: ChallengeX<_> = transcript.squeeze_challenge_scalar();

    // 12. Get the instance evaluations
    let instance_evals = if V::QUERY_INSTANCE {
        // [TRANSCRIPT-16]
        (0..num_proofs)
            .map(|_| -> Result<Vec<_>, _> {
                read_n_scalars(transcript, vk.cs.instance_queries.len())
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let xn = x.pow([params.n()]);
        let (min_rotation, max_rotation) =
            vk.cs
                .instance_queries
                .iter()
                .fold((0, 0), |(min, max), (_, rotation)| {
                    if rotation.0 < min {
                        (rotation.0, max)
                    } else if rotation.0 > max {
                        (min, rotation.0)
                    } else {
                        (min, max)
                    }
                });
        let max_instance_len = instances
            .iter()
            .flat_map(|instance| instance.iter().map(|instance| instance.len()))
            .max_by(Ord::cmp)
            .unwrap_or_default();
        let l_i_s = &vk.domain.l_i_range(
            *x,
            xn,
            -max_rotation..max_instance_len as i32 + min_rotation.abs(),
        );
        instances
            .iter()
            .map(|instances| {
                vk.cs
                    .instance_queries
                    .iter()
                    .map(|(column, rotation)| {
                        let instances = &instances[column.index];
                        let offset = (max_rotation - rotation.0) as usize;
                        compute_inner_product(
                            instances.as_slice(),
                            &l_i_s[offset..offset + instances.len()],
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };

    // [TRANSCRIPT-17]
    let advice_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> { read_n_scalars(transcript, vk.cs.advice_queries.len()) })
        .collect::<Result<Vec<_>, _>>()?;

    // [TRANSCRIPT-18]
    let fixed_evals = read_n_scalars(transcript, vk.cs.fixed_queries.len())?;

    // [TRANSCRIPT-19]
    let vanishing = vanishing.evaluate_after_x(transcript)?;

    // [TRANSCRIPT-20]
    let permutations_common = vk.permutation.evaluate(transcript)?;

    // [TRANSCRIPT-21]
    let permutations_evaluated = permutations_committed
        .into_iter()
        .map(|permutation| permutation.evaluate(transcript))
        .collect::<Result<Vec<_>, _>>()?;

    // [TRANSCRIPT-22]
    let lookups_evaluated = lookups_committed
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            lookups
                .into_iter()
                .map(|lookup| lookup.evaluate(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // [TRANSCRIPT-23]
    let shuffles_evaluated = shuffles_committed
        .into_iter()
        .map(|shuffles| -> Result<Vec<_>, _> {
            shuffles
                .into_iter()
                .map(|shuffle| shuffle.evaluate(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // This check ensures the circuit is satisfied so long as the polynomial
    // commitments open to the correct values.
    let vanishing = {
        // x^n
        let xn = x.pow([params.n()]);

        let blinding_factors = vk.cs.blinding_factors();
        let l_evals = vk
            .domain
            .l_i_range(*x, xn, (-((blinding_factors + 1) as i32))..=0);
        assert_eq!(l_evals.len(), 2 + blinding_factors);
        let l_last = l_evals[0];
        let l_blind: Scheme::Scalar = l_evals[1..(1 + blinding_factors)]
            .iter()
            .fold(Scheme::Scalar::ZERO, |acc, eval| acc + eval);
        let l_0 = l_evals[1 + blinding_factors];

        // Compute the expected value of h(x)
        let expressions = advice_evals
            .iter()
            .zip(instance_evals.iter())
            .zip(permutations_evaluated.iter())
            .zip(lookups_evaluated.iter())
            .zip(shuffles_evaluated.iter())
            .flat_map(
                |((((advice_evals, instance_evals), permutation), lookups), shuffles)| {
                    let challenges = &challenges;
                    let fixed_evals = &fixed_evals;
                    std::iter::empty()
                        // Evaluate the circuit using the custom gates provided
                        .chain(vk.cs.gates.iter().map(move |gate| {
                            gate.poly.evaluate(
                                &|scalar| scalar,
                                &|var| match var {
                                    VarBack::Query(query) => match query.column_type {
                                        Any::Fixed => fixed_evals[query.index],
                                        Any::Advice => advice_evals[query.index],
                                        Any::Instance => instance_evals[query.index],
                                    },
                                    VarBack::Challenge(challenge) => challenges[challenge.index],
                                },
                                &|a| -a,
                                &|a, b| a + b,
                                &|a, b| a * b,
                            )
                        }))
                        .chain(permutation.expressions(
                            vk,
                            &vk.cs.permutation,
                            &permutations_common,
                            advice_evals,
                            fixed_evals,
                            instance_evals,
                            l_0,
                            l_last,
                            l_blind,
                            beta,
                            gamma,
                            x,
                        ))
                        .chain(lookups.iter().zip(vk.cs.lookups.iter()).flat_map(
                            move |(p, argument)| {
                                p.expressions(
                                    l_0,
                                    l_last,
                                    l_blind,
                                    argument,
                                    theta,
                                    beta,
                                    gamma,
                                    advice_evals,
                                    fixed_evals,
                                    instance_evals,
                                    challenges,
                                )
                            },
                        ))
                        .chain(shuffles.iter().zip(vk.cs.shuffles.iter()).flat_map(
                            move |(p, argument)| {
                                p.expressions(
                                    l_0,
                                    l_last,
                                    l_blind,
                                    argument,
                                    theta,
                                    gamma,
                                    advice_evals,
                                    fixed_evals,
                                    instance_evals,
                                    challenges,
                                )
                            },
                        ))
                },
            );

        vanishing.verify(params, expressions, y, xn)
    };

    #[rustfmt::skip]
    let queries = instance_commitments
        .iter()
        .zip(instance_evals.iter())
        .zip(advice_commitments.iter())
        .zip(advice_evals.iter())
        .zip(permutations_evaluated.iter())
        .zip(lookups_evaluated.iter())
        .zip(shuffles_evaluated.iter())
        .flat_map(|((((((instance_commitments, instance_evals), advice_commitments),advice_evals),permutation),lookups),shuffles)| {
                iter::empty()
                    .chain(
                        V::QUERY_INSTANCE
                            .then_some(vk.cs.instance_queries.iter().enumerate().map(
                                move |(query_index, &(column, at))| {
                                    VerifierQuery::new_commitment(
                                        &instance_commitments[column.index],
                                        vk.domain.rotate_omega(*x, at),
                                        instance_evals[query_index],
                                    )
                                },
                            ))
                            .into_iter()
                            .flatten(),
                    )
                    .chain(vk.cs.advice_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| {
                            VerifierQuery::new_commitment(
                                &advice_commitments[column.index],
                                vk.domain.rotate_omega(*x, at),
                                advice_evals[query_index],
                            )
                        },
                    ))
                    .chain(permutation.queries(vk, x))
                    .chain(lookups.iter().flat_map(move |p| p.queries(vk, x)))
                    .chain(shuffles.iter().flat_map(move |p| p.queries(vk, x)))
            },
        )
        .chain(
            vk.cs
                .fixed_queries
                .iter()
                .enumerate()
                .map(|(query_index, &(column, at))| {
                    VerifierQuery::new_commitment(
                        &vk.fixed_commitments[column.index],
                        vk.domain.rotate_omega(*x, at),
                        fixed_evals[query_index],
                    )
                }),
        )
        .chain(permutations_common.queries(&vk.permutation, x))
        .chain(vanishing.queries(x));

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.

    let verifier = V::new();
    strategy.process(|msm| {
        verifier
            .verify_proof(transcript, queries, msm)
            .map_err(|_| Error::Opening)
    })
}
