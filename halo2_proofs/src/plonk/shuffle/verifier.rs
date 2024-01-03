use std::iter;

use super::super::{circuit::Expression, ChallengeGamma, ChallengeTheta, ChallengeX};
use super::Argument;
use crate::{
    arithmetic::CurveAffine,
    plonk::{Error, VerifyingKey},
    poly::{commitment::MSM, Rotation, VerifierQuery},
    transcript::{EncodedChallenge, TranscriptRead},
};
use ff::Field;

pub struct Committed<C: CurveAffine> {
    product_commitment: C,
}

pub struct Evaluated<C: CurveAffine> {
    committed: Committed<C>,
    product_eval: C::Scalar,
    product_next_eval: C::Scalar,
}

impl<F: Field> Argument<F> {
    pub(in crate::plonk) fn read_product_commitment<
        C: CurveAffine<ScalarExt = F>,
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>,
    >(
        &self,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        let product_commitment = transcript.read_point()?;

        Ok(Committed { product_commitment })
    }
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
            let compress_expressions = |expressions: &[Expression<C::Scalar>]| {
                expressions
                    .iter()
                    .map(|expression| {
                        expression.evaluate(
                            &|scalar| scalar,
                            &|_| panic!("virtual selectors are removed during optimization"),
                            &|query| fixed_evals[query.index.unwrap()],
                            &|query| advice_evals[query.index.unwrap()],
                            &|query| instance_evals[query.index.unwrap()],
                            &|challenge| challenges[challenge.index()],
                            &|a| -a,
                            &|a, b| a + &b,
                            &|a, b| a * &b,
                            &|a, scalar| a * &scalar,
                        )
                    })
                    .fold(C::Scalar::ZERO, |acc, eval| acc * &*theta + &eval)
            };
            // z(\omega X) (s(X) + \gamma)
            let left = self.product_next_eval
                * &(compress_expressions(&argument.shuffle_expressions) + &*gamma);
            // z(X) (a(X) + \gamma)
            let right =
                self.product_eval * &(compress_expressions(&argument.input_expressions) + &*gamma);

            (left - &right) * &active_rows
        };

        std::iter::empty()
            .chain(
                // l_0(X) * (1 - z'(X)) = 0
                Some(l_0 * &(C::Scalar::ONE - &self.product_eval)),
            )
            .chain(
                // l_last(X) * (z(X)^2 - z(X)) = 0
                Some(l_last * &(self.product_eval.square() - &self.product_eval)),
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
