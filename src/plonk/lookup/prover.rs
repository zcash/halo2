use super::super::{
    circuit::{Advice, Any, Aux, Column, Fixed},
    ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, Error, ProvingKey,
};
use super::{Argument, Proof};
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, Curve, CurveAffine, FieldExt},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{Hasher, Transcript},
};
use ff::Field;
use std::{collections::BTreeMap, convert::TryFrom, iter};

#[derive(Clone, Debug)]
pub(crate) struct Permuted<C: CurveAffine> {
    permuted_input_value: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_input_commitment: C,
    permuted_table_value: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_table_blind: Blind<C::Scalar>,
    permuted_table_commitment: C,
}

#[derive(Clone, Debug)]
pub(crate) struct Product<C: CurveAffine> {
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_blind: Blind<C::Scalar>,
    product_commitment: C,
}

#[derive(Clone, Debug)]
pub(crate) struct Committed<C: CurveAffine> {
    pub permuted: Permuted<C>,
    pub product: Product<C>,
}

pub(crate) struct Constructed<C: CurveAffine> {
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_input_commitment: C,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_blind: Blind<C::Scalar>,
    permuted_table_commitment: C,
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_blind: Blind<C::Scalar>,
    product_commitment: C,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
    pub product_eval: C::Scalar,
    pub product_inv_eval: C::Scalar,
    pub permuted_input_eval: C::Scalar,
    pub permuted_input_inv_eval: C::Scalar,
    pub permuted_table_eval: C::Scalar,
}

impl Argument {
    /// Given a Lookup with input columns [A_0, A_1, ..., A_m] and table columns
    /// [S_0, S_1, ..., S_m], this method
    /// - constructs A_compressed = A_0 + theta A_1 + theta^2 A_2 + ... and
    ///   S_compressed = S_0 + theta S_1 + theta^2 S_2 + ...,
    /// - permutes A_compressed and S_compressed using permute_column_pair() helper,
    ///   obtaining A' and S', and
    /// - constructs Permuted<C> struct using permuted_input_value = A', and
    ///   permuted_table_value = S'.
    /// The Permuted<C> struct is used to update the Lookup, and is then returned.
    pub(in crate::plonk) fn commit_permuted<
        C: CurveAffine,
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        theta: ChallengeTheta<C::Scalar>,
        advice_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        aux_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<Permuted<C>, Error> {
        // Values of input columns involved in the lookup
        let unpermuted_input_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .input_columns
            .iter()
            .map(|&input| match input.column_type() {
                Any::Advice => advice_values[input.index()].clone(),
                Any::Fixed => fixed_values[input.index()].clone(),
                Any::Aux => aux_values[input.index()].clone(),
            })
            .collect();

        // Compressed version of input columns
        let compressed_input_value = unpermuted_input_values
            .iter()
            .fold(domain.empty_lagrange(), |acc, input| acc * *theta + input);

        // Values of table columns involved in the lookup
        let unpermuted_table_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .table_columns
            .iter()
            .map(|&table| match table.column_type() {
                Any::Advice => advice_values[table.index()].clone(),
                Any::Fixed => fixed_values[table.index()].clone(),
                Any::Aux => aux_values[table.index()].clone(),
            })
            .collect();

        // Compressed version of table columns
        let compressed_table_value = unpermuted_table_values
            .iter()
            .fold(domain.empty_lagrange(), |acc, table| acc * *theta + table);

        // Permute compressed (InputColumn, TableColumn) pair
        let (permuted_input_value, permuted_table_value) =
            permute_column_pair::<C>(domain, &compressed_input_value, &compressed_table_value)?;

        // Construct Permuted struct
        let permuted_input_poly = pk.vk.domain.lagrange_to_coeff(permuted_input_value.clone());
        let permuted_input_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation::default());
        let permuted_input_inv_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation(-1));

        let permuted_input_blind = Blind(C::Scalar::rand());
        let permuted_input_commitment = params
            .commit_lagrange(&permuted_input_value, permuted_input_blind)
            .to_affine();

        let permuted_table_poly = pk.vk.domain.lagrange_to_coeff(permuted_table_value.clone());
        let permuted_table_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_table_poly.clone(), Rotation::default());
        let permuted_table_blind = Blind(C::Scalar::rand());
        let permuted_table_commitment = params
            .commit_lagrange(&permuted_table_value, permuted_table_blind)
            .to_affine();

        // Hash each permuted input commitment
        transcript
            .absorb_point(&permuted_input_commitment)
            .map_err(|_| Error::TranscriptError)?;

        // Hash each permuted table commitment
        transcript
            .absorb_point(&permuted_table_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Permuted {
            permuted_input_value,
            permuted_input_poly,
            permuted_input_coset,
            permuted_input_inv_coset,
            permuted_input_blind,
            permuted_input_commitment,
            permuted_table_value,
            permuted_table_poly,
            permuted_table_coset,
            permuted_table_blind,
            permuted_table_commitment,
        })
    }

    /// Given a Lookup with input columns, table columns, and the permuted
    /// input column and permuted table column, this method constructs the
    /// grand product polynomial over the lookup. The grand product polynomial
    /// is used to populate the Product<C> struct. The Product<C> struct is
    /// added to the Lookup and finally returned by the method.
    pub(in crate::plonk) fn commit_product<
        C: CurveAffine,
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        permuted: &Permuted<C>,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        theta: ChallengeTheta<C::Scalar>,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        advice_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        aux_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<Product<C>, Error> {
        let permuted = permuted.clone();
        let unpermuted_input_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .input_columns
            .iter()
            .map(|&input| match input.column_type() {
                Any::Advice => advice_values[input.index()].clone(),
                Any::Fixed => fixed_values[input.index()].clone(),
                Any::Aux => aux_values[input.index()].clone(),
            })
            .collect();

        let unpermuted_table_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .table_columns
            .iter()
            .map(|&table| match table.column_type() {
                Any::Advice => advice_values[table.index()].clone(),
                Any::Fixed => fixed_values[table.index()].clone(),
                Any::Aux => aux_values[table.index()].clone(),
            })
            .collect();

        // Goal is to compute the products of fractions
        //
        // (a_1(\omega^i) + \theta a_2(\omega^i) + ... + beta)(s_1(\omega^i) + \theta(\omega^i) + ... + \gamma) /
        // (a'(\omega^i) + \beta)(s'(\omega^i) + \gamma)
        //
        // where a_j(X) is the jth input column in this lookup,
        // where a'(X) is the compression of the permuted input columns,
        // s_j(X) is the jth table column in this lookup,
        // s'(X) is the compression of the permuted table columns,
        // and i is the ith row of the column.
        let mut lookup_product = vec![C::Scalar::one(); params.n as usize];

        // Denominator uses the permuted input column and permuted table column
        parallelize(&mut lookup_product, |lookup_product, start| {
            for ((lookup_product, permuted_input_value), permuted_table_value) in lookup_product
                .iter_mut()
                .zip(permuted.permuted_input_value[start..].iter())
                .zip(permuted.permuted_table_value[start..].iter())
            {
                *lookup_product *= &(*beta + permuted_input_value);
                *lookup_product *= &(*gamma + permuted_table_value);
            }
        });

        // Batch invert to obtain the denominators for the lookup product
        // polynomials
        lookup_product.iter_mut().batch_invert();

        // Finish the computation of the entire fraction by computing the numerators
        // (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
        // Compress unpermuted input columns
        let mut input_term = vec![C::Scalar::zero(); params.n as usize];
        for unpermuted_input_value in unpermuted_input_values.iter() {
            parallelize(&mut input_term, |input_term, start| {
                for (input_term, input_value) in input_term
                    .iter_mut()
                    .zip(unpermuted_input_value[start..].iter())
                {
                    *input_term *= &theta;
                    *input_term += input_value;
                }
            });
        }

        // Compress unpermuted table columns
        let mut table_term = vec![C::Scalar::zero(); params.n as usize];
        for unpermuted_table_value in unpermuted_table_values.iter() {
            parallelize(&mut table_term, |table_term, start| {
                for (table_term, fixed_value) in table_term
                    .iter_mut()
                    .zip(unpermuted_table_value[start..].iter())
                {
                    *table_term *= &theta;
                    *table_term += fixed_value;
                }
            });
        }

        // Add \beta and \gamma offsets
        parallelize(&mut lookup_product, |product, start| {
            for ((product, input_term), table_term) in product
                .iter_mut()
                .zip(input_term[start..].iter())
                .zip(table_term[start..].iter())
            {
                *product *= &(*input_term + &beta);
                *product *= &(*table_term + &gamma);
            }
        });

        // The product vector is a vector of products of fractions of the form
        //
        // (a_1(\omega^i) + \theta a_2(\omega^i) + ... + \beta)(s_1(\omega^i) + \theta s_2(\omega^i) + ... + \gamma)/
        // (a'(\omega^i) + \beta) (s'(\omega^i) + \gamma)
        //
        // where a_j(\omega^i) is the jth input column in this lookup,
        // a'j(\omega^i) is the permuted input column,
        // s_j(\omega^i) is the jth table column in this lookup,
        // s'(\omega^i) is the permuted table column,
        // and i is the ith row of the column.

        // Compute the evaluations of the lookup product polynomial
        // over our domain, starting with z[0] = 1
        let mut z = vec![C::Scalar::one()];
        for row in 1..(params.n as usize) {
            let mut tmp = z[row - 1];
            tmp *= &lookup_product[row];
            z.push(tmp);
        }
        let z = pk.vk.domain.lagrange_from_vec(z);

        #[cfg(feature = "sanity-checks")]
        // This test works only with intermediate representations in this method.
        // It can be used for debugging purposes.
        {
            // While in Lagrange basis, check that product is correctly constructed
            let n = params.n as usize;

            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
            for i in 0..n {
                let prev_idx = (n + i - 1) % n;

                let mut left = z[i];
                let permuted_input_value = &permuted.permuted_input_value[i];

                let permuted_table_value = &permuted.permuted_table_value[i];

                left *= &(*beta + permuted_input_value);
                left *= &(*gamma + permuted_table_value);

                let mut right = z[prev_idx];
                let mut input_term = unpermuted_input_values
                    .iter()
                    .fold(C::Scalar::zero(), |acc, input| acc * &theta + &input[i]);

                let mut table_term = unpermuted_table_values
                    .iter()
                    .fold(C::Scalar::zero(), |acc, table| acc * &theta + &table[i]);

                input_term += &(*beta);
                table_term += &(*gamma);
                right *= &(input_term * &table_term);

                assert_eq!(left, right);
            }
        }

        let product_blind = Blind(C::Scalar::rand());
        let product_commitment = params.commit_lagrange(&z, product_blind).to_affine();
        let z = pk.vk.domain.lagrange_to_coeff(z);
        let product_coset = pk
            .vk
            .domain
            .coeff_to_extended(z.clone(), Rotation::default());
        let product_inv_coset = pk.vk.domain.coeff_to_extended(z.clone(), Rotation(-1));

        // Hash each product commitment
        transcript
            .absorb_point(&product_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Product::<C> {
            product_poly: z,
            product_coset,
            product_inv_coset,
            product_commitment,
            product_blind,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    /// Given a Lookup with input columns, table columns, permuted input
    /// column, permuted table column, and grand product polynomial, this
    /// method constructs constraints that must hold between these values.
    /// This method returns the constraints as a vector of polynomials in
    /// the extended evaluation domain.
    pub(in crate::plonk) fn construct<'a>(
        self,
        pk: &'a ProvingKey<C>,
        theta: ChallengeTheta<C::Scalar>,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        argument: &'a Argument,
        advice_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        fixed_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        aux_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
    ) -> Result<
        (
            Constructed<C>,
            impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>> + 'a,
        ),
        Error,
    > {
        let permuted = self.permuted;
        let product = self.product;
        let unpermuted_input_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> = argument
            .input_columns
            .iter()
            .map(|&input| match input.column_type() {
                Any::Advice => advice_cosets[pk
                    .vk
                    .cs
                    .get_advice_query_index(Column::<Advice>::try_from(input).unwrap(), 0)]
                .clone(),
                Any::Fixed => fixed_cosets[pk
                    .vk
                    .cs
                    .get_fixed_query_index(Column::<Fixed>::try_from(input).unwrap(), 0)]
                .clone(),
                Any::Aux => aux_cosets[pk
                    .vk
                    .cs
                    .get_aux_query_index(Column::<Aux>::try_from(input).unwrap(), 0)]
                .clone(),
            })
            .collect();

        let unpermuted_table_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> = argument
            .table_columns
            .iter()
            .map(|&table| match table.column_type() {
                Any::Advice => advice_cosets[pk
                    .vk
                    .cs
                    .get_advice_query_index(Column::<Advice>::try_from(table).unwrap(), 0)]
                .clone(),
                Any::Fixed => fixed_cosets[pk
                    .vk
                    .cs
                    .get_fixed_query_index(Column::<Fixed>::try_from(table).unwrap(), 0)]
                .clone(),
                Any::Aux => aux_cosets[pk
                    .vk
                    .cs
                    .get_aux_query_index(Column::<Aux>::try_from(table).unwrap(), 0)]
                .clone(),
            })
            .collect();

        let expressions = iter::empty()
            // l_0(X) * (1 - z'(X)) = 0
            .chain(Some(
                Polynomial::one_minus(product.product_coset.clone()) * &pk.l0,
            ))
            // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
            // - z'(\omega^{-1} X) (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
            .chain({
                // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
                let mut left = product.product_coset.clone();
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

                //  z'(\omega^{-1} X) (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
                let mut right = product.product_inv_coset;
                let mut input_terms = pk.vk.domain.empty_extended();

                // Compress the unpermuted input columns
                for input in unpermuted_input_cosets.iter() {
                    // (a_1(X) + \theta a_2(X) + ...)
                    parallelize(&mut input_terms, |input_term, start| {
                        for (input_term, input) in input_term.iter_mut().zip(input[start..].iter())
                        {
                            *input_term *= &(*theta);
                            *input_term += input;
                        }
                    });
                }

                let mut table_terms = pk.vk.domain.empty_extended();
                // Compress the unpermuted table columns
                for table in unpermuted_table_cosets.iter() {
                    //  (s_1(X) + \theta s_2(X) + ...)
                    parallelize(&mut table_terms, |table_term, start| {
                        for (table_term, table) in table_term.iter_mut().zip(table[start..].iter())
                        {
                            *table_term *= &(*theta);
                            *table_term += table;
                        }
                    });
                }

                // Add \beta and \gamma offsets
                parallelize(&mut right, |right, start| {
                    for ((right, input_term), table_term) in right
                        .iter_mut()
                        .zip(input_terms[start..].iter())
                        .zip(table_terms[start..].iter())
                    {
                        *right *= &(*input_term + &(*beta));
                        *right *= &(*table_term + &(*gamma));
                    }
                });

                Some(left - &right)
            })
            // Check that the first values in the permuted input column and permuted
            // fixed column are the same.
            // l_0(X) * (a'(X) - s'(X)) = 0
            .chain(Some(
                (permuted.permuted_input_coset.clone() - &permuted.permuted_table_coset) * &pk.l0,
            ))
            // Check that each value in the permuted lookup input column is either
            // equal to the value above it, or the value at the same index in the
            // permuted table column.
            // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
            .chain(Some(
                (permuted.permuted_input_coset.clone() - &permuted.permuted_table_coset)
                    * &(permuted.permuted_input_coset.clone() - &permuted.permuted_input_inv_coset),
            ));

        Ok((
            Constructed {
                permuted_input_poly: permuted.permuted_input_poly,
                permuted_input_blind: permuted.permuted_input_blind,
                permuted_input_commitment: permuted.permuted_input_commitment,
                permuted_table_poly: permuted.permuted_table_poly,
                permuted_table_blind: permuted.permuted_table_blind,
                permuted_table_commitment: permuted.permuted_table_commitment,
                product_poly: product.product_poly,
                product_blind: product.product_blind,
                product_commitment: product.product_commitment,
            },
            expressions,
        ))
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        self,
        pk: &ProvingKey<C>,
        x: ChallengeX<C::Scalar>,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Evaluated<C> {
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
            transcript.absorb_scalar(eval);
        }

        Evaluated {
            constructed: self,
            product_eval,
            product_inv_eval,
            permuted_input_eval,
            permuted_input_inv_eval,
            permuted_table_eval,
        }
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        pk: &'a ProvingKey<C>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let x_inv = pk.vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open lookup product commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
                eval: self.product_eval,
            }))
            // Open lookup input commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permuted_input_poly,
                blind: self.constructed.permuted_input_blind,
                eval: self.permuted_input_eval,
            }))
            // Open lookup table commitments at x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permuted_table_poly,
                blind: self.constructed.permuted_table_blind,
                eval: self.permuted_table_eval,
            }))
            // Open lookup input commitments at x_inv
            .chain(Some(ProverQuery {
                point: x_inv,
                poly: &self.constructed.permuted_input_poly,
                blind: self.constructed.permuted_input_blind,
                eval: self.permuted_input_eval,
            }))
            // Open lookup product commitments at x_inv
            .chain(Some(ProverQuery {
                point: x_inv,
                poly: &self.constructed.product_poly,
                blind: self.constructed.product_blind,
                eval: self.product_eval,
            }))
    }

    pub(crate) fn build(self) -> Proof<C> {
        Proof {
            product_commitment: self.constructed.product_commitment,
            product_eval: self.product_eval,
            product_inv_eval: self.product_inv_eval,
            permuted_input_commitment: self.constructed.permuted_input_commitment,
            permuted_table_commitment: self.constructed.permuted_table_commitment,
            permuted_input_eval: self.permuted_input_eval,
            permuted_input_inv_eval: self.permuted_input_inv_eval,
            permuted_table_eval: self.permuted_table_eval,
        }
    }
}

/// Given a column of input values A and a column of table values S,
/// this method permutes A and S to produce A' and S', such that:
/// - like values in A' are vertically adjacent to each other; and
/// - the first row in a sequence of like values in A' is the row
///   that has the corresponding value in S'.
/// This method returns (A', S') if no errors are encountered.
fn permute_column_pair<C: CurveAffine>(
    domain: &EvaluationDomain<C::Scalar>,
    input_column: &Polynomial<C::Scalar, LagrangeCoeff>,
    table_column: &Polynomial<C::Scalar, LagrangeCoeff>,
) -> Result<
    (
        Polynomial<C::Scalar, LagrangeCoeff>,
        Polynomial<C::Scalar, LagrangeCoeff>,
    ),
    Error,
> {
    let mut permuted_input_column = input_column.clone();

    // Sort input lookup column values
    permuted_input_column.sort();

    // A BTreeMap of each unique element in the table column and its count
    let mut leftover_table_map: BTreeMap<C::Scalar, u32> =
        table_column.iter().fold(BTreeMap::new(), |mut acc, coeff| {
            *acc.entry(*coeff).or_insert(0) += 1;
            acc
        });
    let mut repeated_input_rows = vec![];
    let mut permuted_table_coeffs = vec![C::Scalar::zero(); table_column.len()];

    for row in 0..permuted_input_column.len() {
        let input_value = permuted_input_column[row];

        // If this is the first occurence of `input_value` in the input column
        if row == 0 || input_value != permuted_input_column[row - 1] {
            permuted_table_coeffs[row] = input_value;
            // Remove one instance of input_value from leftover_table_map
            if let Some(count) = leftover_table_map.get_mut(&input_value) {
                assert!(*count > 0);
                *count -= 1;
            } else {
                // Return error if input_value not found
                return Err(Error::ConstraintSystemFailure);
            }
        // If input value is repeated
        } else {
            repeated_input_rows.push(row);
        }
    }

    // Populate permuted table at unfilled rows with leftover table elements
    for (coeff, count) in leftover_table_map.iter() {
        for _ in 0..*count {
            permuted_table_coeffs[repeated_input_rows.pop().unwrap() as usize] = *coeff;
        }
    }
    assert!(repeated_input_rows.is_empty());

    let mut permuted_table_column = domain.empty_lagrange();
    parallelize(
        &mut permuted_table_column,
        |permuted_table_column, start| {
            for (permuted_table_value, permuted_table_coeff) in permuted_table_column
                .iter_mut()
                .zip(permuted_table_coeffs[start..].iter())
            {
                *permuted_table_value += permuted_table_coeff;
            }
        },
    );

    Ok((permuted_input_column, permuted_table_column))
}
