use super::{
    circuit::{AdviceWire, FixedWire},
    ProvingKey,
};
use crate::arithmetic::{parallelize, BatchInvert, Curve, CurveAffine, Field};
use crate::poly::{
    commitment::{Blind, Params},
    Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
};
use std::collections::BTreeSet;
pub mod prover;
pub mod verifier;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum InputWire {
    Advice(AdviceWire),
    Fixed(FixedWire),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TableWire {
    Advice(AdviceWire),
    Fixed(FixedWire),
}

#[derive(Clone, Debug)]
pub struct Lookup<C: CurveAffine> {
    pub input_wires: Vec<InputWire>,
    pub table_wires: Vec<TableWire>,
    pub permuted: Option<Permuted<C>>,
    pub product: Option<Product<C>>,
}

#[derive(Clone, Debug)]
pub struct Proof<C: CurveAffine> {
    pub product_commitment: C,
    pub product_eval: C::Scalar,
    pub product_next_eval: C::Scalar,
    pub permuted_input_commitment: C,
    pub permuted_table_commitment: C,
    pub permuted_input_eval: C::Scalar,
    pub permuted_input_inv_eval: C::Scalar,
    pub permuted_table_eval: C::Scalar,
}

#[derive(Clone, Debug)]
pub struct Permuted<C: CurveAffine> {
    pub permuted_input_value: Polynomial<C::Scalar, LagrangeCoeff>,
    pub permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    pub permuted_input_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    pub permuted_input_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    pub permuted_input_blind: Blind<C::Scalar>,
    pub permuted_input_commitment: C,
    pub permuted_table_value: Polynomial<C::Scalar, LagrangeCoeff>,
    pub permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    pub permuted_table_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    pub permuted_table_blind: Blind<C::Scalar>,
    pub permuted_table_commitment: C,
}

#[derive(Clone, Debug)]
pub struct Product<C: CurveAffine> {
    pub product_poly: Polynomial<C::Scalar, Coeff>,
    pub product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    pub product_next_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    pub product_commitment: C,
    pub product_blind: Blind<C::Scalar>,
}

impl<C: CurveAffine> Lookup<C> {
    pub fn new(input_wires: &[InputWire], table_wires: &[TableWire]) -> Self {
        assert_eq!(input_wires.len(), table_wires.len());
        Lookup {
            input_wires: input_wires.to_vec(),
            table_wires: table_wires.to_vec(),
            permuted: None,
            product: None,
        }
    }

    pub fn construct_permuted(
        &mut self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        theta: C::Scalar,
        advice_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
    ) -> Permuted<C> {
        // Values of input wires involved in the lookup
        let unpermuted_input_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .input_wires
            .iter()
            .map(|&input| match input {
                InputWire::Advice(wire) => advice_values[wire.0].clone(),
                InputWire::Fixed(wire) => fixed_values[wire.0].clone(),
            })
            .collect();

        // Values of table wires involved in the lookup
        let unpermuted_table_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .table_wires
            .iter()
            .map(|&input| match input {
                TableWire::Advice(wire) => advice_values[wire.0].clone(),
                TableWire::Fixed(wire) => fixed_values[wire.0].clone(),
            })
            .collect();

        // Permute each (InputWire, TableWire) pair
        let permuted_values: Vec<(_, _)> = unpermuted_input_values
            .iter()
            .zip(unpermuted_table_values.iter())
            .map(|(unpermuted_input, unpermuted_table)| {
                Lookup::<C>::permute_wire_pair(unpermuted_input, unpermuted_table)
            })
            .collect();

        // Compressed version of input wires
        let permuted_input_value = permuted_values
            .iter()
            .map(|(input, _)| input)
            .fold(domain.empty_lagrange(), |acc, input| acc * theta + input);

        // Compressed version of table wires
        let permuted_table_value = permuted_values
            .iter()
            .map(|(_, table)| table)
            .fold(domain.empty_lagrange(), |acc, table| acc * theta + table);

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

        let permuted_input_blind = Blind(C::Scalar::random());
        let permuted_input_commitment = params
            .commit_lagrange(&permuted_input_value, permuted_input_blind)
            .to_affine();

        let permuted_table_poly = pk.vk.domain.lagrange_to_coeff(permuted_table_value.clone());
        let permuted_table_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_table_poly.clone(), Rotation::default());
        let permuted_table_blind = Blind(C::Scalar::random());
        let permuted_table_commitment = params
            .commit_lagrange(&permuted_table_value, permuted_table_blind)
            .to_affine();

        let permuted = Permuted {
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
        };

        self.permuted = Some(permuted.clone());
        permuted
    }

    fn permute_wire_pair(
        input_value: &Polynomial<C::Scalar, LagrangeCoeff>,
        table_value: &Polynomial<C::Scalar, LagrangeCoeff>,
    ) -> (
        Polynomial<C::Scalar, LagrangeCoeff>,
        Polynomial<C::Scalar, LagrangeCoeff>,
    ) {
        let mut input_coeffs = input_value.get_values().to_vec();
        let mut table_coeffs = table_value.get_values().to_vec();

        // Sort advice lookup wire values
        input_coeffs.sort();
        input_coeffs.reverse();
        let permuted_input_value = Polynomial::new(input_coeffs.to_vec());

        // Get the unique values that appear in the advice wire
        let unique_input_coeffs: BTreeSet<C::Scalar> = input_coeffs.iter().cloned().collect();

        // Sort table wire values according to permuted input lookup wire values
        for &coeff in unique_input_coeffs.iter() {
            // Earliest index of the unique value in the permuted input poly
            let input_idx = input_coeffs
                .iter()
                .position(|&input_coeff| input_coeff == coeff)
                .unwrap();

            // Index of the unique value in the fixed values
            let table_idx = table_coeffs
                .iter()
                .position(|&table_coeff| table_coeff == coeff)
                .unwrap();

            // Move the relevant coeff in the fixed values to match the advice values idx
            table_coeffs.swap(input_idx, table_idx);
        }

        let permuted_table_value = Polynomial::new(table_coeffs.to_vec());

        (permuted_input_value, permuted_table_value)
    }

    pub fn construct_product(
        &mut self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        beta: C::Scalar,
        gamma: C::Scalar,
        theta: C::Scalar,
        advice_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
    ) -> Product<C> {
        let permuted = self.permuted.clone().unwrap();
        let unpermuted_input_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .input_wires
            .iter()
            .map(|&input| match input {
                InputWire::Advice(wire) => advice_values[wire.0].clone(),
                InputWire::Fixed(wire) => fixed_values[wire.0].clone(),
            })
            .collect();

        let unpermuted_table_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .table_wires
            .iter()
            .map(|&table| match table {
                TableWire::Advice(wire) => advice_values[wire.0].clone(),
                TableWire::Fixed(wire) => fixed_values[wire.0].clone(),
            })
            .collect();

        // Goal is to compute the products of fractions
        //
        // (a_1(\omega^i) + \theta a_2(\omega^i) + ... + beta)(s_1(\omega^i) + \theta(\omega^i) + ... + \gamma) /
        // (a'(\omega^i) + \beta)(s'(\omega^i) + \gamma)
        //
        // where a_j(X) is the jth input wire in this lookup,
        // where a'(X) is the compression of the permuted input wires,
        // q_j(X) is the jth table wire in this lookup,
        // q'(X) is the compression of the permuted table wires,
        // and i is the ith row of the wire.
        let mut lookup_product = vec![C::Scalar::one(); params.n as usize];

        // Denominator uses the permuted input wire and permuted table wire
        parallelize(&mut lookup_product, |lookup_product, start| {
            for ((lookup_product, permuted_input_value), permuted_table_value) in lookup_product
                .iter_mut()
                .zip(permuted.permuted_input_value[start..].iter())
                .zip(permuted.permuted_table_value[start..].iter())
            {
                *lookup_product *= &(beta + permuted_input_value);
                *lookup_product *= &(gamma + permuted_table_value);
            }
        });

        // Batch invert to obtain the denominators for the lookup product
        // polynomials
        lookup_product.iter_mut().batch_invert();

        // Finish the computation of the entire fraction by computing the numerators
        // (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
        // Compress unpermuted InputWires
        let mut input_term = vec![C::Scalar::zero(); params.n as usize];
        let mut theta_j = C::Scalar::one();
        for unpermuted_input_value in unpermuted_input_values.iter() {
            parallelize(&mut input_term, |input_term, start| {
                for (input_term, advice_value) in input_term
                    .iter_mut()
                    .zip(unpermuted_input_value.get_values()[start..].iter())
                {
                    *input_term += &(*advice_value * &theta_j);
                }
            });
            theta_j *= &theta;
        }

        // Compress unpermuted TableWires
        let mut table_term = vec![C::Scalar::zero(); params.n as usize];
        let mut theta_j = C::Scalar::one();
        for unpermuted_table_value in unpermuted_table_values.iter() {
            parallelize(&mut table_term, |table_term, start| {
                for (table_term, fixed_value) in table_term
                    .iter_mut()
                    .zip(unpermuted_table_value.get_values()[start..].iter())
                {
                    *table_term += &(*fixed_value * &theta_j);
                }
            });
            theta_j *= &theta;
        }

        // Add blinding \beta and \gamma
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
        // where a_j(\omega^i) is the jth input wire in this lookup,
        // a'j(\omega^i) is the permuted input wire,
        // s_j(\omega^i) is the jth table wire in this lookup,
        // s'(\omega^i) is the permuted table wire,
        // and i is the ith row of the wire.

        // Compute the evaluations of the lookup product polynomial
        // over our domain, starting with z[0] = 1
        let mut z = vec![C::Scalar::one()];
        for row in 1..(params.n as usize) {
            let mut tmp = z[row - 1];
            tmp *= &lookup_product[row - 1];
            z.push(tmp);
        }
        let z = pk.vk.domain.lagrange_from_vec(z);

        // Check lagrange form of product is correctly constructed over the domain
        // z'(X) (a_1(X) + \theta a_2(X) + ... + \beta) (s_1(X) + \theta s_2(X) + ... + \gamma)
        // - z'(omega X) (a'(X) + \beta) (s'(X) + \gamma)
        let n = params.n as usize;

        for i in 0..n {
            let next_idx = (i + 1) % n;

            let mut left = z.get_values().clone()[i];
            let mut input_term = C::Scalar::zero();
            let mut theta_j = C::Scalar::one();
            for unpermuted_input_value in unpermuted_input_values.iter() {
                let input_value = unpermuted_input_value.get_values()[i];
                input_term += &(theta_j * &input_value);
                theta_j *= &theta;
            }

            let mut table_term = C::Scalar::zero();
            let mut theta_j = C::Scalar::one();
            for unpermuted_table_value in unpermuted_table_values.iter() {
                let table_value = unpermuted_table_value.get_values()[i];
                table_term += &(theta_j * &table_value);
                theta_j *= &theta;
            }

            input_term += &beta;
            table_term += &gamma;
            left *= &(input_term * &table_term);

            let mut right = z.get_values().clone()[next_idx];
            let permuted_input_value = &permuted.permuted_input_value.get_values()[i];

            let permuted_table_value = &permuted.permuted_table_value.get_values()[i];

            right *= &(beta + permuted_input_value);
            right *= &(gamma + permuted_table_value);

            assert_eq!(left, right);
        }

        let product_blind = Blind(C::Scalar::random());
        let product_commitment = params.commit_lagrange(&z, product_blind).to_affine();
        let z = pk.vk.domain.lagrange_to_coeff(z);
        let product_coset = pk
            .vk
            .domain
            .coeff_to_extended(z.clone(), Rotation::default());
        let product_next_coset = pk.vk.domain.coeff_to_extended(z.clone(), Rotation(1));

        let product = Product::<C> {
            product_poly: z.clone(),
            product_coset,
            product_next_coset,
            product_commitment,
            product_blind,
        };

        self.product = Some(product.clone());
        product
    }
}
