use ff::{Field, FromUniformBytes, WithSmallOrderMulGroup};
use group::Curve;
use rand_core::RngCore;
use std::collections::{BTreeSet, HashSet};
use std::ops::RangeTo;
use std::{collections::HashMap, iter};

use super::{
    circuit::{
        compile_circuit,
        sealed::{self},
        Advice, Any, Assignment, Challenge, Circuit, Column, Fixed, Instance, Selector,
        WitnessCalculator,
    },
    lookup, permutation, shuffle, vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta,
    ChallengeX, ChallengeY, Error, ProvingKey,
};

use crate::{
    arithmetic::{eval_polynomial, CurveAffine},
    circuit::Value,
    plonk::Assigned,
    poly::{
        commitment::{Blind, CommitmentScheme, Params, Prover},
        Basis, Coeff, LagrangeCoeff, Polynomial, ProverQuery,
    },
};
use crate::{
    poly::batch_invert_assigned,
    transcript::{EncodedChallenge, TranscriptWrite},
};
use group::prime::PrimeCurveAffine;

#[derive(Debug)]
struct InstanceSingle<C: CurveAffine> {
    pub instance_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    pub instance_polys: Vec<Polynomial<C::Scalar, Coeff>>,
}

#[derive(Debug, Clone)]
struct AdviceSingle<C: CurveAffine, B: Basis> {
    pub advice_polys: Vec<Polynomial<C::Scalar, B>>,
    pub advice_blinds: Vec<Blind<C::Scalar>>,
}

/// The prover object used to create proofs interactively by passing the witnesses to commit at
/// each phase.  This works for a single proof.  This is a wrapper over ProverV2.
#[derive(Debug)]
pub struct ProverV2Single<
    'a,
    'params,
    Scheme: CommitmentScheme,
    P: Prover<'params, Scheme>,
    E: EncodedChallenge<Scheme::Curve>,
    R: RngCore,
    T: TranscriptWrite<Scheme::Curve, E>,
>(ProverV2<'a, 'params, Scheme, P, E, R, T>);

impl<
        'a,
        'params,
        Scheme: CommitmentScheme,
        P: Prover<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        R: RngCore,
        T: TranscriptWrite<Scheme::Curve, E>,
    > ProverV2Single<'a, 'params, Scheme, P, E, R, T>
{
    /// Create a new prover object
    pub fn new(
        params: &'params Scheme::ParamsProver,
        pk: &'a ProvingKey<Scheme::Curve>,
        // TODO: If this was a vector the usage would be simpler
        instance: &[&[Scheme::Scalar]],
        rng: R,
        transcript: &'a mut T,
    ) -> Result<Self, Error>
    // TODO: Can I move this `where` to the struct definition?
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        Ok(Self(ProverV2::new(
            params,
            pk,
            &[instance],
            rng,
            transcript,
        )?))
    }

    /// Commit the `witness` at `phase` and return the challenges after `phase`.
    pub fn commit_phase(
        &mut self,
        phase: u8,
        // TODO: Turn this into Vec<Option<Vec<F>>>.  Requires batch_invert_assigned to work with
        // Vec<F>
        // witness: Vec<Option<Polynomial<Assigned<Scheme::Scalar>, LagrangeCoeff>>>,
        witness: Vec<Option<Vec<Assigned<Scheme::Scalar>>>>,
    ) -> Result<HashMap<usize, Scheme::Scalar>, Error>
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        self.0.commit_phase(phase, vec![witness])
    }

    /// Finalizes the proof creation.
    pub fn create_proof(self) -> Result<(), Error>
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        self.0.create_proof()
    }
}

/// The prover object used to create proofs interactively by passing the witnesses to commit at
/// each phase.  This supports batch proving.
#[derive(Debug)]
pub struct ProverV2<
    'a,
    'params,
    Scheme: CommitmentScheme,
    P: Prover<'params, Scheme>,
    E: EncodedChallenge<Scheme::Curve>,
    R: RngCore,
    T: TranscriptWrite<Scheme::Curve, E>,
> {
    // Circuit and setup fields
    params: &'params Scheme::ParamsProver,
    pk: &'a ProvingKey<Scheme::Curve>,
    phases: Vec<sealed::Phase>,
    // State
    instance: Vec<InstanceSingle<Scheme::Curve>>,
    advice: Vec<AdviceSingle<Scheme::Curve, LagrangeCoeff>>,
    challenges: HashMap<usize, Scheme::Scalar>,
    next_phase_index: usize,
    rng: R,
    transcript: &'a mut T, // TODO: maybe &mut T?
    _marker: std::marker::PhantomData<(P, E)>,
}

impl<
        'a,
        'params,
        Scheme: CommitmentScheme,
        P: Prover<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        R: RngCore,
        T: TranscriptWrite<Scheme::Curve, E>,
    > ProverV2<'a, 'params, Scheme, P, E, R, T>
{
    /// Create a new prover object
    pub fn new(
        params: &'params Scheme::ParamsProver,
        pk: &'a ProvingKey<Scheme::Curve>,
        // TODO: If this was a vector the usage would be simpler
        instances: &[&[&[Scheme::Scalar]]],
        rng: R,
        transcript: &'a mut T,
    ) -> Result<Self, Error>
    // TODO: Can I move this `where` to the struct definition?
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        // println!("DBG prove vk.queries.advices {:?}", pk.vk.queries.advice);
        for instance in instances.iter() {
            if instance.len() != pk.vk.cs.num_instance_columns {
                return Err(Error::InvalidInstances);
            }
        }

        // Hash verification key into transcript
        pk.vk.hash_into(transcript)?;

        let meta = &pk.vk.cs;
        // let queries = &pk.vk.queries;
        let phases = meta.phases().collect();

        let domain = &pk.vk.domain;

        // TODO: Name this better
        let mut instance_fn =
            |instance: &[&[Scheme::Scalar]]| -> Result<InstanceSingle<Scheme::Curve>, Error> {
                let instance_values = instance
                    .iter()
                    .map(|values| {
                        let mut poly = domain.empty_lagrange();
                        assert_eq!(poly.len(), params.n() as usize);
                        if values.len() > (poly.len() - (meta.blinding_factors() + 1)) {
                            return Err(Error::InstanceTooLarge);
                        }
                        for (poly, value) in poly.iter_mut().zip(values.iter()) {
                            if !P::QUERY_INSTANCE {
                                // dbg!(1, value);
                                transcript.common_scalar(*value)?;
                            }
                            *poly = *value;
                        }
                        Ok(poly)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if P::QUERY_INSTANCE {
                    let instance_commitments_projective: Vec<_> = instance_values
                        .iter()
                        .map(|poly| params.commit_lagrange(poly, Blind::default()))
                        .collect();
                    let mut instance_commitments =
                        vec![Scheme::Curve::identity(); instance_commitments_projective.len()];
                    <Scheme::Curve as CurveAffine>::CurveExt::batch_normalize(
                        &instance_commitments_projective,
                        &mut instance_commitments,
                    );
                    let instance_commitments = instance_commitments;
                    drop(instance_commitments_projective);

                    for commitment in &instance_commitments {
                        // dbg!(2, commitment);
                        transcript.common_point(*commitment)?;
                    }
                }

                let instance_polys: Vec<_> = instance_values
                    .iter()
                    .map(|poly| {
                        let lagrange_vec = domain.lagrange_from_vec(poly.to_vec());
                        domain.lagrange_to_coeff(lagrange_vec)
                    })
                    .collect();

                Ok(InstanceSingle {
                    instance_values,
                    instance_polys,
                })
            };
        let instance: Vec<InstanceSingle<Scheme::Curve>> = instances
            .iter()
            .map(|instance| instance_fn(instance))
            .collect::<Result<Vec<_>, _>>()?;

        let advice = vec![
            AdviceSingle::<Scheme::Curve, LagrangeCoeff> {
                advice_polys: vec![domain.empty_lagrange(); meta.num_advice_columns],
                advice_blinds: vec![Blind::default(); meta.num_advice_columns],
            };
            instances.len()
        ];
        let challenges = HashMap::<usize, Scheme::Scalar>::with_capacity(meta.num_challenges);

        Ok(ProverV2 {
            params,
            pk,
            phases,
            instance,
            rng,
            transcript,
            advice,
            challenges,
            next_phase_index: 0,
            _marker: std::marker::PhantomData {},
        })
    }

    /// Commit the `witness` at `phase` and return the challenges after `phase`.
    pub fn commit_phase(
        &mut self,
        phase: u8,
        // TODO: Turn this into Vec<Option<Vec<F>>>.  Requires batch_invert_assigned to work with
        // Vec<F>
        // witness: Vec<Vec<Option<Polynomial<Assigned<Scheme::Scalar>, LagrangeCoeff>>>>,
        witness: Vec<Vec<Option<Vec<Assigned<Scheme::Scalar>>>>>,
    ) -> Result<HashMap<usize, Scheme::Scalar>, Error>
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        let current_phase = match self.phases.get(self.next_phase_index) {
            Some(phase) => phase,
            None => {
                panic!("TODO: Return Error instead.  All phases already commited");
            }
        };
        if phase != current_phase.0 {
            panic!("TODO: Return Error instead. Committing invalid phase");
        }

        let params = self.params;
        let meta = &self.pk.vk.cs;
        // let queries = &self.pk.vk.queries;
        // println!("DBG commit_phase gate {:?}", meta.gates()[0]);
        // println!("DBG commit_phase queries {:?}", meta.advice_queries());

        let mut rng = &mut self.rng;

        let advice = &mut self.advice;
        let challenges = &mut self.challenges;

        let column_indices = meta
            .advice_column_phase
            .iter()
            .enumerate()
            .filter_map(|(column_index, phase)| {
                if current_phase == phase {
                    Some(column_index)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        if witness.len() != advice.len() {
            return Err(Error::Other("witness.len() != advice.len()".to_string()));
        }
        for witness_circuit in &witness {
            if witness_circuit.len() != meta.num_advice_columns {
                return Err(Error::Other(format!(
                    "unexpected length in witness_circuitk.  Got {}, expected {}",
                    witness_circuit.len(),
                    meta.num_advice_columns,
                )));
            }
            // Check that all current_phase advice columns are Some, and their length is correct
            for (column_index, advice_column) in witness_circuit.iter().enumerate() {
                if column_indices.contains(&column_index) {
                    match advice_column {
                        None => {
                            return Err(Error::Other(format!(
                                "expected advice column with index {} at phase {}",
                                column_index, current_phase.0
                            )))
                        }
                        Some(advice_column) => {
                            if advice_column.len() != params.n() as usize {
                                return Err(Error::Other(format!(
                                    "expected advice column with index {} to have length {}",
                                    column_index,
                                    params.n(),
                                )));
                            }
                        }
                    }
                } else if advice_column.is_some() {
                    return Err(Error::Other(format!(
                        "expected no advice column with index {} at phase {}",
                        column_index, current_phase.0
                    )));
                };
            }
        }

        let mut commit_phase_fn = |advice: &mut AdviceSingle<Scheme::Curve, LagrangeCoeff>,
                                   witness: Vec<
            Option<Polynomial<Assigned<Scheme::Scalar>, LagrangeCoeff>>,
        >|
         -> Result<(), Error> {
            let unusable_rows_start = params.n() as usize - (meta.blinding_factors() + 1);
            let mut advice_values =
                batch_invert_assigned::<Scheme::Scalar>(witness.into_iter().flatten().collect());
            let unblinded_advice: HashSet<usize> =
                HashSet::from_iter(meta.unblinded_advice_columns.clone());

            // Add blinding factors to advice columns
            for (column_index, advice_values) in column_indices.iter().zip(&mut advice_values) {
                if !unblinded_advice.contains(column_index) {
                    for cell in &mut advice_values[unusable_rows_start..] {
                        *cell = Scheme::Scalar::random(&mut rng);
                    }
                } else {
                    #[cfg(feature = "sanity-checks")]
                    for cell in &advice_values[unusable_rows_start..] {
                        assert_eq!(*cell, Scheme::Scalar::ZERO);
                    }
                }
            }

            // Compute commitments to advice column polynomials
            let blinds: Vec<_> = column_indices
                .iter()
                .map(|i| {
                    if unblinded_advice.contains(i) {
                        Blind::default()
                    } else {
                        Blind(Scheme::Scalar::random(&mut rng))
                    }
                })
                .collect();
            let advice_commitments_projective: Vec<_> = advice_values
                .iter()
                .zip(blinds.iter())
                .map(|(poly, blind)| params.commit_lagrange(poly, *blind))
                .collect();
            let mut advice_commitments =
                vec![Scheme::Curve::identity(); advice_commitments_projective.len()];
            <Scheme::Curve as CurveAffine>::CurveExt::batch_normalize(
                &advice_commitments_projective,
                &mut advice_commitments,
            );
            let advice_commitments = advice_commitments;
            drop(advice_commitments_projective);

            for commitment in &advice_commitments {
                self.transcript.write_point(*commitment)?;
            }
            for ((column_index, advice_values), blind) in
                column_indices.iter().zip(advice_values).zip(blinds)
            {
                advice.advice_polys[*column_index] = advice_values;
                advice.advice_blinds[*column_index] = blind;
            }
            Ok(())
        };

        for (witness, advice) in witness.into_iter().zip(advice.iter_mut()) {
            commit_phase_fn(
                advice,
                witness
                    .into_iter()
                    .map(|v| v.map(Polynomial::new_lagrange_from_vec))
                    .collect(),
            )?;
        }

        for (index, phase) in meta.challenge_phase.iter().enumerate() {
            if current_phase == phase {
                let existing =
                    challenges.insert(index, *self.transcript.squeeze_challenge_scalar::<()>());
                assert!(existing.is_none());
            }
        }

        self.next_phase_index += 1;
        Ok(challenges.clone())
    }

    /// Finalizes the proof creation.
    pub fn create_proof(mut self) -> Result<(), Error>
    where
        Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        let params = self.params;
        let meta = &self.pk.vk.cs;
        // let queries = &self.pk.vk.queries;
        let pk = self.pk;
        let domain = &self.pk.vk.domain;

        let mut rng = self.rng;

        let instance = std::mem::take(&mut self.instance);
        let advice = std::mem::take(&mut self.advice);
        let mut challenges = self.challenges;

        assert_eq!(challenges.len(), meta.num_challenges);
        let challenges = (0..meta.num_challenges)
            .map(|index| challenges.remove(&index).unwrap())
            .collect::<Vec<_>>();

        // Sample theta challenge for keeping lookup columns linearly independent
        let theta: ChallengeTheta<_> = self.transcript.squeeze_challenge_scalar();

        let mut lookups_fn =
            |instance: &InstanceSingle<Scheme::Curve>,
             advice: &AdviceSingle<Scheme::Curve, LagrangeCoeff>|
             -> Result<Vec<lookup::prover::Permuted<Scheme::Curve>>, Error> {
                meta.lookups
                    .iter()
                    .map(|lookup| {
                        lookup.commit_permuted(
                            pk,
                            params,
                            domain,
                            theta,
                            &advice.advice_polys,
                            &pk.fixed_values,
                            &instance.instance_values,
                            &challenges,
                            &mut rng,
                            self.transcript,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
        let lookups: Vec<Vec<lookup::prover::Permuted<Scheme::Curve>>> = instance
            .iter()
            .zip(advice.iter())
            .map(|(instance, advice)| -> Result<Vec<_>, Error> {
                // Construct and commit to permuted values for each lookup
                lookups_fn(instance, advice)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Sample beta challenge
        let beta: ChallengeBeta<_> = self.transcript.squeeze_challenge_scalar();

        // Sample gamma challenge
        let gamma: ChallengeGamma<_> = self.transcript.squeeze_challenge_scalar();

        // Commit to permutation.
        let permutations: Vec<permutation::prover::Committed<Scheme::Curve>> = instance
            .iter()
            .zip(advice.iter())
            .map(|(instance, advice)| {
                meta.permutation.commit(
                    params,
                    pk,
                    &pk.permutation,
                    &advice.advice_polys,
                    &pk.fixed_values,
                    &instance.instance_values,
                    beta,
                    gamma,
                    &mut rng,
                    self.transcript,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let lookups: Vec<Vec<lookup::prover::Committed<Scheme::Curve>>> = lookups
            .into_iter()
            .map(|lookups| -> Result<Vec<_>, _> {
                // Construct and commit to products for each lookup
                lookups
                    .into_iter()
                    .map(|lookup| {
                        lookup.commit_product(pk, params, beta, gamma, &mut rng, self.transcript)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let shuffles: Vec<Vec<shuffle::prover::Committed<Scheme::Curve>>> = instance
            .iter()
            .zip(advice.iter())
            .map(|(instance, advice)| -> Result<Vec<_>, _> {
                // Compress expressions for each shuffle
                meta.shuffles
                    .iter()
                    .map(|shuffle| {
                        shuffle.commit_product(
                            pk,
                            params,
                            domain,
                            theta,
                            gamma,
                            &advice.advice_polys,
                            &pk.fixed_values,
                            &instance.instance_values,
                            &challenges,
                            &mut rng,
                            self.transcript,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Commit to the vanishing argument's random polynomial for blinding h(x_3)
        let vanishing = vanishing::Argument::commit(params, domain, &mut rng, self.transcript)?;

        // Obtain challenge for keeping all separate gates linearly independent
        let y: ChallengeY<_> = self.transcript.squeeze_challenge_scalar();

        // Calculate the advice polys
        let advice: Vec<AdviceSingle<Scheme::Curve, Coeff>> = advice
            .into_iter()
            .map(
                |AdviceSingle {
                     advice_polys,
                     advice_blinds,
                 }| {
                    AdviceSingle {
                        advice_polys: advice_polys
                            .into_iter()
                            .map(|poly| domain.lagrange_to_coeff(poly))
                            .collect::<Vec<_>>(),
                        advice_blinds,
                    }
                },
            )
            .collect();

        // Evaluate the h(X) polynomial
        let h_poly = pk.ev.evaluate_h(
            pk,
            &advice
                .iter()
                .map(|a| a.advice_polys.as_slice())
                .collect::<Vec<_>>(),
            &instance
                .iter()
                .map(|i| i.instance_polys.as_slice())
                .collect::<Vec<_>>(),
            &challenges,
            *y,
            *beta,
            *gamma,
            *theta,
            &lookups,
            &shuffles,
            &permutations,
        );

        // Construct the vanishing argument's h(X) commitments
        let vanishing = vanishing.construct(params, domain, h_poly, &mut rng, self.transcript)?;

        let x: ChallengeX<_> = self.transcript.squeeze_challenge_scalar();
        let xn = x.pow([params.n()]);

        if P::QUERY_INSTANCE {
            // Compute and hash instance evals for the circuit instance
            for instance in instance.iter() {
                // Evaluate polynomials at omega^i x
                let instance_evals: Vec<_> = meta
                    .instance_queries
                    .iter()
                    .map(|&(column, at)| {
                        eval_polynomial(
                            &instance.instance_polys[column.index()],
                            domain.rotate_omega(*x, at),
                        )
                    })
                    .collect();

                // Hash each instance column evaluation
                for eval in instance_evals.iter() {
                    self.transcript.write_scalar(*eval)?;
                }
            }
        }

        // Compute and hash advice evals for the circuit instance
        for advice in advice.iter() {
            // Evaluate polynomials at omega^i x
            let advice_evals: Vec<_> = meta
                .advice_queries
                .iter()
                .map(|&(column, at)| {
                    eval_polynomial(
                        &advice.advice_polys[column.index()],
                        domain.rotate_omega(*x, at),
                    )
                })
                .collect();
            // dbg!(&advice_evals);

            // Hash each advice column evaluation
            for eval in advice_evals.iter() {
                self.transcript.write_scalar(*eval)?;
            }
        }

        // Compute and hash fixed evals
        let fixed_evals: Vec<_> = meta
            .fixed_queries
            .iter()
            .map(|&(column, at)| {
                eval_polynomial(&pk.fixed_polys[column.index()], domain.rotate_omega(*x, at))
            })
            .collect();

        // Hash each fixed column evaluation
        for eval in fixed_evals.iter() {
            self.transcript.write_scalar(*eval)?;
        }

        let vanishing = vanishing.evaluate(x, xn, domain, self.transcript)?;

        // Evaluate common permutation data
        pk.permutation.evaluate(x, self.transcript)?;

        // Evaluate the permutations, if any, at omega^i x.
        let permutations: Vec<permutation::prover::Evaluated<Scheme::Curve>> = permutations
            .into_iter()
            .map(|permutation| -> Result<_, _> {
                permutation.construct().evaluate(pk, x, self.transcript)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Evaluate the lookups, if any, at omega^i x.
        let lookups: Vec<Vec<lookup::prover::Evaluated<Scheme::Curve>>> = lookups
            .into_iter()
            .map(|lookups| -> Result<Vec<_>, _> {
                lookups
                    .into_iter()
                    .map(|p| p.evaluate(pk, x, self.transcript))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Evaluate the shuffles, if any, at omega^i x.
        let shuffles: Vec<Vec<shuffle::prover::Evaluated<Scheme::Curve>>> = shuffles
            .into_iter()
            .map(|shuffles| -> Result<Vec<_>, _> {
                shuffles
                    .into_iter()
                    .map(|p| p.evaluate(pk, x, self.transcript))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let instances = instance
            .iter()
            .zip(advice.iter())
            .zip(permutations.iter())
            .zip(lookups.iter())
            .zip(shuffles.iter())
            .flat_map(|((((instance, advice), permutation), lookups), shuffles)| {
                iter::empty()
                    .chain(
                        P::QUERY_INSTANCE
                            .then_some(meta.instance_queries.iter().map(move |&(column, at)| {
                                ProverQuery {
                                    point: domain.rotate_omega(*x, at),
                                    poly: &instance.instance_polys[column.index()],
                                    blind: Blind::default(),
                                }
                            }))
                            .into_iter()
                            .flatten(),
                    )
                    .chain(
                        meta.advice_queries
                            .iter()
                            .map(move |&(column, at)| ProverQuery {
                                point: domain.rotate_omega(*x, at),
                                poly: &advice.advice_polys[column.index()],
                                blind: advice.advice_blinds[column.index()],
                            }),
                    )
                    .chain(permutation.open(pk, x))
                    .chain(lookups.iter().flat_map(move |p| p.open(pk, x)))
                    .chain(shuffles.iter().flat_map(move |p| p.open(pk, x)))
            })
            .chain(meta.fixed_queries.iter().map(|&(column, at)| ProverQuery {
                point: domain.rotate_omega(*x, at),
                poly: &pk.fixed_polys[column.index()],
                blind: Blind::default(),
            }))
            .chain(pk.permutation.open(x))
            // We query the h(X) polynomial at x
            .chain(vanishing.open(x));

        let prover = P::new(params);
        println!("DBG create_proof");
        prover
            .create_proof(rng, self.transcript, instances)
            .map_err(|_| Error::ConstraintSystemFailure)?;

        Ok(())
    }
}

pub(crate) struct WitnessCollection<'a, F: Field> {
    pub(crate) k: u32,
    pub(crate) current_phase: sealed::Phase,
    pub(crate) advice: Vec<Vec<Assigned<F>>>,
    // pub(crate) unblinded_advice: HashSet<usize>,
    pub(crate) challenges: &'a HashMap<usize, F>,
    pub(crate) instances: &'a [&'a [F]],
    pub(crate) usable_rows: RangeTo<usize>,
    pub(crate) _marker: std::marker::PhantomData<F>,
}

impl<'a, F: Field> Assignment<F> for WitnessCollection<'a, F> {
    fn enter_region<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about regions in this context.
    }

    fn exit_region(&mut self) {
        // Do nothing; we don't care about regions in this context.
    }

    fn enable_selector<A, AR>(&mut self, _: A, _: &Selector, _: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn annotate_column<A, AR>(&mut self, _annotation: A, _column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Do nothing
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Value<F>, Error> {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        self.instances
            .get(column.index())
            .and_then(|column| column.get(row))
            .map(|v| Value::known(*v))
            .ok_or(Error::BoundsFailure)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Ignore assignment of advice column in different phase than current one.
        if self.current_phase != column.column_type().phase {
            return Ok(());
        }

        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        *self
            .advice
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = to().into_field().assign()?;

        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Fixed>,
        _: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn copy(&mut self, _: Column<Any>, _: usize, _: Column<Any>, _: usize) -> Result<(), Error> {
        // We only care about advice columns here

        Ok(())
    }

    fn fill_from_row(
        &mut self,
        _: Column<Fixed>,
        _: usize,
        _: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_challenge(&self, challenge: Challenge) -> Value<F> {
        self.challenges
            .get(&challenge.index())
            .cloned()
            .map(Value::known)
            .unwrap_or_else(Value::unknown)
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about namespaces in this context.
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // Do nothing; we don't care about namespaces in this context.
    }
}

/// This creates a proof for the provided `circuit` when given the public
/// parameters `params` and the proving key [`ProvingKey`] that was
/// generated previously for the same circuit. The provided `instances`
/// are zero-padded internally.
pub fn create_proof<
    'params,
    Scheme: CommitmentScheme,
    P: Prover<'params, Scheme>,
    E: EncodedChallenge<Scheme::Curve>,
    R: RngCore,
    T: TranscriptWrite<Scheme::Curve, E>,
    ConcreteCircuit: Circuit<Scheme::Scalar>,
>(
    params: &'params Scheme::ParamsProver,
    pk: &ProvingKey<Scheme::Curve>,
    circuits: &[ConcreteCircuit],
    instances: &[&[&[Scheme::Scalar]]],
    rng: R,
    transcript: &mut T,
) -> Result<(), Error>
where
    Scheme::Scalar: WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
{
    if circuits.len() != instances.len() {
        return Err(Error::InvalidInstances);
    }
    let (_, config, cs) = compile_circuit(params.k(), &circuits[0], pk.vk.compress_selectors)?;
    let mut witness_calcs: Vec<_> = circuits
        .iter()
        .enumerate()
        .map(|(i, circuit)| WitnessCalculator::new(params.k(), circuit, &config, &cs, instances[i]))
        .collect();
    let mut prover = ProverV2::<Scheme, P, _, _, _>::new(params, pk, instances, rng, transcript)?;
    let mut challenges = HashMap::new();
    let phases = prover.phases.clone();
    for phase in &phases {
        // for phase in [0] {
        println!("DBG phase {}", phase.0);
        let mut witnesses = Vec::with_capacity(circuits.len());
        for witness_calc in witness_calcs.iter_mut() {
            witnesses.push(witness_calc.calc(phase.0, &challenges)?);
        }
        // println!("DBG witness: {:?}", witness);
        challenges = prover.commit_phase(phase.0, witnesses).unwrap();
        // println!("DBG challenges {:?}", challenges);
    }
    prover.create_proof()
}

#[test]
fn test_create_proof() {
    use crate::{
        circuit::SimpleFloorPlanner,
        plonk::{keygen_pk, keygen_vk, ConstraintSystem},
        poly::kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::ProverSHPLONK,
        },
        transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer},
    };
    use halo2curves::bn256::Bn256;
    use rand_core::OsRng;

    #[derive(Clone, Copy)]
    struct MyCircuit;

    impl<F: Field> Circuit<F> for MyCircuit {
        type Config = ();
        type FloorPlanner = SimpleFloorPlanner;
        #[cfg(feature = "circuit-params")]
        type Params = ();

        fn without_witnesses(&self) -> Self {
            *self
        }

        fn configure(_meta: &mut ConstraintSystem<F>) -> Self::Config {}

        fn synthesize(
            &self,
            _config: Self::Config,
            _layouter: impl crate::circuit::Layouter<F>,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    let params: ParamsKZG<Bn256> = ParamsKZG::setup(3, OsRng);
    let vk = keygen_vk(&params, &MyCircuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &MyCircuit).expect("keygen_pk should not fail");
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

    // Create proof with wrong number of instances
    let proof = create_proof::<KZGCommitmentScheme<_>, ProverSHPLONK<_>, _, _, _, _>(
        &params,
        &pk,
        &[MyCircuit, MyCircuit],
        &[],
        OsRng,
        &mut transcript,
    );
    assert!(matches!(proof.unwrap_err(), Error::InvalidInstances));

    // Create proof with correct number of instances
    create_proof::<KZGCommitmentScheme<_>, ProverSHPLONK<_>, _, _, _, _>(
        &params,
        &pk,
        &[MyCircuit, MyCircuit],
        &[&[], &[]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation should not fail");
}
