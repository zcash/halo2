use super::{Argument, Proof};
use crate::{
    arithmetic::{eval_polynomial, Curve, CurveAffine, FieldExt},
    plonk::{ChallengeX, ChallengeY, Error},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, Polynomial,
    },
    transcript::{Hasher, Transcript},
};

pub(in crate::plonk) struct Constructed<C: CurveAffine> {
    h_pieces: Vec<Polynomial<C::Scalar, Coeff>>,
    h_blinds: Vec<Blind<C::Scalar>>,
    h_commitments: Vec<C>,
}

pub(in crate::plonk) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
    h_evals: Vec<C::Scalar>,
}

impl<C: CurveAffine> Argument<C> {
    pub(in crate::plonk) fn construct<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        expressions: impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
        y: ChallengeY<C::Scalar>,
        transcript: &mut Transcript<C, HBase, HScalar>,
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
        let mut h_commitments = vec![C::zero(); h_commitments_projective.len()];
        C::Projective::batch_to_affine(&h_commitments_projective, &mut h_commitments);
        let h_commitments = h_commitments;

        // Hash each h(X) piece
        for c in h_commitments.iter() {
            transcript
                .absorb_point(c)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(Constructed {
            h_pieces,
            h_blinds,
            h_commitments,
        })
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        self,
        x: ChallengeX<C::Scalar>,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Evaluated<C> {
        let h_evals: Vec<_> = self
            .h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, *x))
            .collect();

        // Hash each advice evaluation
        for eval in &h_evals {
            transcript.absorb_scalar(*eval);
        }

        Evaluated {
            constructed: self,
            h_evals,
        }
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        self.constructed
            .h_pieces
            .iter()
            .zip(self.constructed.h_blinds.iter())
            .zip(self.h_evals.iter())
            .map(move |((h_poly, h_blind), h_eval)| ProverQuery {
                point: *x,
                poly: h_poly,
                blind: *h_blind,
                eval: *h_eval,
            })
    }

    pub(in crate::plonk) fn build(self) -> Proof<C> {
        Proof {
            h_commitments: self.constructed.h_commitments,
            h_evals: self.h_evals,
        }
    }
}
