//! Implementation of a PLONK permutation argument.

use crate::arithmetic::CurveAffine;

mod prover;
mod verifier;

#[derive(Debug, Clone)]
pub(crate) struct Proof<C: CurveAffine> {
    permutation_product_commitments: Vec<C>,
    permutation_product_evals: Vec<C::Scalar>,
    permutation_product_inv_evals: Vec<C::Scalar>,
    permutation_evals: Vec<Vec<C::Scalar>>,
}
