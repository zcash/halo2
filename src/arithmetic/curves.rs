//! This module contains the `Curve`/`CurveAffine` abstractions that allow us to
//! write code that generalizes over a pair of groups.

use core::cmp;
use core::ops::{Add, Mul, Sub};
use group::prime::{PrimeCurve, PrimeCurveAffine};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::{FieldExt, Group};

use std::io::{self, Read, Write};

/// This trait is a common interface for dealing with elements of an elliptic
/// curve group in a "projective" form, where that arithmetic is usually more
/// efficient.
pub trait CurveExt:
    PrimeCurve<Affine = <Self as CurveExt>::AffineExt>
    + group::Group<Scalar = <Self as CurveExt>::ScalarExt>
    + Default
    + PartialEq
    + cmp::Eq
    + ConditionallySelectable
    + ConstantTimeEq
    + From<<Self as PrimeCurve>::Affine>
    + Group<Scalar = <Self as group::Group>::Scalar>
{
    /// The scalar field of this elliptic curve.
    type ScalarExt: FieldExt;
    /// The base field over which this elliptic curve is constructed.
    type Base: FieldExt;
    /// The affine version of the curve
    type AffineExt: CurveAffine<CurveExt = Self, ScalarExt = <Self as CurveExt>::ScalarExt>
        + Mul<Self::ScalarExt, Output = Self>
        + for<'r> Mul<Self::ScalarExt, Output = Self>;

    /// CURVE_ID used for hash-to-curve.
    const CURVE_ID: &'static str;

    /// Apply the curve endomorphism by multiplying the x-coordinate
    /// by an element of multiplicative order 3.
    fn endo(&self) -> Self;

    /// Return the Jacobian coordinates of this point.
    fn jacobian_coordinates(&self) -> (Self::Base, Self::Base, Self::Base);

    /// Requests a hasher that accepts messages and returns near-uniformly
    /// distributed elements in the group, given domain prefix `domain_prefix`.
    ///
    /// This method is suitable for use as a random oracle.
    ///
    /// # Example
    ///
    /// ```
    /// use halo2::arithmetic::CurveExt;
    /// fn pedersen_commitment<C: CurveExt>(
    ///     x: C::ScalarExt,
    ///     r: C::ScalarExt,
    /// ) -> C::Affine {
    ///     let hasher = C::hash_to_curve("z.cash:example_pedersen_commitment");
    ///     let g = hasher(b"g");
    ///     let h = hasher(b"h");
    ///     (g * x + &(h * r)).to_affine()
    /// }
    /// ```
    fn hash_to_curve<'a>(domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a>;

    /// Returns whether or not this element is on the curve; should
    /// always be true unless an "unchecked" API was used.
    fn is_on_curve(&self) -> Choice;

    /// Returns the curve constant a.
    fn a() -> Self::Base;

    /// Returns the curve constant b.
    fn b() -> Self::Base;

    /// Obtains a point given Jacobian coordinates $X : Y : Z$, failing
    /// if the coordinates are not on the curve.
    fn new_jacobian(x: Self::Base, y: Self::Base, z: Self::Base) -> CtOption<Self>;
}

/// This trait is the affine counterpart to `Curve` and is used for
/// serialization, storage in memory, and inspection of $x$ and $y$ coordinates.
pub trait CurveAffine:
    PrimeCurveAffine<
        Scalar = <Self as CurveAffine>::ScalarExt,
        Curve = <Self as CurveAffine>::CurveExt,
    > + Default
    + Add<Output = <Self as PrimeCurveAffine>::Curve>
    + Sub<Output = <Self as PrimeCurveAffine>::Curve>
    + ConditionallySelectable
    + ConstantTimeEq
    + From<<Self as PrimeCurveAffine>::Curve>
{
    /// The scalar field of this elliptic curve.
    type ScalarExt: FieldExt;
    /// The base field over which this elliptic curve is constructed.
    type Base: FieldExt;
    /// The projective form of the curve
    type CurveExt: CurveExt<AffineExt = Self, ScalarExt = <Self as CurveAffine>::ScalarExt>;

    /// Gets the $(x, y)$ coordinates of this point.
    fn get_xy(&self) -> CtOption<(Self::Base, Self::Base)>;

    /// Obtains a point given $(x, y)$, failing if it is not on the
    /// curve.
    fn from_xy(x: Self::Base, y: Self::Base) -> CtOption<Self>;

    /// Returns whether or not this element is on the curve; should
    /// always be true unless an "unchecked" API was used.
    fn is_on_curve(&self) -> Choice;

    /// Reads a compressed element from the buffer and attempts to parse it
    /// using `from_bytes`.
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = Self::Repr::default();
        reader.read_exact(compressed.as_mut())?;
        Option::from(Self::from_bytes(&compressed))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid point encoding in proof"))
    }

    /// Writes an element in compressed form to the buffer.
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let compressed = self.to_bytes();
        writer.write_all(compressed.as_ref())
    }

    /// Attempts to obtain a group element from its uncompressed 64-byte little
    /// endian representation.
    fn from_bytes_wide(bytes: &[u8; 64]) -> CtOption<Self>;

    /// Obtains the uncompressed, 64-byte little endian representation of this
    /// element.
    fn to_bytes_wide(&self) -> [u8; 64];

    /// Returns the curve constant $a$.
    fn a() -> Self::Base;

    /// Returns the curve constant $b$.
    fn b() -> Self::Base;
}
