use ff::Field;
use std::iter;

use super::{Argument, Proof, VerifyingKey};
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    plonk::{self, ChallengeBeta, ChallengeGamma, ChallengeX, Error},
    poly::{multiopen::VerifierQuery, Rotation},
    transcript::{Hasher, Transcript},
};

impl<C: CurveAffine> Proof<C> {
    pub(crate) fn check_lengths(&self, p: &Argument) -> Result<(), Error> {
        if self.permutation_evals.len() != p.columns.len() {
            return Err(Error::IncompatibleParams);
        }

        Ok(())
    }

    pub(crate) fn absorb_commitments<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<(), Error> {
        transcript
            .absorb_point(&self.permutation_product_commitment)
            .map_err(|_| Error::TranscriptError)
    }

    pub(in crate::plonk) fn expressions<'a>(
        &'a self,
        vk: &'a plonk::VerifyingKey<C>,
        p: &'a Argument,
        advice_evals: &'a [C::Scalar],
        l_0: C::Scalar,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(Some(
                l_0 * &(C::Scalar::one() - &self.permutation_product_eval),
            ))
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma)
            // - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(Some({
                let mut left = self.permutation_product_eval;
                for (advice_eval, permutation_eval) in p
                    .columns
                    .iter()
                    .map(|&column| advice_evals[vk.cs.get_advice_query_index(column, 0)])
                    .zip(self.permutation_evals.iter())
                {
                    left *= &(advice_eval + &(*beta * permutation_eval) + &gamma);
                }

                let mut right = self.permutation_product_inv_eval;
                let mut current_delta = *beta * &x;
                for advice_eval in p
                    .columns
                    .iter()
                    .map(|&column| advice_evals[vk.cs.get_advice_query_index(column, 0)])
                {
                    right *= &(advice_eval + &current_delta + &gamma);
                    current_delta *= &C::Scalar::DELTA;
                }

                left - &right
            }))
    }

    pub(crate) fn evals(&self) -> impl Iterator<Item = &C::Scalar> {
        iter::empty()
            .chain(Some(&self.permutation_product_eval))
            .chain(Some(&self.permutation_product_inv_eval))
            .chain(self.permutation_evals.iter())
    }

    pub(in crate::plonk) fn queries<'a>(
        &'a self,
        vk: &'a plonk::VerifyingKey<C>,
        vkey: &'a VerifyingKey<C>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = VerifierQuery<'a, C>> + Clone {
        let x_inv = vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open permutation product commitments at x and \omega^{-1} x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.permutation_product_commitment,
                eval: self.permutation_product_eval,
            }))
            .chain(Some(VerifierQuery {
                point: x_inv,
                commitment: &self.permutation_product_commitment,
                eval: self.permutation_product_inv_eval,
            }))
            // Open permutation commitments for each permutation argument at x
            .chain(
                vkey.commitments
                    .iter()
                    .zip(self.permutation_evals.iter())
                    .map(move |(commitment, &eval)| VerifierQuery {
                        point: *x,
                        commitment,
                        eval,
                    }),
            )
    }
}
