//! Monetary values within the Orchard shielded pool.
//!
//! Values are represented in two places within Orchard:
//! - The value of an individual note, which is unsigned.
//! - The sum of note values within an Orchard [`Action`] or [`Bundle`], which is signed.
//!
//! We give these separate types within this crate. Users should map these types to their
//! own general "amount" type as appropriate.
//!
//! Inside the circuit, values are constrained to be 63-bit integers.
//! - TODO: Should this be constrained further to 53 bits? To Zcash's MAX_MONEY?
//!
//! [`Action`]: crate::bundle::Action
//! [`Bundle`]: crate::bundle::Bundle

use bitvec::{array::BitArray, order::Lsb0};

/// The value of an individual Orchard note.
#[derive(Clone, Copy, Debug)]
pub struct NoteValue(u64);

impl NoteValue {
    pub(crate) fn to_le_bits(&self) -> BitArray<Lsb0, [u8; 8]> {
        BitArray::<Lsb0, _>::new(self.0.to_le_bytes())
    }
}

/// A sum of Orchard note values.
#[derive(Debug)]
pub struct ValueSum(i64);

/// A commitment to a [`ValueSum`].
#[derive(Debug)]
pub struct ValueCommitment;
