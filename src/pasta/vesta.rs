//! The Vesta and iso-Vesta elliptic curve groups.

use super::{Eq, EqAffine, Fp, Fq, IsoEq, IsoEqAffine};

/// The base field of the Vesta and iso-Vesta curves.
pub type Base = Fq;

/// The scalar field of the Vesta and iso-Vesta curves.
pub type Scalar = Fp;

/// A Vesta point in the projective coordinate space.
pub type Point = Eq;

/// A Vesta point in the affine coordinate space (or the point at infinity).
pub type Affine = EqAffine;

/// An iso-Vesta point in the projective coordinate space.
pub type IsoPoint = IsoEq;

/// A iso-Vesta point in the affine coordinate space (or the point at infinity).
pub type IsoAffine = IsoEqAffine;

#[test]
fn test_map_to_curve_vesta() {
    use crate::arithmetic::Curve;

    let hash = Point::hasher("z.cash:test");
    let p: Point = hash(b"hello");
    let (x, y, z) = p.jacobian_coordinates();
    println!("{:?}", p);
    assert!(
        format!("{:?}", x) == "0x3984612258b3b43b4f6e046f7f796bbd35ffd8908804bcf47b9537d3ec7645c9"
    );
    assert!(
        format!("{:?}", y) == "0x2573c035293d745a288a65a7a37709ef99bcf31b77cfb3a1126a61e3adeebc4b"
    );
    assert!(
        format!("{:?}", z) == "0x1cb99da94a634842b09a3ee1e5b462233e1fc23d0b357ec7fb0d1c409be30720"
    );
}
