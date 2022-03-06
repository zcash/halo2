use ff::Field;
use group::Curve;
use rand_core::RngCore;
use std::iter;
use std::marker::PhantomData;
use std::ops::Mul;

use super::{
    vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, ChallengeY, Error,
    VerifyingKey,
};
use crate::arithmetic::{BaseExt, CurveAffine, FieldExt, MultiMillerLoop};

use crate::poly::{
    commitment::{Blind, Params, ParamsVerifier},
    multiopen::Decider,
    multiopen::{self, VerifierQuery},
    PairMSM, MSM,
};
use crate::transcript::{read_n_points, read_n_scalars, EncodedChallenge, TranscriptRead};

/// Trait representing a strategy for verifying Halo 2 proofs.
pub trait VerificationStrategy<C: CurveAffine> {
    /// The output type of this verification strategy after processing a proof.
    type Output;

    /// Obtains an MSM from the verifier strategy and yields back the strategy's
    /// output.
    fn process(self, f: impl FnOnce() -> Result<PairMSM<C>, Error>) -> Result<Self::Output, Error>;
}

/// A verifier that checks a single proof at a time.
#[derive(Debug)]
pub struct SingleVerifier<'a, E: MultiMillerLoop> {
    params: &'a ParamsVerifier<E>,
}

impl<'a, C: MultiMillerLoop> SingleVerifier<'a, C> {
    /// Constructs a new single proof verifier.
    pub fn new(params: &'a ParamsVerifier<C>) -> Self {
        SingleVerifier { params }
    }
}

impl<'a, C: MultiMillerLoop> VerificationStrategy<C::G1Affine> for SingleVerifier<'a, C> {
    type Output = ();

    fn process(
        self,
        f: impl FnOnce() -> Result<PairMSM<C::G1Affine>, Error>,
    ) -> Result<Self::Output, Error> {
        let guard = f()?;
        if Decider::verify(self.params, guard) {
            Ok(())
        } else {
            Err(Error::ConstraintSystemFailure)
        }
    }
}

/// A verifier that checks multiple proofs in a batch.
#[derive(Debug)]
pub struct BatchVerifier<'a, E: MultiMillerLoop, R: RngCore> {
    params: &'a ParamsVerifier<E>,
    msm: PairMSM<E::G1Affine>,
    rng: R,
}

impl<'a, E: MultiMillerLoop, R: RngCore> BatchVerifier<'a, E, R> {
    /// Constructs a new batch verifier.
    pub fn new(params: &'a ParamsVerifier<E>, rng: R) -> Self {
        BatchVerifier {
            params,
            msm: PairMSM::default(),
            rng,
        }
    }

    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    #[must_use]
    pub fn finalize(self) -> bool {
        Decider::verify(self.params, self.msm)
    }
}

impl<'a, C: MultiMillerLoop, R: RngCore> VerificationStrategy<C::G1Affine>
    for BatchVerifier<'a, C, R>
{
    type Output = Self;

    fn process(
        mut self,
        f: impl FnOnce() -> Result<PairMSM<C::G1Affine>, Error>,
    ) -> Result<Self::Output, Error> {
        // Scale the MSM by a random factor to ensure that if the existing MSM
        // has is_zero() == false then this argument won't be able to interfere
        // with it to make it true, with high probability.
        self.msm.scale(C::Scalar::random(&mut self.rng));
        let to_add = f()?;
        self.msm.add_msm(to_add);

        Ok(Self {
            msm: self.msm,
            rng: self.rng,
            params: self.params,
        })
    }
}

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<
    'params,
    C: MultiMillerLoop,
    E: EncodedChallenge<C::G1Affine>,
    T: TranscriptRead<C::G1Affine, E>,
    V: VerificationStrategy<C::G1Affine>,
>(
    params: &'params ParamsVerifier<C>,
    vk: &VerifyingKey<C::G1Affine>,
    strategy: V,
    instances: &[&[&[C::Scalar]]],
    transcript: &mut T,
) -> Result<V::Output, Error> {
    // Check that instances matches the expected number of instance columns
    for instances in instances.iter() {
        if instances.len() != vk.cs.num_instance_columns {
            return Err(Error::InvalidInstances);
        }
    }

    let instance_commitments = instances
        .iter()
        .map(|instance| {
            instance
                .iter()
                .map(|instance| {
                    if instance.len() > params.n as usize - (vk.cs.blinding_factors() + 1) {
                        return Err(Error::InstanceTooLarge);
                    }

                    Ok(params.commit_lagrange(instance.to_vec()).to_affine())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let num_proofs = instance_commitments.len();

    // Hash verification key into transcript
    vk.hash_into(transcript)?;

    for instance_commitments in instance_commitments.iter() {
        // Hash the instance (external) commitments into the transcript
        for commitment in instance_commitments {
            transcript.common_point(*commitment)?
        }
    }

    let advice_commitments = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash the prover's advice commitments into the transcript
            read_n_points(transcript, vk.cs.num_advice_columns)
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
        .map(|_| {
            // Hash each permutation product commitment
            vk.cs.permutation.read_product_commitments(vk, transcript)
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

    let vanishing = vanishing::Argument::read_commitments_before_y(transcript)?;

    // Sample y challenge, which keeps the gates linearly independent.
    let y: ChallengeY<_> = transcript.squeeze_challenge_scalar();

    let vanishing = vanishing.read_commitments_after_y(vk, transcript)?;

    // Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability.
    let x: ChallengeX<_> = transcript.squeeze_challenge_scalar();
    let instance_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> { read_n_scalars(transcript, vk.cs.instance_queries.len()) })
        .collect::<Result<Vec<_>, _>>()?;

    let advice_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> { read_n_scalars(transcript, vk.cs.advice_queries.len()) })
        .collect::<Result<Vec<_>, _>>()?;

    let fixed_evals = read_n_scalars(transcript, vk.cs.fixed_queries.len())?;

    let vanishing = vanishing.evaluate_after_x(transcript)?;

    let permutations_common = vk.permutation.evaluate(transcript)?;

    let permutations_evaluated = permutations_committed
        .into_iter()
        .map(|permutation| permutation.evaluate(transcript))
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
    let vanishing = {
        // x^n
        let xn = x.pow(&[params.n as u64, 0, 0, 0]);

        let blinding_factors = vk.cs.blinding_factors();
        let l_evals = vk
            .domain
            .l_i_range(*x, xn, (-((blinding_factors + 1) as i32))..=0);
        assert_eq!(l_evals.len(), 2 + blinding_factors);
        let l_last = l_evals[0];
        let l_blind: C::Scalar = l_evals[1..(1 + blinding_factors)]
            .iter()
            .fold(C::Scalar::zero(), |acc, eval| acc + eval);
        let l_0 = l_evals[1 + blinding_factors];

        // Compute the expected value of h(x)
        let expressions = advice_evals
            .iter()
            .zip(instance_evals.iter())
            .zip(permutations_evaluated.iter())
            .zip(lookups_evaluated.iter())
            .flat_map(|(((advice_evals, instance_evals), permutation), lookups)| {
                let fixed_evals = &fixed_evals;
                std::iter::empty()
                    // Evaluate the circuit using the custom gates provided
                    .chain(vk.cs.gates.iter().flat_map(move |gate| {
                        gate.polynomials().iter().map(move |poly| {
                            poly.evaluate(
                                &|scalar| scalar,
                                &|_| panic!("virtual selectors are removed during optimization"),
                                &|index, _, _| fixed_evals[index],
                                &|index, _, _| advice_evals[index],
                                &|index, _, _| instance_evals[index],
                                &|a| -a,
                                &|a, b| a + &b,
                                &|a, b| a * &b,
                                &|a, scalar| a * &scalar,
                            )
                        })
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
                    .chain(
                        lookups
                            .iter()
                            .zip(vk.cs.lookups.iter())
                            .flat_map(move |(p, argument)| {
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
                                )
                            })
                            .into_iter(),
                    )
            });

        vanishing.verify(expressions, y, xn)
    };

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
                    permutation,
                ),
                lookups,
            )| {
                iter::empty()
                    .chain(vk.cs.instance_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| {
                            VerifierQuery::new_commitment(
                                &instance_commitments[column.index()],
                                vk.domain.rotate_omega(*x, at),
                                at,
                                instance_evals[query_index],
                            )
                        },
                    ))
                    .chain(vk.cs.advice_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| {
                            VerifierQuery::new_commitment(
                                &advice_commitments[column.index()],
                                vk.domain.rotate_omega(*x, at),
                                at,
                                advice_evals[query_index],
                            )
                        },
                    ))
                    .chain(permutation.queries(vk, x))
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
                .map(|(query_index, &(column, at))| {
                    VerifierQuery::new_commitment(
                        &vk.fixed_commitments[column.index()],
                        vk.domain.rotate_omega(*x, at),
                        at,
                        fixed_evals[query_index],
                    )
                }),
        )
        .chain(permutations_common.queries(&vk.permutation, x))
        .chain(vanishing.queries(x));

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.
    strategy.process(|| {
        multiopen::verify_proof(params, transcript, queries).map_err(|_| Error::Opening)
    })
}
