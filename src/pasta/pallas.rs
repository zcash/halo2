//! The Pallas and iso-Pallas elliptic curve groups.

use lazy_static::lazy_static;

use super::SimplifiedSWUWithDegree3Isogeny;
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

lazy_static! {
    /// The iso-Pallas -> Pallas degree 3 isogeny map.
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
fn test_iso_map() {
    use crate::arithmetic::Curve;

    // This is a regression test (it's the same input to iso_map as for hash_to_curve
    // with domain prefix "z.cash:test", Shake128, and input b"hello").
    let r = IsoPoint::new_jacobian(
        Base::from_raw([
            0xc37f111df5c4419e,
            0x593c053e5e2337ad,
            0x9c6cfc47bce1aba6,
            0x0a881e4d556945aa,
        ]),
        Base::from_raw([
            0xf234e04434502b47,
            0x6979f7f2b0acf188,
            0xa62eec46f662cb4e,
            0x035e5c8a06d5cfb4,
        ]),
        Base::from_raw([
            0x11ab791d4fb6f6b4,
            0x575baa717958ef1f,
            0x6ac4e343558dcbf3,
            0x3af37975b0933125,
        ]),
    )
    .unwrap();
    let p = MAP.iso_map(&r);
    let (x, y, z) = p.jacobian_coordinates();
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

#[test]
fn test_map_to_curve_pallas() {
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
        format!("{:?}", x) == "0x318cc15f281662b3f26d0175cab97b924870c837879cac647e877be51a85e898"
    );
    assert!(
        format!("{:?}", y) == "0x1e91e2fa2a5a6a5bc86ff9564ae9336084470e7119dffcb85ae8c1383a3defd7"
    );
    assert!(
        format!("{:?}", z) == "0x1e049436efa754f5f189aec69c2c3a4a559eca6a12b45c3f2e4a769deeca6187"
    );
}
