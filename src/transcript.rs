//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use crate::arithmetic::{CurveAffine, Field};
use std::marker::PhantomData;

/// This is a generic interface for a sponge function that can be used for
/// Fiat-Shamir transformations.
pub trait Hasher<F: Field>: Clone + Send + Sync + 'static {
    /// Initialize the sponge with some key.
    fn init(key: F) -> Self;
    /// Absorb a field element into the sponge.
    fn absorb(&mut self, value: F);
    /// Square a field element out of the sponge.
    fn squeeze(&mut self) -> F;
}

/// This is just a simple (and completely broken) hash function, standing in for
/// some algebraic hash function that we'll switch to later.
#[derive(Debug, Clone)]
pub struct DummyHash<F: Field> {
    power: F,
    state: F,
}

impl<F: Field> Hasher<F> for DummyHash<F> {
    fn init(key: F) -> Self {
        DummyHash {
            power: F::ZETA + F::one() + key,
            state: F::ZETA,
        }
    }
    fn absorb(&mut self, value: F) {
        for _ in 0..10 {
            self.state += value;
            self.state *= self.power;
            self.power += self.power.square();
            self.state += self.power;
        }
    }
    fn squeeze(&mut self) -> F {
        let tmp = self.state;
        self.absorb(tmp);
        tmp
    }
}

/// A transcript that can absorb points from both the base field and scalar
/// field of a curve
#[derive(Debug, Clone)]
pub struct Transcript<C: CurveAffine, HBase, HScalar>
where
    HBase: Hasher<C::Base>,
    HScalar: Hasher<C::Scalar>,
{
    // Hasher over the base field
    base_hasher: HBase,
    // Hasher over the scalar field
    scalar_hasher: HScalar,
    // Indicates if scalar(s) has been hashed but not squeezed
    scalar_needs_squeezing: bool,
    // PhantomData
    _marker: PhantomData<C>,
}

impl<C: CurveAffine, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>
    Transcript<C, HBase, HScalar>
{
    /// Initialise a new transcript with Field::one() as keys
    /// in both the base_hasher and scalar_hasher
    pub fn new() -> Self {
        let base_hasher = HBase::init(C::Base::one());
        let scalar_hasher = HScalar::init(C::Scalar::one());
        Transcript {
            base_hasher,
            scalar_hasher,
            scalar_needs_squeezing: false,
            _marker: PhantomData,
        }
    }

    /// Initialise a new transcript with some given base_hasher and
    /// scalar_hasher
    pub fn init_with_hashers(base_hasher: &HBase, scalar_hasher: &HScalar) -> Self {
        Transcript {
            base_hasher: base_hasher.clone(),
            scalar_hasher: scalar_hasher.clone(),
            scalar_needs_squeezing: false,
            _marker: PhantomData,
        }
    }

    /// Absorb a curve point into the transcript by absorbing
    /// its x and y coordinates
    pub fn absorb_point(&mut self, point: &C) -> Result<(), ()> {
        let tmp = point.get_xy();
        if bool::from(tmp.is_none()) {
            return Err(());
        };
        let tmp = tmp.unwrap();
        self.base_hasher.absorb(tmp.0);
        self.base_hasher.absorb(tmp.1);
        Ok(())
    }

    /// Absorb a base into the base_hasher
    pub fn absorb_base(&mut self, base: C::Base) {
        self.base_hasher.absorb(base);
    }

    /// Absorb a scalar into the scalar_hasher
    pub fn absorb_scalar(&mut self, scalar: C::Scalar) {
        self.scalar_hasher.absorb(scalar);
        self.scalar_needs_squeezing = true;
    }

    /// Squeeze the transcript to obtain a C::Base value.
    pub fn squeeze(&mut self) -> C::Base {
        if self.scalar_needs_squeezing {
            let transcript_scalar_point =
                C::Base::from_bytes(&(self.scalar_hasher.squeeze()).to_bytes()).unwrap();
            self.base_hasher.absorb(transcript_scalar_point);
            self.scalar_needs_squeezing = false;
        }

        self.base_hasher.squeeze()
    }
}
