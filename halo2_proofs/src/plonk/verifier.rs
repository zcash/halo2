use ff::Field;
use pasta_curves::arithmetic::CurveExt;
use std::iter;

use super::{
    vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, ChallengeY, Error,
    VerifyingKey,
};
use crate::arithmetic::CurveAffine;
use crate::poly::{
    commitment::{Guard, Params, MSM},
    multiopen::{self, VerifierQuery},
};
use crate::transcript::{read_n_points, read_n_scalars, EncodedChallenge, TranscriptRead};

#[cfg(feature = "batch")]
mod batch;
#[cfg(feature = "batch")]
pub use batch::BatchVerifier;

// Batch normalization costs more than individual normalization below this.
const MIN_BATCH_NORMALIZE: usize = 4;

/// Trait representing a strategy for verifying Halo 2 proofs.
pub trait VerificationStrategy<'params, C: CurveAffine> {
    /// The output type of this verification strategy after processing a proof.
    type Output;

    /// Obtains an MSM from the verifier strategy and yields back the strategy's
    /// output.
    fn process<E: EncodedChallenge<C>>(
        self,
        f: impl FnOnce(MSM<'params, C>) -> Result<Guard<'params, C, E>, Error>,
    ) -> Result<Self::Output, Error>;
}

/// A verifier that checks a single proof at a time.
#[derive(Debug)]
pub struct SingleVerifier<'params, C: CurveAffine> {
    msm: MSM<'params, C>,
}

impl<'params, C: CurveAffine> SingleVerifier<'params, C> {
    /// Constructs a new single proof verifier.
    pub fn new(params: &'params Params<C>) -> Self {
        SingleVerifier {
            msm: MSM::new(params),
        }
    }
}

impl<'params, C: CurveAffine> VerificationStrategy<'params, C> for SingleVerifier<'params, C> {
    type Output = ();

    fn process<E: EncodedChallenge<C>>(
        self,
        f: impl FnOnce(MSM<'params, C>) -> Result<Guard<'params, C, E>, Error>,
    ) -> Result<Self::Output, Error> {
        let guard = f(self.msm)?;
        let msm = guard.use_challenges();
        if msm.eval() {
            Ok(())
        } else {
            Err(Error::ConstraintSystemFailure)
        }
    }
}

/// Returns a boolean indicating whether or not the proof is valid
pub fn verify_proof<
    'params,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    T: TranscriptRead<C, E>,
    V: VerificationStrategy<'params, C>,
>(
    params: &'params Params<C>,
    vk: &VerifyingKey<C>,
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

    let max_instance_len = params.n as usize - (vk.cs.blinding_factors() + 1);
    if instances
        .iter()
        .flat_map(|instance| instance.iter())
        .any(|instance| instance.len() > max_instance_len)
    {
        return Err(Error::InstanceTooLarge);
    }

    let batch_normalize = instances
        .iter()
        .flat_map(|instance| instance.iter())
        .take(MIN_BATCH_NORMALIZE)
        .count()
        == MIN_BATCH_NORMALIZE;
    let instance_commitments = if !batch_normalize {
        instances
            .iter()
            .map(|instance| {
                instance
                    .iter()
                    .map(|instance| {
                        params
                            .commit_lagrange_instance(instance)
                            .to_affine_vartime()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    } else {
        let instance_commitments_projective = instances
            .iter()
            .flat_map(|instance| instance.iter())
            .map(|instance| params.commit_lagrange_instance(instance))
            .collect::<Vec<_>>();
        let mut normalized_commitments = vec![C::identity(); instance_commitments_projective.len()];
        C::Curve::batch_normalize_vartime(
            &instance_commitments_projective,
            &mut normalized_commitments,
        );
        let mut normalized_commitments = normalized_commitments.into_iter();
        let instance_commitments = instances
            .iter()
            .map(|instance| {
                normalized_commitments
                    .by_ref()
                    .take(instance.len())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert!(normalized_commitments.next().is_none());
        instance_commitments
    };

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
        let xn = pow_by_power_of_two(*x, params.k);

        let blinding_factors = vk.cs.blinding_factors();
        let l_evals = vk
            .domain
            .l_i_range(*x, xn, (-((blinding_factors + 1) as i32))..=0);
        assert_eq!(l_evals.len(), 2 + blinding_factors);
        let l_last = l_evals[0];
        let l_blind: C::Scalar = l_evals[1..(1 + blinding_factors)]
            .iter()
            .fold(C::Scalar::ZERO, |acc, eval| acc + eval);
        let l_0 = l_evals[1 + blinding_factors];

        // Fixed evaluations are shared across every circuit in this proof.
        // Compute their pure gate subexpressions once, then reuse the same
        // scratch slots for each circuit's advice and instance evaluations.
        let mut gate_values = vk.verifier_expressions.common_values(&fixed_evals);

        // Compute the expected value of h(x)
        let expressions = advice_evals
            .iter()
            .zip(instance_evals.iter())
            .zip(permutations_evaluated.iter())
            .zip(lookups_evaluated.iter())
            .flat_map(|(((advice_evals, instance_evals), permutation), lookups)| {
                let fixed_evals = &fixed_evals;
                let gates = vk.verifier_expressions.evaluate(
                    &mut gate_values,
                    fixed_evals,
                    advice_evals,
                    instance_evals,
                );
                std::iter::empty()
                    // Evaluate the circuit using the custom gates provided
                    .chain(gates)
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
                            )
                        },
                    ))
            });

        vanishing.verify(params, expressions, y, xn)
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
                                instance_evals[query_index],
                            )
                        },
                    ))
                    .chain(vk.cs.advice_queries.iter().enumerate().map(
                        move |(query_index, &(column, at))| {
                            VerifierQuery::new_commitment(
                                &advice_commitments[column.index()],
                                vk.domain.rotate_omega(*x, at),
                                advice_evals[query_index],
                            )
                        },
                    ))
                    .chain(permutation.queries(vk, x))
                    .chain(lookups.iter().flat_map(move |p| p.queries(vk, x)))
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
                        fixed_evals[query_index],
                    )
                }),
        )
        .chain(permutations_common.queries(&vk.permutation, x))
        .chain(vanishing.queries(x));

    // We are now convinced the circuit is satisfied so long as the
    // polynomial commitments open to the correct values.
    strategy.process(|msm| {
        multiopen::verify_proof(params, transcript, queries, msm).map_err(|_| Error::Opening)
    })
}

fn pow_by_power_of_two<F: Field>(value: F, exponent: u32) -> F {
    (0..exponent).fold(value, |power, _| power.square())
}

#[cfg(test)]
mod tests {
    mod scalar;

    use super::{verify_proof, SingleVerifier, MIN_BATCH_NORMALIZE};
    use crate::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        pasta::{EqAffine, Fp},
        plonk::{
            create_proof, keygen_pk, keygen_vk, Advice, Circuit, Column, ConstraintSystem, Error,
            Instance, Selector,
        },
        poly::{
            commitment::{Blind, Params},
            EvaluationDomain, Rotation,
        },
        transcript::{Blake2bRead, Blake2bWrite, Challenge255},
    };
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    #[derive(Clone, Debug)]
    struct TestConfig {
        advice: Column<Advice>,
        instance: Column<Instance>,
        selector: Selector,
    }

    #[derive(Clone)]
    struct TestCircuit {
        value: Value<Fp>,
    }

    impl Circuit<Fp> for TestCircuit {
        type Config = TestConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                value: Value::unknown(),
            }
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            let advice = meta.advice_column();
            let instance = meta.instance_column();
            meta.enable_equality(advice);
            meta.enable_equality(instance);
            let selector = meta.complex_selector();
            meta.create_gate("instance equality", |meta| {
                let selector = meta.query_selector(selector);
                let difference = meta.query_advice(advice, Rotation::cur())
                    - meta.query_instance(instance, Rotation::cur());
                let constraint = selector.clone() * selector * difference;
                vec![constraint.clone(), constraint]
            });

            TestConfig {
                advice,
                instance,
                selector,
            }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let cell = layouter.assign_region(
                || "assign instance value",
                |mut region| {
                    config.selector.enable(&mut region, 0)?;
                    region.assign_advice(|| "instance value", config.advice, 0, || self.value)
                },
            )?;
            layouter.constrain_instance(cell.cell(), config.instance, 0)
        }
    }

    #[test]
    fn instance_commitment_matches_zero_padded_commitment() {
        const K: u32 = 5;

        let params = Params::<EqAffine>::new(K);
        let domain = EvaluationDomain::new(1, K);

        for len in [0, 1, 9, 1 << K] {
            let instance = (0..len).map(|i| Fp::from(i as u64)).collect::<Vec<_>>();
            let mut padded = domain.empty_lagrange();
            for (coefficient, value) in padded.iter_mut().zip(instance.iter()) {
                *coefficient = *value;
            }

            assert_eq!(
                params.commit_lagrange_instance(&instance),
                params.commit_lagrange(&padded, Blind::default()),
            );
        }
    }

    #[test]
    fn batch_normalized_instance_commitments_verify() {
        const K: u32 = 4;

        let rng = UnwrapErr(SysRng);

        let params = Params::<EqAffine>::new(K);
        let empty_circuit = TestCircuit {
            value: Value::unknown(),
        };
        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");
        let vk_clone = pk.get_vk().clone();
        assert!(std::sync::Arc::ptr_eq(
            &pk.get_vk().verifier_expressions,
            &vk_clone.verifier_expressions,
        ));
        assert_eq!(pk.get_vk().transcript_repr, vk_clone.transcript_repr);
        assert_eq!(
            format!("{:?}", pk.get_vk().pinned()),
            format!("{:?}", vk_clone.pinned())
        );
        let public_inputs = (0..MIN_BATCH_NORMALIZE)
            .map(|i| vec![Fp::from(i as u64 + 1)])
            .collect::<Vec<_>>();
        let circuits = public_inputs
            .iter()
            .map(|values| TestCircuit {
                value: Value::known(values[0]),
            })
            .collect::<Vec<_>>();
        let instance_columns = public_inputs
            .iter()
            .map(|values| [values.as_slice()])
            .collect::<Vec<_>>();
        let instances = instance_columns
            .iter()
            .map(|columns| columns.as_slice())
            .collect::<Vec<_>>();

        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(Vec::new());
        create_proof(&params, &pk, &circuits, &instances, rng, &mut transcript)
            .expect("proof generation should not fail");
        let proof = transcript.finalize();

        let strategy = SingleVerifier::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        verify_proof(&params, pk.get_vk(), strategy, &instances, &mut transcript)
            .expect("proof verification should not fail");

        #[cfg(feature = "batch")]
        {
            use ff::Field;

            let owned_instances = public_inputs
                .iter()
                .map(|values| vec![values.clone()])
                .collect::<Vec<_>>();
            let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(Vec::new());
            create_proof(
                &params,
                &pk,
                &circuits,
                &instances,
                UnwrapErr(SysRng),
                &mut transcript,
            )
            .expect("second proof generation should not fail");
            let second_proof = transcript.finalize();
            let check = || {
                let mut one = super::BatchVerifier::new();
                one.add_proof(owned_instances.clone(), proof.clone());
                assert!(one.finalize(&params, pk.get_vk()));
                let mut two = super::BatchVerifier::new();
                two.add_proof(owned_instances.clone(), proof.clone());
                two.add_proof(owned_instances.clone(), second_proof.clone());
                assert!(two.finalize(&params, pk.get_vk()));
                let mut invalid_instances = owned_instances.clone();
                invalid_instances[0][0][0] += Fp::ONE;
                let mut invalid = super::BatchVerifier::new();
                invalid.add_proof(invalid_instances, proof.clone());
                assert!(!invalid.finalize(&params, pk.get_vk()));
            };
            check();
            #[cfg(feature = "multicore")]
            maybe_rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build()
                .unwrap()
                .install(check);
        }

        scalar::check_tampered_and_batch_verification(
            &params,
            &pk,
            proof,
            &instances,
            #[cfg(feature = "batch")]
            &public_inputs,
        );
    }
}
