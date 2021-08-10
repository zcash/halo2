//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.
use crate::poseidon::{ConstantLength, Domain, Duplex as PoseidonState, Spec};

use group::Curve;
use std::convert::TryInto;

use crate::arithmetic::{Coordinates, CurveAffine, FieldExt};

use std::io::{self, Read, Write};
use std::marker::PhantomData;

/// Poseidon width
const T: usize = 3;

/// Poseidon rate
const RATE: usize = 2;

/// Prefix to a prover's message soliciting a challenge
const POSEIDON_PREFIX_CHALLENGE: u64 = 0;

/// Prefix to a prover's message containing a curve point
const POSEIDON_PREFIX_POINT: u64 = 1;

/// Prefix to a prover's message containing a scalar
const POSEIDON_PREFIX_SCALAR: u64 = 2;

/// Generic transcript view (from either the prover or verifier's perspective)
pub trait Transcript<C: CurveAffine, E: EncodedChallenge<C>> {
    /// Squeeze an encoded verifier challenge from the transcript.
    fn squeeze_challenge(&mut self) -> io::Result<E>;

    /// Squeeze a typed challenge (in the scalar field) from the transcript.
    fn squeeze_challenge_scalar<T>(&mut self) -> io::Result<ChallengeScalar<C, T>> {
        // Get scalar
        self.squeeze_challenge().map(|challenge| ChallengeScalar {
            inner: challenge.get_scalar(),
            _marker: PhantomData,
        })
    }

    /// Writing the point to the base transcript without writing it to the proof,
    /// treating it as a common input.
    fn common_point(&mut self, point: C) -> io::Result<()>;

    /// Writing the point to the scalar transcript without writing it to the proof,
    /// treating it as a common input.
    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()>;
}

/// Transcript view from the perspective of a verifier that has access to an
// input stream of data from the prover to the verifier.
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

/// TODO
#[derive(Debug)]
pub struct PoseidonRead<
    R: Read,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    BaseSpec: Spec<C::Base, T, RATE>,
    ScalarSpec: Spec<C::Scalar, T, RATE>,
> {
    base_state: PoseidonState<C::Base, BaseSpec, T, RATE>,
    scalar_state: PoseidonState<C::Scalar, ScalarSpec, T, RATE>,
    squeezable: bool,
    commitment_base: C,
    reader: R,
    _marker: PhantomData<E>,
}

impl<
        R: Read,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > PoseidonRead<R, C, E, BaseSpec, ScalarSpec>
{
    /// Initialize a transcript given an input buffer.
    pub fn init(
        commitment_base: C,
        reader: R,
        base_spec: BaseSpec,
        scalar_spec: ScalarSpec,
    ) -> Self {
        PoseidonRead {
            base_state: PoseidonState::new(
                base_spec,
                ConstantLength::<2, T, RATE>.initial_capacity_element(),
                ConstantLength::<2, T, RATE>.pad_and_add(),
            ),
            scalar_state: PoseidonState::new(
                scalar_spec,
                ConstantLength::<2, T, RATE>.initial_capacity_element(),
                ConstantLength::<2, T, RATE>.pad_and_add(),
            ),
            squeezable: false,
            commitment_base,
            reader,
            _marker: PhantomData,
        }
    }
}

impl<
        R: Read,
        C: CurveAffine,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > TranscriptRead<C, Challenge255<C>>
    for PoseidonRead<R, C, Challenge255<C>, BaseSpec, ScalarSpec>
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

impl<
        R: Read,
        C: CurveAffine,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > Transcript<C, Challenge255<C>> for PoseidonRead<R, C, Challenge255<C>, BaseSpec, ScalarSpec>
{
    fn squeeze_challenge(&mut self) -> io::Result<Challenge255<C>> {
        if self.squeezable {
            let scalar = self.scalar_state.squeeze();
            self.squeezable = false;
            let point = self.commitment_base * scalar;
            self.common_point(point.to_affine())?;
        }

        self.base_state
            .absorb(C::Base::from_u64(POSEIDON_PREFIX_CHALLENGE));
        let challenge: [u8; 64] = {
            [
                self.base_state.squeeze().to_bytes(),
                self.base_state.squeeze().to_bytes(),
            ]
            .concat()
            .try_into()
            .unwrap()
        };
        Ok(Challenge255::<C>::new(&challenge))
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        self.base_state
            .absorb(C::Base::from_u64(POSEIDON_PREFIX_POINT));
        let coords: Coordinates<C> = Option::from(point.coordinates()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.base_state.absorb(*coords.x());
        self.base_state.absorb(*coords.y());

        Ok(())
    }

    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.scalar_state
            .absorb(C::Scalar::from_u64(POSEIDON_PREFIX_SCALAR));
        self.scalar_state.absorb(scalar);

        self.squeezable = true;
        Ok(())
    }
}

/// TODO
#[derive(Debug)]
pub struct PoseidonWrite<
    W: Write,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    BaseSpec: Spec<C::Base, T, RATE>,
    ScalarSpec: Spec<C::Scalar, T, RATE>,
> {
    base_state: PoseidonState<C::Base, BaseSpec, T, RATE>,
    scalar_state: PoseidonState<C::Scalar, ScalarSpec, T, RATE>,
    squeezable: bool,
    commitment_base: C,
    writer: W,
    _marker: PhantomData<(C, E)>,
}

impl<
        W: Write,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > PoseidonWrite<W, C, E, BaseSpec, ScalarSpec>
{
    /// Initialize a transcript given an output buffer.
    pub fn init(
        commitment_base: C,
        writer: W,
        base_spec: BaseSpec,
        scalar_spec: ScalarSpec,
    ) -> Self {
        PoseidonWrite {
            base_state: PoseidonState::new(
                base_spec,
                ConstantLength::<2, T, RATE>.initial_capacity_element(),
                ConstantLength::<2, T, RATE>.pad_and_add(),
            ),
            scalar_state: PoseidonState::new(
                scalar_spec,
                ConstantLength::<2, T, RATE>.initial_capacity_element(),
                ConstantLength::<2, T, RATE>.pad_and_add(),
            ),
            squeezable: false,
            commitment_base,
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

impl<
        W: Write,
        C: CurveAffine,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > TranscriptWrite<C, Challenge255<C>>
    for PoseidonWrite<W, C, Challenge255<C>, BaseSpec, ScalarSpec>
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

impl<
        W: Write,
        C: CurveAffine,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
    > Transcript<C, Challenge255<C>>
    for PoseidonWrite<W, C, Challenge255<C>, BaseSpec, ScalarSpec>
{
    fn squeeze_challenge(&mut self) -> io::Result<Challenge255<C>> {
        if self.squeezable {
            let scalar = self.scalar_state.squeeze();
            self.squeezable = false;
            let point = self.commitment_base * scalar;
            self.common_point(point.to_affine())?;
        }

        self.base_state
            .absorb(C::Base::from_u64(POSEIDON_PREFIX_CHALLENGE));
        let challenge: [u8; 64] = {
            [
                self.base_state.squeeze().to_bytes(),
                self.base_state.squeeze().to_bytes(),
            ]
            .concat()
            .try_into()
            .unwrap()
        };
        Ok(Challenge255::<C>::new(&challenge))
    }

    fn common_point(&mut self, point: C) -> io::Result<()> {
        self.base_state
            .absorb(C::Base::from_u64(POSEIDON_PREFIX_POINT));
        let coords: Coordinates<C> = Option::from(point.coordinates()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "cannot write points at infinity to the transcript",
            )
        })?;
        self.base_state.absorb(*coords.x());
        self.base_state.absorb(*coords.y());

        Ok(())
    }

    fn common_scalar(&mut self, scalar: C::Scalar) -> io::Result<()> {
        self.scalar_state
            .absorb(C::Scalar::from_u64(POSEIDON_PREFIX_SCALAR));
        self.scalar_state.absorb(scalar);

        self.squeezable = true;
        Ok(())
    }
}

/// The scalar representation of a verifier challenge.
///
/// The `Type` type can be used to scope the challenge to a specific context, or
/// set to `()` if no context is required.
#[derive(Copy, Clone, Debug, Default)]
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
