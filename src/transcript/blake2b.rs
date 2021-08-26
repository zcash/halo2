use super::{EncodedChallenge, Transcript, TranscriptRead, TranscriptWrite};
use blake2b_simd::{Params as Blake2bParams, State as Blake2bState};

use crate::arithmetic::{Coordinates, CurveAffine, FieldExt};

use std::convert::TryInto;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// Prefix to a prover's message soliciting a challenge
const BLAKE2B_PREFIX_CHALLENGE: u8 = 0;

/// Prefix to a prover's message containing a curve point
const BLAKE2B_PREFIX_POINT: u8 = 1;

/// Prefix to a prover's message containing a scalar
const BLAKE2B_PREFIX_SCALAR: u8 = 2;

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bRead<R: Read, C: CurveAffine, E: EncodedChallenge<C>> {
    state: Blake2bState,
    reader: R,
    _marker: PhantomData<(C, E)>,
}

impl<R: Read, C: CurveAffine, E: EncodedChallenge<C>> Blake2bRead<R, C, E> {
    /// Initialize a transcript given an input buffer.
    pub fn init(reader: R) -> Self {
        Blake2bRead {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
                .to_state(),
            reader,
            _marker: PhantomData,
        }
    }
}

impl<R: Read, C: CurveAffine> TranscriptRead<C, Challenge255<C>>
    for Blake2bRead<R, C, Challenge255<C>>
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

impl<R: Read, C: CurveAffine> Transcript<C, Challenge255<C>>
    for Blake2bRead<R, C, Challenge255<C>>
{
    fn squeeze_challenge(&mut self) -> io::Result<Challenge255<C>> {
        self.state.update(&[BLAKE2B_PREFIX_CHALLENGE]);
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        Ok(Challenge255::<C>::new(&result))
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        self.state.update(&[BLAKE2B_PREFIX_POINT]);
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
        self.state.update(&[BLAKE2B_PREFIX_SCALAR]);
        self.state.update(&scalar.to_bytes());

        Ok(())
    }
}

/// We will replace BLAKE2b with an algebraic hash function in a later version.
#[derive(Debug, Clone)]
pub struct Blake2bWrite<W: Write, C: CurveAffine, E: EncodedChallenge<C>> {
    state: Blake2bState,
    writer: W,
    _marker: PhantomData<(C, E)>,
}

impl<W: Write, C: CurveAffine, E: EncodedChallenge<C>> Blake2bWrite<W, C, E> {
    /// Initialize a transcript given an output buffer.
    pub fn init(writer: W) -> Self {
        Blake2bWrite {
            state: Blake2bParams::new()
                .hash_length(64)
                .personal(b"Halo2-Transcript")
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

impl<W: Write, C: CurveAffine> TranscriptWrite<C, Challenge255<C>>
    for Blake2bWrite<W, C, Challenge255<C>>
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

impl<W: Write, C: CurveAffine> Transcript<C, Challenge255<C>>
    for Blake2bWrite<W, C, Challenge255<C>>
{
    fn squeeze_challenge(&mut self) -> io::Result<Challenge255<C>> {
        self.state.update(&[BLAKE2B_PREFIX_CHALLENGE]);
        let hasher = self.state.clone();
        let result: [u8; 64] = hasher.finalize().as_bytes().try_into().unwrap();
        Ok(Challenge255::<C>::new(&result))
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        self.state.update(&[BLAKE2B_PREFIX_POINT]);
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
        self.state.update(&[BLAKE2B_PREFIX_SCALAR]);
        self.state.update(&scalar.to_bytes());

        Ok(())
    }
}

/// A 255-bit challenge.
#[derive(Copy, Clone, Debug)]
pub struct Challenge255<C: CurveAffine>([u8; 32], PhantomData<C>);

impl<C: CurveAffine> std::ops::Deref for Challenge255<C> {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: CurveAffine> EncodedChallenge<C> for Challenge255<C> {
    type Input = [u8; 64];

    fn new(challenge_input: &[u8; 64]) -> Self {
        Challenge255(
            C::Scalar::from_bytes_wide(challenge_input).to_bytes(),
            PhantomData,
        )
    }
    fn get_scalar(&self) -> C::Scalar {
        C::Scalar::from_bytes(&self.0).unwrap()
    }
}
