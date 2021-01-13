//! The Vesta and iso-Vesta elliptic curve groups.

use lazy_static::lazy_static;

use super::{Eq, EqAffine, Fp, Fq, IsoEq, IsoEqAffine};
use crate::arithmetic::{FieldExt, SimplifiedSWUWithDegree3Isogeny};

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
    pub static ref MAP: SimplifiedSWUWithDegree3Isogeny<Base, IsoAffine, Affine> = {
        let isogeny_constants: [Base; 13] = [
            Base::from_raw([
                0x43cd42c800000001,
                0x0205dd51cfa0961a,
                0x8e38e38e38e38e39,
                0x38e38e38e38e38e3,
            ]),
            Base::from_raw([
                0x8b95c6aaf703bcc5,
                0x216b8861ec72bd5d,
                0xacecf10f5f7c09a2,
                0x1d935247b4473d17,
            ]),
            Base::from_raw([
                0xaeac67bbeb586a3d,
                0xd59d03d23b39cb11,
                0xed7ee4a9cdf78f8f,
                0x18760c7f7a9ad20d,
            ]),
            Base::from_raw([
                0xfb539a6f0000002b,
                0xe1c521a795ac8356,
                0x1c71c71c71c71c71,
                0x31c71c71c71c71c7,
            ]),
            Base::from_raw([
                0xb7284f7eaf21a2e9,
                0xa3ad678129b604d3,
                0x1454798a5b5c56b2,
                0x0a2de485568125d5,
            ]),
            Base::from_raw([
                0xf169c187d2533465,
                0x30cd6d53df49d235,
                0x0c621de8b91c242a,
                0x14735171ee542778,
            ]),
            Base::from_raw([
                0x6bef1642aaaaaaab,
                0x5601f4709a8adcb3,
                0xda12f684bda12f68,
                0x12f684bda12f684b,
            ]),
            Base::from_raw([
                0x8bee58e5fb81de63,
                0x21d910aefb03b31d,
                0xd6767887afbe04d1,
                0x2ec9a923da239e8b,
            ]),
            Base::from_raw([
                0x4986913ab4443034,
                0x97a3ca5c24e9ea63,
                0x66d1466e9de10e64,
                0x19b0d87e16e25788,
            ]),
            Base::from_raw([
                0x8f64842c55555533,
                0x8bc32d36fb21a6a3,
                0x425ed097b425ed09,
                0x1ed097b425ed097b,
            ]),
            Base::from_raw([
                0x58dfecce86b2745e,
                0x06a767bfc35b5bac,
                0x9e7eb64f890a820c,
                0x2f44d6c801c1b8bf,
            ]),
            Base::from_raw([
                0xd43d449776f99d2f,
                0x926847fb9ddd76a1,
                0x252659ba2b546c7e,
                0x3d59f455cafc7668,
            ]),
            Base::from_raw([
                0x8c46eb20fffffde5,
                0x224698fc0994a8dd,
                0x0000000000000000,
                0x4000000000000000,
            ]),
        ];

        let z = -Base::from_u64(13);
        SimplifiedSWUWithDegree3Isogeny::new(&z, &isogeny_constants)
    };
}

#[test]
fn test_map_to_curve_vesta() {
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
        format!("{:?}", x) == "0x3984612258b3b43b4f6e046f7f796bbd35ffd8908804bcf47b9537d3ec7645c9"
    );
    assert!(
        format!("{:?}", y) == "0x2573c035293d745a288a65a7a37709ef99bcf31b77cfb3a1126a61e3adeebc4b"
    );
    assert!(
        format!("{:?}", z) == "0x1cb99da94a634842b09a3ee1e5b462233e1fc23d0b357ec7fb0d1c409be30720"
    );
}
