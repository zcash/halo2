use crate::arithmetic::CurveAffine;
use crate::poly::{commitment::Blind, Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial};

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
    permuted: Permuted<C>,
    product: Product<C>,
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
