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
    auxs: &[&[Polynomial<C::Scalar, LagrangeCoeff>]],
    transcript: &mut T,
) -> Result<(), Error> {
    for aux in auxs.iter() {
        if aux.len() != pk.vk.cs.num_aux_columns {
            return Err(Error::IncompatibleParams);
        }
    }

    let domain = &pk.vk.domain;
    let mut meta = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut meta);

    struct AuxSingle<C: CurveAffine> {
        pub aux_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        pub aux_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub aux_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    }

    let aux_vec: Result<Vec<_>, _> = auxs
        .iter()
        .map(|aux| -> Result<AuxSingle<C>, Error> {
            let aux_commitments_projective: Vec<_> = aux
                .iter()
                .map(|poly| params.commit_lagrange(poly, Blind::default()))
                .collect();
            let mut aux_commitments = vec![C::zero(); aux_commitments_projective.len()];
            C::Projective::batch_to_affine(&aux_commitments_projective, &mut aux_commitments);
            let aux_commitments = aux_commitments;
            drop(aux_commitments_projective);
            metrics::counter!("aux_commitments", aux_commitments.len() as u64);

            for commitment in &aux_commitments {
                transcript
                    .common_point(*commitment)
                    .map_err(|_| Error::TranscriptError)?;
            }

            let aux_polys: Vec<_> = aux
                .iter()
                .map(|poly| {
                    let lagrange_vec = domain.lagrange_from_vec(poly.to_vec());
                    domain.lagrange_to_coeff(lagrange_vec)
                })
                .collect();

            let aux_cosets: Vec<_> = meta
                .aux_queries
                .iter()
                .map(|&(column, at)| {
                    let poly = aux_polys[column.index()].clone();
                    domain.coeff_to_extended(poly, at)
                })
                .collect();

            Ok(AuxSingle {
                aux_values: aux.to_vec(),
                aux_polys,
                aux_cosets,
            })
        })
        .collect();

    let aux_vec = match aux_vec {
        Ok(aux_vec) => aux_vec,
        Err(err) => return Err(err),
    };

    struct AdviceSingle<C: CurveAffine> {
        pub advice_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
        pub advice_polys: Vec<Polynomial<C::Scalar, Coeff>>,
        pub advice_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
        pub advice_blinds: Vec<Blind<C::Scalar>>,
    }

    let advice_vec: Result<Vec<_>, _> = circuits
        .iter()
        .map(|circuit| -> Result<AdviceSingle<C>, Error> {
            struct WitnessCollection<F: Field> {
                pub advice: Vec<Polynomial<F, LagrangeCoeff>>,
                _marker: std::marker::PhantomData<F>,
            }

            impl<F: Field> Assignment<F> for WitnessCollection<F> {
                fn assign_advice(
                    &mut self,
                    column: Column<Advice>,
                    row: usize,
                    to: impl FnOnce() -> Result<F, Error>,
                ) -> Result<(), Error> {
                    *self
                        .advice
                        .get_mut(column.index())
                        .and_then(|v| v.get_mut(row))
                        .ok_or(Error::BoundsFailure)? = to()?;

                    Ok(())
                }

                fn assign_fixed(
                    &mut self,
                    _: Column<Fixed>,
                    _: usize,
                    _: impl FnOnce() -> Result<F, Error>,
                ) -> Result<(), Error> {
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
            }

            let mut witness = WitnessCollection {
                advice: vec![domain.empty_lagrange(); meta.num_advice_columns],
                _marker: std::marker::PhantomData,
            };

            // Synthesize the circuit to obtain the witness and other information.
            circuit.synthesize(&mut witness, config)?;

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
        .collect();

    let advice_vec = match advice_vec {
        Ok(advice_vec) => advice_vec,
        Err(err) => return Err(err),
    };

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta = ChallengeTheta::get(transcript);

    let lookups_vec: Result<Vec<Vec<_>>, _> = aux_vec
        .iter()
        .zip(advice_vec.iter())
        .map(
            |(aux, advice)| -> Result<Vec<lookup::prover::Permuted<'_, C>>, Error> {
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
                            &aux.aux_values,
                            &advice.advice_cosets,
                            &pk.fixed_cosets,
                            &aux.aux_cosets,
                            transcript,
                        )
                    })
                    .collect()
            },
        )
        .collect();

    let lookups_vec = match lookups_vec {
        Ok(lookups_vec) => lookups_vec,
        Err(err) => return Err(err),
    };

    // Sample beta challenge
    let beta = ChallengeBeta::get(transcript);

    // Sample gamma challenge
    let gamma = ChallengeGamma::get(transcript);

    let permutations_vec: Result<Vec<Vec<_>>, _> = advice_vec
        .iter()
        .map(
            |advice| -> Result<Vec<permutation::prover::Committed<C>>, Error> {
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
                            beta,
                            gamma,
                            transcript,
                        )
                    })
                    .collect()
            },
        )
        .collect();

    let permutations_vec = match permutations_vec {
        Ok(permutations_vec) => permutations_vec,
        Err(err) => return Err(err),
    };

    let lookups_vec: Result<Vec<Vec<lookup::prover::Committed<'_, C>>>, _> = lookups_vec
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            // Construct and commit to products for each lookup
            lookups
                .into_iter()
                .map(|lookup| lookup.commit_product(&pk, &params, theta, beta, gamma, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect();

    let lookups_vec = match lookups_vec {
        Ok(lookups_vec) => lookups_vec,
        Err(err) => return Err(err),
    };

    // Obtain challenge for keeping all separate gates linearly independent
    let y = ChallengeY::get(transcript);

    let (permutations_vec, permutation_expressions_vec): (Vec<Vec<_>>, Vec<Vec<_>>) =
        permutations_vec
            .into_iter()
            .zip(advice_vec.iter())
            .map(|(permutations, advice)| {
                let tmp: Vec<_> = permutations
                    .into_iter()
                    .zip(pk.vk.cs.permutations.iter())
                    .zip(pk.permutations.iter())
                    .map(|((p, argument), pkey)| {
                        p.construct(pk, argument, pkey, &advice.advice_cosets, beta, gamma)
                    })
                    .collect();

                tmp.into_iter().unzip()
            })
            .collect::<Vec<(Vec<_>, Vec<_>)>>()
            .into_iter()
            .unzip();

    let (lookups_vec, lookup_expressions_vec): (Vec<Vec<_>>, Vec<Vec<_>>) = lookups_vec
        .into_iter()
        .map(|lookups| {
            // Evaluate the h(X) polynomial's constraint system expressions for the lookup constraints, if any.
            let tmp: Vec<_> = lookups
                .into_iter()
                .map(|p| p.construct(pk, theta, beta, gamma))
                .collect();

            tmp.into_iter().unzip()
        })
        .collect::<Vec<(Vec<_>, Vec<_>)>>()
        .into_iter()
        .unzip();

    let expressions = advice_vec
        .iter()
        .zip(aux_vec.iter())
        .zip(permutation_expressions_vec.into_iter())
        .zip(lookup_expressions_vec.into_iter())
        .flat_map(
            |(((advice, aux), permutation_expressions), lookup_expressions)| {
                iter::empty()
                    // Custom constraints
                    .chain(meta.gates.iter().map(move |poly| {
                        poly.evaluate(
                            &|index| pk.fixed_cosets[index].clone(),
                            &|index| advice.advice_cosets[index].clone(),
                            &|index| aux.aux_cosets[index].clone(),
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
        )
        .collect::<Vec<_>>()
        .into_iter();

    // Construct the vanishing argument
    let vanishing = vanishing::Argument::construct(params, domain, expressions, y, transcript)?;

    let x = ChallengeX::get(transcript);

    // Compute and hash aux evals for each circuit instance
    for aux in aux_vec.iter() {
        // Evaluate polynomials at omega^i x
        let aux_evals: Vec<_> = meta
            .aux_queries
            .iter()
            .map(|&(column, at)| {
                eval_polynomial(&aux.aux_polys[column.index()], domain.rotate_omega(*x, at))
            })
            .collect();

        // Hash each aux column evaluation
        for eval in aux_evals.iter() {
            transcript
                .write_scalar(*eval)
                .map_err(|_| Error::TranscriptError)?;
        }
    }

    // Compute and hash advice evals for each circuit instance
    for advice in advice_vec.iter() {
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
    let permutations_vec: Result<Vec<Vec<permutation::prover::Evaluated<C>>>, _> = permutations_vec
        .into_iter()
        .map(|permutations| -> Result<Vec<_>, _> {
            permutations
                .into_iter()
                .zip(pk.permutations.iter())
                .map(|(p, pkey)| p.evaluate(pk, pkey, x, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect();

    let permutations_vec = match permutations_vec {
        Ok(permutations_vec) => permutations_vec,
        Err(err) => return Err(err),
    };

    // Evaluate the lookups, if any, at omega^i x.
    let lookups_vec: Result<Vec<Vec<lookup::prover::Evaluated<C>>>, _> = lookups_vec
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            lookups
                .into_iter()
                .map(|p| p.evaluate(pk, x, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect();

    let lookups_vec = match lookups_vec {
        Ok(lookups_vec) => lookups_vec,
        Err(err) => return Err(err),
    };

    let instances = aux_vec
        .iter()
        .zip(advice_vec.iter())
        .zip(permutations_vec.iter())
        .zip(lookups_vec.iter())
        .flat_map(|(((aux, advice), permutations), lookups)| {
            iter::empty()
                .chain(
                    pk.vk
                        .cs
                        .aux_queries
                        .iter()
                        .map(move |&(column, at)| ProverQuery {
                            point: domain.rotate_omega(*x, at),
                            poly: &aux.aux_polys[column.index()],
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
                        .map(move |(p, pkey)| p.open(pk, pkey, x))
                        .into_iter()
                        .flatten(),
                )
                .chain(
                    lookups
                        .iter()
                        .map(move |p| p.open(pk, x))
                        .into_iter()
                        .flatten(),
                )
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(
            pk.vk
                .cs
                .fixed_queries
                .iter()
                .map(move |&(column, at)| ProverQuery {
                    point: domain.rotate_omega(*x, at),
                    poly: &pk.fixed_polys[column.index()],
                    blind: Blind::default(),
                }),
        )
        // We query the h(X) polynomial at x
        .chain(vanishing.open(x));

    multiopen::create_proof(params, transcript, instances).map_err(|_| Error::OpeningError)
}
