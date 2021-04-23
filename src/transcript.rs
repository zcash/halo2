//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use blake2b_simd::{Params as Blake2bParams, State as Blake2bState};
use ff::Field;
use std::convert::TryInto;

use crate::arithmetic::{Coordinates, CurveAffine, FieldExt};

use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// Generic transcript view (from either the prover or verifier's perspective)
pub trait Transcript<C: CurveAffine, S: ChallengeSpace<C>> {
    /// Squeeze a verifier challenge from the transcript. The length of the
    /// challenge is determined by the `ChallengeSpace`.
    fn squeeze_challenge(&mut self) -> S::Challenge;

    /// Squeeze a challenge (in the scalar field) from the transcript.
    fn squeeze_challenge_scalar<T>(&mut self) -> ChallengeScalar<C, T> {
        S::to_challenge_scalar(self.squeeze_challenge())
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
pub trait TranscriptRead<C: CurveAffine, S: ChallengeSpace<C>>: Transcript<C, S> {
    /// Read a curve point from the prover.
    fn read_point(&mut self) -> io::Result<C>;

    /// Read a curve scalar from the prover.
    fn read_scalar(&mut self) -> io::Result<C::Scalar>;
}

/// Transcript view from the perspective of a prover that has access to an
/// output stream of messages from the prover to the verifier.
pub trait TranscriptWrite<C: CurveAffine, S: ChallengeSpace<C>>: Transcript<C, S> {
    /// Write a curve point to the proof and the transcript.
    fn write_point(&mut self, point: C) -> io::Result<()>;

    /// Write a scalar to the proof and the transcript.
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bRead<R: Read, C: CurveAffine, S: ChallengeSpace<C>> {
    state: Blake2bState,
    reader: R,
    _marker: PhantomData<C>,
    _marker_s: PhantomData<S>,
}

impl<R: Read, C: CurveAffine, S: ChallengeSpace<C>> Blake2bRead<R, C, S> {
    /// Initialize a transcript given an input buffer and a key.
    pub fn init(reader: R) -> Self {
        Blake2bRead {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
                .to_state(),
            reader,
            _marker: PhantomData,
            _marker_s: PhantomData,
        }
    }
}

impl<R: Read, C: CurveAffine, S: ChallengeSpace<C>> TranscriptRead<C, S> for Blake2bRead<R, C, S> {
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

impl<R: Read, C: CurveAffine, S: ChallengeSpace<C>> Transcript<C, S> for Blake2bRead<R, C, S> {
    fn squeeze_challenge(&mut self) -> S::Challenge {
        let hasher = self.state.clone();
        let mut result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        result[S::NUM_BYTES - 1] &= S::BYTE_MASK;
        // self.state.update(&result[..S::NUM_BYTES]);
        self.state.update(&result[..]);
        S::Challenge::new(&result[..S::NUM_BYTES])
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
pub struct Blake2bWrite<W: Write, C: CurveAffine, S: ChallengeSpace<C>> {
    state: Blake2bState,
    writer: W,
    _marker: PhantomData<C>,
    _marker_s: PhantomData<S>,
}

impl<W: Write, C: CurveAffine, S: ChallengeSpace<C>> Blake2bWrite<W, C, S> {
    /// Initialize a transcript given an output buffer and a key.
    pub fn init(writer: W) -> Self {
        Blake2bWrite {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
                .to_state(),
            writer,
            _marker: PhantomData,
            _marker_s: PhantomData,
        }
    }

    /// Conclude the interaction and return the output buffer (writer).
    pub fn finalize(self) -> W {
        // TODO: handle outstanding scalars? see issue #138
        self.writer
    }
}

impl<W: Write, C: CurveAffine, S: ChallengeSpace<C>> TranscriptWrite<C, S>
    for Blake2bWrite<W, C, S>
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

impl<W: Write, C: CurveAffine, S: ChallengeSpace<C>> Transcript<C, S> for Blake2bWrite<W, C, S> {
    fn squeeze_challenge(&mut self) -> S::Challenge {
        let hasher = self.state.clone();
        let mut result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        result[S::NUM_BYTES - 1] &= S::BYTE_MASK;
        // self.state.update(&result[..S::NUM_BYTES]);
        self.state.update(&result[..]);
        S::Challenge::new(&result[..S::NUM_BYTES])
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

/// This is a 16-byte verifier challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge16(pub(crate) [u8; 16]);

impl Challenge for Challenge16 {
    fn new(challenge: &[u8]) -> Self {
        Self(challenge.try_into().unwrap())
    }
}

impl std::ops::Deref for Challenge16 {
    type Target = [u8; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This is a 64-byte verifier challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge64(pub(crate) [u8; 64]);

impl Challenge for Challenge64 {
    fn new(challenge: &[u8]) -> Self {
        Self(challenge.try_into().unwrap())
    }
}

impl std::ops::Deref for Challenge64 {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The scalar representation of a verifier challenge.
///
/// The `Type` type can be used to scope the challenge to a specific context, or
/// set to `Default` if no context is required.
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

/// The challenge space used to sample a scalar from a 512-bit challenge.
/// This protocol supports implementations for `ChallengeScalarEndo`, which
/// uses an endomorphism, and `ChallengeScalarFull`, which samples the
/// full-width field.
pub trait ChallengeSpace<C: CurveAffine>: Copy + Clone + std::fmt::Debug {
    /// TODO
    const NUM_BYTES: usize;
    /// TODO
    const BYTE_MASK: u8;
    /// TODO
    type Challenge: Challenge;

    /// Derive a scalar from a challenge in a certain challenge space.
    fn to_challenge_scalar<T>(challenge: Self::Challenge) -> ChallengeScalar<C, T>;
}

/// The scalar challenge space that applies the mapping of Algorithm 1 from the
/// [Halo](https://eprint.iacr.org/2019/1021) paper.
#[derive(Copy, Clone, Debug)]
pub struct ChallengeScalarEndo<C: CurveAffine> {
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> ChallengeSpace<C> for ChallengeScalarEndo<C> {
    const NUM_BYTES: usize = 16;
    const BYTE_MASK: u8 = 0b11111111;
    type Challenge = Challenge16;

    fn to_challenge_scalar<T>(challenge: Self::Challenge) -> ChallengeScalar<C, T> {
        let mut acc = (C::Scalar::ZETA + &C::Scalar::one()).double();

        let challenge: u128 = u128::from_le_bytes(challenge.0);

        for i in (0..64).rev() {
            let should_negate = ((challenge >> ((i << 1) + 1)) & 1) == 1;
            let should_endo = ((challenge >> (i << 1)) & 1) == 1;

            let q = if should_negate {
                -C::Scalar::one()
            } else {
                C::Scalar::one()
            };
            let q = if should_endo { q * &C::Scalar::ZETA } else { q };
            acc = acc + &q + &acc;
        }

        ChallengeScalar {
            inner: acc,
            _marker: PhantomData,
        }
    }
}

/// The scalar challenge space that samples from the full-width field.
#[derive(Copy, Clone, Debug)]
pub struct ChallengeScalarFull<C: CurveAffine> {
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> ChallengeSpace<C> for ChallengeScalarFull<C> {
    const NUM_BYTES: usize = 64;
    const BYTE_MASK: u8 = 0b111111;
    type Challenge = Challenge64;

    fn to_challenge_scalar<T>(challenge: Self::Challenge) -> ChallengeScalar<C, T> {
        ChallengeScalar {
            inner: C::Scalar::from_bytes_wide(&challenge.0),
            _marker: PhantomData,
        }
    }
}

pub(crate) fn read_n_points<C: CurveAffine, S: ChallengeSpace<C>, T: TranscriptRead<C, S>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    (0..n).map(|_| transcript.read_point()).collect()
}

pub(crate) fn read_n_scalars<C: CurveAffine, S: ChallengeSpace<C>, T: TranscriptRead<C, S>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    (0..n).map(|_| transcript.read_scalar()).collect()
}
