//! The Tweedledee elliptic curve group.

/// A Tweedledee point in the projective coordinate space.
pub type Point = super::Eq;

/// A Tweedledee point in the affine coordinate space (or the point at infinity).
pub type Affine = super::EqAffine;

/// The base field of the Tweedledee group.
pub type Base = super::Fq;

/// The scalar field of the Tweedledee group.
pub type Scalar = super::Fp;
