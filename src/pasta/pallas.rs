//! The Pallas and iso-Pallas elliptic curve groups.

use super::{Ep, EpAffine, Fp, Fq, IsoEp, IsoEpAffine};

/// The base field of the Pallas and iso-Pallas curves.
pub type Base = Fp;

/// The scalar field of the Pallas and iso-Pallas curves.
pub type Scalar = Fq;

/// A Pallas point in the projective coordinate space.
pub type Point = Ep;

/// A Pallas point in the affine coordinate space (or the point at infinity).
pub type Affine = EpAffine;

/// An iso-Pallas point in the projective coordinate space.
pub type IsoPoint = IsoEp;

/// A iso-Pallas point in the affine coordinate space (or the point at infinity).
pub type IsoAffine = IsoEpAffine;

#[test]
fn test_map_to_curve_pallas() {
    use crate::arithmetic::Curve;

    let hash = Point::hash_to_curve("z.cash:test");
    let p: Point = hash(b"hello");
    let (x, y, z) = p.jacobian_coordinates();
    println!("{:?}", p);
    assert!(
        format!("{:?}", x) == "0x318cc15f281662b3f26d0175cab97b924870c837879cac647e877be51a85e898"
    );
    assert!(
        format!("{:?}", y) == "0x1e91e2fa2a5a6a5bc86ff9564ae9336084470e7119dffcb85ae8c1383a3defd7"
    );
    assert!(
        format!("{:?}", z) == "0x1e049436efa754f5f189aec69c2c3a4a559eca6a12b45c3f2e4a769deeca6187"
    );
}
