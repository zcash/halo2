use std::iter;

use ff::Field;
use group::Curve;
use rand_core::RngCore;

use super::Argument;
use crate::{
    arithmetic::{eval_polynomial, CurveAffine, FieldExt},
    plonk::{ChallengeX, ChallengeY, Error},
    poly::{
        self,
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, Polynomial,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};

pub(in crate::plonk) struct Committed<C: CurveAffine> {
    random_poly: Polynomial<C::Scalar, Coeff>,
}

pub(in crate::plonk) struct Constructed<C: CurveAffine> {
    h_pieces: Vec<Polynomial<C::Scalar, Coeff>>,
    committed: Committed<C>,
}

pub(in crate::plonk) struct Evaluated<C: CurveAffine> {
    h_poly: Polynomial<C::Scalar, Coeff>,
    committed: Committed<C>,
}

impl<C: CurveAffine> Argument<C> {
    pub(in crate::plonk) fn commit<E: EncodedChallenge<C>, R: RngCore, T: TranscriptWrite<C, E>>(
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        mut rng: R,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        // Sample a random polynomial of degree n - 1
        let mut random_poly = domain.empty_coeff();
        for coeff in random_poly.iter_mut() {
            *coeff = C::Scalar::random(&mut rng);
        }

        // Commit
        let c = params.commit(&random_poly).to_affine();
        transcript.write_point(c)?;

        Ok(Committed { random_poly })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(in crate::plonk) fn construct<
        E: EncodedChallenge<C>,
        Ev: Copy + Send + Sync,
        T: TranscriptWrite<C, E>,
    >(
        self,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        evaluator: poly::Evaluator<Ev, C::Scalar, ExtendedLagrangeCoeff>,
        expressions: impl Iterator<Item = poly::Ast<Ev, C::Scalar, ExtendedLagrangeCoeff>>,
        y: ChallengeY<C>,
        transcript: &mut T,
    ) -> Result<Constructed<C>, Error> {
        // Evaluate the h(X) polynomial's constraint system expressions for the constraints provided
        let h_poly = expressions
            .reduce(|h_poly, v| &(&h_poly * *y) + &v) // Fold the gates together with the y challenge
            .unwrap_or_else(|| poly::Ast::ConstantTerm(C::Scalar::zero()));
        let h_poly = evaluator.evaluate(&h_poly, domain); // Evaluate the h(X) polynomial

        // Divide by t(X) = X^{params.n} - 1.
        let h_poly = domain.divide_by_vanishing_poly(h_poly);

        // Obtain final h(X) polynomial
        let h_poly = domain.extended_to_coeff(h_poly);

        // Split h(X) up into pieces
        let h_pieces = h_poly
            .chunks_exact(params.n as usize)
            .map(|v| domain.coeff_from_vec(v.to_vec()))
            .collect::<Vec<_>>();
        drop(h_poly);

        // Compute commitments to each h(X) piece
        let h_commitments_projective: Vec<_> = h_pieces
            .iter()
            .map(|h_piece| params.commit(h_piece))
            .collect();
        let mut h_commitments = vec![C::identity(); h_commitments_projective.len()];
        C::Curve::batch_normalize(&h_commitments_projective, &mut h_commitments);
        let h_commitments = h_commitments;

        // Hash each h(X) piece
        for c in h_commitments.iter() {
            transcript.write_point(*c)?;
        }

        Ok(Constructed {
            h_pieces,
            committed: self,
        })
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        self,
        x: ChallengeX<C>,
        xn: C::Scalar,
        domain: &EvaluationDomain<C::Scalar>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let h_poly = self
            .h_pieces
            .iter()
            .rev()
            .fold(domain.empty_coeff(), |acc, eval| acc * xn + eval);

        let random_eval = eval_polynomial(&self.committed.random_poly, *x);
        transcript.write_scalar(random_eval)?;

        Ok(Evaluated {
            h_poly,
            committed: self.committed,
        })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open(
        &self,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'_, C>> + Clone {
        iter::empty()
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.h_poly,
            }))
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.committed.random_poly,
            }))
    }
}
