//! The Vesta and iso-Vesta elliptic curve groups.

use lazy_static::lazy_static;

use super::SimplifiedSWUWithDegree3Isogeny;
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

lazy_static! {
    /// The iso-Vesta -> Vesta degree 3 isogeny map.
    pub static ref MAP: SimplifiedSWUWithDegree3Isogeny<Base, Affine, IsoAffine> = {
        SimplifiedSWUWithDegree3Isogeny::new(
            IsoAffine::Z,
            IsoAffine::ISOGENY_CONSTANTS,
            IsoAffine::MINUS_B_OVER_A,
            IsoAffine::B_OVER_ZA,
            IsoAffine::THETA
        )
    };
}

#[test]
fn test_map_to_curve_vesta() {
    use crate::arithmetic::{Curve, CurveAffine, FieldExt};
    use std::collections::HashSet;

    assert!(MAP.minus_b_over_a * IsoAffine::a() == -IsoAffine::b());
    assert!(MAP.b_over_za * MAP.z * IsoAffine::a() == IsoAffine::b());
    assert!(MAP.theta.square() * Base::ROOT_OF_UNITY == MAP.z);

    let set: HashSet<_> = (0..10000)
        .map(|i| MAP.map_to_curve(&Base::from(i)).to_affine())
        .collect();
    assert!(set.len() == 10000);

    let hash = MAP.hash_to_curve("z.cash:test");
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
