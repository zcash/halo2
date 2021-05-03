//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use blake2b_simd::{Params as Blake2bParams, State as Blake2bState};
use ff::Field;
use std::convert::TryInto;

use crate::arithmetic::{Coordinates, CurveAffine, FieldExt};

use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// Generic transcript view (from either the prover or verifier's perspective)
pub trait Transcript<C: CurveAffine, I, E: EncodedChallenge<C, I>> {
    /// Squeeze an encoded verifier challenge from the transcript.
    fn squeeze_challenge(&mut self) -> E;

    /// Squeeze a typed challenge (in the scalar field) from the transcript.
    fn squeeze_challenge_scalar<T>(&mut self) -> ChallengeScalar<C, T> {
        ChallengeScalar {
            inner: self.squeeze_challenge().get_scalar(),
            _marker: PhantomData,
        }
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
pub trait TranscriptRead<C: CurveAffine, I, E: EncodedChallenge<C, I>>:
    Transcript<C, I, E>
{
    /// Read a curve point from the prover.
    fn read_point(&mut self) -> io::Result<C>;

    /// Read a curve scalar from the prover.
    fn read_scalar(&mut self) -> io::Result<C::Scalar>;
}

/// Transcript view from the perspective of a prover that has access to an
/// output stream of messages from the prover to the verifier.
pub trait TranscriptWrite<C: CurveAffine, I, E: EncodedChallenge<C, I>>:
    Transcript<C, I, E>
{
    /// Write a curve point to the proof and the transcript.
    fn write_point(&mut self, point: C) -> io::Result<()>;

    /// Write a scalar to the proof and the transcript.
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bRead<R: Read, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> {
    state: Blake2bState,
    reader: R,
    _marker_c: PhantomData<C>,
    _marker_e: PhantomData<E>,
}

impl<R: Read, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> Blake2bRead<R, C, E> {
    /// Initialize a transcript given an input buffer and a key.
    pub fn init(reader: R) -> Self {
        Blake2bRead {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
                .to_state(),
            reader,
            _marker_c: PhantomData,
            _marker_e: PhantomData,
        }
    }
}

impl<R: Read, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> TranscriptRead<C, [u8; 64], E>
    for Blake2bRead<R, C, E>
{
    fn read_point(&mut self) -> io::Result<C> {
        let mut compressed = C::Repr::default();
        self.reader.read_exact(compressed.as_mut())?;
        let point: C = Option::from(C::from_bytes(&compressed)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "invalid point encoding in proof")
        })?;
        self.common_point(point)?;

        Ok(point)
    }

    fn read_scalar(&mut self) -> io::Result<C::Scalar> {
        let mut data = [0u8; 32];
        self.reader.read_exact(&mut data)?;
        let scalar: C::Scalar = Option::from(C::Scalar::from_bytes(&data)).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "invalid field element encoding in proof",
            )
        })?;
        self.common_scalar(scalar)?;

        Ok(scalar)
    }
}

impl<R: Read, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> Transcript<C, [u8; 64], E>
    for Blake2bRead<R, C, E>
{
    fn squeeze_challenge(&mut self) -> E {
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        self.state.update(&result[..]);
        E::new(&result)
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        let coords: Coordinates<C> = Option::from(point.coordinates()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.state.update(&coords.x().to_bytes());
        self.state.update(&coords.y().to_bytes());

        Ok(())
    }

    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.state.update(&scalar.to_bytes());

        Ok(())
    }
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bWrite<W: Write, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> {
    state: Blake2bState,
    writer: W,
    _marker_c: PhantomData<C>,
    _marker_e: PhantomData<E>,
}

impl<W: Write, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> Blake2bWrite<W, C, E> {
    /// Initialize a transcript given an output buffer and a key.
    pub fn init(writer: W) -> Self {
        Blake2bWrite {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
                .to_state(),
            writer,
            _marker_c: PhantomData,
            _marker_e: PhantomData,
        }
    }

    /// Conclude the interaction and return the output buffer (writer).
    pub fn finalize(self) -> W {
        // TODO: handle outstanding scalars? see issue #138
        self.writer
    }
}

impl<W: Write, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> TranscriptWrite<C, [u8; 64], E>
    for Blake2bWrite<W, C, E>
{
    fn write_point(&mut self, point: C) -> io::Result<()> {
        self.common_point(point)?;
        let compressed = point.to_bytes();
        self.writer.write_all(compressed.as_ref())
    }
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.common_scalar(scalar)?;
        let data = scalar.to_bytes();
        self.writer.write_all(&data[..])
    }
}

impl<W: Write, C: CurveAffine, E: EncodedChallenge<C, [u8; 64]>> Transcript<C, [u8; 64], E>
    for Blake2bWrite<W, C, E>
{
    fn squeeze_challenge(&mut self) -> E {
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        self.state.update(&result[..]);
        E::new(&result)
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        let coords: Coordinates<C> = Option::from(point.coordinates()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.state.update(&coords.x().to_bytes());
        self.state.update(&coords.y().to_bytes());

        Ok(())
    }

    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.state.update(&scalar.to_bytes());

        Ok(())
    }
}

/// `Challenge` trait implemented for challenges of different lengths
pub trait Challenge: Copy + Clone + std::fmt::Debug {
    /// Try to create challenge of appropriate length.
    fn new(challenge: &[u8]) -> Self;
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

/// `EncodedChallenge<C, I>` defines a challenge encoding where `I` is the input
/// that is used to derive the challenge encoding and `get_challenge` obtains
/// the _real_ `C::Scalar` that the challenge encoding represents.
pub trait EncodedChallenge<C: CurveAffine, I> {
    /// Get an encoded challenge from a given input challenge.
    fn new(challenge_input: &I) -> Self;

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

/// A 128-bit challenge. Note that using this challenge space may result
/// in less than 128-bit security.
#[derive(Copy, Clone, Debug)]
pub struct Challenge128(u128);

impl std::ops::Deref for Challenge128 {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: CurveAffine> EncodedChallenge<C, C::Base> for Challenge128 {
    fn new(challenge_input: &C::Base) -> Self {
        Challenge128(challenge_input.get_lower_128())
    }

    // This applies the mapping of Algorithm 1 from the [Halo](https://eprint.iacr.org/2019/1021) paper.
    fn get_scalar(&self) -> C::Scalar {
        let mut acc = (C::Scalar::ZETA + &C::Scalar::one()).double();

        for i in (0..64).rev() {
            let should_negate = ((self.0 >> ((i << 1) + 1)) & 1) == 1;
            let should_endo = ((self.0 >> (i << 1)) & 1) == 1;

            let q = if should_negate {
                -C::Scalar::one()
            } else {
                C::Scalar::one()
            };
            let q = if should_endo { q * &C::Scalar::ZETA } else { q };
            acc = acc + &q + &acc;
        }

        acc
    }
}

/// A 255-bit challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge255([u8; 32]);

impl std::ops::Deref for Challenge255 {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: CurveAffine> EncodedChallenge<C, [u8; 64]> for Challenge255 {
    fn new(challenge_input: &[u8; 64]) -> Self {
        Challenge255(C::Scalar::from_bytes_wide(challenge_input).to_bytes())
    }
    fn get_scalar(&self) -> C::Scalar {
        C::Scalar::from_bytes(&self.0).unwrap()
    }
}

pub(crate) fn read_n_points<
    C: CurveAffine,
    I,
    E: EncodedChallenge<C, I>,
    T: TranscriptRead<C, I, E>,
>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    (0..n).map(|_| transcript.read_point()).collect()
}

pub(crate) fn read_n_scalars<
    C: CurveAffine,
    I,
    E: EncodedChallenge<C, I>,
    T: TranscriptRead<C, I, E>,
>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    (0..n).map(|_| transcript.read_scalar()).collect()
}
