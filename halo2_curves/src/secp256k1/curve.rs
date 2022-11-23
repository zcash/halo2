use crate::secp256k1::Fp;
use crate::secp256k1::Fq;
use crate::{Coordinates, CurveAffine, CurveAffineExt, CurveExt, Group};
use core::cmp;
use core::fmt::Debug;
use core::iter::Sum;
use core::ops::{Add, Mul, Neg, Sub};
use ff::{Field, PrimeField};
use group::Curve;
use group::{prime::PrimeCurveAffine, Group as _, GroupEncoding};

use rand::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

impl Secp256k1 {
    fn endomorphism_base(&self) -> Self {
        unimplemented!();
    }
}

impl group::cofactor::CofactorGroup for Secp256k1 {
    type Subgroup = Secp256k1;

    fn clear_cofactor(&self) -> Self {
        *self
    }

    fn into_subgroup(self) -> CtOption<Self::Subgroup> {
        CtOption::new(self, 1.into())
    }

    fn is_torsion_free(&self) -> Choice {
        1.into()
    }
}

// Reference: https://neuromancer.sk/std/secg/secp256k1
const SECP_GENERATOR_X: Fp = Fp::from_raw([
    0x59F2815B16F81798,
    0x029BFCDB2DCE28D9,
    0x55A06295CE870B07,
    0x79BE667EF9DCBBAC,
]);
const SECP_GENERATOR_Y: Fp = Fp::from_raw([
    0x9C47D08FFB10D4B8,
    0xFD17B448A6855419,
    0x5DA4FBFC0E1108A8,
    0x483ADA7726A3C465,
]);
const SECP_B: Fp = Fp::from_raw([7, 0, 0, 0]);

use crate::{
    batch_add, impl_add_binop_specify_output, impl_binops_additive,
    impl_binops_additive_specify_output, impl_binops_multiplicative,
    impl_binops_multiplicative_mixed, impl_sub_binop_specify_output, new_curve_impl,
};

// macro_rules! new_curve_impl {
//     (($($privacy:tt)*),
//     $name:ident,
//     $name_affine:ident,
//     $name_compressed:ident,
//     $base:ident,
//     $scalar:ident,
//     $generator:expr,
//     $constant_b:expr,
//     $curve_id:literal,
//     )

new_curve_impl!(
    (pub),
    Secp256k1,
    Secp256k1Affine,
    Fp,
    Fq,
    (SECP_GENERATOR_X,SECP_GENERATOR_Y),
    SECP_B,
    "secp256k1",
);

impl CurveAffineExt for Secp256k1Affine {
    batch_add!();
}

#[test]
fn test_curve() {
    crate::tests::curve::curve_tests::<Secp256k1>();
}

#[test]
fn ecdsa_example() {
    use crate::group::Curve;
    use crate::{CurveAffine, FieldExt};
    use rand_core::OsRng;

    fn mod_n(x: Fp) -> Fq {
        let mut x_repr = [0u8; 32];
        x_repr.copy_from_slice(x.to_repr().as_ref());
        let mut x_bytes = [0u8; 64];
        x_bytes[..32].copy_from_slice(&x_repr[..]);
        Fq::from_bytes_wide(&x_bytes)
    }

    let g = Secp256k1::generator();

    for _ in 0..1000 {
        // Generate a key pair
        let sk = Fq::random(OsRng);
        let pk = (g * sk).to_affine();

        // Generate a valid signature
        // Suppose `m_hash` is the message hash
        let msg_hash = Fq::random(OsRng);

        let (r, s) = {
            // Draw arandomness
            let k = Fq::random(OsRng);
            let k_inv = k.invert().unwrap();

            // Calculate `r`
            let r_point = (g * k).to_affine().coordinates().unwrap();
            let x = r_point.x();
            let r = mod_n(*x);

            // Calculate `s`
            let s = k_inv * (msg_hash + (r * sk));

            (r, s)
        };

        {
            // Verify
            let s_inv = s.invert().unwrap();
            let u_1 = msg_hash * s_inv;
            let u_2 = r * s_inv;

            let v_1 = g * u_1;
            let v_2 = pk * u_2;

            let r_point = (v_1 + v_2).to_affine().coordinates().unwrap();
            let x_candidate = r_point.x();
            let r_candidate = mod_n(*x_candidate);

            assert_eq!(r, r_candidate);
        }
    }
}
