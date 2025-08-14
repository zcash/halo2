//! This module provides common utilities, traits and structures for group and
//! field arithmetic.
//!
//! This module is temporary, and the extension traits defined here are expected to be
//! upstreamed into the `ff` and `group` crates after some refactoring.

mod curves;
mod fields;

pub use curves::*;
pub use fields::*;

/// This represents an element of a group with basic operations that can be
/// performed. This allows an FFT implementation (for example) to operate
/// generically over either a field or elliptic curve group.
pub trait Group: Copy + Clone + Send + Sync + 'static {
    /// The group is assumed to be of prime order $p$. `Scalar` is the
    /// associated scalar field of size $p$.
    type Scalar: FieldExt;

    /// Returns the additive identity of the group.
    fn group_zero() -> Self;

    /// Adds `rhs` to this group element.
    fn group_add(&mut self, rhs: &Self);

    /// Subtracts `rhs` from this group element.
    fn group_sub(&mut self, rhs: &Self);

    /// Scales this group element by a scalar.
    fn group_scale(&mut self, by: &Self::Scalar);
}
