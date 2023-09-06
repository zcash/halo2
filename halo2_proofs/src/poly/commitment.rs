use super::{
    query::{ProverQuery, VerifierQuery},
    strategy::Guard,
    Coeff, LagrangeCoeff, Polynomial,
};
use crate::poly::Error;
use crate::transcript::{EncodedChallenge, TranscriptRead, TranscriptWrite};
use ff::Field;
use halo2curves::CurveAffine;
use rand_core::RngCore;
use std::{
    fmt::Debug,
    io::{self},
    ops::{Add, AddAssign, Mul, MulAssign},
};

/// Defines components of a commitment scheme.
pub trait CommitmentScheme {
    /// Application field of this commitment scheme
    type Scalar: Field;

    /// Elliptic curve used to commit the application and witnesses
    type Curve: CurveAffine<ScalarExt = Self::Scalar>;

    /// Constant prover parameters
    type ParamsProver: for<'params> ParamsProver<
        'params,
        Self::Curve,
        ParamsVerifier = Self::ParamsVerifier,
    >;

    /// Constant verifier parameters
    type ParamsVerifier: for<'params> ParamsVerifier<'params, Self::Curve>;

    /// Wrapper for parameter generator
    fn new_params(k: u32) -> Self::ParamsProver;

    /// Wrapper for parameter reader
    fn read_params<R: io::Read>(reader: &mut R) -> io::Result<Self::ParamsProver>;
}

/// Parameters for circuit sysnthesis and prover parameters.
pub trait Params<'params, C: CurveAffine>: Sized + Clone {
    /// Multi scalar multiplication engine
    type MSM: MSM<C> + 'params;

    /// Logaritmic size of the circuit
    fn k(&self) -> u32;

    /// Size of the circuit
    fn n(&self) -> u64;

    /// Downsize `Params` with smaller `k`.
    fn downsize(&mut self, k: u32);

    /// Generates an empty multiscalar multiplication struct using the
    /// appropriate params.
    fn empty_msm(&'params self) -> Self::MSM;

    /// This commits to a polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment will be blinded by the blinding factor
    /// `r`.
    fn commit_lagrange(
        &self,
        poly: &Polynomial<C::ScalarExt, LagrangeCoeff>,
        r: Blind<C::ScalarExt>,
    ) -> C::CurveExt;

    /// Writes params to a buffer.
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;

    /// Reads params from a buffer.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self>;
}

/// Parameters for circuit sysnthesis and prover parameters.
pub trait ParamsProver<'params, C: CurveAffine>: Params<'params, C> {
    /// Constant verifier parameters.
    type ParamsVerifier: ParamsVerifier<'params, C>;

    /// Returns new instance of parameters
    fn new(k: u32) -> Self;

    /// This computes a commitment to a polynomial described by the provided
    /// slice of coefficients. The commitment may be blinded by the blinding
    /// factor `r`.
    fn commit(&self, poly: &Polynomial<C::ScalarExt, Coeff>, r: Blind<C::ScalarExt>)
        -> C::CurveExt;

    /// Getter for g generators
    fn get_g(&self) -> &[C];

    /// Returns verification parameters.
    fn verifier_params(&'params self) -> &'params Self::ParamsVerifier;
}

/// Verifier specific functionality with circuit constaints
pub trait ParamsVerifier<'params, C: CurveAffine>: Params<'params, C> {}

/// Multi scalar multiplication engine
pub trait MSM<C: CurveAffine>: Clone + Debug + Send + Sync {
    /// Add arbitrary term (the scalar and the point)
    fn append_term(&mut self, scalar: C::Scalar, point: C::CurveExt);

    /// Add another multiexp into this one
    fn add_msm(&mut self, other: &Self)
    where
        Self: Sized;

    /// Scale all scalars in the MSM by some scaling factor
    fn scale(&mut self, factor: C::Scalar);

    /// Perform multiexp and check that it results in zero
    fn check(&self) -> bool;

    /// Perform multiexp and return the result
    fn eval(&self) -> C::CurveExt;

    /// Return base points
    fn bases(&self) -> Vec<C::CurveExt>;

    /// Scalars
    fn scalars(&self) -> Vec<C::Scalar>;
}

/// Common multi-open prover interface for various commitment schemes
pub trait Prover<'params, Scheme: CommitmentScheme> {
    /// Query instance or not
    const QUERY_INSTANCE: bool;

    /// Creates new prover instance
    fn new(params: &'params Scheme::ParamsProver) -> Self;

    /// Create a multi-opening proof
    fn create_proof<
        'com,
        E: EncodedChallenge<Scheme::Curve>,
        T: TranscriptWrite<Scheme::Curve, E>,
        R,
        I,
    >(
        &self,
        rng: R,
        transcript: &mut T,
        queries: I,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = ProverQuery<'com, Scheme::Curve>> + Clone,
        R: RngCore;
}

/// Common multi-open verifier interface for various commitment schemes
pub trait Verifier<'params, Scheme: CommitmentScheme> {
    /// Unfinalized verification result. This is returned in verification
    /// to allow developer to compress or combined verification results
    type Guard: Guard<Scheme, MSMAccumulator = Self::MSMAccumulator>;

    /// Accumulator fot comressed verification
    type MSMAccumulator;

    /// Query instance or not
    const QUERY_INSTANCE: bool;

    /// Creates new verifier instance
    fn new(params: &'params Scheme::ParamsVerifier) -> Self;

    /// Process the proof and returns unfinished result named `Guard`
    fn verify_proof<
        'com,
        E: EncodedChallenge<Scheme::Curve>,
        T: TranscriptRead<Scheme::Curve, E>,
        I,
    >(
        &self,
        transcript: &mut T,
        queries: I,
        msm: Self::MSMAccumulator,
    ) -> Result<Self::Guard, Error>
    where
        'params: 'com,
        I: IntoIterator<
                Item = VerifierQuery<
                    'com,
                    Scheme::Curve,
                    <Scheme::ParamsVerifier as Params<'params, Scheme::Curve>>::MSM,
                >,
            > + Clone;
}

/// Wrapper type around a blinding factor.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Blind<F>(pub F);

impl<F: Field> Default for Blind<F> {
    fn default() -> Self {
        Blind(F::ONE)
    }
}

impl<F: Field> Blind<F> {
    /// Given `rng` creates new blinding scalar
    pub fn new<R: RngCore>(rng: &mut R) -> Self {
        Blind(F::random(rng))
    }
}

impl<F: Field> Add for Blind<F> {
    type Output = Self;

    fn add(self, rhs: Blind<F>) -> Self {
        Blind(self.0 + rhs.0)
    }
}

impl<F: Field> Mul for Blind<F> {
    type Output = Self;

    fn mul(self, rhs: Blind<F>) -> Self {
        Blind(self.0 * rhs.0)
    }
}

impl<F: Field> AddAssign for Blind<F> {
    fn add_assign(&mut self, rhs: Blind<F>) {
        self.0 += rhs.0;
    }
}

impl<F: Field> MulAssign for Blind<F> {
    fn mul_assign(&mut self, rhs: Blind<F>) {
        self.0 *= rhs.0;
    }
}

impl<F: Field> AddAssign<F> for Blind<F> {
    fn add_assign(&mut self, rhs: F) {
        self.0 += rhs;
    }
}

impl<F: Field> MulAssign<F> for Blind<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.0 *= rhs;
    }
}
