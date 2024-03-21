use std::iter;

use super::Argument;
use crate::{
    arithmetic::CurveAffine,
    plonk::circuit::{ExpressionBack, QueryBack, VarBack},
    plonk::{ChallengeGamma, ChallengeTheta, ChallengeX, Error, VerifyingKey},
    poly::{commitment::MSM, VerifierQuery},
    transcript::{EncodedChallenge, TranscriptRead},
};
use halo2_middleware::circuit::Any;
use halo2_middleware::ff::Field;
use halo2_middleware::poly::Rotation;

pub(crate) struct Committed<C: CurveAffine> {
    product_commitment: C,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    committed: Committed<C>,
    product_eval: C::Scalar,
    product_next_eval: C::Scalar,
}

pub(in crate::plonk) fn shuffle_read_product_commitment<
    F: Field,
    C: CurveAffine<ScalarExt = F>,
    E: EncodedChallenge<C>,
    T: TranscriptRead<C, E>,
>(
    transcript: &mut T,
) -> Result<Committed<C>, Error> {
    let product_commitment = transcript.read_point()?;

    Ok(Committed { product_commitment })
}

impl<C: CurveAffine> Committed<C> {
    pub(crate) fn evaluate<E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
        self,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let product_eval = transcript.read_scalar()?;
        let product_next_eval = transcript.read_scalar()?;

        Ok(Evaluated {
            committed: self,
            product_eval,
            product_next_eval,
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
        gamma: ChallengeGamma<C>,
        advice_evals: &[C::Scalar],
        fixed_evals: &[C::Scalar],
        instance_evals: &[C::Scalar],
        challenges: &[C::Scalar],
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        let active_rows = C::Scalar::ONE - (l_last + l_blind);

        let product_expression = || {
            // z(\omega X) (s(X) + \gamma) - z(X) (a(X) + \gamma)
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
                            &|a, scalar| a * scalar,
                        )
                    })
                    .fold(C::Scalar::ZERO, |acc, eval| acc * *theta + eval)
            };
            // z(\omega X) (s(X) + \gamma)
            let left = self.product_next_eval
                * (compress_expressions(&argument.shuffle_expressions) + *gamma);
            // z(X) (a(X) + \gamma)
            let right =
                self.product_eval * (compress_expressions(&argument.input_expressions) + *gamma);

            (left - right) * active_rows
        };

        std::iter::empty()
            .chain(
                // l_0(X) * (1 - z'(X)) = 0
                Some(l_0 * (C::Scalar::ONE - self.product_eval)),
            )
            .chain(
                // l_last(X) * (z(X)^2 - z(X)) = 0
                Some(l_last * (self.product_eval.square() - self.product_eval)),
            )
            .chain(
                // (1 - (l_last(X) + l_blind(X))) * ( z(\omega X) (s(X) + \gamma) - z(X) (a(X) + \gamma))
                Some(product_expression()),
            )
    }

    pub(in crate::plonk) fn queries<'r, M: MSM<C> + 'r>(
        &'r self,
        vk: &'r VerifyingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'r, C, M>> + Clone {
        let x_next = vk.domain.rotate_omega(*x, Rotation::next());

        iter::empty()
            // Open shuffle product commitment at x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                *x,
                self.product_eval,
            )))
            // Open shuffle product commitment at \omega x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                x_next,
                self.product_next_eval,
            )))
    }
}
