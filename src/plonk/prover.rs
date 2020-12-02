use ff::Field;
use std::iter;

use super::{
    circuit::{Advice, Assignment, Circuit, Column, ConstraintSystem, Fixed},
    permutation, vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, ChallengeY,
    Error, Proof, ProvingKey,
};
use crate::arithmetic::{eval_polynomial, Curve, CurveAffine, FieldExt};
use crate::poly::{
    commitment::{Blind, Params},
    multiopen::{self, ProverQuery},
    LagrangeCoeff, Polynomial,
};
use crate::transcript::{Hasher, Transcript};

impl<C: CurveAffine> Proof<C> {
    /// This creates a proof for the provided `circuit` when given the public
    /// parameters `params` and the proving key [`ProvingKey`] that was
    /// generated previously for the same circuit.
    pub fn create<
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
        ConcreteCircuit: Circuit<C::Scalar>,
    >(
        params: &Params<C>,
        pk: &ProvingKey<C>,
        circuit: &ConcreteCircuit,
        aux: &[Polynomial<C::Scalar, LagrangeCoeff>],
    ) -> Result<Self, Error> {
        if aux.len() != pk.vk.cs.num_aux_columns {
            return Err(Error::IncompatibleParams);
        }

        struct WitnessCollection<F: Field> {
            advice: Vec<Polynomial<F, LagrangeCoeff>>,
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

        let domain = &pk.vk.domain;
        let mut meta = ConstraintSystem::default();
        let config = ConcreteCircuit::configure(&mut meta);

        let mut witness = WitnessCollection {
            advice: vec![domain.empty_lagrange(); meta.num_advice_columns],
            _marker: std::marker::PhantomData,
        };

        // Synthesize the circuit to obtain the witness and other information.
        circuit.synthesize(&mut witness, config)?;

        let witness = witness;

        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = Transcript::<C, HBase, HScalar>::new();

        // Compute commitments to aux column polynomials
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
                .absorb_point(commitment)
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
                .absorb_point(commitment)
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

        // Sample theta challenge for keeping lookup columns linearly independent
        let theta = ChallengeTheta::get(&mut transcript);

        // Construct and commit to permuted values for each lookup
        let lookups = pk
            .vk
            .cs
            .lookups
            .iter()
            .map(|lookup| {
                lookup.commit_permuted(
                    &pk,
                    &params,
                    &domain,
                    theta,
                    &witness.advice,
                    &pk.fixed_values,
                    &aux,
                    &advice_cosets,
                    &pk.fixed_cosets,
                    &aux_cosets,
                    &mut transcript,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Sample beta challenge
        let beta = ChallengeBeta::get(&mut transcript);

        // Sample gamma challenge
        let gamma = ChallengeGamma::get(&mut transcript);

        // Commit to permutations, if any.
        let permutations = if !pk.vk.cs.permutations.is_empty() {
            Some(permutation::Argument::commit(
                params,
                pk,
                &witness.advice,
                beta,
                gamma,
                &mut transcript,
            )?)
        } else {
            None
        };

        // Construct and commit to products for each lookup
        let lookups = lookups
            .into_iter()
            .map(|lookup| lookup.commit_product(&pk, &params, theta, beta, gamma, &mut transcript))
            .collect::<Result<Vec<_>, _>>()?;

        // Obtain challenge for keeping all separate gates linearly independent
        let y = ChallengeY::get(&mut transcript);

        // Evaluate the h(X) polynomial's constraint system expressions for the permutation constraints, if any.
        let (permutations, permutation_expressions) = permutations
            .map(|p| p.construct(pk, &advice_cosets, beta, gamma))
            .transpose()?
            .map(|(p, expressions)| (Some(p), Some(expressions)))
            .unwrap_or_default();

        // Evaluate the h(X) polynomial's constraint system expressions for the lookup constraints, if any.
        let (lookups, lookup_expressions): (Vec<_>, Vec<_>) = {
            let tmp = lookups
                .into_iter()
                .map(|p| p.construct(pk, theta, beta, gamma))
                .collect::<Result<Vec<_>, _>>()?;

            tmp.into_iter().unzip()
        };

        // Evaluate the h(X) polynomial's constraint system expressions for the constraints provided
        let expressions = iter::empty()
            // Custom constraints
            .chain(meta.gates.iter().map(|poly| {
                poly.evaluate(
                    &|index| pk.fixed_cosets[index].clone(),
                    &|index| advice_cosets[index].clone(),
                    &|index| aux_cosets[index].clone(),
                    &|a, b| a + &b,
                    &|a, b| a * &b,
                    &|a, scalar| a * scalar,
                )
            }))
            // Permutation constraints, if any.
            .chain(permutation_expressions.into_iter().flatten())
            // Lookup constraints, if any.
            .chain(lookup_expressions.into_iter().flatten());

        // Construct the vanishing argument
        let vanishing =
            vanishing::Argument::construct(params, domain, expressions, y, &mut transcript)?;

        let x = ChallengeX::get(&mut transcript);

        // Evaluate polynomials at omega^i x
        let advice_evals: Vec<_> = meta
            .advice_queries
            .iter()
            .map(|&(column, at)| {
                eval_polynomial(&advice_polys[column.index()], domain.rotate_omega(*x, at))
            })
            .collect();

        let aux_evals: Vec<_> = meta
            .aux_queries
            .iter()
            .map(|&(column, at)| {
                eval_polynomial(&aux_polys[column.index()], domain.rotate_omega(*x, at))
            })
            .collect();

        let fixed_evals: Vec<_> = meta
            .fixed_queries
            .iter()
            .map(|&(column, at)| {
                eval_polynomial(&pk.fixed_polys[column.index()], domain.rotate_omega(*x, at))
            })
            .collect();

        // Hash each column evaluation
        for eval in advice_evals
            .iter()
            .chain(aux_evals.iter())
            .chain(fixed_evals.iter())
        {
            transcript.absorb_scalar(*eval);
        }

        let vanishing = vanishing.evaluate(x, &mut transcript);

        // Evaluate the permutations, if any, at omega^i x.
        let permutations = permutations.map(|p| p.evaluate(pk, x, &mut transcript));

        // Evaluate the lookups, if any, at omega^i x.
        let lookups = lookups
            .into_iter()
            .map(|p| p.evaluate(pk, x, &mut transcript))
            .collect::<Vec<_>>();

        let instances =
            iter::empty()
                .chain(pk.vk.cs.advice_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| ProverQuery {
                        point: domain.rotate_omega(*x, at),
                        poly: &advice_polys[column.index()],
                        blind: advice_blinds[column.index()],
                        eval: advice_evals[query_index],
                    },
                ))
                .chain(pk.vk.cs.aux_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| ProverQuery {
                        point: domain.rotate_omega(*x, at),
                        poly: &aux_polys[column.index()],
                        blind: Blind::default(),
                        eval: aux_evals[query_index],
                    },
                ))
                .chain(pk.vk.cs.fixed_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| ProverQuery {
                        point: domain.rotate_omega(*x, at),
                        poly: &pk.fixed_polys[column.index()],
                        blind: Blind::default(),
                        eval: fixed_evals[query_index],
                    },
                ))
                // We query the h(X) polynomial at x
                .chain(vanishing.open(x));

        let multiopening = multiopen::Proof::create(
            params,
            &mut transcript,
            instances
                .chain(
                    permutations
                        .as_ref()
                        .map(|p| p.open(pk, x))
                        .into_iter()
                        .flatten(),
                )
                .chain(lookups.iter().map(|p| p.open(pk, x)).into_iter().flatten()),
        )
        .map_err(|_| Error::OpeningError)?;

        Ok(Proof {
            advice_commitments,
            permutations: permutations.map(|p| p.build()),
            lookups: lookups.into_iter().map(|p| p.build()).collect(),
            advice_evals,
            fixed_evals,
            aux_evals,
            vanishing: vanishing.build(),
            multiopening,
        })
    }
}
