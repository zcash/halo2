use std::{io, fmt};
use std::marker::PhantomData;

use ff::{BatchInvert, Field};
use group::{Curve, Group};
use halo2_proofs::{
    arithmetic::{eval_polynomial, kate_division},
    poly::{
        commitment::{Blind, Params},
        Coeff, EvaluationDomain, Polynomial,
    },
    transcript::{EncodedChallenge, TranscriptRead, TranscriptWrite},
};
use pasta_curves::arithmetic::CurveAffine;
use rand_core::RngCore;

use super::AccumulationScheme;

/// An abstract interface into a univariate polynomial commitment
pub trait Commitment<F: Field>: Clone + fmt::Debug + PartialEq + Eq {

}

/// Represents a restriction of a bivariate polynomial
pub trait Restriction {
    type Dual: Restriction;
}

/// Restriction on `X`
pub struct X;
/// Restriction on `Y`
pub struct Y;

impl Restriction for X {
    type Dual = Y;
}
impl Restriction for Y {
    type Dual = X;
}

/// An accumulation scheme for claims on the partial evaluation of bivariate
/// polynomials.
///
/// Given a public bivariate polynomial $s(X, Y)$, we can commit to either $s(X,
/// y)$ for some value $y$ using commitment.
#[derive(Debug)]
pub struct BivariateAccumulation<'a, C: CurveAffine, C1: Commitment<C::Scalar>, C2: Commitment<C::Scalar>> {
    params: &'a Params<C>,
    domain: &'a EvaluationDomain<C::Scalar>,
    polys: Vec<Polynomial<C::Scalar, Coeff>>,
    _marker: PhantomData<(C1, C2)>
}

/// An accumulator for a 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BivariateAccumulator<C: CurveAffine, Comm: Commitment<C::Scalar>, R: Restriction> {
    commitment: Comm,
    restriction: C::Scalar,
    _marker: PhantomData<R>
}

/*
/// This provides an implementation of a bivariate polynomial evaluation proof
/// with an attached accumulation scheme.
#[derive(Debug, Clone)]
pub struct BivariateAccumulation<'a, C: CurveAffine> {
    params: &'a Params<C>,
    domain: &'a EvaluationDomain<C::Scalar>,
    polys: Vec<Polynomial<C::Scalar, Coeff>>,
}

/// A private accumulator that consists of a commitment to the restriction $s(x,
/// Y)$ of a public bivariate polynomial $s$ as well as the value of $x$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BivariateAccumulator<F: Field> {
    commitment: Vec<F>,
    y: F,
}

impl<'a, C: CurveAffine> BivariateAccumulation<'a, C> {
    /// Initializes this private accumulation context given parameters and an
    /// evaluation domain.
    pub fn new(
        params: &'a Params<C>,
        domain: &'a EvaluationDomain<C::Scalar>,
        polys: Vec<Polynomial<C::Scalar, Coeff>>,
    ) -> Self {
        BivariateAccumulation { params, domain, polys }
    }
}

impl<'a, C: CurveAffine> AccumulationScheme<C> for BivariateAccumulation<'a, C, U> {
    type Accumulator = BivariateAccumulator<C, U>;
    type Witness = ();

    fn blank(&self) -> (Self::Accumulator, Self::Witness) {
        todo!()
    }

    fn verify_combine<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[Self::Accumulator],
    ) -> io::Result<Self::Accumulator>
    where
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>,
    {
        todo!()
    }

    fn prove_combine<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[(Self::Accumulator, Self::Witness)],
        mut rng: impl RngCore,
    ) -> io::Result<(Self::Accumulator, Self::Witness)>
    where
        E: EncodedChallenge<C>,
        T: TranscriptWrite<C, E>,
    {
        todo!()
    }

    fn decide(&self, accumulator: &Self::Accumulator, witness: &Self::Witness) -> bool {
        todo!()
    }
}
*/