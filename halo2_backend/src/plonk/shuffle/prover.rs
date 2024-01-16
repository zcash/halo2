use super::super::{
    circuit::Expression, ChallengeGamma, ChallengeTheta, ChallengeX, Error, ProvingKey,
};
use super::Argument;
use crate::plonk::evaluation::evaluate;
use crate::{
    arithmetic::{eval_polynomial, parallelize, CurveAffine},
    poly::{
        commitment::{Blind, Params},
        Coeff, EvaluationDomain, LagrangeCoeff, Polynomial, ProverQuery, Rotation,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};
use ff::WithSmallOrderMulGroup;
use group::{ff::BatchInvert, Curve};
use rand_core::RngCore;
use std::{
    iter,
    ops::{Mul, MulAssign},
};

#[derive(Debug)]
struct Compressed<C: CurveAffine> {
    input_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    shuffle_expression: Polynomial<C::Scalar, LagrangeCoeff>,
}

#[derive(Debug)]
pub(in crate::plonk) struct Committed<C: CurveAffine> {
    pub(in crate::plonk) product_poly: Polynomial<C::Scalar, Coeff>,
    product_blind: Blind<C::Scalar>,
}

pub(in crate::plonk) struct Evaluated<C: CurveAffine> {
    constructed: Committed<C>,
}

impl<F: WithSmallOrderMulGroup<3>> Argument<F> {
    /// Given a Shuffle with input expressions [A_0, A_1, ..., A_{m-1}] and table expressions
    /// [S_0, S_1, ..., S_{m-1}], this method
    /// - constructs A_compressed = \theta^{m-1} A_0 + theta^{m-2} A_1 + ... + \theta A_{m-2} + A_{m-1}
    ///   and S_compressed = \theta^{m-1} S_0 + theta^{m-2} S_1 + ... + \theta S_{m-2} + S_{m-1},
    #[allow(clippy::too_many_arguments)]
    fn compress<'a, 'params: 'a, C, P: Params<'params, C>>(
        &self,
        pk: &ProvingKey<C>,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        theta: ChallengeTheta<C>,
        advice_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        instance_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        challenges: &'a [C::Scalar],
    ) -> Compressed<C>
    where
        C: CurveAffine<ScalarExt = F>,
        C::Curve: Mul<F, Output = C::Curve> + MulAssign<F>,
    {
        // Closure to get values of expressions and compress them
        let compress_expressions = |expressions: &[Expression<C::Scalar>]| {
            let compressed_expression = expressions
                .iter()
                .map(|expression| {
                    pk.vk.domain.lagrange_from_vec(evaluate(
                        expression,
                        params.n() as usize,
                        1,
                        fixed_values,
                        advice_values,
                        instance_values,
                        challenges,
                    ))
                })
                .fold(domain.empty_lagrange(), |acc, expression| {
                    acc * *theta + &expression
                });
            compressed_expression
        };

        // Get values of input expressions involved in the shuffle and compress them
        let input_expression = compress_expressions(&self.input_expressions);

        // Get values of table expressions involved in the shuffle and compress them
        let shuffle_expression = compress_expressions(&self.shuffle_expressions);

        Compressed {
            input_expression,
            shuffle_expression,
        }
    }

    /// Given a Shuffle with input expressions and table expressions this method
    /// constructs the grand product polynomial over the shuffle.
    /// The grand product polynomial is used to populate the Product<C> struct.
    /// The Product<C> struct is added to the Shuffle and finally returned by the method.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn commit_product<
        'a,
        'params: 'a,
        C,
        P: Params<'params, C>,
        E: EncodedChallenge<C>,
        R: RngCore,
        T: TranscriptWrite<C, E>,
    >(
        &self,
        pk: &ProvingKey<C>,
        params: &P,
        domain: &EvaluationDomain<C::Scalar>,
        theta: ChallengeTheta<C>,
        gamma: ChallengeGamma<C>,
        advice_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        instance_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        challenges: &'a [C::Scalar],
        mut rng: R,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error>
    where
        C: CurveAffine<ScalarExt = F>,
        C::Curve: Mul<F, Output = C::Curve> + MulAssign<F>,
    {
        let compressed = self.compress(
            pk,
            params,
            domain,
            theta,
            advice_values,
            fixed_values,
            instance_values,
            challenges,
        );

        let blinding_factors = pk.vk.cs.blinding_factors();

        let mut shuffle_product = vec![C::Scalar::ZERO; params.n() as usize];
        parallelize(&mut shuffle_product, |shuffle_product, start| {
            for (shuffle_product, shuffle_value) in shuffle_product
                .iter_mut()
                .zip(compressed.shuffle_expression[start..].iter())
            {
                *shuffle_product = *gamma + shuffle_value;
            }
        });

        shuffle_product.iter_mut().batch_invert();

        parallelize(&mut shuffle_product, |product, start| {
            for (i, product) in product.iter_mut().enumerate() {
                let i = i + start;
                *product *= &(*gamma + compressed.input_expression[i]);
            }
        });

        // Compute the evaluations of the shuffle product polynomial
        // over our domain, starting with z[0] = 1
        let z = iter::once(C::Scalar::ONE)
            .chain(shuffle_product)
            .scan(C::Scalar::ONE, |state, cur| {
                *state *= &cur;
                Some(*state)
            })
            // Take all rows including the "last" row which should
            // be a boolean (and ideally 1, else soundness is broken)
            .take(params.n() as usize - blinding_factors)
            // Chain random blinding factors.
            .chain((0..blinding_factors).map(|_| C::Scalar::random(&mut rng)))
            .collect::<Vec<_>>();
        assert_eq!(z.len(), params.n() as usize);
        let z = pk.vk.domain.lagrange_from_vec(z);

        #[cfg(feature = "sanity-checks")]
        {
            // While in Lagrange basis, check that product is correctly constructed
            let u = (params.n() as usize) - (blinding_factors + 1);
            assert_eq!(z[0], C::Scalar::ONE);
            for i in 0..u {
                let mut left = z[i + 1];
                let input_value = &compressed.input_expression[i];
                let shuffle_value = &compressed.shuffle_expression[i];
                left *= &(*gamma + shuffle_value);
                let mut right = z[i];
                right *= &(*gamma + input_value);
                assert_eq!(left, right);
            }
            assert_eq!(z[u], C::Scalar::ONE);
        }

        let product_blind = Blind(C::Scalar::random(rng));
        let product_commitment = params.commit_lagrange(&z, product_blind).to_affine();
        let z = pk.vk.domain.lagrange_to_coeff(z);

        // Hash product commitment
        transcript.write_point(product_commitment)?;

        Ok(Committed::<C> {
            product_poly: z,
            product_blind,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        self,
        pk: &ProvingKey<C>,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let domain = &pk.vk.domain;
        let x_next = domain.rotate_omega(*x, Rotation::next());

        let product_eval = eval_polynomial(&self.product_poly, *x);
        let product_next_eval = eval_polynomial(&self.product_poly, x_next);

        // Hash each advice evaluation
        for eval in iter::empty()
            .chain(Some(product_eval))
            .chain(Some(product_next_eval))
        {
            transcript.write_scalar(eval)?;
        }

        Ok(Evaluated { constructed: self })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        pk: &'a ProvingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let x_next = pk.vk.domain.rotate_omega(*x, Rotation::next());

        iter::empty()
            // Open shuffle product commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
            }))
            // Open shuffle product commitments at x_next
            .chain(Some(ProverQuery {
                point: x_next,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
            }))
    }
}
