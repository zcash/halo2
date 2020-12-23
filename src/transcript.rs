//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use ff::Field;
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
pub trait TranscriptRead<R: Read, C: CurveAffine>: Transcript<C> {
    /// Initialize the transcript with a key and an input stream.
    fn init(reader: R, key: C::Base) -> Self;

    /// Read a curve point from the prover.
    fn read_point(&mut self) -> io::Result<C>;

    /// Read a curve scalar from the prover.
    fn read_scalar(&mut self) -> io::Result<C::Scalar>;
}

/// Transcript view from the perspective of a prover that has access to an
/// output stream of messages from the prover to the verifier.
pub trait TranscriptWrite<W: Write, C: CurveAffine>: Transcript<C> {
    /// Forked transcript that does not write to the proof structure.
    type ForkedTranscript: TranscriptWrite<io::Sink, C>;

    /// Initialize the transcript with a key and an output stream.
    fn init(writer: W, key: C::Base) -> Self;

    /// Write a curve point to the proof and the transcript.
    fn write_point(&mut self, point: C) -> io::Result<()>;

    /// Write a scalar to the proof and the transcript.
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;

    /// Fork the transcript, creating a variant of this `TranscriptWrite` which
    /// does not output anything to the writer.
    fn fork(&self) -> Self::ForkedTranscript;

    /// Return the writer to conclude the interaction and take possession of the
    /// proof.
    fn finalize(self) -> W;
}

/// This is just a simple (and completely broken) transcript reader
/// implementation, standing in for some algebraic hash function that we'll
/// switch to later.
#[derive(Debug, Clone)]
pub struct DummyHashRead<R: Read, C: CurveAffine> {
    base_state: C::Base,
    scalar_state: C::Scalar,
    read_scalar: bool,
    reader: R,
}

impl<R: Read, C: CurveAffine> TranscriptRead<R, C> for DummyHashRead<R, C> {
    fn init(reader: R, key: C::Base) -> Self {
        DummyHashRead {
            base_state: key + &C::Base::from_u64(1013),
            scalar_state: C::Scalar::from_u64(1013),
            read_scalar: false,
            reader,
        }
    }

    fn read_point(&mut self) -> io::Result<C> {
        let mut compressed = [0u8; 32];
        self.reader.read_exact(&mut compressed[..])?;
        let point: C = Option::from(C::from_bytes(&compressed)).ok_or(io::Error::new(
            io::ErrorKind::Other,
            "invalid point encoding in proof",
        ))?;
        self.common_point(point)?;

        Ok(point)
    }

    fn read_scalar(&mut self) -> io::Result<C::Scalar> {
        let mut data = [0u8; 32];
        self.reader.read_exact(&mut data)?;
        let scalar = Option::from(C::Scalar::from_bytes(&data)).ok_or(io::Error::new(
            io::ErrorKind::Other,
            "invalid field element encoding in proof",
        ))?;
        self.scalar_state += &(scalar * &C::Scalar::ZETA);
        self.scalar_state = self.scalar_state.square();
        self.read_scalar = true;

        Ok(scalar)
    }
}

impl<R: Read, C: CurveAffine> Transcript<C> for DummyHashRead<R, C> {
    fn common_point(&mut self, point: C) -> io::Result<()> {
        let (x, y) = Option::from(point.get_xy()).ok_or(io::Error::new(
            io::ErrorKind::Other,
            "cannot write points at infinity to the transcript",
        ))?;
        self.base_state += &(x * &C::Base::ZETA);
        self.base_state = self.base_state.square();
        self.base_state += &(y * &C::Base::ZETA);
        self.base_state = self.base_state.square();

        Ok(())
    }

    fn squeeze_challenge(&mut self) -> C::Base {
        if self.read_scalar {
            let x = C::Base::from_bytes(&self.scalar_state.to_bytes()).unwrap();
            self.base_state += &(x * &C::Base::ZETA);
            self.base_state = self.base_state.square();
            self.scalar_state = self.scalar_state.square();
            self.read_scalar = false;
        }

        let tmp = self.base_state;
        for _ in 0..5 {
            self.base_state *= &(C::Base::ZETA + &C::Base::ZETA);
            self.base_state += &C::Base::ZETA;
            self.base_state = self.base_state.square();
        }

        tmp
    }
}

/// This is just a simple (and completely broken) transcript writer
/// implementation, standing in for some algebraic hash function that we'll
/// switch to later.
#[derive(Debug, Clone)]
pub struct DummyHashWrite<W: Write, C: CurveAffine> {
    base_state: C::Base,
    scalar_state: C::Scalar,
    written_scalar: bool,
    writer: W,
}

impl<W: Write, C: CurveAffine> TranscriptWrite<W, C> for DummyHashWrite<W, C> {
    type ForkedTranscript = DummyHashWrite<io::Sink, C>;

    fn init(writer: W, key: C::Base) -> Self {
        DummyHashWrite {
            base_state: key + &C::Base::from_u64(1013),
            scalar_state: C::Scalar::from_u64(1013),
            written_scalar: false,
            writer,
        }
    }
    fn write_point(&mut self, point: C) -> io::Result<()> {
        self.common_point(point)?;
        let compressed = point.to_bytes();
        self.writer.write_all(&compressed[..])
    }
    fn write_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.scalar_state += &(scalar * &C::Scalar::ZETA);
        self.scalar_state = self.scalar_state.square();
        self.written_scalar = true;
        let data = scalar.to_bytes();
        self.writer.write_all(&data[..])
    }
    fn fork(&self) -> Self::ForkedTranscript {
        DummyHashWrite {
            base_state: self.base_state,
            scalar_state: self.scalar_state,
            written_scalar: self.written_scalar,
            writer: io::sink(),
        }
    }
    fn finalize(self) -> W {
        // TODO: handle outstanding scalars?
        self.writer
    }
}

impl<W: Write, C: CurveAffine> Transcript<C> for DummyHashWrite<W, C> {
    fn common_point(&mut self, point: C) -> io::Result<()> {
        let (x, y) = Option::from(point.get_xy()).ok_or(io::Error::new(
            io::ErrorKind::Other,
            "cannot write points at infinity to the transcript",
        ))?;
        self.base_state += &(x * &C::Base::ZETA);
        self.base_state = self.base_state.square();
        self.base_state += &(y * &C::Base::ZETA);
        self.base_state = self.base_state.square();

        Ok(())
    }

    fn squeeze_challenge(&mut self) -> C::Base {
        if self.written_scalar {
            let x = C::Base::from_bytes(&self.scalar_state.to_bytes()).unwrap();
            self.base_state += &(x * &C::Base::ZETA);
            self.base_state = self.base_state.square();
            self.scalar_state = self.scalar_state.square();
            self.written_scalar = false;
        }

        let tmp = self.base_state;
        for _ in 0..5 {
            self.base_state *= &(C::Base::ZETA + &C::Base::ZETA);
            self.base_state += &C::Base::ZETA;
            self.base_state = self.base_state.square();
        }

        tmp
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

pub(crate) fn read_n_points<C: CurveAffine, R: Read, T: TranscriptRead<R, C>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(transcript.read_point()?);
    }
    Ok(v)
}

pub(crate) fn read_n_scalars<C: CurveAffine, R: Read, T: TranscriptRead<R, C>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(transcript.read_scalar()?);
    }
    Ok(v)
}
