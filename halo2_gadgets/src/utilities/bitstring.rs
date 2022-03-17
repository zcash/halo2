//! Bitstring utility.
use std::convert::TryInto;

use ff::PrimeFieldBits;
use halo2_proofs::{circuit::AssignedCell, plonk::Assigned};
use pasta_curves::arithmetic::FieldExt;

/// A bit.
#[derive(Clone, Copy, Debug)]
pub struct Bit(bool);

impl std::ops::Deref for Bit {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F: FieldExt> From<&Bit> for Assigned<F> {
    fn from(bit: &Bit) -> Self {
        Assigned::Trivial(F::from(bit.0))
    }
}

impl From<bool> for Bit {
    fn from(bit: bool) -> Self {
        Self(bit)
    }
}

/// An assigned bit.
#[derive(Clone, Debug)]
pub struct AssignedBit<F: FieldExt>(AssignedCell<Bit, F>);

impl<F: FieldExt> From<&AssignedCell<Bit, F>> for AssignedBit<F> {
    fn from(assigned: &AssignedCell<Bit, F>) -> Self {
        Self(assigned.clone())
    }
}

impl<F: FieldExt> std::ops::Deref for AssignedBit<F> {
    type Target = AssignedCell<Bit, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A little-endian bitstring.
#[derive(Clone, Debug)]
pub struct Bits<const LEN: usize>([bool; LEN]);

impl<const LEN: usize> std::ops::Deref for Bits<LEN> {
    type Target = [bool; LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const LEN: usize> From<[bool; LEN]> for Bits<LEN> {
    fn from(bits: [bool; LEN]) -> Self {
        Self(bits)
    }
}

impl<F: FieldExt, const LEN: usize> From<&Bits<LEN>> for Assigned<F> {
    fn from(bits: &Bits<LEN>) -> Assigned<F> {
        assert!(LEN <= F::NUM_BITS as usize);
        let val = bits
            .0
            .iter()
            .rev()
            .fold(F::zero(), |acc, &b| acc.double() + F::from(b));

        Assigned::Trivial(val)
    }
}

impl<F, const LEN: usize> From<&F> for Bits<LEN>
where
    F: FieldExt + PrimeFieldBits,
{
    fn from(word: &F) -> Bits<LEN> {
        let word = word
            .to_le_bits()
            .into_iter()
            .take(F::NUM_BITS as usize)
            .collect::<Vec<_>>();
        let (word, zeros) = word.split_at(LEN);

        // Assert that all bits after LEN are zero.
        assert!(zeros.iter().all(|b| !b));

        Bits(word.try_into().unwrap())
    }
}

/// An assigned bitstring.
#[derive(Clone, Debug)]
pub struct AssignedBits<F: FieldExt, const LEN: usize>(AssignedCell<Bits<LEN>, F>);

impl<F: FieldExt, const LEN: usize> From<&AssignedCell<Bits<LEN>, F>> for AssignedBits<F, LEN> {
    fn from(assigned: &AssignedCell<Bits<LEN>, F>) -> Self {
        Self(assigned.clone())
    }
}

impl<F: FieldExt, const LEN: usize> std::ops::Deref for AssignedBits<F, LEN> {
    type Target = AssignedCell<Bits<LEN>, F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
