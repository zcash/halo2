use super::super::ProvingKey;
use super::{InputWire, LookupData, Proof, TableWire};
use crate::arithmetic::{eval_polynomial, parallelize, CurveAffine, Field};
use crate::poly::{EvaluationDomain, ExtendedLagrangeCoeff, Polynomial, Rotation};

impl<C: CurveAffine> LookupData<C> {
    pub fn construct_constraints(
        &self,
        pk: &ProvingKey<C>,
        beta: C::Scalar,
        gamma: C::Scalar,
        theta: C::Scalar,
        advice_cosets: &[Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        fixed_cosets: &[Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
    ) -> Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> {
        let permuted = self.permuted.clone().unwrap();
        let product = self.product.clone().unwrap();
        let unpermuted_input_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> = self
            .lookup
            .input_wires
            .iter()
            .map(|&input| match input {
                InputWire::Advice(wire) => {
                    advice_cosets[pk.vk.cs.get_advice_query_index(wire, 0)].clone()
                }
                InputWire::Fixed(wire) => {
                    fixed_cosets[pk.vk.cs.get_fixed_query_index(wire, 0)].clone()
                }
            })
            .collect();

        let unpermuted_table_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> = self
            .lookup
            .table_wires
            .iter()
            .map(|&table| match table {
                TableWire::Advice(wire) => {
                    advice_cosets[pk.vk.cs.get_advice_query_index(wire, 0)].clone()
                }
                TableWire::Fixed(wire) => {
                    fixed_cosets[pk.vk.cs.get_fixed_query_index(wire, 0)].clone()
                }
            })
            .collect();

        let mut constraints: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>> =
            Vec::with_capacity(4);

        // l_0(X) * (1 - z'(X)) = 0
        {
            let mut first_product_poly = pk.vk.domain.empty_extended();
            parallelize(&mut first_product_poly, |first_product_poly, start| {
                for ((first_product_poly, product), l0) in first_product_poly
                    .iter_mut()
                    .zip(product.product_coset[start..].iter())
                    .zip(pk.l0[start..].iter())
                {
                    *first_product_poly += &(*l0 * &(C::Scalar::one() - product));
                }
            });
            constraints.push(first_product_poly);
        }

        // z'(X) (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
        // - z'(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
        {
            let mut left = product.product_coset.clone();
            let mut input_terms = pk.vk.domain.empty_extended();

            // Compress the unpermuted Input wires
            for input in unpermuted_input_cosets.iter() {
                // (a_1(X) + \theta a_2(X) + ...)
                parallelize(&mut input_terms, |input_term, start| {
                    for (input_term, input) in input_term.iter_mut().zip(input[start..].iter()) {
                        *input_term *= &theta;
                        *input_term += input;
                    }
                });
            }

            let mut table_terms = pk.vk.domain.empty_extended();
            // Compress the unpermuted Table wires
            for table in unpermuted_table_cosets.iter() {
                //  (s_1(X) + \theta s_2(X) + ...)
                parallelize(&mut table_terms, |table_term, start| {
                    for (table_term, table) in table_term.iter_mut().zip(table[start..].iter()) {
                        *table_term *= &theta;
                        *table_term += table;
                    }
                });
            }

            // add \beta and \gamma blinding
            parallelize(&mut left, |left, start| {
                for ((left, input_term), table_term) in left
                    .iter_mut()
                    .zip(input_terms[start..].iter())
                    .zip(table_terms[start..].iter())
                {
                    *left *= &(*input_term + &beta);
                    *left *= &(*table_term + &gamma);
                }
            });

            //  z'(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
            let mut right = product.product_next_coset.clone();
            parallelize(&mut right, |right, start| {
                for ((right, permuted_input), permuted_table) in right
                    .iter_mut()
                    .zip(permuted.permuted_input_coset[start..].iter())
                    .zip(permuted.permuted_table_coset[start..].iter())
                {
                    *right *= &(*permuted_input + &beta);
                    *right *= &(*permuted_table + &gamma);
                }
            });
            constraints.push(left - &right);
        }

        // Check that the first values in the permuted input wire and permuted
        // fixed wire are the same.
        // l_0(X) * (a'(X) - s'(X)) = 0
        {
            let mut first_lookup_poly = pk.vk.domain.empty_extended();
            parallelize(&mut first_lookup_poly, |first_lookup_poly, start| {
                for (((first_lookup_poly, input), table), l0) in first_lookup_poly
                    .iter_mut()
                    .zip(permuted.permuted_input_coset[start..].iter())
                    .zip(permuted.permuted_table_coset[start..].iter())
                    .zip(pk.l0[start..].iter())
                {
                    *first_lookup_poly += &(*l0 * &(*input - table));
                }
            });
            constraints.push(first_lookup_poly);
        }

        // Check that each value in the permuted lookup input wire is either
        // equal to the value above it, or the value at the same index in the
        // permuted table wire.
        // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
        {
            let mut lookup_poly =
                permuted.permuted_input_coset.clone() - &permuted.permuted_table_coset;
            lookup_poly = lookup_poly
                * &(permuted.permuted_input_coset.clone() - &permuted.permuted_input_inv_coset);
            constraints.push(lookup_poly);
        }

        constraints
    }

    pub fn construct_proof(
        &mut self,
        domain: &EvaluationDomain<C::Scalar>,
        point: C::Scalar,
    ) -> Proof<C> {
        let product = self.product.clone().unwrap();
        let permuted = self.permuted.clone().unwrap();

        let product_eval: C::Scalar = eval_polynomial(&product.product_poly.get_values(), point);

        let product_next_eval: C::Scalar = eval_polynomial(
            &product.product_poly.get_values(),
            domain.rotate_omega(point, Rotation(1)),
        );

        let permuted_input_eval: C::Scalar = eval_polynomial(&permuted.permuted_input_poly, point);
        let permuted_input_inv_eval: C::Scalar = eval_polynomial(
            &permuted.permuted_input_poly,
            domain.rotate_omega(point, Rotation(-1)),
        );
        let permuted_table_eval: C::Scalar = eval_polynomial(&permuted.permuted_table_poly, point);

        Proof {
            product_commitment: product.product_commitment,
            product_eval,
            product_next_eval,
            permuted_input_commitment: permuted.permuted_input_commitment,
            permuted_table_commitment: permuted.permuted_table_commitment,
            permuted_input_eval,
            permuted_input_inv_eval,
            permuted_table_eval,
        }
    }
}
