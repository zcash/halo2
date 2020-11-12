//! The Tweedledum elliptic curve group.

/// A Tweedledum point in the projective coordinate space.
pub type Point = super::Ep;

/// A Tweedledum point in the affine coordinate space (or the point at infinity).
pub type Affine = super::EpAffine;

/// The base field of the Tweedledum group.
pub type Base = super::Fp;

/// The scalar field of the Tweedledum group.
pub type Scalar = super::Fq;
