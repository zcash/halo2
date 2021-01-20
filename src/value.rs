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

use std::fmt;
use std::marker::PhantomData;

/// The constraints applied to Orchard values.
pub trait Constraint: fmt::Debug {}

/// The value of an individual Orchard note.
#[derive(Debug)]
pub struct NoteValue<C: Constraint>(u64, PhantomData<C>);

/// A sum of Orchard note values.
#[derive(Debug)]
pub struct ValueSum<C: Constraint>(i64, PhantomData<C>);

/// A commitment to a [`ValueSum`].
#[derive(Debug)]
pub struct ValueCommitment;
