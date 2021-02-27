use ff::Field;
use group::{Curve, Group};

use crate::{
    arithmetic::CurveAffine,
    plonk::{Error, VerifyingKey},
    poly::multiopen::VerifierQuery,
    transcript::{read_n_points, EncodedChallenge, TranscriptRead},
};

use super::super::{ChallengeX, ChallengeY};
use super::Argument;

pub struct Committed<C: CurveAffine> {
    h_commitments: Vec<C>,
}

pub struct Evaluated<C: CurveAffine> {
    h_commitment: C,
    expected_h_eval: C::Scalar,
}

impl<C: CurveAffine> Argument<C> {
    pub(in crate::plonk) fn read_commitments<E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
        vk: &VerifyingKey<C>,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        // Obtain a commitment to h(X) in the form of multiple pieces of degree n - 1
        let h_commitments = read_n_points(transcript, vk.domain.get_quotient_poly_degree())
            .map_err(|_| Error::TranscriptError)?;

        Ok(Committed { h_commitments })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(in crate::plonk) fn verify(
        self,
        expressions: impl Iterator<Item = C::Scalar>,
        y: ChallengeY<C>,
        xn: C::Scalar,
    ) -> Result<Evaluated<C>, Error> {
        let expected_h_eval = expressions.fold(C::Scalar::zero(), |h_eval, v| h_eval * &*y + &v);
        let expected_h_eval = expected_h_eval * ((xn - C::Scalar::one()).invert().unwrap());

        let h_commitment = self
            .h_commitments
            .iter()
            .rev()
            .fold(C::CurveExt::identity(), |acc, eval| {
                acc * xn + eval.to_curve()
            })
            .to_affine();

        Ok(Evaluated {
            expected_h_eval,
            h_commitment,
        })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn queries(
        &self,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'_, C>> + Clone {
        Some(VerifierQuery {
            point: *x,
            commitment: &self.h_commitment,
            eval: self.expected_h_eval,
        })
        .into_iter()
    }
}
