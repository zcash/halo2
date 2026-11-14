use ff::Field;
use rand_core::Rng;
use std::iter;
use std::ops::RangeTo;

use crate::multicore::prelude::*;

use super::{
    circuit::{
        Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Fixed, FloorPlanner, Instance,
        Selector,
    },
    lookup, permutation, vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX,
    ChallengeY, Error, ProvingKey,
};
use crate::{
    arithmetic::{evaluate_batch, CurveAffine, CurveExt},
    circuit::Value,
    plonk::Assigned,
    poly::{
        self,
        commitment::{Blind, Params},
        multiopen::{self, ProverQuery},
        Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial,
    },
};
use crate::{
    poly::batch_invert_assigned,
    transcript::{EncodedChallenge, TranscriptWrite},
};

/// Field elements drawn from the prover's RNG ahead of time.
///
/// Work that consumes randomness (blinding rows, commitment blinds) can only
/// run in parallel if it does not share the RNG. The prover therefore draws
/// each step's field elements up front, in exactly the order a sequential
/// prover would draw them, and hands each parallel task its own `Draws`. The
/// proof is then the same as a sequential prover's for the same RNG stream.
pub(in crate::plonk) struct Draws<F>(std::vec::IntoIter<F>);

impl<F: Field> Draws<F> {
    /// Draws `count` field elements from `rng`.
    pub(in crate::plonk) fn take<R: Rng>(rng: &mut R, count: usize) -> Self {
        Draws(
            (0..count)
                .map(|_| F::random(&mut *rng))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    /// Returns the next drawn field element.
    ///
    /// Panics if the step consumes more elements than were drawn for it,
    /// which would be a bug in the step's draw count. A step that consumes
    /// fewer would shift every later draw; the pinned proofs in
    /// `tests/prover_determinism.rs` catch that.
    pub(in crate::plonk) fn next(&mut self) -> F {
        self.0
            .next()
            .expect("a prover step consumed more randomness than was drawn for it")
    }
}

/// This creates a proof for the provided `circuit` when given the public
/// parameters `params` and the proving key [`ProvingKey`] that was
/// generated previously for the same circuit. The provided `instances`
/// are zero-padded internally.
pub fn create_proof<
    C: CurveAffine,
    E: EncodedChallenge<C>,
    R: Rng,
    T: TranscriptWrite<C, E>,
    ConcreteCircuit: Circuit<C::Scalar>,
>(
    params: &Params<C>,
    pk: &ProvingKey<C>,
    circuits: &[ConcreteCircuit],
    instances: &[&[&[C::Scalar]]],
    mut rng: R,
    transcript: &mut T,
) -> Result<(), Error> {
    if circuits.len() != instances.len() {
        return Err(Error::InvalidInstances);
    }

    for instance in instances.iter() {
        if instance.len() != pk.vk.cs.num_instance_columns {
            return Err(Error::InvalidInstances);
        }
    }

    // Hash verification key into transcript
    pk.vk.hash_into(transcript)?;

    let domain = &pk.vk.domain;
    let mut meta = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut meta);

    // Selector optimizations cannot be applied here; use the ConstraintSystem
    // from the verification key.
    let meta = &pk.vk.cs;

    struct InstanceSingle<C: CurveAffine> {
        pub instance_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        pub instance_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub instance_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    }

    // Pad each instance column to the domain size.
    let instance_values: Vec<Vec<Polynomial<C::Scalar, LagrangeCoeff>>> = instances
        .iter()
        .map(|instance| {
            instance
                .iter()
                .map(|values| {
                    let mut poly = domain.empty_lagrange();
                    assert_eq!(poly.len(), params.n as usize);
                    if values.len() > (poly.len() - (meta.blinding_factors() + 1)) {
                        return Err(Error::InstanceTooLarge);
                    }
                    for (poly, value) in poly.iter_mut().zip(values.iter()) {
                        *poly = *value;
                    }
                    Ok(poly)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Commit to every instance column of every circuit at once, then hash the
    // commitments circuit by circuit.
    let instance_commitments: Vec<Vec<C>> = instances
        .into_par_iter()
        .map(|instance| {
            let projective: Vec<_> = instance
                .into_par_iter()
                .map(|values| params.commit_lagrange_instance(values))
                .collect();
            let mut commitments = vec![C::identity(); projective.len()];
            C::Curve::batch_normalize_vartime(&projective, &mut commitments);
            commitments
        })
        .collect();
    for commitment in instance_commitments.iter().flatten() {
        transcript.common_point(*commitment)?;
    }

    let instance: Vec<InstanceSingle<C>> = instance_values
        .into_par_iter()
        .map(|instance_values| {
            let instance_polys: Vec<_> = instance_values
                .par_iter()
                .map(|poly| domain.lagrange_to_coeff(poly.clone()))
                .collect();
            let instance_cosets: Vec<_> = instance_polys
                .par_iter()
                .map(|poly| domain.coeff_to_extended(poly.clone()))
                .collect();
            InstanceSingle {
                instance_values,
                instance_polys,
                instance_cosets,
            }
        })
        .collect();

    struct AdviceSingle<C: CurveAffine> {
        pub advice_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        pub advice_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub advice_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
        pub advice_blinds: Vec<Blind<C::Scalar>>,
    }

    // Synthesis runs circuit by circuit, since a circuit need not be `Sync`.
    // It also draws the advice blinding factors and blinds, in the same order
    // as before; everything after it runs in parallel across circuits and
    // columns.
    #[allow(clippy::type_complexity)]
    let synthesized: Vec<(
        Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        Vec<Blind<C::Scalar>>,
    )> = circuits
        .iter()
        .zip(instances.iter())
        .map(|(circuit, instances)| -> Result<_, Error> {
            struct WitnessCollection<'a, F: Field> {
                k: u32,
                pub advice: Vec<Polynomial<Assigned<F>, LagrangeCoeff>>,
                instances: &'a [&'a [F]],
                usable_rows: RangeTo<usize>,
                _marker: std::marker::PhantomData<F>,
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

                fn enable_selector<A, AR>(
                    &mut self,
                    _: A,
                    _: &Selector,
                    _: usize,
                ) -> Result<(), Error>
                where
                    A: FnOnce() -> AR,
                    AR: Into<String>,
                {
                    // We only care about advice columns here

                    Ok(())
                }

                fn query_instance(
                    &self,
                    column: Column<Instance>,
                    row: usize,
                ) -> Result<Value<F>, Error> {
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

                fn copy(
                    &mut self,
                    _: Column<Any>,
                    _: usize,
                    _: Column<Any>,
                    _: usize,
                ) -> Result<(), Error> {
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

            let unusable_rows_start = params.n as usize - (meta.blinding_factors() + 1);

            let mut witness = WitnessCollection {
                k: params.k,
                advice: vec![domain.empty_lagrange_assigned(); meta.num_advice_columns],
                instances,
                // The prover will not be allowed to assign values to advice
                // cells that exist within inactive rows, which include some
                // number of blinding factors and an extra row for use in the
                // permutation argument.
                usable_rows: ..unusable_rows_start,
                _marker: std::marker::PhantomData,
            };

            // Synthesize the circuit to obtain the witness and other information.
            ConcreteCircuit::FloorPlanner::synthesize(
                &mut witness,
                circuit,
                config.clone(),
                meta.constants.clone(),
            )?;

            let mut advice = batch_invert_assigned(witness.advice);

            // Add blinding factors to advice columns
            for advice in &mut advice {
                for cell in &mut advice[unusable_rows_start..] {
                    *cell = C::Scalar::random(&mut rng);
                }
            }

            // Blinds for the commitments to the advice column polynomials
            let advice_blinds: Vec<_> = advice
                .iter()
                .map(|_| Blind(C::Scalar::random(&mut rng)))
                .collect();

            Ok((advice, advice_blinds))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Commit to every advice column of every circuit at once, then hash the
    // commitments circuit by circuit.
    let advice_commitments: Vec<Vec<C>> = synthesized
        .par_iter()
        .map(|(advice, advice_blinds)| {
            let projective: Vec<_> = advice
                .par_iter()
                .zip(advice_blinds.par_iter())
                .map(|(poly, blind)| params.commit_lagrange(poly, *blind))
                .collect();
            let mut commitments = vec![C::identity(); projective.len()];
            C::Curve::batch_normalize_vartime(&projective, &mut commitments);
            commitments
        })
        .collect();
    for commitment in advice_commitments.iter().flatten() {
        transcript.write_point(*commitment)?;
    }

    let advice: Vec<AdviceSingle<C>> = synthesized
        .into_par_iter()
        .map(|(advice_values, advice_blinds)| {
            let advice_polys: Vec<_> = advice_values
                .par_iter()
                .map(|poly| domain.lagrange_to_coeff(poly.clone()))
                .collect();
            let advice_cosets: Vec<_> = advice_polys
                .par_iter()
                .map(|poly| domain.coeff_to_extended(poly.clone()))
                .collect();
            AdviceSingle {
                advice_values,
                advice_polys,
                advice_cosets,
                advice_blinds,
            }
        })
        .collect();

    // Create polynomial evaluator context for values.
    let mut value_evaluator = poly::new_evaluator(|| {});

    // Register fixed values with the polynomial evaluator.
    let fixed_values: Vec<_> = pk
        .fixed_values
        .iter()
        .map(|poly| value_evaluator.register_poly(poly.clone()))
        .collect();

    // Register advice values with the polynomial evaluator.
    let advice_values: Vec<_> = advice
        .iter()
        .map(|advice| {
            advice
                .advice_values
                .iter()
                .map(|poly| value_evaluator.register_poly(poly.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    // Register instance values with the polynomial evaluator.
    let instance_values: Vec<_> = instance
        .iter()
        .map(|instance| {
            instance
                .instance_values
                .iter()
                .map(|poly| value_evaluator.register_poly(poly.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    // Create polynomial evaluator context for cosets.
    let mut coset_evaluator = poly::new_evaluator(|| {});

    // Register fixed cosets with the polynomial evaluator.
    let fixed_cosets: Vec<_> = pk
        .fixed_cosets
        .iter()
        .map(|poly| coset_evaluator.register_poly(poly.clone()))
        .collect();

    // Register advice cosets with the polynomial evaluator.
    let advice_cosets: Vec<_> = advice
        .iter()
        .map(|advice| {
            advice
                .advice_cosets
                .iter()
                .map(|poly| coset_evaluator.register_poly(poly.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    // Register instance cosets with the polynomial evaluator.
    let instance_cosets: Vec<_> = instance
        .iter()
        .map(|instance| {
            instance
                .instance_cosets
                .iter()
                .map(|poly| coset_evaluator.register_poly(poly.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    // Register permutation cosets with the polynomial evaluator.
    let permutation_cosets: Vec<_> = pk
        .permutation
        .cosets
        .iter()
        .map(|poly| coset_evaluator.register_poly(poly.clone()))
        .collect();

    // Register boundary polynomials used in the lookup and permutation arguments.
    let l0 = coset_evaluator.register_poly(pk.l0.clone());
    let l_blind = coset_evaluator.register_poly(pk.l_blind.clone());
    let l_last = coset_evaluator.register_poly(pk.l_last.clone());

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta: ChallengeTheta<_> = transcript.squeeze_challenge_scalar();

    // Permute and commit to every lookup of every circuit at once, then hash
    // the commitments in order.
    let num_lookups = pk.vk.cs.lookups.len();
    let permuted_draws = lookup::prover::permuted_draws(pk);
    let lookup_tasks: Vec<_> = (0..instances.len())
        .flat_map(|circuit| pk.vk.cs.lookups.iter().map(move |lookup| (circuit, lookup)))
        .map(|(circuit, lookup)| (circuit, lookup, Draws::take(&mut rng, permuted_draws)))
        .collect();
    let pending: Vec<_> = lookup_tasks
        .into_par_iter()
        .map(|(circuit, lookup, draws)| {
            lookup.commit_permuted(
                pk,
                params,
                domain,
                &value_evaluator,
                theta,
                &advice_values[circuit],
                &fixed_values,
                &instance_values[circuit],
                &advice_cosets[circuit],
                &fixed_cosets,
                &instance_cosets[circuit],
                draws,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut pending = pending.into_iter();
    let lookups: Vec<Vec<lookup::prover::Permuted<C, _>>> = (0..instances.len())
        .map(|_| {
            (&mut pending)
                .take(num_lookups)
                .map(|permuted| permuted.finish(&mut coset_evaluator, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample beta challenge
    let beta: ChallengeBeta<_> = transcript.squeeze_challenge_scalar();

    // Sample gamma challenge
    let gamma: ChallengeGamma<_> = transcript.squeeze_challenge_scalar();

    // Commit to permutations, every circuit at once, then hash in order.
    let permutation_draws = pk.vk.cs.permutation.commit_draws(pk);
    let permutation_tasks: Vec<_> = instance
        .iter()
        .zip(advice.iter())
        .map(|(instance, advice)| (instance, advice, Draws::take(&mut rng, permutation_draws)))
        .collect();
    let pending: Vec<_> = permutation_tasks
        .into_par_iter()
        .map(|(instance, advice, draws)| {
            pk.vk.cs.permutation.commit(
                params,
                pk,
                &pk.permutation,
                &advice.advice_values,
                &pk.fixed_values,
                &instance.instance_values,
                beta,
                gamma,
                draws,
            )
        })
        .collect();
    let permutations: Vec<permutation::prover::Committed<C, _>> = pending
        .into_iter()
        .map(|permutation| permutation.finish(&mut coset_evaluator, transcript))
        .collect::<Result<Vec<_>, _>>()?;

    // Construct and commit to products for each lookup, every circuit at
    // once, then hash in order.
    let product_draws = lookup::prover::product_draws(pk);
    let product_tasks: Vec<_> = lookups
        .into_iter()
        .flatten()
        .map(|permuted| (permuted, Draws::take(&mut rng, product_draws)))
        .collect();
    let pending: Vec<_> = product_tasks
        .into_par_iter()
        .map(|(permuted, draws)| permuted.commit_product(pk, params, beta, gamma, draws))
        .collect();
    let mut pending = pending.into_iter();
    let lookups: Vec<Vec<lookup::prover::Committed<C, _>>> = (0..instances.len())
        .map(|_| {
            (&mut pending)
                .take(num_lookups)
                .map(|committed| committed.finish(&mut coset_evaluator, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Commit to the vanishing argument's random polynomial for blinding h(x_3)
    let vanishing = vanishing::Argument::commit(params, domain, &mut rng, transcript)?;

    // Obtain challenge for keeping all separate gates linearly independent
    let y: ChallengeY<_> = transcript.squeeze_challenge_scalar();

    // Evaluate the h(X) polynomial's constraint system expressions for the permutation constraints.
    let (permutations, permutation_expressions): (Vec<_>, Vec<_>) = permutations
        .into_iter()
        .zip(advice_cosets.iter())
        .zip(instance_cosets.iter())
        .map(|((permutation, advice), instance)| {
            permutation.construct(
                pk,
                &pk.vk.cs.permutation,
                advice,
                &fixed_cosets,
                instance,
                &permutation_cosets,
                l0,
                l_blind,
                l_last,
                beta,
                gamma,
            )
        })
        .unzip();

    let (lookups, lookup_expressions): (Vec<Vec<_>>, Vec<Vec<_>>) = lookups
        .into_iter()
        .map(|lookups| {
            // Evaluate the h(X) polynomial's constraint system expressions for the lookup constraints, if any.
            lookups
                .into_iter()
                .map(|p| p.construct(beta, gamma, l0, l_blind, l_last))
                .unzip()
        })
        .unzip();

    let expressions = advice_cosets
        .iter()
        .zip(instance_cosets.iter())
        .zip(permutation_expressions)
        .zip(lookup_expressions)
        .flat_map(
            |(((advice_cosets, instance_cosets), permutation_expressions), lookup_expressions)| {
                let fixed_cosets = &fixed_cosets;
                iter::empty()
                    // Custom constraints
                    .chain(meta.gates.iter().flat_map(move |gate| {
                        gate.polynomials().iter().map(move |expr| {
                            expr.evaluate(
                                &poly::Ast::ConstantTerm,
                                &|_| panic!("virtual selectors are removed during optimization"),
                                &|query| {
                                    fixed_cosets[query.column_index]
                                        .with_rotation(query.rotation)
                                        .into()
                                },
                                &|query| {
                                    advice_cosets[query.column_index]
                                        .with_rotation(query.rotation)
                                        .into()
                                },
                                &|query| {
                                    instance_cosets[query.column_index]
                                        .with_rotation(query.rotation)
                                        .into()
                                },
                                &|a| -a,
                                &|a, b| a + b,
                                &|a, b| a * b,
                                &|a, scalar| a * scalar,
                            )
                        })
                    }))
                    // Permutation constraints, if any.
                    .chain(permutation_expressions)
                    // Lookup constraints, if any.
                    .chain(lookup_expressions.into_iter().flatten())
            },
        );

    // Construct the vanishing argument's h(X) commitments
    let vanishing = vanishing.construct(
        params,
        domain,
        coset_evaluator,
        expressions,
        y,
        &mut rng,
        transcript,
    )?;

    let x: ChallengeX<_> = transcript.squeeze_challenge_scalar();
    let xn = x.pow([params.n, 0, 0, 0]);

    // Every evaluation at a rotation of x is independent: compute them all at
    // once, then hash them in transcript order (the instance evals of each
    // circuit, the advice evals of each circuit, then the fixed evals, which
    // are shared across circuits).
    let instance_queries = instance.iter().flat_map(|instance| {
        meta.instance_queries.iter().map(move |&(column, at)| {
            (
                &instance.instance_polys[column.index()][..],
                domain.rotate_omega(*x, at),
            )
        })
    });
    let advice_queries = advice.iter().flat_map(|advice| {
        meta.advice_queries.iter().map(move |&(column, at)| {
            (
                &advice.advice_polys[column.index()][..],
                domain.rotate_omega(*x, at),
            )
        })
    });
    let fixed_queries = meta.fixed_queries.iter().map(|&(column, at)| {
        (
            &pk.fixed_polys[column.index()][..],
            domain.rotate_omega(*x, at),
        )
    });
    let queries: Vec<(&[C::Scalar], C::Scalar)> = instance_queries
        .chain(advice_queries)
        .chain(fixed_queries)
        .collect();
    for eval in evaluate_batch(&queries) {
        transcript.write_scalar(eval)?;
    }

    let vanishing = vanishing.evaluate(x, xn, domain, transcript)?;

    // Evaluate common permutation data
    pk.permutation.evaluate(x, transcript)?;

    // Evaluate the permutations, if any, at omega^i x.
    let permutations: Vec<permutation::prover::Evaluated<C>> = permutations
        .into_iter()
        .map(|permutation| -> Result<_, _> { permutation.evaluate(pk, x, transcript) })
        .collect::<Result<Vec<_>, _>>()?;

    // Evaluate the lookups, if any, at omega^i x.
    let lookups: Vec<Vec<lookup::prover::Evaluated<C>>> = lookups
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            lookups
                .into_iter()
                .map(|p| p.evaluate(pk, x, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let instances = instance
        .iter()
        .zip(advice.iter())
        .zip(permutations.iter())
        .zip(lookups.iter())
        .flat_map(|(((instance, advice), permutation), lookups)| {
            iter::empty()
                .chain(
                    pk.vk
                        .cs
                        .instance_queries
                        .iter()
                        .map(move |&(column, at)| ProverQuery {
                            point: domain.rotate_omega(*x, at),
                            poly: &instance.instance_polys[column.index()],
                            blind: Blind::default(),
                        }),
                )
                .chain(
                    pk.vk
                        .cs
                        .advice_queries
                        .iter()
                        .map(move |&(column, at)| ProverQuery {
                            point: domain.rotate_omega(*x, at),
                            poly: &advice.advice_polys[column.index()],
                            blind: advice.advice_blinds[column.index()],
                        }),
                )
                .chain(permutation.open(pk, x))
                .chain(lookups.iter().flat_map(move |p| p.open(pk, x)))
        })
        .chain(
            pk.vk
                .cs
                .fixed_queries
                .iter()
                .map(|&(column, at)| ProverQuery {
                    point: domain.rotate_omega(*x, at),
                    poly: &pk.fixed_polys[column.index()],
                    blind: Blind::default(),
                }),
        )
        .chain(pk.permutation.open(x))
        // We query the h(X) polynomial at x
        .chain(vanishing.open(x));

    multiopen::create_proof(params, rng, transcript, instances).map_err(|_| Error::Opening)
}

#[test]
fn test_commit_instance() {
    use pasta_curves::{EqAffine, Fp};

    let params: Params<EqAffine> = Params::new(3);
    let domain = crate::poly::EvaluationDomain::new(1, 3);
    let instance = [Fp::from(3), Fp::ZERO, Fp::from(5)];

    for instance in [&[][..], &instance[..]] {
        let mut poly = domain.empty_lagrange();
        for (coefficient, value) in poly.iter_mut().zip(instance.iter()) {
            *coefficient = *value;
        }

        assert_eq!(
            params.commit_lagrange_instance(instance),
            params.commit_lagrange(&poly, Blind::default())
        );
    }
}

#[test]
fn test_create_proof() {
    use crate::{
        circuit::SimpleFloorPlanner,
        plonk::{keygen_pk, keygen_vk},
        transcript::{Blake2bWrite, Challenge255},
    };
    use pasta_curves::EqAffine;
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    #[derive(Clone, Copy)]
    struct MyCircuit;

    impl<F: Field> Circuit<F> for MyCircuit {
        type Config = ();

        type FloorPlanner = SimpleFloorPlanner;

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

    let params: Params<EqAffine> = Params::new(3);
    let vk = keygen_vk(&params, &MyCircuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &MyCircuit).expect("keygen_pk should not fail");
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

    // Create proof with wrong number of instances
    let proof = create_proof(
        &params,
        &pk,
        &[MyCircuit, MyCircuit],
        &[],
        UnwrapErr(SysRng),
        &mut transcript,
    );
    assert!(matches!(proof.unwrap_err(), Error::InvalidInstances));

    // Create proof with correct number of instances
    create_proof(
        &params,
        &pk,
        &[MyCircuit, MyCircuit],
        &[&[], &[]],
        UnwrapErr(SysRng),
        &mut transcript,
    )
    .expect("proof generation should not fail");
}
