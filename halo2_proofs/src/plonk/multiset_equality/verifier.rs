use std::iter;

use super::super::{circuit::Expression, ChallengeBeta, ChallengeTheta, ChallengeX};
use super::Argument;
use crate::{
    arithmetic::{CurveAffine, FieldExt},
    plonk::{Error, VerifyingKey},
    poly::{multiopen::VerifierQuery, Rotation},
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

impl<F: FieldExt> Argument<F> {
    pub(in crate::plonk) fn read_product_commitment<
        C: CurveAffine,
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>,
    >(
        self,
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
        beta: ChallengeBeta<C>,
        advice_evals: &[C::Scalar],
        fixed_evals: &[C::Scalar],
        instance_evals: &[C::Scalar],
    ) -> impl Iterator<Item = C::Scalar> + 'a {
        let active_rows = C::Scalar::one() - (l_last + l_blind);

        let product_expression = || {
            let compress_expressions = |expressions: &[Expression<C::Scalar>]| {
                expressions
                    .iter()
                    .map(|expression| {
                        expression.evaluate(
                            &|scalar| scalar,
                            &|_| panic!("virtual selectors are removed during optimization"),
                            &|query| fixed_evals[query.index],
                            &|query| advice_evals[query.index],
                            &|query| instance_evals[query.index],
                            &|a| -a,
                            &|a, b| a + &b,
                            &|a, b| a * &b,
                            &|a, scalar| a * &scalar,
                        )
                    })
                    .fold(C::Scalar::zero(), |acc, eval| acc * &*theta + &eval)
            };
            // z(\omega X) (\theta^{m-1} a'_0(X) + ... + a'_{m-1}(X) + \beta)
            // - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta)
            let left = self.product_next_eval
                * &(compress_expressions(&argument.permuted_expressions) + &*beta);

            let right = self.product_eval
                * &(compress_expressions(&argument.original_expressions) + &*beta);

            (left - &right) * &active_rows
        };

        std::iter::empty()
            .chain(
                // l_0(X) * (1 - z(X)) = 0
                Some(l_0 * &(C::Scalar::one() - &self.product_eval)),
            )
            .chain(
                // l_last(X) * (z(X)^2 - z(X)) = 0
                Some(l_last * &(self.product_eval.square() - &self.product_eval)),
            )
            .chain(
                // (1 - (l_last(X) + l_blind(X))) * (
                //   z(\omega X) (\theta^{m-1} a'_0(X) + ... + a'_{m-1}(X) + \beta)
                //   - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta)
                // ) = 0
                Some(product_expression()),
            )
    }

    pub(in crate::plonk) fn queries<'r, 'params: 'r>(
        &'r self,
        vk: &'r VerifyingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = VerifierQuery<'r, 'params, C>> + Clone {
        let x_next = vk.domain.rotate_omega(*x, Rotation::next());

        iter::empty()
            // Open multiset product commitment at x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                *x,
                self.product_eval,
            )))
            // Open multiset product commitment at \omega x
            .chain(Some(VerifierQuery::new_commitment(
                &self.committed.product_commitment,
                x_next,
                self.product_next_eval,
            )))
    }
}
