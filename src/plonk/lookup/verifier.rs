use std::iter;

use super::super::circuit::{Any, Column};
use super::{Argument, Proof};
use crate::{
    arithmetic::CurveAffine,
    plonk::{ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error, VerifyingKey},
    poly::{multiopen::VerifierQuery, Rotation},
    transcript::{Hasher, Transcript},
};
use ff::Field;

impl<C: CurveAffine> Proof<C> {
    pub(in crate::plonk) fn absorb_permuted_commitments<
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<(), Error> {
        transcript
            .absorb_point(&self.permuted_input_commitment)
            .map_err(|_| Error::TranscriptError)?;
        transcript
            .absorb_point(&self.permuted_table_commitment)
            .map_err(|_| Error::TranscriptError)
    }

    pub(in crate::plonk) fn absorb_product_commitment<
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<(), Error> {
        transcript
            .absorb_point(&self.product_commitment)
            .map_err(|_| Error::TranscriptError)
    }

    pub(in crate::plonk) fn expressions<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        l_0: C::Scalar,
        argument: &'a Argument,
        theta: ChallengeTheta<C::Scalar>,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        advice_evals: &[C::Scalar],
        fixed_evals: &[C::Scalar],
        aux_evals: &[C::Scalar],
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        let product_expression = || {
            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            let left = self.product_eval
                * &(self.permuted_input_eval + &beta)
                * &(self.permuted_table_eval + &gamma);

            let compress_columns = |columns: &[Column<Any>]| {
                columns
                    .iter()
                    .map(|column| {
                        let index = vk.cs.get_any_query_index(*column, 0);
                        match column.column_type() {
                            Any::Advice => advice_evals[index],
                            Any::Fixed => fixed_evals[index],
                            Any::Aux => aux_evals[index],
                        }
                    })
                    .fold(C::Scalar::zero(), |acc, eval| acc * &theta + &eval)
            };
            let right = self.product_inv_eval
                * &(compress_columns(&argument.input_columns) + &beta)
                * &(compress_columns(&argument.table_columns) + &gamma);

            left - &right
        };

        std::iter::empty()
            .chain(
                // l_0(X) * (1 - z'(X)) = 0
                Some(l_0 * &(C::Scalar::one() - &self.product_eval)),
            )
            .chain(
                // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
                // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
                Some(product_expression()),
            )
            .chain(Some(
                // l_0(X) * (a'(X) - s'(X)) = 0
                l_0 * &(self.permuted_input_eval - &self.permuted_table_eval),
            ))
            .chain(Some(
                // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
                (self.permuted_input_eval - &self.permuted_table_eval)
                    * &(self.permuted_input_eval - &self.permuted_input_inv_eval),
            ))
    }

    pub(in crate::plonk) fn evals(&self) -> impl Iterator<Item = &C::Scalar> {
        iter::empty()
            .chain(Some(&self.product_eval))
            .chain(Some(&self.product_inv_eval))
            .chain(Some(&self.permuted_input_eval))
            .chain(Some(&self.permuted_input_inv_eval))
            .chain(Some(&self.permuted_table_eval))
    }

    pub(in crate::plonk) fn queries<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = VerifierQuery<'a, C>> + Clone {
        let x_inv = vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open lookup product commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.product_commitment,
                eval: self.product_eval,
            }))
            // Open lookup input commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.permuted_input_commitment,
                eval: self.permuted_input_eval,
            }))
            // Open lookup table commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.permuted_table_commitment,
                eval: self.permuted_table_eval,
            }))
            // Open lookup input commitments at \omega^{-1} x
            .chain(Some(VerifierQuery {
                point: x_inv,
                commitment: &self.permuted_input_commitment,
                eval: self.permuted_input_inv_eval,
            }))
            // Open lookup product commitments at \omega^{-1} x
            .chain(Some(VerifierQuery {
                point: x_inv,
                commitment: &self.product_commitment,
                eval: self.product_inv_eval,
            }))
    }
}
