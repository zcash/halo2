//! The Pallas and iso-Pallas elliptic curve groups.

use super::{Ep, EpAffine, Fp, Fq};

/// The base field of the Pallas and iso-Pallas curves.
pub type Base = Fp;

/// The scalar field of the Pallas and iso-Pallas curves.
pub type Scalar = Fq;

/// A Pallas point in the projective coordinate space.
pub type Point = Ep;

/// A Pallas point in the affine coordinate space (or the point at infinity).
pub type Affine = EpAffine;

#[test]
fn test_iso_map() {
    use crate::arithmetic::Curve;

    // This is a regression test (it's the same input to iso_map as for hash_to_curve
    // with domain prefix "z.cash:test", Shake128, and input b"hello").
    let r = super::IsoEp::new_jacobian(
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
    let p =
        super::hashtocurve::iso_map::<_, Affine, super::IsoEpAffine>(&r, &Ep::ISOGENY_CONSTANTS);
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
fn test_iso_map_identity() {
    use crate::arithmetic::Curve;

    let r = super::IsoEp::new_jacobian(
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
    let r = (r * -Fq::one()) + r;
    assert!(bool::from(r.is_on_curve()));
    let p =
        super::hashtocurve::iso_map::<_, Affine, super::IsoEpAffine>(&r, &Ep::ISOGENY_CONSTANTS);
    assert!(bool::from(p.is_on_curve()));
}

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
    assert!(bool::from(p.is_on_curve()));
    let p = (p * -Fq::one()) + p;
    assert!(bool::from(p.is_on_curve()));
}
