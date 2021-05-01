use group::Curve;

use super::super::{ChallengeX, ChallengeY};
use super::Argument;
use crate::{
    arithmetic::{eval_polynomial, CurveAffine, FieldExt},
    plonk::Error,
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, Polynomial,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};

pub(in crate::plonk) struct Constructed<C: CurveAffine> {
    h_pieces: Vec<Polynomial<C::Scalar, Coeff>>,
    h_blinds: Vec<Blind<C::Scalar>>,
}

pub(in crate::plonk) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
}

impl<C: CurveAffine> Argument<C> {
    pub(in crate::plonk) fn construct<I, E: EncodedChallenge<C, I>, T: TranscriptWrite<C, I, E>>(
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        expressions: impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
        y: ChallengeY<C>,
        transcript: &mut T,
    ) -> Result<Constructed<C>, Error> {
        // Evaluate the h(X) polynomial's constraint system expressions for the constraints provided
        let h_poly = expressions.fold(domain.empty_extended(), |h_poly, v| h_poly * *y + &v);

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
        let h_blinds: Vec<_> = h_pieces.iter().map(|_| Blind(C::Scalar::rand())).collect();

        // Compute commitments to each h(X) piece
        let h_commitments_projective: Vec<_> = h_pieces
            .iter()
            .zip(h_blinds.iter())
            .map(|(h_piece, blind)| params.commit(&h_piece, *blind))
            .collect();
        let mut h_commitments = vec![C::identity(); h_commitments_projective.len()];
        C::Curve::batch_normalize(&h_commitments_projective, &mut h_commitments);
        let h_commitments = h_commitments;

        // Hash each h(X) piece
        for c in h_commitments.iter() {
            transcript
                .write_point(*c)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(Constructed { h_pieces, h_blinds })
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<I, E: EncodedChallenge<C, I>, T: TranscriptWrite<C, I, E>>(
        self,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let h_evals: Vec<_> = self
            .h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, *x))
            .collect();

        // Hash each advice evaluation
        for eval in &h_evals {
            transcript
                .write_scalar(*eval)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(Evaluated { constructed: self })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open(
        &self,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'_, C>> + Clone {
        self.constructed
            .h_pieces
            .iter()
            .zip(self.constructed.h_blinds.iter())
            .map(move |(h_poly, h_blind)| ProverQuery {
                point: *x,
                poly: h_poly,
                blind: *h_blind,
            })
    }
}
