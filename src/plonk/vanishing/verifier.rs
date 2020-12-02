use ff::Field;

use super::Proof;
use crate::{
    arithmetic::CurveAffine,
    plonk::{ChallengeX, ChallengeY, Error, VerifyingKey},
    poly::multiopen::VerifierQuery,
    transcript::{Hasher, Transcript},
};

impl<C: CurveAffine> Proof<C> {
    pub(in crate::plonk) fn check_lengths(&self, _vk: &VerifyingKey<C>) -> Result<(), Error> {
        // TODO: check h_evals

        // TODO: check h_commitments

        Ok(())
    }

    pub(in crate::plonk) fn absorb_commitments<
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<(), Error> {
        // Obtain a commitment to h(X) in the form of multiple pieces of degree n - 1
        for c in &self.h_commitments {
            transcript
                .absorb_point(c)
                .map_err(|_| Error::TranscriptError)?;
        }
        Ok(())
    }

    pub(in crate::plonk) fn verify(
        &self,
        expressions: impl Iterator<Item = C::Scalar>,
        y: ChallengeY<C::Scalar>,
        xn: C::Scalar,
    ) -> Result<(), Error> {
        let expected_h_eval = expressions.fold(C::Scalar::zero(), |h_eval, v| h_eval * &y + &v);

        // Compute h(x) from the prover
        let h_eval = self
            .h_evals
            .iter()
            .rev()
            .fold(C::Scalar::zero(), |acc, eval| acc * &xn + eval);

        // Did the prover commit to the correct polynomial?
        if expected_h_eval != (h_eval * &(xn - &C::Scalar::one())) {
            return Err(Error::ConstraintSystemFailure);
        }

        Ok(())
    }

    pub(in crate::plonk) fn evals(&self) -> impl Iterator<Item = &C::Scalar> {
        self.h_evals.iter()
    }

    pub(in crate::plonk) fn queries<'a>(
        &'a self,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = VerifierQuery<'a, C>> + Clone {
        self.h_commitments
            .iter()
            .zip(self.h_evals.iter())
            .map(move |(commitment, &eval)| VerifierQuery {
                point: *x,
                commitment,
                eval,
            })
    }
}
