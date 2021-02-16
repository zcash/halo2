use super::super::{
    circuit::Expression, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error,
    ProvingKey,
};
use super::Argument;
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, Curve, CurveAffine, FieldExt},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::TranscriptWrite,
};
use ff::Field;
use std::{
    collections::BTreeMap,
    iter,
    ops::{Mul, MulAssign},
};

#[derive(Debug)]
pub(in crate::plonk) struct Permuted<C: CurveAffine> {
    unpermuted_input_expressions: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    unpermuted_input_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permuted_input_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_input_commitment: C,
    unpermuted_table_expressions: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    unpermuted_table_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permuted_table_expression: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_table_blind: Blind<C::Scalar>,
    permuted_table_commitment: C,
}

#[derive(Debug)]
pub(in crate::plonk) struct Committed<C: CurveAffine> {
    permuted: Permuted<C>,
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_blind: Blind<C::Scalar>,
    product_commitment: C,
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

impl<F: FieldExt> Argument<F> {
    /// Given a Lookup with input expressions [A_0, A_1, ..., A_{m-1}] and table expressions
    /// [S_0, S_1, ..., S_{m-1}], this method
    /// - constructs A_compressed = \theta^{m-1} A_0 + theta^{m-2} A_1 + ... + \theta A_{m-2} + A_{m-1}
    ///   and S_compressed = \theta^{m-1} S_0 + theta^{m-2} S_1 + ... + \theta S_{m-2} + S_{m-1},
    /// - permutes A_compressed and S_compressed using permute_expression_pair() helper,
    ///   obtaining A' and S', and
    /// - constructs Permuted<C> struct using permuted_input_value = A', and
    ///   permuted_table_expression = S'.
    /// The Permuted<C> struct is used to update the Lookup, and is then returned.
    pub(in crate::plonk) fn commit_permuted<'a, C, T: TranscriptWrite<C>>(
        &self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        theta: ChallengeTheta<C>,
        advice_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        instance_values: &'a [Polynomial<C::Scalar, LagrangeCoeff>],
        advice_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        fixed_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        instance_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        transcript: &mut T,
    ) -> Result<Permuted<C>, Error>
    where
        C: CurveAffine<Scalar = F>,
        C::Projective: Mul<F, Output = C::Projective> + MulAssign<F>,
    {
        // Closure to get values of expressions and compress them
        let compress_expressions = |expressions: &[Expression<C::Scalar>]| {
            // Values of input expressions involved in the lookup
            let unpermuted_expressions: Vec<_> = expressions
                .iter()
                .map(|expression| {
                    expression.evaluate(
                        &|index| {
                            let query = pk.vk.cs.fixed_queries[index];
                            let column_index = query.0.index();
                            let rotation = query.1;
                            fixed_values[column_index].clone().rotate(rotation)
                        },
                        &|index| {
                            let query = pk.vk.cs.advice_queries[index];
                            let column_index = query.0.index();
                            let rotation = query.1;
                            advice_values[column_index].clone().rotate(rotation)
                        },
                        &|index| {
                            let query = pk.vk.cs.instance_queries[index];
                            let column_index = query.0.index();
                            let rotation = query.1;
                            instance_values[column_index].clone().rotate(rotation)
                        },
                        &|a, b| a + &b,
                        &|a, b| {
                            let mut modified_a = vec![C::Scalar::one(); params.n as usize];
                            parallelize(&mut modified_a, |modified_a, start| {
                                for ((modified_a, a), b) in modified_a
                                    .iter_mut()
                                    .zip(a[start..].iter())
                                    .zip(b[start..].iter())
                                {
                                    *modified_a *= *a * b;
                                }
                            });
                            pk.vk.domain.lagrange_from_vec(modified_a)
                        },
                        &|a, scalar| a * scalar,
                    )
                })
                .collect();

            let unpermuted_cosets: Vec<_> = expressions
                .iter()
                .map(|expression| {
                    expression.evaluate(
                        &|index| fixed_cosets[index].clone(),
                        &|index| advice_cosets[index].clone(),
                        &|index| instance_cosets[index].clone(),
                        &|a, b| a + &b,
                        &|a, b| a * &b,
                        &|a, scalar| a * scalar,
                    )
                })
                .collect();

            // Compressed version of expressions
            let compressed_expression = unpermuted_expressions
                .iter()
                .fold(domain.empty_lagrange(), |acc, expression| {
                    acc * *theta + expression
                });

            (
                unpermuted_expressions,
                unpermuted_cosets,
                compressed_expression,
            )
        };

        // Closure to construct commitment to vector of values
        let commit_values = |values: &Polynomial<C::Scalar, LagrangeCoeff>| {
            let poly = pk.vk.domain.lagrange_to_coeff(values.clone());
            let blind = Blind(C::Scalar::rand());
            let commitment = params.commit_lagrange(&values, blind).to_affine();
            (poly, blind, commitment)
        };

        // Get values of input expressions involved in the lookup and compress them
        let (unpermuted_input_expressions, unpermuted_input_cosets, compressed_input_expression) =
            compress_expressions(&self.input_expressions);

        // Get values of table expressions involved in the lookup and compress them
        let (unpermuted_table_expressions, unpermuted_table_cosets, compressed_table_expression) =
            compress_expressions(&self.table_expressions);

        // Permute compressed (InputExpression, TableExpression) pair
        let (permuted_input_expression, permuted_table_expression) = permute_expression_pair::<C>(
            domain,
            &compressed_input_expression,
            &compressed_table_expression,
        )?;

        // Commit to permuted input expression
        let (permuted_input_poly, permuted_input_blind, permuted_input_commitment) =
            commit_values(&permuted_input_expression);

        // Commit to permuted table expression
        let (permuted_table_poly, permuted_table_blind, permuted_table_commitment) =
            commit_values(&permuted_table_expression);

        // Hash permuted input commitment
        transcript
            .write_point(permuted_input_commitment)
            .map_err(|_| Error::TranscriptError)?;

        // Hash permuted table commitment
        transcript
            .write_point(permuted_table_commitment)
            .map_err(|_| Error::TranscriptError)?;

        let permuted_input_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation::cur());
        let permuted_input_inv_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation(-1));
        let permuted_table_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_table_poly.clone(), Rotation::cur());

        Ok(Permuted {
            unpermuted_input_expressions,
            unpermuted_input_cosets,
            permuted_input_expression,
            permuted_input_poly,
            permuted_input_coset,
            permuted_input_inv_coset,
            permuted_input_blind,
            permuted_input_commitment,
            unpermuted_table_expressions,
            unpermuted_table_cosets,
            permuted_table_expression,
            permuted_table_poly,
            permuted_table_coset,
            permuted_table_blind,
            permuted_table_commitment,
        })
    }
}

impl<C: CurveAffine> Permuted<C> {
    /// Given a Lookup with input expressions, table expressions, and the permuted
    /// input expression and permuted table expression, this method constructs the
    /// grand product polynomial over the lookup. The grand product polynomial
    /// is used to populate the Product<C> struct. The Product<C> struct is
    /// added to the Lookup and finally returned by the method.
    pub(in crate::plonk) fn commit_product<T: TranscriptWrite<C>>(
        self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        theta: ChallengeTheta<C>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
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
        let mut lookup_product = vec![C::Scalar::zero(); params.n as usize];
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
        lookup_product.iter_mut().batch_invert();

        // Finish the computation of the entire fraction by computing the numerators
        // (\theta^{m-1} a_0(\omega^i) + \theta^{m-2} a_1(\omega^i) + ... + \theta a_{m-2}(\omega^i) + a_{m-1}(\omega^i) + \beta)
        // * (\theta^{m-1} s_0(\omega^i) + \theta^{m-2} s_1(\omega^i) + ... + \theta s_{m-2}(\omega^i) + s_{m-1}(\omega^i) + \gamma)
        parallelize(&mut lookup_product, |product, start| {
            for (i, product) in product.iter_mut().enumerate() {
                let i = i + start;

                // Compress unpermuted input expressions
                let mut input_term = C::Scalar::zero();
                for unpermuted_input_expression in self.unpermuted_input_expressions.iter() {
                    input_term *= &*theta;
                    input_term += &unpermuted_input_expression[i];
                }

                // Compress unpermuted table expressions
                let mut table_term = C::Scalar::zero();
                for unpermuted_table_expression in self.unpermuted_table_expressions.iter() {
                    table_term *= &*theta;
                    table_term += &unpermuted_table_expression[i];
                }

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
        let z = iter::once(C::Scalar::one())
            .chain(lookup_product.into_iter().skip(1))
            .scan(C::Scalar::one(), |state, cur| {
                *state *= &cur;
                Some(*state)
            })
            .collect::<Vec<_>>();
        let z = pk.vk.domain.lagrange_from_vec(z);

        #[cfg(feature = "sanity-checks")]
        // This test works only with intermediate representations in this method.
        // It can be used for debugging purposes.
        {
            // While in Lagrange basis, check that product is correctly constructed
            let n = params.n as usize;

            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            for i in 0..n {
                let prev_idx = (n + i - 1) % n;

                let mut left = z[i];
                let permuted_input_value = &self.permuted_input_expression[i];

                let permuted_table_value = &self.permuted_table_expression[i];

                left *= &(*beta + permuted_input_value);
                left *= &(*gamma + permuted_table_value);

                let mut right = z[prev_idx];
                let mut input_term = self
                    .unpermuted_input_expressions
                    .iter()
                    .fold(C::Scalar::zero(), |acc, input| acc * &*theta + &input[i]);

                let mut table_term = self
                    .unpermuted_table_expressions
                    .iter()
                    .fold(C::Scalar::zero(), |acc, table| acc * &*theta + &table[i]);

                input_term += &(*beta);
                table_term += &(*gamma);
                right *= &(input_term * &table_term);

                assert_eq!(left, right);
            }
        }

        let product_blind = Blind(C::Scalar::rand());
        let product_commitment = params.commit_lagrange(&z, product_blind).to_affine();
        let z = pk.vk.domain.lagrange_to_coeff(z);
        let product_coset = pk.vk.domain.coeff_to_extended(z.clone(), Rotation::cur());
        let product_inv_coset = pk.vk.domain.coeff_to_extended(z.clone(), Rotation::prev());

        // Hash product commitment
        transcript
            .write_point(product_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Committed::<C> {
            permuted: self,
            product_poly: z,
            product_coset,
            product_inv_coset,
            product_commitment,
            product_blind,
        })
    }
}

impl<'a, C: CurveAffine> Committed<C> {
    /// Given a Lookup with input expressions, table expressions, permuted input
    /// expression, permuted table expression, and grand product polynomial, this
    /// method constructs constraints that must hold between these values.
    /// This method returns the constraints as a vector of polynomials in
    /// the extended evaluation domain.
    pub(in crate::plonk) fn construct(
        self,
        pk: &'a ProvingKey<C>,
        theta: ChallengeTheta<C>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
    ) -> (
        Constructed<C>,
        impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>> + 'a,
    ) {
        let permuted = self.permuted;

        let expressions = iter::empty()
            // l_0(X) * (1 - z'(X)) = 0
            .chain(Some(
                Polynomial::one_minus(self.product_coset.clone()) * &pk.l0,
            ))
            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
            .chain({
                // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
                let mut left = self.product_coset.clone();
                parallelize(&mut left, |left, start| {
                    for ((left, permuted_input), permuted_table) in left
                        .iter_mut()
                        .zip(permuted.permuted_input_coset[start..].iter())
                        .zip(permuted.permuted_table_coset[start..].iter())
                    {
                        *left *= &(*permuted_input + &(*beta));
                        *left *= &(*permuted_table + &(*gamma));
                    }
                });

                //  z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
                let mut right = self.product_inv_coset;
                parallelize(&mut right, |right, start| {
                    for (i, right) in right.iter_mut().enumerate() {
                        let i = i + start;

                        // Compress the unpermuted input expressions
                        let mut input_term = C::Scalar::zero();
                        for input in permuted.unpermuted_input_cosets.iter() {
                            input_term *= &*theta;
                            input_term += &input[i];
                        }

                        // Compress the unpermuted table expressions
                        let mut table_term = C::Scalar::zero();
                        for table in permuted.unpermuted_table_cosets.iter() {
                            table_term *= &*theta;
                            table_term += &table[i];
                        }

                        // Add \beta and \gamma offsets
                        *right *= &(input_term + &*beta);
                        *right *= &(table_term + &*gamma);
                    }
                });

                Some(left - &right)
            })
            // Check that the first values in the permuted input expression and permuted
            // fixed expression are the same.
            // l_0(X) * (a'(X) - s'(X)) = 0
            .chain(Some(
                (permuted.permuted_input_coset.clone() - &permuted.permuted_table_coset) * &pk.l0,
            ))
            // Check that each value in the permuted lookup input expression is either
            // equal to the value above it, or the value at the same index in the
            // permuted table expression.
            // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
            .chain(Some(
                (permuted.permuted_input_coset.clone() - &permuted.permuted_table_coset)
                    * &(permuted.permuted_input_coset.clone() - &permuted.permuted_input_inv_coset),
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
    pub(in crate::plonk) fn evaluate<T: TranscriptWrite<C>>(
        self,
        pk: &ProvingKey<C>,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let domain = &pk.vk.domain;
        let x_inv = domain.rotate_omega(*x, Rotation(-1));

        let product_eval = eval_polynomial(&self.product_poly, *x);
        let product_inv_eval = eval_polynomial(&self.product_poly, x_inv);
        let permuted_input_eval = eval_polynomial(&self.permuted_input_poly, *x);
        let permuted_input_inv_eval = eval_polynomial(&self.permuted_input_poly, x_inv);
        let permuted_table_eval = eval_polynomial(&self.permuted_table_poly, *x);

        // Hash each advice evaluation
        for eval in iter::empty()
            .chain(Some(product_eval))
            .chain(Some(product_inv_eval))
            .chain(Some(permuted_input_eval))
            .chain(Some(permuted_input_inv_eval))
            .chain(Some(permuted_table_eval))
        {
            transcript
                .write_scalar(eval)
                .map_err(|_| Error::TranscriptError)?;
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
        let x_inv = pk.vk.domain.rotate_omega(*x, Rotation(-1));

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
            // Open lookup product commitments at x_inv
            .chain(Some(ProverQuery {
                point: x_inv,
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
/// This method returns (A', S') if no errors are encountered.
fn permute_expression_pair<C: CurveAffine>(
    domain: &EvaluationDomain<C::Scalar>,
    input_expression: &Polynomial<C::Scalar, LagrangeCoeff>,
    table_expression: &Polynomial<C::Scalar, LagrangeCoeff>,
) -> Result<ExpressionPair<C::Scalar>, Error> {
    let mut permuted_input_expression = input_expression.clone();

    // Sort input lookup expression values
    permuted_input_expression.sort();

    // A BTreeMap of each unique element in the table expression and its count
    let mut leftover_table_map: BTreeMap<C::Scalar, u32> =
        table_expression
            .iter()
            .fold(BTreeMap::new(), |mut acc, coeff| {
                *acc.entry(*coeff).or_insert(0) += 1;
                acc
            });
    let mut permuted_table_coeffs = vec![C::Scalar::zero(); table_expression.len()];

    let mut repeated_input_rows = permuted_input_expression
        .iter()
        .zip(permuted_table_coeffs.iter_mut())
        .enumerate()
        .filter_map(|(row, (input_value, table_value))| {
            // If this is the first occurence of `input_value` in the input expression
            if row == 0 || *input_value != permuted_input_expression[row - 1] {
                *table_value = *input_value;
                // Remove one instance of input_value from leftover_table_map
                if let Some(count) = leftover_table_map.get_mut(&input_value) {
                    assert!(*count > 0);
                    *count -= 1;
                    None
                } else {
                    // Return error if input_value not found
                    Some(Err(Error::ConstraintSystemFailure))
                }
            // If input value is repeated
            } else {
                Some(Ok(row))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Populate permuted table at unfilled rows with leftover table elements
    for (coeff, count) in leftover_table_map.iter() {
        for _ in 0..*count {
            permuted_table_coeffs[repeated_input_rows.pop().unwrap() as usize] = *coeff;
        }
    }
    assert!(repeated_input_rows.is_empty());

    let mut permuted_table_expression = domain.empty_lagrange();
    parallelize(
        &mut permuted_table_expression,
        |permuted_table_expression, start| {
            for (permuted_table_value, permuted_table_coeff) in permuted_table_expression
                .iter_mut()
                .zip(permuted_table_coeffs[start..].iter())
            {
                *permuted_table_value += permuted_table_coeff;
            }
        },
    );

    Ok((permuted_input_expression, permuted_table_expression))
}
