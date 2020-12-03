//! The Vesta elliptic curve group.

/// A Vesta point in the projective coordinate space.
pub type Point = super::Eq;

/// A Vesta point in the affine coordinate space (or the point at infinity).
pub type Affine = super::EqAffine;

/// The base field of the Vesta group.
pub type Base = super::Fq;

/// The scalar field of the Vesta group.
pub type Scalar = super::Fp;
