use std::iter;

use super::super::circuit::{Any, Column};
use super::Argument;
use crate::{
    arithmetic::CurveAffine,
    plonk::{ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error, VerifyingKey},
    poly::{multiopen::VerifierQuery, Rotation},
    transcript::TranscriptRead,
};
use ff::Field;

pub struct PermutationCommitments<C: CurveAffine> {
    permuted_input_commitment: C,
    permuted_table_commitment: C,
}

pub struct Committed<C: CurveAffine> {
    permuted: PermutationCommitments<C>,
    product_commitment: C,
}

pub struct Evaluated<C: CurveAffine> {
    committed: Committed<C>,
    product_eval: C::Scalar,
    product_inv_eval: C::Scalar,
    permuted_input_eval: C::Scalar,
    permuted_input_inv_eval: C::Scalar,
    permuted_table_eval: C::Scalar,
}

impl Argument {
    pub(in crate::plonk) fn read_permuted_commitments<C: CurveAffine, T: TranscriptRead<C>>(
        &self,
        transcript: &mut T,
    ) -> Result<PermutationCommitments<C>, Error> {
        let permuted_input_commitment = transcript
            .read_point()
            .map_err(|_| Error::TranscriptError)?;
        let permuted_table_commitment = transcript
            .read_point()
            .map_err(|_| Error::TranscriptError)?;

        Ok(PermutationCommitments {
            permuted_input_commitment,
            permuted_table_commitment,
        })
    }
}

impl<C: CurveAffine> PermutationCommitments<C> {
    pub(in crate::plonk) fn read_product_commitment<T: TranscriptRead<C>>(
        self,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        let product_commitment = transcript
            .read_point()
            .map_err(|_| Error::TranscriptError)?;

        Ok(Committed {
            permuted: self,
            product_commitment,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(crate) fn evaluate<T: TranscriptRead<C>>(
        self,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let product_eval = transcript
            .read_scalar()
            .map_err(|_| Error::TranscriptError)?;
        let product_inv_eval = transcript
            .read_scalar()
            .map_err(|_| Error::TranscriptError)?;
        let permuted_input_eval = transcript
            .read_scalar()
            .map_err(|_| Error::TranscriptError)?;
        let permuted_input_inv_eval = transcript
            .read_scalar()
            .map_err(|_| Error::TranscriptError)?;
        let permuted_table_eval = transcript
            .read_scalar()
            .map_err(|_| Error::TranscriptError)?;

        Ok(Evaluated {
            committed: self,
            product_eval,
            product_inv_eval,
            permuted_input_eval,
            permuted_input_inv_eval,
            permuted_table_eval,
        })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn expressions<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        l_0: C::Scalar,
        argument: &'a Argument,
        theta: ChallengeTheta<C>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        advice_evals: &[C::Scalar],
        fixed_evals: &[C::Scalar],
        instance_evals: &[C::Scalar],
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        let product_expression = || {
            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            let left = self.product_eval
                * &(self.permuted_input_eval + &*beta)
                * &(self.permuted_table_eval + &*gamma);

            let compress_columns = |columns: &[Column<Any>]| {
                columns
                    .iter()
                    .map(|column| {
                        let index = vk.cs.get_any_query_index(*column, Rotation::cur());
                        match column.column_type() {
                            Any::Advice => advice_evals[index],
                            Any::Fixed => fixed_evals[index],
                            Any::Instance => instance_evals[index],
                        }
                    })
                    .fold(C::Scalar::zero(), |acc, eval| acc * &*theta + &eval)
            };
            let right = self.product_inv_eval
                * &(compress_columns(&argument.input_columns) + &*beta)
                * &(compress_columns(&argument.table_columns) + &*gamma);

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

    pub(in crate::plonk) fn queries<'a>(
        &'a self,
        vk: &'a VerifyingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'a, C>> + Clone {
        let x_inv = vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open lookup product commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.committed.product_commitment,
                eval: self.product_eval,
            }))
            // Open lookup input commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.committed.permuted.permuted_input_commitment,
                eval: self.permuted_input_eval,
            }))
            // Open lookup table commitments at x
            .chain(Some(VerifierQuery {
                point: *x,
                commitment: &self.committed.permuted.permuted_table_commitment,
                eval: self.permuted_table_eval,
            }))
            // Open lookup input commitments at \omega^{-1} x
            .chain(Some(VerifierQuery {
                point: x_inv,
                commitment: &self.committed.permuted.permuted_input_commitment,
                eval: self.permuted_input_inv_eval,
            }))
            // Open lookup product commitments at \omega^{-1} x
            .chain(Some(VerifierQuery {
                point: x_inv,
                commitment: &self.committed.product_commitment,
                eval: self.product_inv_eval,
            }))
    }
}
