//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.
use crate::arithmetic::CurveAffine;

use std::io;
use std::marker::PhantomData;

pub(crate) mod blake2b;
pub(crate) mod poseidon;

pub use self::poseidon::{Challenge255 as PoseidonChallenge255, PoseidonRead, PoseidonWrite};
pub use blake2b::{Blake2bRead, Blake2bWrite, Challenge255 as Blake2bChallenge255};

/// Generic transcript view (from either the prover or verifier's perspective)
pub trait Transcript<C: CurveAffine, E: EncodedChallenge<C>> {
    /// Squeeze an encoded verifier challenge from the transcript.
    fn squeeze_challenge(&mut self) -> io::Result<E>;

    /// Squeeze a typed challenge (in the scalar field) from the transcript.
    fn squeeze_challenge_scalar<T>(&mut self) -> io::Result<ChallengeScalar<C, T>> {
        self.squeeze_challenge().map(|challenge| ChallengeScalar {
            inner: challenge.get_scalar(),
            _marker: PhantomData,
        })
    }

    /// Writing the point to the transcript without writing it to the proof,
    /// treating it as a common input.
    fn common_point(&mut self, point: C) -> io::Result<()>;

    /// Writing the scalar to the transcript without writing it to the proof,
    /// treating it as a common input.
    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// Transcript view from the perspective of a verifier that has access to an
/// input stream of data from the prover to the verifier.
pub trait TranscriptRead<C: CurveAffine, E: EncodedChallenge<C>>: Transcript<C, E> {
    /// Read a curve point from the prover.
    fn read_point(&mut self) -> io::Result<C>;

    /// Read a curve scalar from the prover.
    fn read_scalar(&mut self) -> io::Result<C::Scalar>;
}

/// Transcript view from the perspective of a prover that has access to an
/// output stream of messages from the prover to the verifier.
pub trait TranscriptWrite<C: CurveAffine, E: EncodedChallenge<C>>: Transcript<C, E> {
    /// Write a curve point to the proof and the transcript.
    fn write_point(&mut self, point: C) -> io::Result<()>;

    /// Write a scalar to the proof and the transcript.
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// The scalar representation of a verifier challenge.
///
/// The `Type` type can be used to scope the challenge to a specific context, or
/// set to `()` if no context is required.
#[derive(Copy, Clone, Debug)]
pub struct ChallengeScalar<C: CurveAffine, T> {
    inner: C::Scalar,
    _marker: PhantomData<T>,
}

impl<C: CurveAffine, T> std::ops::Deref for ChallengeScalar<C, T> {
    type Target = C::Scalar;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// `EncodedChallenge<C>` defines a challenge encoding with a [`Self::Input`]
/// that is used to derive the challenge encoding and `get_challenge` obtains
/// the _real_ `C::Scalar` that the challenge encoding represents.
pub trait EncodedChallenge<C: CurveAffine> {
    /// The Input type used to derive the challenge encoding. For example,
    /// an input from the Poseidon hash would be a base field element;
    /// an input from the Blake2b hash would be a [u8; 64].
    type Input;

    /// Get an encoded challenge from a given input challenge.
    fn new(challenge_input: &Self::Input) -> Self;

    /// Get a scalar field element from an encoded challenge.
    fn get_scalar(&self) -> C::Scalar;

    /// Cast an encoded challenge as a typed `ChallengeScalar`.
    fn as_challenge_scalar<T>(&self) -> ChallengeScalar<C, T> {
        ChallengeScalar {
            inner: self.get_scalar(),
            _marker: PhantomData,
        }
    }
}

pub(crate) fn read_n_points<C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    (0..n).map(|_| transcript.read_point()).collect()
}

pub(crate) fn read_n_scalars<C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    (0..n).map(|_| transcript.read_scalar()).collect()
}
