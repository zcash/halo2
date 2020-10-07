//! This module contains an implementation of the multipoint opening polynomial
//! commitment scheme described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use super::*;
use crate::arithmetic::CurveAffine;

mod prover;
mod verifier;

/// This is a multi-point opening proof used in the polynomial commitment scheme opening.
#[derive(Debug, Clone)]
pub struct Proof<C: CurveAffine> {
    /// A vector of evaluations at each set of query points
    pub q_evals: Vec<C::Scalar>,

    /// Commitment to final polynomial
    pub f_commitment: C,

    /// Commitment proof
    pub opening: commitment::Proof<C>,
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct ProverQuery<'a, C: CurveAffine> {
    /// point at which polynomial is queried
    pub point: C::Scalar,
    /// coefficients of polynomial
    pub poly: &'a Polynomial<C::Scalar, Coeff>,
    /// blinding factor of polynomial
    pub blind: commitment::Blind<C::Scalar>,
    /// evaluation of polynomial at query point
    pub eval: C::Scalar,
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct VerifierQuery<'a, C: CurveAffine> {
    /// point at which polynomial is queried
    pub point: C::Scalar,
    /// commitment to polynomial
    pub commitment: &'a C,
    /// evaluation of polynomial at query point
    pub eval: C::Scalar,
}
