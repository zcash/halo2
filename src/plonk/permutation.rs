//! Implementation of a PLONK permutation argument.

use super::circuit::{Advice, Column};
use crate::{
    arithmetic::CurveAffine,
    poly::{Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial},
};

pub(crate) mod keygen;
mod prover;
mod verifier;

/// A permutation argument.
#[derive(Debug, Clone)]
pub(crate) struct Argument {
    /// A sequence of columns involved in the argument.
    columns: Vec<Column<Advice>>,
}

impl Argument {
    pub(crate) fn new(columns: Vec<Column<Advice>>) -> Self {
        Argument { columns }
    }

    pub(crate) fn required_degree(&self) -> usize {
        // The permutation argument will serve alongside the gates, so must be
        // accounted for.
        self.columns.len() + 1
    }
}

/// The verifying key for a single permutation argument.
#[derive(Debug)]
pub(crate) struct VerifyingKey<C: CurveAffine> {
    commitments: Vec<C>,
}

/// The proving key for a single permutation argument.
#[derive(Debug)]
pub(crate) struct ProvingKey<C: CurveAffine> {
    permutations: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    polys: Vec<Polynomial<C::Scalar, Coeff>>,
    cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Proof<C: CurveAffine> {
    permutation_product_commitments: Vec<C>,
    permutation_product_evals: Vec<C::Scalar>,
    permutation_product_inv_evals: Vec<C::Scalar>,
    permutation_evals: Vec<Vec<C::Scalar>>,
}
