use ff::Field;

use crate::{
    arithmetic::CurveAffine,
    plonk::{Error, VerifyingKey},
    poly::{
        commitment::{Params, MSM},
        multiopen::VerifierQuery,
    },
    transcript::{read_n_points, EncodedChallenge, TranscriptRead},
};

use super::super::{ChallengeX, ChallengeY};
use super::Argument;

pub struct Committed<C: CurveAffine> {
    h_commitments: Vec<C>,
}

pub struct Evaluated<'params, C: CurveAffine> {
    h_commitment: MSM<'params, C>,
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
        params: &Params<C>,
        expressions: impl Iterator<Item = C::Scalar>,
        y: ChallengeY<C>,
        xn: C::Scalar,
    ) -> Evaluated<C> {
        let expected_h_eval = expressions.fold(C::Scalar::zero(), |h_eval, v| h_eval * &*y + &v);
        let expected_h_eval = expected_h_eval * ((xn - C::Scalar::one()).invert().unwrap());

        let h_commitment =
            self.h_commitments
                .iter()
                .rev()
                .fold(params.empty_msm(), |mut acc, commitment| {
                    acc.scale(xn);
                    acc.append_term(C::Scalar::one(), *commitment);
                    acc
                });

        Evaluated {
            expected_h_eval,
            h_commitment,
        }
    }
}

impl<'params, C: CurveAffine> Evaluated<'params, C> {
    pub(in crate::plonk) fn queries<'r>(
        &'r self,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'r, 'params, C>> + Clone
    where
        'params: 'r,
    {
        Some(VerifierQuery::new_msm(
            &self.h_commitment,
            *x,
            self.expected_h_eval,
        ))
        .into_iter()
    }
}
