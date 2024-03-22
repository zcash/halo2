use std::iter;

use super::Argument;
use crate::{
    arithmetic::CurveAffine,
    plonk::circuit::{ExpressionBack, QueryBack, VarBack},
    plonk::{ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error, VerifyingKey},
    poly::{commitment::MSM, VerifierQuery},
    transcript::{EncodedChallenge, TranscriptRead},
};
use halo2_middleware::circuit::Any;
use halo2_middleware::ff::Field;
use halo2_middleware::poly::Rotation;

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
    product_next_eval: C::Scalar,
    permuted_input_eval: C::Scalar,
    permuted_input_inv_eval: C::Scalar,
    permuted_table_eval: C::Scalar,
}

pub(in crate::plonk) fn lookup_read_permuted_commitments<
    C: CurveAffine,
    E: EncodedChallenge<C>,
    T: TranscriptRead<C, E>,
>(
    transcript: &mut T,
) -> Result<PermutationCommitments<C>, Error> {
    let permuted_input_commitment = transcript.read_point()?;
    let permuted_table_commitment = transcript.read_point()?;

    Ok(PermutationCommitments {
        permuted_input_commitment,
        permuted_table_commitment,
    })
}

impl<C: CurveAffine> PermutationCommitments<C> {
    pub(in crate::plonk) fn read_product_commitment<
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>,
    >(
        self,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        let product_commitment = transcript.read_point()?;

        Ok(Committed {
            permuted: self,
            product_commitment,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(crate) fn evaluate<E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
        self,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let product_eval = transcript.read_scalar()?;
        let product_next_eval = transcript.read_scalar()?;
        let permuted_input_eval = transcript.read_scalar()?;
        let permuted_input_inv_eval = transcript.read_scalar()?;
        let permuted_table_eval = transcript.read_scalar()?;

        Ok(Evaluated {
            committed: self,
            product_eval,
            product_next_eval,
            permuted_input_eval,
            permuted_input_inv_eval,
            permuted_table_eval,
        })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn expressions<'a>(
        &'a self,
        l_0: C::Scalar,
        l_last: C::Scalar,
        l_blind: C::Scalar,
        argument: &'a Argument<C::Scalar>,
        theta: ChallengeTheta<C>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        advice_evals: &[C::Scalar],
        fixed_evals: &[C::Scalar],
        instance_evals: &[C::Scalar],
        challenges: &[C::Scalar],
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        let active_rows = C::Scalar::ONE - (l_last + l_blind);

        let product_expression = || {
            // z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            let left = self.product_next_eval
                * (self.permuted_input_eval + *beta)
                * (self.permuted_table_eval + *gamma);

            let compress_expressions = |expressions: &[ExpressionBack<C::Scalar>]| {
                expressions
                    .iter()
                    .map(|expression| {
                        expression.evaluate(
                            &|scalar| scalar,
                            &|var| match var {
                                VarBack::Challenge(challenge) => challenges[challenge.index],
                                VarBack::Query(QueryBack {
                                    index, column_type, ..
                                }) => match column_type {
                                    Any::Fixed => fixed_evals[index],
                                    Any::Advice => advice_evals[index],
                                    Any::Instance => instance_evals[index],
                                },
                            },
                            &|a| -a,
                            &|a, b| a + b,
                            &|a, b| a * b,
                        )
                    })
                    .fold(C::Scalar::ZERO, |acc, eval| acc * *theta + eval)
            };
            let right = self.product_eval
                * (compress_expressions(&argument.input_expressions) + *beta)
                * (compress_expressions(&argument.table_expressions) + *gamma);

            (left - right) * active_rows
        };

        std::iter::empty()
            .chain(
                // l_0(X) * (1 - z(X)) = 0
                Some(l_0 * (C::Scalar::ONE - self.product_eval)),
            )
            .chain(
                // l_last(X) * (z(X)^2 - z(X)) = 0
                Some(l_last * (self.product_eval.square() - self.product_eval)),
            )
            .chain(
                // (1 - (l_last(X) + l_blind(X))) * (
                //   z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
                //   - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
                // ) = 0
                Some(product_expression()),
            )
            .chain(Some(
                // l_0(X) * (a'(X) - s'(X)) = 0
                l_0 * (self.permuted_input_eval - self.permuted_table_eval),
            ))
            .chain(Some(
                // (1 - (l_last(X) + l_blind(X))) * (a′(X) − s′(X))⋅(a′(X) − a′(\omega^{-1} X)) = 0
                (self.permuted_input_eval - self.permuted_table_eval)
                    * (self.permuted_input_eval - self.permuted_input_inv_eval)
                    * active_rows,
            ))
    }

    pub(in crate::plonk) fn queries<'r, M: MSM<C> + 'r>(
        &'r self,
        vk: &'r VerifyingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'r, C, M>> + Clone {
        let x_inv = vk.domain.rotate_omega(*x, Rotation::prev());
        let x_next = vk.domain.rotate_omega(*x, Rotation::next());

        iter::empty()
            // Open lookup product commitment at x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                *x,
                self.product_eval,
            )))
            // Open lookup input commitments at x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.permuted.permuted_input_commitment,
                *x,
                self.permuted_input_eval,
            )))
            // Open lookup table commitments at x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.permuted.permuted_table_commitment,
                *x,
                self.permuted_table_eval,
            )))
            // Open lookup input commitments at \omega^{-1} x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.permuted.permuted_input_commitment,
                x_inv,
                self.permuted_input_inv_eval,
            )))
            // Open lookup product commitment at \omega x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                x_next,
                self.product_next_eval,
            )))
    }
}
