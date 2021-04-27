//! Monetary values within the Orchard shielded pool.
//!
//! Values are represented in two places within Orchard:
//! - The value of an individual note, which is an unsigned 63-bit integer.
//! - The sum of note values within an Orchard [`Action`] or [`Bundle`], which is a signed
//!   63-bit integer.
//!
//! We give these separate types within this crate. Users should map these types to their
//! own general "amount" type as appropriate, and apply their own bounds checks if smaller
//! than the Orchard protocol supports.
//!
//! Inside the circuit, note values are constrained to be unsigned 64-bit integers.
//!
//! [`Action`]: crate::bundle::Action
//! [`Bundle`]: crate::bundle::Bundle

use std::convert::TryInto;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Sub};

use bitvec::{array::BitArray, order::Lsb0};
use ff::{Field, PrimeField};
use group::{Group, GroupEncoding};
use pasta_curves::{
    arithmetic::{CurveExt, FieldExt},
    pallas,
};
use rand::RngCore;

use crate::primitives::redpallas::{self, Binding};

/// A value operation overflowed.
#[derive(Debug)]
pub struct OverflowError;

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Orchard value operation overflowed")
    }
}

impl std::error::Error for OverflowError {}

/// The non-negative value of an individual Orchard note.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoteValue(u64);

impl NoteValue {
    pub(crate) fn zero() -> Self {
        // Default for u64 is zero.
        Default::default()
    }

    /// Creates a note value from its raw numeric value.
    ///
    /// This only enforces that the value is an unsigned 64-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: u64) -> Self {
        NoteValue(value)
    }

    pub(crate) fn to_le_bits(self) -> BitArray<Lsb0, [u8; 8]> {
        BitArray::<Lsb0, _>::new(self.0.to_le_bytes())
    }
}

impl Sub for NoteValue {
    type Output = Result<ValueSum, OverflowError>;

    fn sub(self, rhs: Self) -> Self::Output {
        let a: i64 = self.0.try_into().map_err(|_| OverflowError)?;
        let b: i64 = rhs.0.try_into().map_err(|_| OverflowError)?;
        Ok(ValueSum(a - b))
    }
}

/// A sum of Orchard note values.
#[derive(Clone, Copy, Debug, Default)]
pub struct ValueSum(i64);

impl ValueSum {
    /// Creates a value sum from its raw numeric value.
    ///
    /// This only enforces that the value is a signed 63-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: i64) -> Self {
        ValueSum(value)
    }
}

impl Add for ValueSum {
    type Output = Result<ValueSum, OverflowError>;

    fn add(self, rhs: Self) -> Self::Output {
        self.0.checked_add(rhs.0).map(ValueSum).ok_or(OverflowError)
    }
}

impl<'a> Sum<&'a ValueSum> for Result<ValueSum, OverflowError> {
    fn sum<I: Iterator<Item = &'a ValueSum>>(iter: I) -> Self {
        iter.fold(Ok(ValueSum(0)), |acc, cv| acc? + *cv)
    }
}

/// The blinding factor for a [`ValueCommitment`].
#[derive(Clone, Debug)]
pub struct ValueCommitTrapdoor(pallas::Scalar);

impl Add<&ValueCommitTrapdoor> for ValueCommitTrapdoor {
    type Output = ValueCommitTrapdoor;

    fn add(self, rhs: &Self) -> Self::Output {
        ValueCommitTrapdoor(self.0 + rhs.0)
    }
}

impl<'a> Sum<&'a ValueCommitTrapdoor> for ValueCommitTrapdoor {
    fn sum<I: Iterator<Item = &'a ValueCommitTrapdoor>>(iter: I) -> Self {
        iter.fold(ValueCommitTrapdoor::zero(), |acc, cv| acc + cv)
    }
}

impl ValueCommitTrapdoor {
    /// Generates a new value commitment trapdoor.
    pub(crate) fn random(rng: impl RngCore) -> Self {
        ValueCommitTrapdoor(pallas::Scalar::random(rng))
    }

    /// Returns the zero trapdoor, which provides no blinding.
    pub(crate) fn zero() -> Self {
        ValueCommitTrapdoor(pallas::Scalar::zero())
    }

    pub(crate) fn into_bsk(self) -> redpallas::SigningKey<Binding> {
        // TODO: impl From<pallas::Scalar> for redpallas::SigningKey.
        self.0.to_repr().try_into().unwrap()
    }
}

/// A commitment to a [`ValueSum`].
#[derive(Clone, Debug)]
pub struct ValueCommitment(pallas::Point);

impl Add<&ValueCommitment> for ValueCommitment {
    type Output = ValueCommitment;

    fn add(self, rhs: &Self) -> Self::Output {
        ValueCommitment(self.0 + rhs.0)
    }
}

impl Sub for ValueCommitment {
    type Output = ValueCommitment;

    fn sub(self, rhs: Self) -> Self::Output {
        ValueCommitment(self.0 - rhs.0)
    }
}

impl Sum for ValueCommitment {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ValueCommitment(pallas::Point::identity()), |acc, cv| {
            acc + &cv
        })
    }
}

impl<'a> Sum<&'a ValueCommitment> for ValueCommitment {
    fn sum<I: Iterator<Item = &'a ValueCommitment>>(iter: I) -> Self {
        iter.fold(ValueCommitment(pallas::Point::identity()), |acc, cv| {
            acc + cv
        })
    }
}

impl ValueCommitment {
    /// $ValueCommit^Orchard$.
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.8.3: Homomorphic Pedersen commitments (Sapling and Orchard)][concretehomomorphiccommit].
    ///
    /// [concretehomomorphiccommit]: https://zips.z.cash/protocol/nu5.pdf#concretehomomorphiccommit
    #[allow(non_snake_case)]
    pub(crate) fn derive(value: ValueSum, rcv: ValueCommitTrapdoor) -> Self {
        let hasher = pallas::Point::hash_to_curve("z.cash:Orchard-cv");
        let V = hasher(b"v");
        let R = hasher(b"r");

        let value = if value.0.is_negative() {
            -pallas::Scalar::from_u64((-value.0) as u64)
        } else {
            pallas::Scalar::from_u64(value.0 as u64)
        };

        ValueCommitment(V * value + R * rcv.0)
    }

    pub(crate) fn into_bvk(self) -> redpallas::VerificationKey<Binding> {
        // TODO: impl From<pallas::Point> for redpallas::VerificationKey.
        self.0.to_bytes().try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use proptest::prelude::*;

    use super::{OverflowError, ValueCommitTrapdoor, ValueCommitment, ValueSum};
    use crate::primitives::redpallas;

    /// Zcash's maximum money amount. Used as a bound in proptests so we don't artifically
    /// overflow `ValueSum`'s size.
    const MAX_MONEY: i64 = 21_000_000 * 1_0000_0000;

    prop_compose! {
        fn arb_scalar()(bytes in prop::array::uniform32(0u8..)) -> pallas::Scalar {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Scalar::from_bytes_wide(&buf)
        }
    }

    prop_compose! {
        fn arb_value_sum(bound: i64)(value in -bound..bound) -> ValueSum {
            ValueSum(value)
        }
    }

    prop_compose! {
        fn arb_trapdoor()(rcv in arb_scalar()) -> ValueCommitTrapdoor {
            ValueCommitTrapdoor(rcv)
        }
    }

    proptest! {
        #[test]
        fn bsk_consistent_with_bvk(
            values in prop::collection::vec((arb_value_sum(MAX_MONEY), arb_trapdoor()), 1..10),
        ) {
            let value_balance = values
                .iter()
                .map(|(value, _)| value)
                .sum::<Result<ValueSum, OverflowError>>()
                .expect("we generate values that won't overflow");

            let bsk = values
                .iter()
                .map(|(_, rcv)| rcv)
                .sum::<ValueCommitTrapdoor>()
                .into_bsk();

            let bvk = (values
                .into_iter()
                .map(|(value, rcv)| ValueCommitment::derive(value, rcv))
                .sum::<ValueCommitment>()
                - ValueCommitment::derive(value_balance, ValueCommitTrapdoor::zero()))
            .into_bvk();

            assert_eq!(redpallas::VerificationKey::from(&bsk), bvk);
        }
    }
}
