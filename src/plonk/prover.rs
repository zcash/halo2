use ff::Field;
use std::iter;

use super::{
    circuit::{Advice, Assignment, Circuit, Column, ConstraintSystem, Fixed},
    lookup, permutation, vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX,
    ChallengeY, Error, ProvingKey,
};
use crate::arithmetic::{eval_polynomial, Curve, CurveAffine, FieldExt};
use crate::poly::{
    commitment::{Blind, Params},
    multiopen::{self, ProverQuery},
    Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial,
};
use crate::transcript::TranscriptWrite;

/// This creates a proof for the provided `circuit` when given the public
/// parameters `params` and the proving key [`ProvingKey`] that was
/// generated previously for the same circuit.
pub fn create_proof<C: CurveAffine, T: TranscriptWrite<C>, ConcreteCircuit: Circuit<C::Scalar>>(
    params: &Params<C>,
    pk: &ProvingKey<C>,
    circuits: &[ConcreteCircuit],
    instances: &[&[Polynomial<C::Scalar, LagrangeCoeff>]],
    transcript: &mut T,
) -> Result<(), Error> {
    for instance in instances.iter() {
        if instance.len() != pk.vk.cs.num_instance_columns {
            return Err(Error::IncompatibleParams);
        }
    }

    // Hash verification key into transcript
    pk.vk.hash(transcript).map_err(|_| Error::TranscriptError)?;

    let domain = &pk.vk.domain;
    let mut meta = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut meta);

    struct InstanceSingle<'a, C: CurveAffine> {
        pub instance_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        pub instance_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub instance_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    }

    let instance: Vec<InstanceSingle<C>> = instances
        .iter()
        .map(|instance| -> Result<InstanceSingle<C>, Error> {
            let instance_commitments_projective: Vec<_> = instance
                .iter()
                .map(|poly| params.commit_lagrange(poly, Blind::default()))
                .collect();
            let mut instance_commitments = vec![C::zero(); instance_commitments_projective.len()];
            C::Projective::batch_to_affine(
                &instance_commitments_projective,
                &mut instance_commitments,
            );
            let instance_commitments = instance_commitments;
            drop(instance_commitments_projective);
            metrics::counter!("instance_commitments", instance_commitments.len() as u64);

            for commitment in &instance_commitments {
                transcript
                    .common_point(*commitment)
                    .map_err(|_| Error::TranscriptError)?;
            }

            let instance_polys: Vec<_> = instance
                .iter()
                .map(|poly| {
                    let lagrange_vec = domain.lagrange_from_vec(poly.to_vec());
                    domain.lagrange_to_coeff(lagrange_vec)
                })
                .collect();

            let instance_cosets: Vec<_> = meta
                .instance_queries
                .iter()
                .map(|&(column, at)| {
                    let poly = instance_polys[column.index()].clone();
                    domain.coeff_to_extended(poly, at)
                })
                .collect();

            Ok(InstanceSingle {
                instance_values: *instance,
                instance_polys,
                instance_cosets,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    struct AdviceSingle<C: CurveAffine> {
        pub advice_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        pub advice_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub advice_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
        pub advice_blinds: Vec<Blind<C::Scalar>>,
    }

    let advice: Vec<AdviceSingle<C>> = circuits
        .iter()
        .map(|circuit| -> Result<AdviceSingle<C>, Error> {
            struct WitnessCollection<F: Field> {
                pub advice: Vec<Polynomial<F, LagrangeCoeff>>,
                _marker: std::marker::PhantomData<F>,
            }

            impl<F: Field> Assignment<F> for WitnessCollection<F> {
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

                fn assign_advice<V, A, AR>(
                    &mut self,
                    _: A,
                    column: Column<Advice>,
                    row: usize,
                    to: V,
                ) -> Result<(), Error>
                where
                    V: FnOnce() -> Result<F, Error>,
                    A: FnOnce() -> AR,
                    AR: Into<String>,
                {
                    *self
                        .advice
                        .get_mut(column.index())
                        .and_then(|v| v.get_mut(row))
                        .ok_or(Error::BoundsFailure)? = to()?;

                    Ok(())
                }

                fn assign_fixed<V, A, AR>(
                    &mut self,
                    _: A,
                    _: Column<Fixed>,
                    _: usize,
                    _: V,
                ) -> Result<(), Error>
                where
                    V: FnOnce() -> Result<F, Error>,
                    A: FnOnce() -> AR,
                    AR: Into<String>,
                {
                    // We only care about advice columns here

                    Ok(())
                }

                fn copy(
                    &mut self,
                    _: usize,
                    _: usize,
                    _: usize,
                    _: usize,
                    _: usize,
                ) -> Result<(), Error> {
                    // We only care about advice columns here

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

            let mut witness = WitnessCollection {
                advice: vec![domain.empty_lagrange(); meta.num_advice_columns],
                _marker: std::marker::PhantomData,
            };

            // Synthesize the circuit to obtain the witness and other information.
            circuit.synthesize(&mut witness, config.clone())?;

            let witness = witness;

            // Compute commitments to advice column polynomials
            let advice_blinds: Vec<_> = witness
                .advice
                .iter()
                .map(|_| Blind(C::Scalar::rand()))
                .collect();
            let advice_commitments_projective: Vec<_> = witness
                .advice
                .iter()
                .zip(advice_blinds.iter())
                .map(|(poly, blind)| params.commit_lagrange(poly, *blind))
                .collect();
            let mut advice_commitments = vec![C::zero(); advice_commitments_projective.len()];
            C::Projective::batch_to_affine(&advice_commitments_projective, &mut advice_commitments);
            let advice_commitments = advice_commitments;
            drop(advice_commitments_projective);
            metrics::counter!("advice_commitments", advice_commitments.len() as u64);

            for commitment in &advice_commitments {
                transcript
                    .write_point(*commitment)
                    .map_err(|_| Error::TranscriptError)?;
            }

            let advice_polys: Vec<_> = witness
                .advice
                .clone()
                .into_iter()
                .map(|poly| domain.lagrange_to_coeff(poly))
                .collect();

            let advice_cosets: Vec<_> = meta
                .advice_queries
                .iter()
                .map(|&(column, at)| {
                    let poly = advice_polys[column.index()].clone();
                    domain.coeff_to_extended(poly, at)
                })
                .collect();

            Ok(AdviceSingle {
                advice_values: witness.advice,
                advice_polys,
                advice_cosets,
                advice_blinds,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta = ChallengeTheta::get(transcript);

    let lookups: Vec<Vec<lookup::prover::Permuted<'_, C>>> = instance
        .iter()
        .zip(advice.iter())
        .map(|(instance, advice)| -> Result<Vec<_>, Error> {
            // Construct and commit to permuted values for each lookup
            pk.vk
                .cs
                .lookups
                .iter()
                .map(|lookup| {
                    lookup.commit_permuted(
                        &pk,
                        &params,
                        &domain,
                        theta,
                        &advice.advice_values,
                        &pk.fixed_values,
                        &instance.instance_values,
                        &advice.advice_cosets,
                        &pk.fixed_cosets,
                        &instance.instance_cosets,
                        transcript,
                    )
                })
                .collect()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample beta challenge
    let beta = ChallengeBeta::get(transcript);

    // Sample gamma challenge
    let gamma = ChallengeGamma::get(transcript);

    let permutations: Vec<Vec<permutation::prover::Committed<C>>> = instance
        .iter()
        .zip(advice.iter())
        .map(|(instance, advice)| -> Result<Vec<_>, Error> {
            // Commit to permutations, if any.
            pk.vk
                .cs
                .permutations
                .iter()
                .zip(pk.permutations.iter())
                .map(|(p, pkey)| {
                    p.commit(
                        params,
                        pk,
                        pkey,
                        &advice.advice_values,
                        &pk.fixed_values,
                        &instance.instance_values,
                        beta,
                        gamma,
                        transcript,
                    )
                })
                .collect()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let lookups: Vec<Vec<lookup::prover::Committed<'_, C>>> = lookups
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            // Construct and commit to products for each lookup
            lookups
                .into_iter()
                .map(|lookup| lookup.commit_product(&pk, &params, theta, beta, gamma, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Obtain challenge for keeping all separate gates linearly independent
    let y = ChallengeY::get(transcript);

    let (permutations, permutation_expressions): (Vec<Vec<_>>, Vec<Vec<_>>) = permutations
        .into_iter()
        .zip(advice.iter())
        .zip(instance.iter())
        .map(|((permutations, advice), instance)| {
            // Evaluate the h(X) polynomial's constraint system expressions for the permutation constraints, if any.
            let tmp: Vec<_> = permutations
                .into_iter()
                .zip(pk.vk.cs.permutations.iter())
                .zip(pk.permutations.iter())
                .map(|((p, argument), pkey)| {
                    p.construct(
                        pk,
                        argument,
                        pkey,
                        &advice.advice_cosets,
                        &pk.fixed_cosets,
                        &instance.instance_cosets,
                        beta,
                        gamma,
                    )
                })
                .collect();

            tmp.into_iter().unzip()
        })
        .unzip();

    let (lookups, lookup_expressions): (Vec<Vec<_>>, Vec<Vec<_>>) = lookups
        .into_iter()
        .map(|lookups| {
            // Evaluate the h(X) polynomial's constraint system expressions for the lookup constraints, if any.
            let tmp: Vec<_> = lookups
                .into_iter()
                .map(|p| p.construct(pk, theta, beta, gamma))
                .collect();

            tmp.into_iter().unzip()
        })
        .unzip();

    let expressions = advice
        .iter()
        .zip(instance.iter())
        .zip(permutation_expressions.into_iter())
        .zip(lookup_expressions.into_iter())
        .flat_map(
            |(((advice, instance), permutation_expressions), lookup_expressions)| {
                iter::empty()
                    // Custom constraints
                    .chain(meta.gates.iter().map(move |(_, poly)| {
                        poly.evaluate(
                            &|index| pk.fixed_cosets[index].clone(),
                            &|index| advice.advice_cosets[index].clone(),
                            &|index| instance.instance_cosets[index].clone(),
                            &|a, b| a + &b,
                            &|a, b| a * &b,
                            &|a, scalar| a * scalar,
                        )
                    }))
                    // Permutation constraints, if any.
                    .chain(permutation_expressions.into_iter().flatten())
                    // Lookup constraints, if any.
                    .chain(lookup_expressions.into_iter().flatten())
            },
        );

    // Construct the vanishing argument
    let vanishing = vanishing::Argument::construct(params, domain, expressions, y, transcript)?;

    let x = ChallengeX::get(transcript);

    // Compute and hash instance evals for each circuit instance
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
            transcript
                .write_scalar(*eval)
                .map_err(|_| Error::TranscriptError)?;
        }
    }

    // Compute and hash advice evals for each circuit instance
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

        // Hash each advice column evaluation
        for eval in advice_evals.iter() {
            transcript
                .write_scalar(*eval)
                .map_err(|_| Error::TranscriptError)?;
        }
    }

    // Compute and hash fixed evals (shared across all circuit instances)
    let fixed_evals: Vec<_> = meta
        .fixed_queries
        .iter()
        .map(|&(column, at)| {
            eval_polynomial(&pk.fixed_polys[column.index()], domain.rotate_omega(*x, at))
        })
        .collect();

    // Hash each fixed column evaluation
    for eval in fixed_evals.iter() {
        transcript
            .write_scalar(*eval)
            .map_err(|_| Error::TranscriptError)?;
    }

    let vanishing = vanishing.evaluate(x, transcript)?;

    // Evaluate the permutations, if any, at omega^i x.
    let permutations: Vec<Vec<permutation::prover::Evaluated<C>>> = permutations
        .into_iter()
        .map(|permutations| -> Result<Vec<_>, _> {
            permutations
                .into_iter()
                .zip(pk.permutations.iter())
                .map(|(p, pkey)| p.evaluate(pk, pkey, x, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
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
        .flat_map(|(((instance, advice), permutations), lookups)| {
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
                .chain(
                    permutations
                        .iter()
                        .zip(pk.permutations.iter())
                        .flat_map(move |(p, pkey)| p.open(pk, pkey, x))
                        .into_iter(),
                )
                .chain(lookups.iter().flat_map(move |p| p.open(pk, x)).into_iter())
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
        // We query the h(X) polynomial at x
        .chain(vanishing.open(x));

    multiopen::create_proof(params, transcript, instances).map_err(|_| Error::OpeningError)
}
