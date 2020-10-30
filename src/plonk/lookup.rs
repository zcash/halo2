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
}