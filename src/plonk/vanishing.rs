use std::marker::PhantomData;

use crate::arithmetic::CurveAffine;

mod prover;
mod verifier;

/// A vanishing argument.
pub(crate) struct Argument<C: CurveAffine> {
    _marker: PhantomData<C>,
}

#[derive(Debug, Clone)]
pub(crate) struct Proof<C: CurveAffine> {
    h_commitments: Vec<C>,
    h_evals: Vec<C::Scalar>,
}
