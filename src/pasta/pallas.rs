//! The Pallas elliptic curve group.

/// A Pallas point in the projective coordinate space.
pub type Point = super::Ep;

/// A Pallas point in the affine coordinate space (or the point at infinity).
pub type Affine = super::EpAffine;

/// The base field of the Pallas group.
pub type Base = super::Fp;

/// The scalar field of the Pallas group.
pub type Scalar = super::Fq;
