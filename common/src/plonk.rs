//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;
use group::ff::{Field, FromUniformBytes, PrimeField};

use crate::arithmetic::CurveAffine;
use crate::helpers::{
    polynomial_slice_byte_length, read_polynomial_vec, write_polynomial_slice, SerdeCurveAffine,
    SerdePrimeField,
};
use crate::poly::{
    Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, PinnedEvaluationDomain,
    Polynomial,
};
use crate::transcript::{ChallengeScalar, EncodedChallenge, Transcript};
use crate::SerdeFormat;
use halo2_middleware::circuit::{
    Advice, AdviceQueryMid, Challenge, Column, ExpressionMid, Fixed, FixedQueryMid, GateV2Backend,
    Instance, InstanceQueryMid, PreprocessingV2,
};
use halo2_middleware::poly::Rotation;

pub mod assigned;
pub mod circuit;
pub mod error;
// pub mod evaluation;
pub mod keygen;
pub mod lookup;
pub mod permutation;
pub mod shuffle;
// pub mod vanishing;

// pub mod verifier;

pub use assigned::*;
pub use circuit::*;
pub use error::*;
pub use keygen::*;
// pub use verifier::*;

use std::io;

/// List of queries (columns and rotations) used by a circuit
#[derive(Debug, Clone)]
pub struct Queries {
    /// List of unique advice queries
    pub advice: Vec<(Column<Advice>, Rotation)>,
    /// List of unique instance queries
    pub instance: Vec<(Column<Instance>, Rotation)>,
    /// List of unique fixed queries
    pub fixed: Vec<(Column<Fixed>, Rotation)>,
    /// Contains an integer for each advice column
    /// identifying how many distinct queries it has
    /// so far; should be same length as cs.num_advice_columns.
    pub num_advice_queries: Vec<usize>,
}

impl Queries {
    /// Returns the minimum necessary rows that need to exist in order to
    /// account for e.g. blinding factors.
    pub fn minimum_rows(&self) -> usize {
        self.blinding_factors() // m blinding factors
            + 1 // for l_{-(m + 1)} (l_last)
            + 1 // for l_0 (just for extra breathing room for the permutation
                // argument, to essentially force a separation in the
                // permutation polynomial between the roles of l_last, l_0
                // and the interstitial values.)
            + 1 // for at least one row
    }

    /// Compute the number of blinding factors necessary to perfectly blind
    /// each of the prover's witness polynomials.
    pub fn blinding_factors(&self) -> usize {
        // All of the prover's advice columns are evaluated at no more than
        let factors = *self.num_advice_queries.iter().max().unwrap_or(&1);
        // distinct points during gate checks.

        // - The permutation argument witness polynomials are evaluated at most 3 times.
        // - Each lookup argument has independent witness polynomials, and they are
        //   evaluated at most 2 times.
        let factors = std::cmp::max(3, factors);

        // Each polynomial is evaluated at most an additional time during
        // multiopen (at x_3 to produce q_evals):
        let factors = factors + 1;

        // h(x) is derived by the other evaluations so it does not reveal
        // anything; in fact it does not even appear in the proof.

        // h(x_3) is also not revealed; the verifier only learns a single
        // evaluation of a polynomial in x_1 which has h(x_3) and another random
        // polynomial evaluated at x_3 as coefficients -- this random polynomial
        // is "random_poly" in the vanishing argument.

        // Add an additional blinding factor as a slight defense against
        // off-by-one errors.
        factors + 1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theta;
pub type ChallengeTheta<F> = ChallengeScalar<F, Theta>;

#[derive(Clone, Copy, Debug)]
pub struct Beta;
pub type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
pub struct Gamma;
pub type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
pub struct Y;
pub type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
pub struct X;
pub type ChallengeX<F> = ChallengeScalar<F, X>;
