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
// pub mod prover;
// pub mod verifier;

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
}