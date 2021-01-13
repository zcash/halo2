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
        // accounted for. There are constraints of degree 2 regardless of the
        // number of columns involved. (It doesn't make sense to make a
        // permutation argument with zero columns but to be rigorous we account
        // for it here.)

        // degree 2:
        // l_0(X) * (1 - z(X)) = 0
        //
        // degree columns + 1
        // z(X) \prod (p(X) + \beta s_i(X) + \gamma)
        // - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
        std::cmp::max(self.columns.len() + 1, 2)
    }

    pub(crate) fn get_columns(&self) -> Vec<Column<Advice>> {
        self.columns.clone()
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
