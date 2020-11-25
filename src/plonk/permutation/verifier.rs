use ff::Field;
use std::iter;

use super::Proof;
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    plonk::{ChallengeBeta, ChallengeGamma, ChallengeX, Error, VerifyingKey},
    poly::{multiopen::VerifierQuery, Rotation},
    transcript::{Hasher, Transcript},
};

impl<C: CurveAffine> Proof<C> {
    pub(crate) fn check_lengths(&self, vk: &VerifyingKey<C>) -> Result<(), Error> {
        if self.permutation_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        for (permutation_evals, permutation) in
            self.permutation_evals.iter().zip(vk.cs.permutations.iter())
        {
            if permutation_evals.len() != permutation.len() {
                return Err(Error::IncompatibleParams);
            }
        }

        if self.permutation_product_inv_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        if self.permutation_product_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        if self.permutation_product_commitments.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        Ok(())
    }

    pub(crate) fn absorb_commitments<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<(), Error> {
        for c in &self.permutation_product_commitments {
            transcript
                .absorb_point(c)
                .map_err(|_| Error::TranscriptError)?;
        }
        Ok(())
    }

    pub(crate) fn expressions<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        advice_evals: &'a [C::Scalar],
        l_0: C::Scalar,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(
                self.permutation_product_evals
                    .iter()
                    .map(move |product_eval| l_0 * &(C::Scalar::one() - product_eval)),
            )
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma)
            // - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(
                vk.cs
                    .permutations
                    .iter()
                    .zip(self.permutation_evals.iter())
                    .zip(self.permutation_product_evals.iter())
                    .zip(self.permutation_product_inv_evals.iter())
                    .map(
                        move |(((columns, permutation_evals), product_eval), product_inv_eval)| {
                            let mut left = *product_eval;
                            for (advice_eval, permutation_eval) in columns
                                .iter()
                                .map(|&column| {
                                    advice_evals[vk.cs.get_advice_query_index(column, 0)]
                                })
                                .zip(permutation_evals.iter())
                            {
                                left *= &(advice_eval + &(*beta * permutation_eval) + &gamma);
                            }

                            let mut right = *product_inv_eval;
                            let mut current_delta = *beta * &x;
                            for advice_eval in columns.iter().map(|&column| {
                                advice_evals[vk.cs.get_advice_query_index(column, 0)]
                            }) {
                                right *= &(advice_eval + &current_delta + &gamma);
                                current_delta *= &C::Scalar::DELTA;
                            }

                            left - &right
                        },
                    ),
            )
    }

    pub(crate) fn evals(&self) -> impl Iterator<Item = &C::Scalar> {
        self.permutation_product_evals
            .iter()
            .chain(self.permutation_product_inv_evals.iter())
            .chain(self.permutation_evals.iter().flat_map(|evals| evals.iter()))
    }

    pub(crate) fn queries<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = VerifierQuery<'a, C>> + Clone {
        let x_inv = vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open permutation product commitments at x
            .chain(
                self.permutation_product_commitments
                    .iter()
                    .enumerate()
                    .zip(self.permutation_product_evals.iter())
                    .map(move |((idx, _), &eval)| VerifierQuery {
                        point: *x,
                        commitment: &self.permutation_product_commitments[idx],
                        eval,
                    }),
            )
            // Open permutation commitments for each permutation argument at x
            .chain(
                (0..vk.permutation_commitments.len())
                    .map(move |outer_idx| {
                        let inner_len = vk.permutation_commitments[outer_idx].len();
                        (0..inner_len).map(move |inner_idx| VerifierQuery {
                            point: *x,
                            commitment: &vk.permutation_commitments[outer_idx][inner_idx],
                            eval: self.permutation_evals[outer_idx][inner_idx],
                        })
                    })
                    .flatten(),
            )
            // Open permutation product commitments at \omega^{-1} x
            .chain(
                self.permutation_product_commitments
                    .iter()
                    .enumerate()
                    .zip(self.permutation_product_inv_evals.iter())
                    .map(move |((idx, _), &eval)| VerifierQuery {
                        point: x_inv,
                        commitment: &self.permutation_product_commitments[idx],
                        eval,
                    }),
            )
    }
}
