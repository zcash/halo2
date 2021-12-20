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

use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug};
use std::iter::Sum;
use std::ops::{Add, Sub};

use bitvec::{array::BitArray, order::Lsb0};
use ff::{Field, PrimeField};
use group::{Curve, Group, GroupEncoding};
use pasta_curves::{
    arithmetic::{CurveAffine, CurveExt},
    pallas,
};
use rand::RngCore;
use subtle::CtOption;

use crate::{
    constants::{
        VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_R_BYTES, VALUE_COMMITMENT_V_BYTES,
    },
    primitives::redpallas::{self, Binding},
};

use std::ops::RangeInclusive;

/// Maximum note value.
pub const MAX_NOTE_VALUE: u64 = u64::MAX;

/// The valid range of the scalar multiplication used in ValueCommit^Orchard.
///
/// Defined in a note in [Zcash Protocol Spec § 4.17.4: Action Statement (Orchard)][actionstatement].
///
/// [actionstatement]: https://zips.z.cash/protocol/nu5.pdf#actionstatement
pub const VALUE_SUM_RANGE: RangeInclusive<i128> =
    -(MAX_NOTE_VALUE as i128)..=MAX_NOTE_VALUE as i128;

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

    /// Returns the raw underlying value.
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Creates a note value from its raw numeric value.
    ///
    /// This only enforces that the value is an unsigned 64-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: u64) -> Self {
        NoteValue(value)
    }

    pub(crate) fn from_bytes(bytes: [u8; 8]) -> Self {
        NoteValue(u64::from_le_bytes(bytes))
    }

    pub(crate) fn to_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub(crate) fn to_le_bits(self) -> BitArray<Lsb0, [u8; 8]> {
        BitArray::<Lsb0, _>::new(self.0.to_le_bytes())
    }
}

impl Sub for NoteValue {
    type Output = Option<ValueSum>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        let a = self.0 as i128;
        let b = rhs.0 as i128;
        a.checked_sub(b)
            .filter(|v| VALUE_SUM_RANGE.contains(v))
            .map(ValueSum)
    }
}

/// A sum of Orchard note values
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ValueSum(i128);

impl ValueSum {
    pub(crate) fn zero() -> Self {
        // Default for i64 is zero.
        Default::default()
    }

    /// Creates a value sum from its raw numeric value.
    ///
    /// This only enforces that the value is a signed 63-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: i64) -> Self {
        ValueSum(value as i128)
    }
}

impl Add for ValueSum {
    type Output = Option<ValueSum>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self.0
            .checked_add(rhs.0)
            .filter(|v| VALUE_SUM_RANGE.contains(v))
            .map(ValueSum)
    }
}

impl<'a> Sum<&'a ValueSum> for Result<ValueSum, OverflowError> {
    fn sum<I: Iterator<Item = &'a ValueSum>>(iter: I) -> Self {
        iter.fold(Ok(ValueSum(0)), |acc, v| (acc? + *v).ok_or(OverflowError))
    }
}

impl Sum<ValueSum> for Result<ValueSum, OverflowError> {
    fn sum<I: Iterator<Item = ValueSum>>(iter: I) -> Self {
        iter.fold(Ok(ValueSum(0)), |acc, v| (acc? + v).ok_or(OverflowError))
    }
}

impl TryFrom<ValueSum> for i64 {
    type Error = OverflowError;

    fn try_from(v: ValueSum) -> Result<i64, Self::Error> {
        i64::try_from(v.0).map_err(|_| OverflowError)
    }
}

/// The blinding factor for a [`ValueCommitment`].
#[derive(Clone, Debug)]
pub struct ValueCommitTrapdoor(pallas::Scalar);

impl ValueCommitTrapdoor {
    pub(crate) fn inner(&self) -> pallas::Scalar {
        self.0
    }
}

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
        let hasher = pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION);
        let V = hasher(&VALUE_COMMITMENT_V_BYTES);
        let R = hasher(&VALUE_COMMITMENT_R_BYTES);
        let abs_value = u64::try_from(value.0.abs()).expect("value must be in valid range");

        let value = if value.0.is_negative() {
            -pallas::Scalar::from(abs_value)
        } else {
            pallas::Scalar::from(abs_value)
        };

        ValueCommitment(V * value + R * rcv.0)
    }

    pub(crate) fn into_bvk(self) -> redpallas::VerificationKey<Binding> {
        // TODO: impl From<pallas::Point> for redpallas::VerificationKey.
        self.0.to_bytes().try_into().unwrap()
    }

    /// Deserialize a value commitment from its byte representation
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<ValueCommitment> {
        pallas::Point::from_bytes(bytes).map(ValueCommitment)
    }

    /// Serialize this value commitment to its canonical byte representation.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// x-coordinate of this value commitment.
    pub(crate) fn x(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().x()
        }
    }

    /// y-coordinate of this value commitment.
    pub(crate) fn y(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().y()
        }
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use proptest::prelude::*;

    use super::{NoteValue, ValueCommitTrapdoor, ValueSum, MAX_NOTE_VALUE, VALUE_SUM_RANGE};

    prop_compose! {
        /// Generate an arbitrary Pallas scalar.
        pub fn arb_scalar()(bytes in prop::array::uniform32(0u8..)) -> pallas::Scalar {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Scalar::from_bytes_wide(&buf)
        }
    }

    prop_compose! {
        /// Generate an arbitrary [`ValueSum`] in the range of valid Zcash values.
        pub fn arb_value_sum()(value in VALUE_SUM_RANGE) -> ValueSum {
            ValueSum(value as i128)
        }
    }

    prop_compose! {
        /// Generate an arbitrary [`ValueSum`] in the range of valid Zcash values.
        pub fn arb_value_sum_bounded(bound: NoteValue)(value in -(bound.0 as i128)..=(bound.0 as i128)) -> ValueSum {
            ValueSum(value as i128)
        }
    }

    prop_compose! {
        /// Generate an arbitrary ValueCommitTrapdoor
        pub fn arb_trapdoor()(rcv in arb_scalar()) -> ValueCommitTrapdoor {
            ValueCommitTrapdoor(rcv)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid nonnegative Zcash amounts.
        pub fn arb_note_value()(value in 0u64..MAX_NOTE_VALUE) -> NoteValue {
            NoteValue(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid positive Zcash amounts
        /// less than a specified value.
        pub fn arb_note_value_bounded(max: u64)(value in 0u64..max) -> NoteValue {
            NoteValue(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid positive Zcash amounts
        /// less than a specified value.
        pub fn arb_positive_note_value(max: u64)(value in 1u64..max) -> NoteValue {
            NoteValue(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{
        testing::{arb_note_value_bounded, arb_trapdoor, arb_value_sum_bounded},
        OverflowError, ValueCommitTrapdoor, ValueCommitment, ValueSum, MAX_NOTE_VALUE,
    };
    use crate::primitives::redpallas;

    proptest! {
        #[test]
        fn bsk_consistent_with_bvk(
            values in (1usize..10).prop_flat_map(|n_values|
                arb_note_value_bounded(MAX_NOTE_VALUE / n_values as u64).prop_flat_map(move |bound|
                    prop::collection::vec((arb_value_sum_bounded(bound), arb_trapdoor()), n_values)
                )
            )
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
