//! This module contains utilities and traits for dealing with Fiat-Shamir
//! transcripts.

use crate::arithmetic::Field;

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
    fn init(value: F) -> Self {
        DummyHash {
            power: F::ZETA + F::one() + value,
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
