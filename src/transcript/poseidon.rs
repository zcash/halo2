use poseidon::primitive::{Domain, Duplex, Spec};

use super::{EncodedChallenge, Transcript, TranscriptRead, TranscriptWrite};

use crate::arithmetic::{Coordinates, CurveAffine, FieldExt};

use std::io::{self, Read, Write};
use std::marker::PhantomData;

use group::Curve;

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

#[derive(Debug)]
/// A transcript reader instantiated over a Poseidon sponge.
pub struct PoseidonRead<
    R: Read,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    BaseSpec: Spec<C::Base, T, RATE>,
    ScalarSpec: Spec<C::Scalar, T, RATE>,
    BaseDomain: Domain<C::Base, T, RATE>,
    ScalarDomain: Domain<C::Scalar, T, RATE>,
> {
    base_state: Duplex<C::Base, BaseSpec, T, RATE>,
    scalar_state: Duplex<C::Scalar, ScalarSpec, T, RATE>,
    squeezable: bool,
    commitment_base: C,
    reader: R,
    _marker: PhantomData<(E, BaseDomain, ScalarDomain)>,
}

impl<
        R: Read,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > PoseidonRead<R, C, E, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
{
    #[allow(dead_code)]
    /// Initialize a transcript given an input buffer.
    pub(crate) fn init(
        commitment_base: C,
        reader: R,
        base_spec: BaseSpec,
        scalar_spec: ScalarSpec,
        base_domain: BaseDomain,
        scalar_domain: ScalarDomain,
    ) -> Self {
        PoseidonRead {
            base_state: Duplex::new(
                base_spec,
                base_domain.initial_capacity_element(),
                base_domain.pad_and_add(),
            ),
            scalar_state: Duplex::new(
                scalar_spec,
                scalar_domain.initial_capacity_element(),
                scalar_domain.pad_and_add(),
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
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > TranscriptRead<C, Challenge255<C>>
    for PoseidonRead<R, C, Challenge255<C>, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
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
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > Transcript<C, Challenge255<C>>
    for PoseidonRead<R, C, Challenge255<C>, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
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
        let challenge = self.base_state.squeeze();
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

#[derive(Debug)]
/// A transcript reader instantiated over a Poseidon sponge.
pub struct PoseidonWrite<
    W: Write,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    BaseSpec: Spec<C::Base, T, RATE>,
    ScalarSpec: Spec<C::Scalar, T, RATE>,
    BaseDomain: Domain<C::Base, T, RATE>,
    ScalarDomain: Domain<C::Scalar, T, RATE>,
> {
    base_state: Duplex<C::Base, BaseSpec, T, RATE>,
    scalar_state: Duplex<C::Scalar, ScalarSpec, T, RATE>,
    squeezable: bool,
    commitment_base: C,
    writer: W,
    _marker: PhantomData<(E, BaseDomain, ScalarDomain)>,
}

impl<
        W: Write,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > PoseidonWrite<W, C, E, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
{
    #[allow(dead_code)]
    /// Initialize a transcript given an output buffer.
    pub(crate) fn init(
        commitment_base: C,
        writer: W,
        base_spec: BaseSpec,
        scalar_spec: ScalarSpec,
        base_domain: BaseDomain,
        scalar_domain: ScalarDomain,
    ) -> Self {
        PoseidonWrite {
            base_state: Duplex::new(
                base_spec,
                base_domain.initial_capacity_element(),
                base_domain.pad_and_add(),
            ),
            scalar_state: Duplex::new(
                scalar_spec,
                scalar_domain.initial_capacity_element(),
                scalar_domain.pad_and_add(),
            ),
            squeezable: false,
            commitment_base,
            writer,
            _marker: PhantomData,
        }
    }

    #[allow(dead_code)]
    /// Conclude the interaction and return the output buffer (writer).
    pub(crate) fn finalize(self) -> W {
        // TODO: handle outstanding scalars? see issue #138
        self.writer
    }
}

impl<
        W: Write,
        C: CurveAffine,
        BaseSpec: Spec<C::Base, T, RATE>,
        ScalarSpec: Spec<C::Scalar, T, RATE>,
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > TranscriptWrite<C, Challenge255<C>>
    for PoseidonWrite<W, C, Challenge255<C>, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
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
        BaseDomain: Domain<C::Base, T, RATE>,
        ScalarDomain: Domain<C::Scalar, T, RATE>,
    > Transcript<C, Challenge255<C>>
    for PoseidonWrite<W, C, Challenge255<C>, BaseSpec, ScalarSpec, BaseDomain, ScalarDomain>
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
        let challenge = self.base_state.squeeze();
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
    type Input = C::Base;

    fn new(challenge_input: &C::Base) -> Self {
        Challenge255(challenge_input.to_bytes(), PhantomData)
    }
    fn get_scalar(&self) -> C::Scalar {
        C::Scalar::from_bytes(&self.0).unwrap()
    }
}
