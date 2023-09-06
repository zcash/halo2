use std::fmt::Debug;

use super::commitment::{Blind, MSM};
use crate::{
    arithmetic::eval_polynomial,
    poly::{Coeff, Polynomial},
};
use halo2curves::CurveAffine;

pub trait Query<F>: Sized + Clone + Send + Sync {
    type Commitment: PartialEq + Copy + Send + Sync;
    type Eval: Clone + Default + Debug;

    fn get_point(&self) -> F;
    fn get_eval(&self) -> Self::Eval;
    fn get_commitment(&self) -> Self::Commitment;
}

/// A polynomial query at a point
#[derive(Debug, Clone)]
pub struct ProverQuery<'com, C: CurveAffine> {
    /// point at which polynomial is queried
    pub(crate) point: C::Scalar,
    /// coefficients of polynomial
    pub(crate) poly: &'com Polynomial<C::Scalar, Coeff>,
    /// blinding factor of polynomial
    pub(crate) blind: Blind<C::Scalar>,
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct PolynomialPointer<'com, C: CurveAffine> {
    pub(crate) poly: &'com Polynomial<C::Scalar, Coeff>,
    pub(crate) blind: Blind<C::Scalar>,
}

impl<'com, C: CurveAffine> PartialEq for PolynomialPointer<'com, C> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.poly, other.poly)
    }
}

impl<'com, C: CurveAffine> Query<C::Scalar> for ProverQuery<'com, C> {
    type Commitment = PolynomialPointer<'com, C>;
    type Eval = C::Scalar;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> Self::Eval {
        eval_polynomial(&self.poly[..], self.get_point())
    }
    fn get_commitment(&self) -> Self::Commitment {
        PolynomialPointer {
            poly: self.poly,
            blind: self.blind,
        }
    }
}

impl<'com, C: CurveAffine, M: MSM<C>> VerifierQuery<'com, C, M> {
    /// Create a new verifier query based on a commitment
    pub fn new_commitment(commitment: &'com C, point: C::Scalar, eval: C::Scalar) -> Self {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::Commitment(commitment),
        }
    }

    /// Create a new verifier query based on a linear combination of commitments
    pub fn new_msm(msm: &'com M, point: C::Scalar, eval: C::Scalar) -> VerifierQuery<'com, C, M> {
        VerifierQuery {
            point,
            eval,
            commitment: CommitmentReference::MSM(msm),
        }
    }
}

/// A polynomial query at a point
#[derive(Debug)]
pub struct VerifierQuery<'com, C: CurveAffine, M: MSM<C>> {
    /// point at which polynomial is queried
    pub(crate) point: C::Scalar,
    /// commitment to polynomial
    pub(crate) commitment: CommitmentReference<'com, C, M>,
    /// evaluation of polynomial at query point
    pub(crate) eval: C::Scalar,
}

impl<'com, C: CurveAffine, M: MSM<C>> Clone for VerifierQuery<'com, C, M> {
    fn clone(&self) -> Self {
        Self {
            point: self.point,
            commitment: self.commitment,
            eval: self.eval,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum CommitmentReference<'r, C: CurveAffine, M: MSM<C>> {
    Commitment(&'r C),
    MSM(&'r M),
}

impl<'r, C: CurveAffine, M: MSM<C>> Copy for CommitmentReference<'r, C, M> {}

impl<'r, C: CurveAffine, M: MSM<C>> PartialEq for CommitmentReference<'r, C, M> {
    #![allow(clippy::vtable_address_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&CommitmentReference::Commitment(a), &CommitmentReference::Commitment(b)) => {
                std::ptr::eq(a, b)
            }
            (&CommitmentReference::MSM(a), &CommitmentReference::MSM(b)) => std::ptr::eq(a, b),
            _ => false,
        }
    }
}

impl<'com, C: CurveAffine, M: MSM<C>> Query<C::Scalar> for VerifierQuery<'com, C, M> {
    type Eval = C::Scalar;
    type Commitment = CommitmentReference<'com, C, M>;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        self.commitment
    }
}
