//! The Pallas and iso-Pallas elliptic curve groups.

use lazy_static::lazy_static;

use super::{Ep, EpAffine, Fp, Fq, IsoEp, IsoEpAffine};
use crate::arithmetic::{FieldExt, SimplifiedSWUWithDegree3Isogeny};

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
    pub static ref MAP: SimplifiedSWUWithDegree3Isogeny<Base, IsoAffine, Affine> = {
        let isogeny_constants: [Base; 13] = [
            Base::from_raw([
                0x775f6034aaaaaaab,
                0x4081775473d8375b,
                0xe38e38e38e38e38e,
                0x0e38e38e38e38e38,
            ]),
            Base::from_raw([
                0x8cf863b02814fb76,
                0x0f93b82ee4b99495,
                0x267c7ffa51cf412a,
                0x3509afd51872d88e,
            ]),
            Base::from_raw([
                0x0eb64faef37ea4f7,
                0x380af066cfeb6d69,
                0x98c7d7ac3d98fd13,
                0x17329b9ec5253753,
            ]),
            Base::from_raw([
                0xeebec06955555580,
                0x8102eea8e7b06eb6,
                0xc71c71c71c71c71c,
                0x1c71c71c71c71c71,
            ]),
            Base::from_raw([
                0xc47f2ab668bcd71f,
                0x9c434ac1c96b6980,
                0x5a607fcce0494a79,
                0x1d572e7ddc099cff,
            ]),
            Base::from_raw([
                0x2aa3af1eae5b6604,
                0xb4abf9fb9a1fc81c,
                0x1d13bf2a7f22b105,
                0x325669becaecd5d1,
            ]),
            Base::from_raw([
                0x5ad985b5e38e38e4,
                0x7642b01ad461bad2,
                0x4bda12f684bda12f,
                0x1a12f684bda12f68,
            ]),
            Base::from_raw([
                0xc67c31d8140a7dbb,
                0x07c9dc17725cca4a,
                0x133e3ffd28e7a095,
                0x1a84d7ea8c396c47,
            ]),
            Base::from_raw([
                0x02e2be87d225b234,
                0x1765e924f7459378,
                0x303216cce1db9ff1,
                0x3fb98ff0d2ddcadd,
            ]),
            Base::from_raw([
                0x93e53ab371c71c4f,
                0x0ac03e8e134eb3e4,
                0x7b425ed097b425ed,
                0x025ed097b425ed09,
            ]),
            Base::from_raw([
                0x5a28279b1d1b42ae,
                0x5941a3a4a97aa1b3,
                0x0790bfb3506defb6,
                0x0c02c5bcca0e6b7f,
            ]),
            Base::from_raw([
                0x4d90ab820b12320a,
                0xd976bbfabbc5661d,
                0x573b3d7f7d681310,
                0x17033d3c60c68173,
            ]),
            Base::from_raw([
                0x992d30ecfffffde5,
                0x224698fc094cf91b,
                0x0000000000000000,
                0x4000000000000000,
            ]),
        ];

        let z = -Base::from_u64(13);
        SimplifiedSWUWithDegree3Isogeny::new(&z, &isogeny_constants)
    };
}

#[test]
fn test_iso_map() {
    use crate::arithmetic::{Curve, HashToCurve};

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
    use crate::arithmetic::{Curve, CurveAffine, HashToCurve, Shake128};
    use std::collections::HashSet;

    assert!(MAP.minus_b_over_a * IsoAffine::a() == -IsoAffine::b());
    assert!(MAP.b_over_za * MAP.z * IsoAffine::a() == IsoAffine::b());
    assert!(MAP.theta.square() * Base::ROOT_OF_UNITY == MAP.z);

    let set: HashSet<_> = (0..10000)
        .map(|i| MAP.map_to_curve(&Base::from(i)).to_affine())
        .collect();
    assert!(set.len() == 10000);

    let hash = MAP.hash_to_curve("z.cash:test", Shake128::default());
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
