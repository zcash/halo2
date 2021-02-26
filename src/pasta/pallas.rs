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
    use crate::arithmetic::CurveExt;
    use group::Group;

    // This is a regression test (it's the same input to iso_map as for hash_to_curve
    // with domain prefix "z.cash:test", Shake128, and input b"hello"). We don't
    // implement Shake128 any more but that's fine.
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
    let p = super::hashtocurve::iso_map::<_, Point, super::IsoEp>(&r, &Ep::ISOGENY_CONSTANTS);
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

    // check that iso_map([2] r) = [2] iso_map(r)
    let r2 = r.double();
    assert!(bool::from(r2.is_on_curve()));
    let p2 = super::hashtocurve::iso_map::<_, Point, super::IsoEp>(&r2, &Ep::ISOGENY_CONSTANTS);
    assert!(bool::from(p2.is_on_curve()));
    assert!(p2 == p.double());
}

#[test]
fn test_iso_map_identity() {
    use crate::arithmetic::CurveExt;
    use group::Group;

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
    assert!(bool::from(r.is_identity()));
    let p = super::hashtocurve::iso_map::<_, Point, super::IsoEp>(&r, &Ep::ISOGENY_CONSTANTS);
    assert!(bool::from(p.is_on_curve()));
    assert!(bool::from(p.is_identity()));
}

#[test]
fn test_map_to_curve_simple_swu() {
    use crate::arithmetic::CurveExt;
    use crate::pasta::curves::IsoEp;
    use crate::pasta::hashtocurve::map_to_curve_simple_swu;

    // The zero input is a special case.
    let p: IsoEp = map_to_curve_simple_swu::<Fp, Ep, IsoEp>(&Fp::zero(), Ep::THETA, Ep::Z);
    let (x, y, z) = p.jacobian_coordinates();

    assert!(
        format!("{:?}", x) == "0x28c1a6a534f56c52e25295b339129a8af5f42525dea727f485ca3433519b096e"
    );
    assert!(
        format!("{:?}", y) == "0x3bfc658bee6653c63c7d7f0927083fd315d29c270207b7c7084fa1ee6ac5ae8d"
    );
    assert!(
        format!("{:?}", z) == "0x054b3ba10416dc104157b1318534a19d5d115472da7d746f8a5f250cd8cdef36"
    );

    let p: IsoEp = map_to_curve_simple_swu::<Fp, Ep, IsoEp>(&Fp::one(), Ep::THETA, Ep::Z);
    let (x, y, z) = p.jacobian_coordinates();

    assert!(
        format!("{:?}", x) == "0x010cba5957e876534af5e967c026a1856d64b071068280837913b9a5a3561505"
    );
    assert!(
        format!("{:?}", y) == "0x062fc61f9cd3118e7d6e65a065ebf46a547514d6b08078e976fa6d515dcc9c81"
    );
    assert!(
        format!("{:?}", z) == "0x3f86cb8c311250c3101c4e523e7793605ccff5623de1753a7c75bc9a29a73688"
    );
}

#[test]
fn test_hash_to_curve() {
    use crate::arithmetic::CurveExt;
    use group::Group;

    // This test vector is chosen so that the first map_to_curve_simple_swu takes the gx1 square
    // "branch" and the second takes the gx1 non-square "branch" (opposite to the Vesta test vector).
    let hash = Point::hash_to_curve("z.cash:test");
    let p: Point = hash(b"world");
    let (x, y, z) = p.jacobian_coordinates();

    assert!(
        format!("{:?}", x) == "0x2ae2d9bde5a5b4bc1f1e7154f18a407ac826c9d7cd23c3b33efa0f237e99cd35"
    );
    assert!(
        format!("{:?}", y) == "0x3ca16b5bf2e6c41cdf781ead8ba61400becbc16430d026b65b707560b98f8b31"
    );
    assert!(
        format!("{:?}", z) == "0x2502d25cc3b1129d933af3ac34822111bfd070609fdebdfb778dd25cf40f9b82"
    );
    assert!(bool::from(p.is_on_curve()));

    let p = (p * -Fq::one()) + p;
    assert!(bool::from(p.is_on_curve()));
    assert!(bool::from(p.is_identity()));
}
