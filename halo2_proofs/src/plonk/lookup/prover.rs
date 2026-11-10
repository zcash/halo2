use super::super::{
    circuit::Expression, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error,
    ProvingKey,
};
use super::Argument;
use crate::{
    arithmetic::{evaluate_batch, grand_product, parallel_batch_invert, parallelize, CurveAffine},
    poly::{
        self,
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};
use ff::WithSmallOrderMulGroup;
use group::ff::{Field, PrimeField};
use maybe_rayon::prelude::*;
use pasta_curves::arithmetic::CurveExt;
use rand_core::Rng;
use std::{
    cmp::Ordering,
    iter,
    ops::{Mul, MulAssign},
};

#[derive(Debug)]
pub(in crate::plonk) struct Permuted<C: CurveAffine, Ev> {
    compressed_input_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_input_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    compressed_input_coset: poly::Ast<Ev, C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_coset: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
    permuted_input_blind: Blind<C::Scalar>,
    compressed_table_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    compressed_table_coset: poly::Ast<Ev, C::Scalar, ExtendedLagrangeCoeff>,
    permuted_table_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_coset: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
    permuted_table_blind: Blind<C::Scalar>,
}

#[derive(Debug)]
pub(in crate::plonk) struct Committed<C: CurveAffine, Ev> {
    permuted: Permuted<C, Ev>,
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_coset: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
    product_blind: Blind<C::Scalar>,
}

pub(in crate::plonk) struct Constructed<C: CurveAffine> {
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_blind: Blind<C::Scalar>,
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_blind: Blind<C::Scalar>,
}

pub(in crate::plonk) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
}

impl<F: WithSmallOrderMulGroup<3>> Argument<F> {
    /// Given a Lookup with input expressions [A_0, A_1, ..., A_{m-1}] and table expressions
    /// [S_0, S_1, ..., S_{m-1}], this method
    /// - constructs A_compressed = \theta^{m-1} A_0 + theta^{m-2} A_1 + ... + \theta A_{m-2} + A_{m-1}
    ///   and S_compressed = \theta^{m-1} S_0 + theta^{m-2} S_1 + ... + \theta S_{m-2} + S_{m-1},
    /// - permutes A_compressed and S_compressed using permute_expression_pair() helper,
    ///   obtaining A' and S', and
    /// - constructs Permuted<C> struct using permuted_input_value = A', and
    ///   permuted_table_expression = S'.
    ///
    /// The Permuted<C> struct is used to update the Lookup, and is then returned.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn commit_permuted<
        'a,
        C,
        E: EncodedChallenge<C>,
        Ev: Copy + Send + Sync,
        Ec: Copy + Send + Sync,
        R: Rng,
        T: TranscriptWrite<C, E>,
    >(
        &self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        value_evaluator: &poly::Evaluator<Ev, C::Scalar, LagrangeCoeff>,
        coset_evaluator: &mut poly::Evaluator<Ec, C::Scalar, ExtendedLagrangeCoeff>,
        theta: ChallengeTheta<C>,
        advice_values: &'a [poly::AstLeaf<Ev, LagrangeCoeff>],
        fixed_values: &'a [poly::AstLeaf<Ev, LagrangeCoeff>],
        instance_values: &'a [poly::AstLeaf<Ev, LagrangeCoeff>],
        advice_cosets: &'a [poly::AstLeaf<Ec, ExtendedLagrangeCoeff>],
        fixed_cosets: &'a [poly::AstLeaf<Ec, ExtendedLagrangeCoeff>],
        instance_cosets: &'a [poly::AstLeaf<Ec, ExtendedLagrangeCoeff>],
        mut rng: R,
        transcript: &mut T,
    ) -> Result<Permuted<C, Ec>, Error>
    where
        C: CurveAffine<ScalarExt = F>,
        C::Curve: Mul<F, Output = C::Curve> + MulAssign<F>,
    {
        // Closure to get values of expressions and compress them
        let compress_expressions = |expressions: &[Expression<C::Scalar>]| {
            // Values of input expressions involved in the lookup
            let unpermuted_expressions: Vec<_> = expressions
                .iter()
                .map(|expression| {
                    expression.evaluate(
                        &|scalar| poly::Ast::ConstantTerm(scalar),
                        &|_| panic!("virtual selectors are removed during optimization"),
                        &|query| {
                            fixed_values[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|query| {
                            advice_values[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|query| {
                            instance_values[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|a| -a,
                        &|a, b| a + b,
                        &|a, b| a * b,
                        &|a, scalar| a * scalar,
                    )
                })
                .collect();

            let unpermuted_cosets: Vec<_> = expressions
                .iter()
                .map(|expression| {
                    expression.evaluate(
                        &|scalar| poly::Ast::ConstantTerm(scalar),
                        &|_| panic!("virtual selectors are removed during optimization"),
                        &|query| {
                            fixed_cosets[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|query| {
                            advice_cosets[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|query| {
                            instance_cosets[query.column_index]
                                .with_rotation(query.rotation)
                                .into()
                        },
                        &|a| -a,
                        &|a, b| a + b,
                        &|a, b| a * b,
                        &|a, scalar| a * scalar,
                    )
                })
                .collect();

            // Compressed version of expressions
            let compressed_expression = unpermuted_expressions.iter().fold(
                poly::Ast::ConstantTerm(C::Scalar::ZERO),
                |acc, expression| &(acc * *theta) + expression,
            );

            // Compressed version of cosets
            let compressed_coset = unpermuted_cosets.iter().fold(
                poly::Ast::<_, _, ExtendedLagrangeCoeff>::ConstantTerm(C::Scalar::ZERO),
                |acc, eval| acc * poly::Ast::ConstantTerm(*theta) + eval.clone(),
            );

            (
                compressed_coset,
                value_evaluator.evaluate(&compressed_expression, domain),
            )
        };

        // Get values of input expressions involved in the lookup and compress them
        let (compressed_input_coset, compressed_input_expression) =
            compress_expressions(&self.input_expressions);

        // Get values of table expressions involved in the lookup and compress them
        let (compressed_table_coset, compressed_table_expression) =
            compress_expressions(&self.table_expressions);

        // Permute compressed (InputExpression, TableExpression) pair
        let (permuted_input_expression, permuted_table_expression) = permute_expression_pair::<C, _>(
            pk,
            params,
            domain,
            &mut rng,
            &compressed_input_expression,
            &compressed_table_expression,
        )?;

        // Closure to construct commitment to vector of values
        let mut commit_values = |values: &Polynomial<C::Scalar, LagrangeCoeff>| {
            let poly = pk.vk.domain.lagrange_to_coeff(values.clone());
            let blind = Blind(C::Scalar::random(&mut rng));
            let commitment = params.commit_lagrange(values, blind).to_affine_vartime();
            (poly, blind, commitment)
        };

        // Commit to permuted input expression
        let (permuted_input_poly, permuted_input_blind, permuted_input_commitment) =
            commit_values(&permuted_input_expression);

        // Commit to permuted table expression
        let (permuted_table_poly, permuted_table_blind, permuted_table_commitment) =
            commit_values(&permuted_table_expression);

        // Hash permuted input commitment
        transcript.write_point(permuted_input_commitment)?;

        // Hash permuted table commitment
        transcript.write_point(permuted_table_commitment)?;

        let permuted_input_coset = coset_evaluator
            .register_poly(pk.vk.domain.coeff_to_extended(permuted_input_poly.clone()));
        let permuted_table_coset = coset_evaluator
            .register_poly(pk.vk.domain.coeff_to_extended(permuted_table_poly.clone()));

        Ok(Permuted {
            compressed_input_expression,
            compressed_input_coset,
            permuted_input_expression,
            permuted_input_poly,
            permuted_input_coset,
            permuted_input_blind,
            compressed_table_expression,
            compressed_table_coset,
            permuted_table_expression,
            permuted_table_poly,
            permuted_table_coset,
            permuted_table_blind,
        })
    }
}

impl<C: CurveAffine, Ev: Copy + Send + Sync> Permuted<C, Ev> {
    /// Given a Lookup with input expressions, table expressions, and the permuted
    /// input expression and permuted table expression, this method constructs the
    /// grand product polynomial over the lookup. The grand product polynomial
    /// is used to populate the Product<C> struct. The Product<C> struct is
    /// added to the Lookup and finally returned by the method.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn commit_product<
        E: EncodedChallenge<C>,
        R: Rng,
        T: TranscriptWrite<C, E>,
    >(
        self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        evaluator: &mut poly::Evaluator<Ev, C::Scalar, ExtendedLagrangeCoeff>,
        mut rng: R,
        transcript: &mut T,
    ) -> Result<Committed<C, Ev>, Error> {
        let blinding_factors = pk.vk.cs.blinding_factors();
        // Goal is to compute the products of fractions
        //
        // Numerator: (\theta^{m-1} a_0(\omega^i) + \theta^{m-2} a_1(\omega^i) + ... + \theta a_{m-2}(\omega^i) + a_{m-1}(\omega^i) + \beta)
        //            * (\theta^{m-1} s_0(\omega^i) + \theta^{m-2} s_1(\omega^i) + ... + \theta s_{m-2}(\omega^i) + s_{m-1}(\omega^i) + \gamma)
        // Denominator: (a'(\omega^i) + \beta) (s'(\omega^i) + \gamma)
        //
        // where a_j(X) is the jth input expression in this lookup,
        // where a'(X) is the compression of the permuted input expressions,
        // s_j(X) is the jth table expression in this lookup,
        // s'(X) is the compression of the permuted table expressions,
        // and i is the ith row of the expression.
        let mut lookup_product = vec![C::Scalar::ZERO; params.n as usize];
        // Denominator uses the permuted input expression and permuted table expression
        parallelize(&mut lookup_product, |lookup_product, start| {
            for ((lookup_product, permuted_input_value), permuted_table_value) in lookup_product
                .iter_mut()
                .zip(self.permuted_input_expression[start..].iter())
                .zip(self.permuted_table_expression[start..].iter())
            {
                *lookup_product = (*beta + permuted_input_value) * &(*gamma + permuted_table_value);
            }
        });

        // Batch invert to obtain the denominators for the lookup product
        // polynomials
        parallel_batch_invert(&mut lookup_product);

        // Finish the computation of the entire fraction by computing the numerators
        // (\theta^{m-1} a_0(\omega^i) + \theta^{m-2} a_1(\omega^i) + ... + \theta a_{m-2}(\omega^i) + a_{m-1}(\omega^i) + \beta)
        // * (\theta^{m-1} s_0(\omega^i) + \theta^{m-2} s_1(\omega^i) + ... + \theta s_{m-2}(\omega^i) + s_{m-1}(\omega^i) + \gamma)
        parallelize(&mut lookup_product, |product, start| {
            for ((product, &input_term), &table_term) in product
                .iter_mut()
                .zip(self.compressed_input_expression[start..].iter())
                .zip(self.compressed_table_expression[start..].iter())
            {
                *product *= &(input_term + &*beta);
                *product *= &(table_term + &*gamma);
            }
        });

        // The product vector is a vector of products of fractions of the form
        //
        // Numerator: (\theta^{m-1} a_0(\omega^i) + \theta^{m-2} a_1(\omega^i) + ... + \theta a_{m-2}(\omega^i) + a_{m-1}(\omega^i) + \beta)
        //            * (\theta^{m-1} s_0(\omega^i) + \theta^{m-2} s_1(\omega^i) + ... + \theta s_{m-2}(\omega^i) + s_{m-1}(\omega^i) + \gamma)
        // Denominator: (a'(\omega^i) + \beta) (s'(\omega^i) + \gamma)
        //
        // where there are m input expressions and m table expressions,
        // a_j(\omega^i) is the jth input expression in this lookup,
        // a'j(\omega^i) is the permuted input expression,
        // s_j(\omega^i) is the jth table expression in this lookup,
        // s'(\omega^i) is the permuted table expression,
        // and i is the ith row of the expression.

        // Compute the evaluations of the lookup product polynomial
        // over our domain, starting with z[0] = 1
        // Take all rows including the "last" row which should be a boolean
        // (and ideally 1, else soundness is broken), then the random
        // blinding factors.
        let mut z = grand_product(&lookup_product, params.n as usize - blinding_factors);
        z.extend((0..blinding_factors).map(|_| C::Scalar::random(&mut rng)));
        assert_eq!(z.len(), params.n as usize);
        let z = pk.vk.domain.lagrange_from_vec(z);

        #[cfg(feature = "sanity-checks")]
        // This test works only with intermediate representations in this method.
        // It can be used for debugging purposes.
        {
            // While in Lagrange basis, check that product is correctly constructed
            let u = (params.n as usize) - (blinding_factors + 1);

            // l_0(X) * (1 - z(X)) = 0
            assert_eq!(z[0], C::Scalar::ONE);

            // z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            for i in 0..u {
                let mut left = z[i + 1];
                let permuted_input_value = &self.permuted_input_expression[i];

                let permuted_table_value = &self.permuted_table_expression[i];

                left *= &(*beta + permuted_input_value);
                left *= &(*gamma + permuted_table_value);

                let mut right = z[i];
                let mut input_term = self.compressed_input_expression[i];

                let mut table_term = self.compressed_table_expression[i];

                input_term += &(*beta);
                table_term += &(*gamma);
                right *= &(input_term * &table_term);

                assert_eq!(left, right);
            }

            // l_last(X) * (z(X)^2 - z(X)) = 0
            // Assertion will fail only when soundness is broken, in which
            // case this z[u] value will be zero. (bad!)
            assert_eq!(z[u], C::Scalar::ONE);
        }

        let product_blind = Blind(C::Scalar::random(&mut rng));
        let product_commitment = params
            .commit_lagrange(&z, product_blind)
            .to_affine_vartime();
        let z = pk.vk.domain.lagrange_to_coeff(z);
        let product_coset = evaluator.register_poly(pk.vk.domain.coeff_to_extended(z.clone()));

        // Hash product commitment
        transcript.write_point(product_commitment)?;

        Ok(Committed::<C, _> {
            permuted: self,
            product_poly: z,
            product_coset,
            product_blind,
        })
    }
}

impl<'a, C: CurveAffine, Ev: Copy + Send + Sync + 'a> Committed<C, Ev> {
    /// Given a Lookup with input expressions, table expressions, permuted input
    /// expression, permuted table expression, and grand product polynomial, this
    /// method constructs constraints that must hold between these values.
    /// This method returns the constraints as a vector of ASTs for polynomials in
    /// the extended evaluation domain.
    pub(in crate::plonk) fn construct(
        self,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        l0: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
        l_blind: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
        l_last: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
    ) -> (
        Constructed<C>,
        impl Iterator<Item = poly::Ast<Ev, C::Scalar, ExtendedLagrangeCoeff>> + 'a,
    ) {
        let permuted = self.permuted;

        let active_rows = poly::Ast::one() - (poly::Ast::from(l_last) + l_blind);
        let beta = poly::Ast::ConstantTerm(*beta);
        let gamma = poly::Ast::ConstantTerm(*gamma);

        let expressions = iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(Some((poly::Ast::one() - self.product_coset) * l0))
            // l_last(X) * (z(X)^2 - z(X)) = 0
            .chain(Some(
                (poly::Ast::from(self.product_coset) * self.product_coset - self.product_coset)
                    * l_last,
            ))
            // (1 - (l_last(X) + l_blind(X))) * (
            //   z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
            //   - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            // ) = 0
            .chain({
                // z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
                let left: poly::Ast<_, _, _> = poly::Ast::<_, C::Scalar, _>::from(
                    self.product_coset.with_rotation(Rotation::next()),
                ) * (poly::Ast::from(permuted.permuted_input_coset)
                    + beta.clone())
                    * (poly::Ast::from(permuted.permuted_table_coset) + gamma.clone());

                //  z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
                let right: poly::Ast<_, _, _> = poly::Ast::from(self.product_coset)
                    * (permuted.compressed_input_coset + beta)
                    * (permuted.compressed_table_coset + gamma);

                Some((left - right) * active_rows.clone())
            })
            // Check that the first values in the permuted input expression and permuted
            // fixed expression are the same.
            // l_0(X) * (a'(X) - s'(X)) = 0
            .chain(Some(
                (poly::Ast::from(permuted.permuted_input_coset) - permuted.permuted_table_coset)
                    * l0,
            ))
            // Check that each value in the permuted lookup input expression is either
            // equal to the value above it, or the value at the same index in the
            // permuted table expression.
            // (1 - (l_last + l_blind)) * (a′(X) − s′(X))⋅(a′(X) − a′(\omega^{-1} X)) = 0
            .chain(Some(
                (poly::Ast::<_, C::Scalar, _>::from(permuted.permuted_input_coset)
                    - permuted.permuted_table_coset)
                    * (poly::Ast::from(permuted.permuted_input_coset)
                        - permuted
                            .permuted_input_coset
                            .with_rotation(Rotation::prev()))
                    * active_rows,
            ));

        (
            Constructed {
                permuted_input_poly: permuted.permuted_input_poly,
                permuted_input_blind: permuted.permuted_input_blind,
                permuted_table_poly: permuted.permuted_table_poly,
                permuted_table_blind: permuted.permuted_table_blind,
                product_poly: self.product_poly,
                product_blind: self.product_blind,
            },
            expressions,
        )
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        self,
        pk: &ProvingKey<C>,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let domain = &pk.vk.domain;
        let x_inv = domain.rotate_omega(*x, Rotation::prev());
        let x_next = domain.rotate_omega(*x, Rotation::next());

        let queries: [(&[C::Scalar], C::Scalar); 5] = [
            (&self.product_poly, *x),
            (&self.product_poly, x_next),
            (&self.permuted_input_poly, *x),
            (&self.permuted_input_poly, x_inv),
            (&self.permuted_table_poly, *x),
        ];

        // Hash each lookup evaluation
        for eval in evaluate_batch(&queries) {
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
        let x_inv = pk.vk.domain.rotate_omega(*x, Rotation::prev());
        let x_next = pk.vk.domain.rotate_omega(*x, Rotation::next());

        iter::empty()
            // Open lookup product commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
            }))
            // Open lookup input commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permuted_input_poly,
                blind: self.constructed.permuted_input_blind,
            }))
            // Open lookup table commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permuted_table_poly,
                blind: self.constructed.permuted_table_blind,
            }))
            // Open lookup input commitments at x_inv
            .chain(Some(ProverQuery {
                point: x_inv,
                poly: &self.constructed.permuted_input_poly,
                blind: self.constructed.permuted_input_blind,
            }))
            // Open lookup product commitments at x_next
            .chain(Some(ProverQuery {
                point: x_next,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
            }))
    }
}

type ExpressionPair<F> = (Polynomial<F, LagrangeCoeff>, Polynomial<F, LagrangeCoeff>);

/// Given a vector of input values A and a vector of table values S,
/// this method permutes A and S to produce A' and S', such that:
/// - like values in A' are vertically adjacent to each other; and
/// - the first row in a sequence of like values in A' is the row
///   that has the corresponding value in S'.
///
/// This method returns (A', S') if no errors are encountered.
fn permute_expression_pair<C: CurveAffine, R: Rng>(
    pk: &ProvingKey<C>,
    params: &Params<C>,
    domain: &EvaluationDomain<C::Scalar>,
    mut rng: R,
    input_expression: &Polynomial<C::Scalar, LagrangeCoeff>,
    table_expression: &Polynomial<C::Scalar, LagrangeCoeff>,
) -> Result<ExpressionPair<C::Scalar>, Error> {
    let blinding_factors = pk.vk.cs.blinding_factors();
    let usable_rows = params.n as usize - (blinding_factors + 1);

    let (mut permuted_input_expression, mut permuted_table_coeffs) = permute_lookup_values(
        &input_expression[..][..usable_rows],
        &table_expression[..][..usable_rows],
    )?;

    permuted_input_expression
        .extend((0..(blinding_factors + 1)).map(|_| C::Scalar::random(&mut rng)));
    permuted_table_coeffs.extend((0..(blinding_factors + 1)).map(|_| C::Scalar::random(&mut rng)));
    assert_eq!(permuted_input_expression.len(), params.n as usize);
    assert_eq!(permuted_table_coeffs.len(), params.n as usize);

    #[cfg(feature = "sanity-checks")]
    {
        let mut last = None;
        for (a, b) in permuted_input_expression
            .iter()
            .zip(permuted_table_coeffs.iter())
            .take(usable_rows)
        {
            if *a != *b {
                assert_eq!(*a, last.unwrap());
            }
            last = Some(*a);
        }
    }

    Ok((
        domain.lagrange_from_vec(permuted_input_expression),
        domain.lagrange_from_vec(permuted_table_coeffs),
    ))
}

/// Below this many rows, the lookup values are sorted on one thread.
#[cfg_attr(not(feature = "multicore"), allow(dead_code))]
const LOOKUP_SORT_PARALLEL_THRESHOLD: usize = 1 << 14;

/// Orders field elements by their canonical byte representation read as a
/// little-endian integer, which is the order of the integers in `[0, p)` and
/// therefore the order pasta's `Ord` implementation defines.
///
/// Any total order in which equal keys mean equal values would do for the
/// lookup argument; this one keeps proofs identical to those of earlier
/// versions, which sorted with `Ord` directly.
fn cmp_repr<F: PrimeField>(a: &F::Repr, b: &F::Repr) -> Ordering {
    a.as_ref().iter().rev().cmp(b.as_ref().iter().rev())
}

/// Returns `values` sorted, together with their sort keys.
///
/// Comparing field elements with `Ord` converts both operands out of
/// Montgomery form on every comparison, so a sort performs `2 n log n`
/// conversions. Computing each key once costs `n`.
fn sort_by_repr<F: PrimeField>(values: &[F]) -> Vec<(F::Repr, F)> {
    let mut keyed: Vec<(F::Repr, F)> = values
        .into_par_iter()
        .map(|value| (value.to_repr(), *value))
        .collect();
    let by_key = |a: &(F::Repr, F), b: &(F::Repr, F)| cmp_repr::<F>(&a.0, &b.0);
    // Elements with equal keys are equal, so an unstable sort is
    // indistinguishable from a stable one.
    #[cfg(feature = "multicore")]
    if keyed.len() >= LOOKUP_SORT_PARALLEL_THRESHOLD {
        keyed.par_sort_unstable_by(by_key);
        return keyed;
    }
    keyed.sort_unstable_by(by_key);
    keyed
}

/// Given the usable rows of a compressed input expression A and table
/// expression S, returns (A', S') such that:
/// - A' is A sorted, so like values in A' are vertically adjacent; and
/// - S' is a permutation of S in which the first row of each run of like
///   values in A' holds that value.
///
/// Both columns are sorted by key and then merged, so every input value is
/// matched against the table in a single linear pass. Returns
/// `Error::ConstraintSystemFailure` if an input value is not in the table.
pub(crate) fn permute_lookup_values<F: PrimeField>(
    input: &[F],
    table: &[F],
) -> Result<(Vec<F>, Vec<F>), Error> {
    assert_eq!(input.len(), table.len());
    let input = sort_by_repr(input);
    let table = sort_by_repr(table);

    // The table as runs of equal values, each with how many of its copies
    // are not yet placed in S'.
    let mut leftover: Vec<(F::Repr, F, u32)> = Vec::new();
    for (key, value) in table {
        match leftover.last_mut() {
            Some((last, _, count)) if cmp_repr::<F>(last, &key) == Ordering::Equal => *count += 1,
            _ => leftover.push((key, value, 1)),
        }
    }

    let mut permuted_table = vec![F::ZERO; input.len()];
    let mut repeated_input_rows = Vec::new();
    let mut run = 0;
    for (row, (key, value)) in input.iter().enumerate() {
        // If this is the first occurrence of `value` in the input expression
        if row == 0 || cmp_repr::<F>(key, &input[row - 1].0) != Ordering::Equal {
            permuted_table[row] = *value;
            // Remove one copy of `value` from the leftover table runs. Both
            // lists are sorted, so the matching run is at or after `run`.
            while run < leftover.len() && cmp_repr::<F>(&leftover[run].0, key) == Ordering::Less {
                run += 1;
            }
            match leftover.get_mut(run) {
                Some((table_key, _, count)) if cmp_repr::<F>(table_key, key) == Ordering::Equal => {
                    assert!(*count > 0);
                    *count -= 1;
                }
                // Return error if `value` is not in the table
                _ => return Err(Error::ConstraintSystemFailure),
            }
        } else {
            repeated_input_rows.push(row);
        }
    }

    // Populate permuted table at unfilled rows with leftover table elements
    for (_, value, count) in leftover {
        for _ in 0..count {
            permuted_table[repeated_input_rows.pop().unwrap()] = value;
        }
    }
    assert!(repeated_input_rows.is_empty());

    Ok((
        input.into_iter().map(|(_, value)| value).collect(),
        permuted_table,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use group::ff::Field;
    use proptest::prelude::*;

    use super::{permute_lookup_values, LOOKUP_SORT_PARALLEL_THRESHOLD};
    use crate::{pasta::Fp, plonk::Error};

    /// The algorithm `permute_lookup_values` replaced: a stable sort with
    /// `Ord` and a `BTreeMap` of leftover table values.
    fn reference(input: &[Fp], table: &[Fp]) -> Result<(Vec<Fp>, Vec<Fp>), Error> {
        let mut permuted_input = input.to_vec();
        permuted_input.sort();
        let mut leftover = BTreeMap::<Fp, u32>::new();
        for value in table {
            *leftover.entry(*value).or_insert(0) += 1;
        }
        let mut permuted_table = vec![Fp::ZERO; input.len()];
        let mut repeated = vec![];
        for row in 0..permuted_input.len() {
            let value = permuted_input[row];
            if row == 0 || value != permuted_input[row - 1] {
                permuted_table[row] = value;
                match leftover.get_mut(&value) {
                    Some(count) => *count -= 1,
                    None => return Err(Error::ConstraintSystemFailure),
                }
            } else {
                repeated.push(row);
            }
        }
        for (value, count) in leftover {
            for _ in 0..count {
                permuted_table[repeated.pop().unwrap()] = value;
            }
        }
        Ok((permuted_input, permuted_table))
    }

    /// An input column drawn from a small range, so values repeat, and a
    /// table column of the same length that contains every input value.
    fn arb_lookup(max_len: usize) -> impl Strategy<Value = (Vec<Fp>, Vec<Fp>)> {
        (1..max_len).prop_flat_map(|len| {
            (
                prop::collection::vec(0u64..8, len),
                prop::collection::vec(0u64..16, len),
            )
                .prop_map(|(input, mut extra)| {
                    // Place each distinct input value in the table once, then
                    // pad with arbitrary values, some outside the input range.
                    let mut distinct = input.clone();
                    distinct.sort_unstable();
                    distinct.dedup();
                    extra[..distinct.len()].copy_from_slice(&distinct);
                    // Large values exercise the most significant bytes.
                    let field = |v: &u64| -Fp::from(*v);
                    (
                        input.iter().map(field).collect(),
                        extra.iter().map(field).collect(),
                    )
                })
        })
    }

    fn is_err<T>(r: &Result<T, Error>) -> bool {
        matches!(r, Err(Error::ConstraintSystemFailure))
    }

    proptest! {
        #[test]
        fn matches_the_btreemap_algorithm((input, table) in arb_lookup(64)) {
            prop_assert_eq!(
                permute_lookup_values(&input, &table).unwrap(),
                reference(&input, &table).unwrap()
            );
        }

        #[test]
        fn rejects_an_input_missing_from_the_table(
            (input, mut table) in arb_lookup(64),
        ) {
            // Remove every copy of one input value from the table.
            let missing = input[0];
            for value in table.iter_mut().filter(|v| **v == missing) {
                *value = Fp::from(1000);
            }
            prop_assert!(is_err(&permute_lookup_values(&input, &table)));
            prop_assert!(is_err(&reference(&input, &table)));
        }
    }

    #[test]
    fn matches_the_btreemap_algorithm_above_the_parallel_threshold() {
        let len = LOOKUP_SORT_PARALLEL_THRESHOLD + 7;
        let input: Vec<Fp> = (0..len as u64)
            .map(|i| -Fp::from((i * 7919) % 1000))
            .collect();
        let table: Vec<Fp> = (0..len as u64).map(|i| -Fp::from(i % 1500)).collect();
        assert_eq!(
            permute_lookup_values(&input, &table).unwrap(),
            reference(&input, &table).unwrap()
        );
    }
}
