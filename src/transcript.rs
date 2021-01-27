//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use blake2b_simd::{Params as Blake2bParams, State as Blake2bState};
use ff::Field;
use std::convert::TryInto;
use std::ops::Deref;

use crate::arithmetic::{CurveAffine, FieldExt};

use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// Generic transcript view (from either the prover or verifier's perspective)
pub trait Transcript<C: CurveAffine> {
    /// Squeeze a challenge (in the base field) from the transcript.
    fn squeeze_challenge(&mut self) -> C::Base;

    /// Writing the point to the transcript without writing it to the proof,
    /// treating it as a common input.
    fn common_point(&mut self, point: C) -> io::Result<()>;
}

/// Transcript view from the perspective of a verifier that has access to an
/// input stream of data from the prover to the verifier.
pub trait TranscriptRead<C: CurveAffine>: Transcript<C> {
    /// Read a curve point from the prover.
    fn read_point(&mut self) -> io::Result<C>;

    /// Read a curve scalar from the prover.
    fn read_scalar(&mut self) -> io::Result<C::Scalar>;
}

/// Transcript view from the perspective of a prover that has access to an
/// output stream of messages from the prover to the verifier.
pub trait TranscriptWrite<C: CurveAffine>: Transcript<C> {
    /// Write a curve point to the proof and the transcript.
    fn write_point(&mut self, point: C) -> io::Result<()>;

    /// Write a scalar to the proof and the transcript.
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bRead<R: Read, C: CurveAffine> {
    state: Blake2bState,
    reader: R,
    _marker: PhantomData<C>,
}

impl<R: Read, C: CurveAffine> Blake2bRead<R, C> {
    /// Initialize a transcript given an input buffer and a key.
    pub fn init(reader: R) -> Self {
        Blake2bRead {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(C::BLAKE2B_PERSONALIZATION)
                .to_state(),
            reader,
            _marker: PhantomData,
        }
    }
}

impl<R: Read, C: CurveAffine> TranscriptRead<C> for Blake2bRead<R, C> {
    fn read_point(&mut self) -> io::Result<C> {
        let mut compressed = [0u8; 32];
        self.reader.read_exact(&mut compressed[..])?;
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
        self.state.update(&scalar.to_bytes());

        Ok(scalar)
    }
}

impl<R: Read, C: CurveAffine> Transcript<C> for Blake2bRead<R, C> {
    fn common_point(&mut self, point: C) -> io::Result<()> {
        let (x, y) = Option::from(point.get_xy()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.state.update(&x.to_bytes());
        self.state.update(&y.to_bytes());

        Ok(())
    }

    fn squeeze_challenge(&mut self) -> C::Base {
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        self.state.update(&result[..]);
        C::Base::from_bytes_wide(&result)
    }
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bWrite<W: Write, C: CurveAffine> {
    state: Blake2bState,
    writer: W,
    _marker: PhantomData<C>,
}

impl<W: Write, C: CurveAffine> Blake2bWrite<W, C> {
    /// Initialize a transcript given an output buffer and a key.
    pub fn init(writer: W) -> Self {
        Blake2bWrite {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(C::BLAKE2B_PERSONALIZATION)
                .to_state(),
            writer,
            _marker: PhantomData,
        }
    }

    /// Conclude the interaction and return the output buffer (writer).
    pub fn finalize(self) -> W {
        // TODO: handle outstanding scalars? see issue #138
        self.writer
    }
}

impl<W: Write, C: CurveAffine> TranscriptWrite<C> for Blake2bWrite<W, C> {
    fn write_point(&mut self, point: C) -> io::Result<()> {
        self.common_point(point)?;
        let compressed = point.to_bytes();
        self.writer.write_all(&compressed[..])
    }
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.state.update(&scalar.to_bytes());
        let data = scalar.to_bytes();
        self.writer.write_all(&data[..])
    }
}

impl<W: Write, C: CurveAffine> Transcript<C> for Blake2bWrite<W, C> {
    fn common_point(&mut self, point: C) -> io::Result<()> {
        let (x, y) = Option::from(point.get_xy()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.state.update(&x.to_bytes());
        self.state.update(&y.to_bytes());

        Ok(())
    }

    fn squeeze_challenge(&mut self) -> C::Base {
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        self.state.update(&result[..]);
        C::Base::from_bytes_wide(&result)
    }
}

/// This is a 128-bit verifier challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge(pub(crate) u128);

impl Challenge {
    /// Obtains a new challenge from the transcript.
    pub fn get<C: CurveAffine, T: Transcript<C>>(transcript: &mut T) -> Challenge {
        Challenge(transcript.squeeze_challenge().get_lower_128())
    }
}

/// The scalar representation of a verifier challenge.
///
/// The `Type` type can be used to scope the challenge to a specific context, or
/// set to `()` if no context is required.
#[derive(Copy, Clone, Debug)]
pub struct ChallengeScalar<C: CurveAffine, Type> {
    inner: C::Scalar,
    _marker: PhantomData<Type>,
}

impl<C: CurveAffine, Type> From<Challenge> for ChallengeScalar<C, Type> {
    /// This algorithm applies the mapping of Algorithm 1 from the
    /// [Halo](https://eprint.iacr.org/2019/1021) paper.
    fn from(challenge: Challenge) -> Self {
        let mut acc = (C::Scalar::ZETA + &C::Scalar::one()).double();

        for i in (0..64).rev() {
            let should_negate = ((challenge.0 >> ((i << 1) + 1)) & 1) == 1;
            let should_endo = ((challenge.0 >> (i << 1)) & 1) == 1;

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

impl<C: CurveAffine, Type> ChallengeScalar<C, Type> {
    /// Obtains a new challenge from the transcript.
    pub fn get<T: Transcript<C>>(transcript: &mut T) -> Self
    where
        C: CurveAffine,
    {
        Challenge::get(transcript).into()
    }
}

impl<C: CurveAffine, Type> Deref for ChallengeScalar<C, Type> {
    type Target = C::Scalar;

    fn deref(&self) -> &C::Scalar {
        &self.inner
    }
}

pub(crate) fn read_n_points<C: CurveAffine, T: TranscriptRead<C>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    (0..n).map(|_| transcript.read_point()).collect()
}

pub(crate) fn read_n_scalars<C: CurveAffine, T: TranscriptRead<C>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    (0..n).map(|_| transcript.read_scalar()).collect()
}
