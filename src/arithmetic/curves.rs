//! This module contains the `Curve`/`CurveAffine` abstractions that allow us to
//! write code that generalizes over a pair of groups.

use core::cmp;
use core::fmt::Debug;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::{FieldExt, Group};

use std::io::{self, Read, Write};

/// This trait is a common interface for dealing with elements of an elliptic
/// curve group in a "projective" form, where that arithmetic is usually more
/// efficient.
pub trait Curve:
    Sized
    + Default
    + Copy
    + Clone
    + Send
    + Sync
    + 'static
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<<Self as Curve>::Scalar, Output = Self>
    + Neg<Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + MulAssign<<Self as Curve>::Scalar>
    + AddAssign
    + SubAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + AddAssign<<Self as Curve>::Affine>
    + SubAssign<<Self as Curve>::Affine>
    + PartialEq
    + cmp::Eq
    + ConditionallySelectable
    + ConstantTimeEq
    + From<<Self as Curve>::Affine>
    + Group<Scalar = <Self as Curve>::Scalar>
{
    /// The representation of a point on this curve in the affine coordinate space.
    type Affine: CurveAffine<
            Projective = Self,
            Scalar = <Self as Curve>::Scalar,
            Base = <Self as Curve>::Base,
        > + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<<Self as Curve>::Scalar, Output = Self>
        + Neg<Output = <Self as Curve>::Affine>
        + From<Self>;
    /// The scalar field of this elliptic curve.
    type Scalar: FieldExt;
    /// The base field over which this elliptic curve is constructed.
    type Base: FieldExt;

    /// Obtains the additive identity.
    fn zero() -> Self;

    /// Obtains the base point of the curve.
    fn one() -> Self;

    /// Doubles this element.
    fn double(&self) -> Self;

    /// Returns whether or not this element is the identity.
    fn is_zero(&self) -> Choice;

    /// Apply the curve endomorphism by multiplying the x-coordinate
    /// by an element of multiplicative order 3.
    fn endo(&self) -> Self;

    /// Converts this element into its affine form.
    fn to_affine(&self) -> Self::Affine;

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
    /// use halo2::arithmetic::{Curve, CurveAffine};
    /// fn pedersen_commitment<C: CurveAffine>(x: C::Scalar, r: C::Scalar) -> C {
    ///     let hasher = C::Projective::hash_to_curve("z.cash:example_pedersen_commitment");
    ///     let g = hasher(b"g");
    ///     let h = hasher(b"h");
    ///     (g * x + &(h * r)).to_affine()
    /// }
    /// ```
    fn hash_to_curve<'a>(domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a>;

    /// Returns whether or not this element is on the curve; should
    /// always be true unless an "unchecked" API was used.
    fn is_on_curve(&self) -> Choice;

    /// Converts many elements into their affine form. Panics if the
    /// sizes of the slices are different.
    fn batch_to_affine(v: &[Self], target: &mut [Self::Affine]);

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
    Sized
    + Default
    + Copy
    + Clone
    + Send
    + Sync
    + 'static
    + Debug
    + Add<Output = <Self as CurveAffine>::Projective>
    + Sub<Output = <Self as CurveAffine>::Projective>
    + Mul<<Self as CurveAffine>::Scalar, Output = <Self as CurveAffine>::Projective>
    + Neg<Output = Self>
    + PartialEq
    + cmp::Eq
    + ConditionallySelectable
    + ConstantTimeEq
    + From<<Self as CurveAffine>::Projective>
{
    /// The representation of a point on this curve in the projective coordinate space.
    type Projective: Curve<
            Affine = Self,
            Scalar = <Self as CurveAffine>::Scalar,
            Base = <Self as CurveAffine>::Base,
        > + Mul<<Self as CurveAffine>::Scalar, Output = <Self as CurveAffine>::Projective>
        + MulAssign<<Self as CurveAffine>::Scalar>
        + AddAssign<Self>
        + SubAssign<Self>
        + From<Self>;
    /// The scalar field of this elliptic curve.
    type Scalar: FieldExt;
    /// The base field over which this elliptic curve is constructed.
    type Base: FieldExt;

    /// Personalization of BLAKE2b hasher used to generate the uniform
    /// random string.
    const BLAKE2B_PERSONALIZATION: &'static [u8; 16];

    /// CURVE_ID used for hash-to-curve.
    const CURVE_ID: &'static str;

    /// Obtains the additive identity.
    fn zero() -> Self;

    /// Obtains the base point of the curve.
    fn one() -> Self;

    /// Returns whether or not this element is the identity.
    fn is_zero(&self) -> Choice;

    /// Converts this element into its projective form.
    fn to_projective(&self) -> Self::Projective;

    /// Gets the $(x, y)$ coordinates of this point.
    fn get_xy(&self) -> CtOption<(Self::Base, Self::Base)>;

    /// Obtains a point given $(x, y)$, failing if it is not on the
    /// curve.
    fn from_xy(x: Self::Base, y: Self::Base) -> CtOption<Self>;

    /// Returns whether or not this element is on the curve; should
    /// always be true unless an "unchecked" API was used.
    fn is_on_curve(&self) -> Choice;

    /// Attempts to obtain a group element from its compressed 32-byte little
    /// endian representation.
    fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self>;

    /// Reads a compressed element from the buffer and attempts to parse it
    /// using `from_bytes`.
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut compressed = [0u8; 32];
        reader.read_exact(&mut compressed[..])?;
        Option::from(Self::from_bytes(&compressed))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid point encoding in proof"))
    }

    /// Obtains the compressed, 32-byte little endian representation of this
    /// element.
    fn to_bytes(&self) -> [u8; 32];

    /// Writes an element in compressed form to the buffer.
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let compressed = self.to_bytes();
        writer.write_all(&compressed[..])
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
