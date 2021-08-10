use std::marker::PhantomData;

use crate::arithmetic::CurveAffine;

mod prover;
mod verifier;

/// TODO: Documenetation
#[derive(Default, Debug)]
pub struct Proof<C: CurveAffine> {
    random_commitment: C,
    h_commitments: Vec<C>,
    random_eval: C::Scalar,
}

/// A vanishing argument.
pub(crate) struct Argument<C: CurveAffine> {
    _marker: PhantomData<C>,
}
