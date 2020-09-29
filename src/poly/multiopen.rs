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
