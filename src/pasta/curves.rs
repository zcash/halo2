//! This module contains implementations for the Pallas and Vesta elliptic curve
//! groups.

use core::cmp;
use core::fmt::Debug;
use core::ops::{Add, Mul, Neg, Sub};
use ff::Field;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::{Fp, Fq};
use crate::arithmetic::{Curve, CurveAffine, FieldExt, Group};

macro_rules! new_curve_impl {
    (($($privacy:tt)*), $name:ident, $name_affine:ident, $iso_affine:ident, $base:ident, $scalar:ident, $blake2b_personalization:literal,
     $curve_id:literal, $a_raw:expr, $b_raw:expr, $curve_type:ident) => {
        /// Represents a point in the projective coordinate space.
        #[derive(Copy, Clone, Debug)]
        $($privacy)* struct $name {
            x: $base,
            y: $base,
            z: $base,
        }

        impl $name {
            const fn curve_constant_a() -> $base {
                $base::from_raw($a_raw)
            }

            const fn curve_constant_b() -> $base {
                $base::from_raw($b_raw)
            }
        }

        /// Represents a point in the affine coordinate space (or the point at
        /// infinity).
        #[derive(Copy, Clone)]
        $($privacy)* struct $name_affine {
            x: $base,
            y: $base,
            infinity: Choice,
        }

        impl std::fmt::Debug for $name_affine {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                if self.infinity.into() {
                    write!(f, "Infinity")
                } else {
                    write!(f, "({:?}, {:?})", self.x, self.y)
                }
            }
        }

        impl Curve for $name {
            type Affine = $name_affine;
            type Scalar = $scalar;
            type Base = $base;

            impl_projective_curve_specific!($name, $name_affine, $iso_affine, $base, $curve_type);

            fn zero() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    z: $base::zero(),
                }
            }

            fn is_zero(&self) -> Choice {
                self.z.ct_is_zero()
            }

            fn to_affine(&self) -> Self::Affine {
                let zinv = self.z.invert().unwrap_or($base::zero());
                let zinv2 = zinv.square();
                let x = self.x * zinv2;
                let zinv3 = zinv2 * zinv;
                let y = self.y * zinv3;

                let tmp = $name_affine {
                    x,
                    y,
                    infinity: Choice::from(0u8),
                };

                $name_affine::conditional_select(&tmp, &$name_affine::zero(), zinv.ct_is_zero())
            }

            fn a() -> Self::Base {
                $name::curve_constant_a()
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }

            fn jacobian_coordinates(&self) -> ($base, $base, $base) {
               (self.x, self.y, self.z)
            }

            fn is_on_curve(&self) -> Choice {
                // Y^2 = X^3 + AX(Z^4) + b(Z^6)
                // Y^2 - (X^2 + A(Z^4))X = b(Z^6)

                let z2 = self.z.square();
                let z4 = z2.square();
                let z6 = z4 * z2;
                (self.y.square() - (self.x.square() + $name::curve_constant_a() * z4) * self.x)
                    .ct_eq(&(z6 * $name::curve_constant_b()))
                    | self.z.ct_is_zero()
            }

            fn batch_to_affine(p: &[Self], q: &mut [Self::Affine]) {
                assert_eq!(p.len(), q.len());

                let mut acc = $base::one();
                for (p, q) in p.iter().zip(q.iter_mut()) {
                    // We use the `x` field of $name_affine to store the product
                    // of previous z-coordinates seen.
                    q.x = acc;

                    // We will end up skipping all identities in p
                    acc = $base::conditional_select(&(acc * p.z), &acc, p.is_zero());
                }

                // This is the inverse, as all z-coordinates are nonzero and the ones
                // that are not are skipped.
                acc = acc.invert().unwrap();

                for (p, q) in p.iter().rev().zip(q.iter_mut().rev()) {
                    let skip = p.is_zero();

                    // Compute tmp = 1/z
                    let tmp = q.x * acc;

                    // Cancel out z-coordinate in denominator of `acc`
                    acc = $base::conditional_select(&(acc * p.z), &acc, skip);

                    // Set the coordinates to the correct value
                    let tmp2 = tmp.square();
                    let tmp3 = tmp2 * tmp;

                    q.x = p.x * tmp2;
                    q.y = p.y * tmp3;
                    q.infinity = Choice::from(0u8);

                    *q = $name_affine::conditional_select(&q, &$name_affine::zero(), skip);
                }
            }

            fn new_jacobian(x: Self::Base, y: Self::Base, z: Self::Base) -> CtOption<Self> {
                let p = $name { x, y, z };
                CtOption::new(p, p.is_on_curve())
            }
        }

        impl<'a> From<&'a $name_affine> for $name {
            fn from(p: &'a $name_affine) -> $name {
                p.to_projective()
            }
        }

        impl From<$name_affine> for $name {
            fn from(p: $name_affine) -> $name {
                p.to_projective()
            }
        }

        impl Default for $name {
            fn default() -> $name {
                $name::zero()
            }
        }

        impl ConstantTimeEq for $name {
            fn ct_eq(&self, other: &Self) -> Choice {
                // Is (xz^2, yz^3, z) equal to (x'z'^2, yz'^3, z') when converted to affine?

                let z = other.z.square();
                let x1 = self.x * z;
                let z = z * other.z;
                let y1 = self.y * z;
                let z = self.z.square();
                let x2 = other.x * z;
                let z = z * self.z;
                let y2 = other.y * z;

                let self_is_zero = self.is_zero();
                let other_is_zero = other.is_zero();

                (self_is_zero & other_is_zero) // Both point at infinity
                            | ((!self_is_zero) & (!other_is_zero) & x1.ct_eq(&x2) & y1.ct_eq(&y2))
                // Neither point at infinity, coordinates are the same
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.ct_eq(other).into()
            }
        }

        impl cmp::Eq for $name {}

        impl ConditionallySelectable for $name {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $name {
                    x: $base::conditional_select(&a.x, &b.x, choice),
                    y: $base::conditional_select(&a.y, &b.y, choice),
                    z: $base::conditional_select(&a.z, &b.z, choice),
                }
            }
        }

        impl<'a> Neg for &'a $name {
            type Output = $name;

            fn neg(self) -> $name {
                $name {
                    x: self.x,
                    y: -self.y,
                    z: self.z,
                }
            }
        }

        impl Neg for $name {
            type Output = $name;

            fn neg(self) -> $name {
                -&self
            }
        }

        impl<'a, 'b> Add<&'a $name> for &'b $name {
            type Output = $name;

            fn add(self, rhs: &'a $name) -> $name {
                if bool::from(self.is_zero()) {
                    *rhs
                } else if bool::from(rhs.is_zero()) {
                    *self
                } else {
                    let z1z1 = self.z.square();
                    let z2z2 = rhs.z.square();
                    let u1 = self.x * z2z2;
                    let u2 = rhs.x * z1z1;
                    let s1 = self.y * z2z2 * rhs.z;
                    let s2 = rhs.y * z1z1 * self.z;

                    if u1 == u2 {
                        if s1 == s2 {
                            self.double()
                        } else {
                            $name::zero()
                        }
                    } else {
                        let h = u2 - u1;
                        let i = (h + h).square();
                        let j = h * i;
                        let r = s2 - s1;
                        let r = r + r;
                        let v = u1 * i;
                        let x3 = r.square() - j - v - v;
                        let s1 = s1 * j;
                        let s1 = s1 + s1;
                        let y3 = r * (v - x3) - s1;
                        let z3 = (self.z + rhs.z).square() - z1z1 - z2z2;
                        let z3 = z3 * h;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Add<&'a $name_affine> for &'b $name {
            type Output = $name;

            fn add(self, rhs: &'a $name_affine) -> $name {
                if bool::from(self.is_zero()) {
                    rhs.to_projective()
                } else if bool::from(rhs.is_zero()) {
                    *self
                } else {
                    let z1z1 = self.z.square();
                    let u2 = rhs.x * z1z1;
                    let s2 = rhs.y * z1z1 * self.z;

                    if self.x == u2 {
                        if self.y == s2 {
                            self.double()
                        } else {
                            $name::zero()
                        }
                    } else {
                        let h = u2 - self.x;
                        let hh = h.square();
                        let i = hh + hh;
                        let i = i + i;
                        let j = h * i;
                        let r = s2 - self.y;
                        let r = r + r;
                        let v = self.x * i;
                        let x3 = r.square() - j - v - v;
                        let j = self.y * j;
                        let j = j + j;
                        let y3 = r * (v - x3) - j;
                        let z3 = (self.z + h).square() - z1z1 - hh;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Sub<&'a $name> for &'b $name {
            type Output = $name;

            fn sub(self, other: &'a $name) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Sub<&'a $name_affine> for &'b $name {
            type Output = $name;

            fn sub(self, other: &'a $name_affine) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Mul<&'b $scalar> for &'a $name {
            type Output = $name;

            fn mul(self, other: &'b $scalar) -> Self::Output {
                // TODO: make this faster

                let mut acc = $name::zero();

                // This is a simple double-and-add implementation of point
                // multiplication, moving from most significant to least
                // significant bit of the scalar.
                //
                // NOTE: We skip the leading bit because it's always unset.
                for bit in other
                    .to_bytes()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
                    .skip(1)
                {
                    acc = acc.double();
                    acc = $name::conditional_select(&acc, &(acc + self), bit);
                }

                acc
            }
        }

        impl<'a> Neg for &'a $name_affine {
            type Output = $name_affine;

            fn neg(self) -> $name_affine {
                $name_affine {
                    x: self.x,
                    y: -self.y,
                    infinity: self.infinity,
                }
            }
        }

        impl Neg for $name_affine {
            type Output = $name_affine;

            fn neg(self) -> $name_affine {
                -&self
            }
        }

        impl<'a, 'b> Add<&'a $name> for &'b $name_affine {
            type Output = $name;

            fn add(self, rhs: &'a $name) -> $name {
                rhs + self
            }
        }

        impl<'a, 'b> Add<&'a $name_affine> for &'b $name_affine {
            type Output = $name;

            fn add(self, rhs: &'a $name_affine) -> $name {
                if bool::from(self.is_zero()) {
                    rhs.to_projective()
                } else if bool::from(rhs.is_zero()) {
                    self.to_projective()
                } else {
                    if self.x == rhs.x {
                        if self.y == rhs.y {
                            self.to_projective().double()
                        } else {
                            $name::zero()
                        }
                    } else {
                        let h = rhs.x - self.x;
                        let hh = h.square();
                        let i = hh + hh;
                        let i = i + i;
                        let j = h * i;
                        let r = rhs.y - self.y;
                        let r = r + r;
                        let v = self.x * i;
                        let x3 = r.square() - j - v - v;
                        let j = self.y * j;
                        let j = j + j;
                        let y3 = r * (v - x3) - j;
                        let z3 = h + h;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Sub<&'a $name_affine> for &'b $name_affine {
            type Output = $name;

            fn sub(self, other: &'a $name_affine) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Sub<&'a $name> for &'b $name_affine {
            type Output = $name;

            fn sub(self, other: &'a $name) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Mul<&'b $scalar> for &'a $name_affine {
            type Output = $name;

            fn mul(self, other: &'b $scalar) -> Self::Output {
                // TODO: make this faster

                let mut acc = $name::zero();

                // This is a simple double-and-add implementation of point
                // multiplication, moving from most significant to least
                // significant bit of the scalar.
                //
                // NOTE: We skip the leading bit because it's always unset.
                for bit in other
                    .to_bytes()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
                    .skip(1)
                {
                    acc = acc.double();
                    acc = $name::conditional_select(&acc, &(acc + self), bit);
                }

                acc
            }
        }

        impl CurveAffine for $name_affine {
            type Projective = $name;
            type Scalar = $scalar;
            type Base = $base;

            const BLAKE2B_PERSONALIZATION: &'static [u8; 16] = $blake2b_personalization;
            const CURVE_ID: &'static str = $curve_id;

            impl_affine_curve_specific!($name, $base, $curve_type);

            fn zero() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    infinity: Choice::from(1u8),
                }
            }

            fn is_zero(&self) -> Choice {
                self.infinity
            }

            fn is_on_curve(&self) -> Choice {
                // y^2 - x^3 - ax ?= b
                (self.y.square() - (self.x.square() + &$name::curve_constant_a()) * self.x).ct_eq(&$name::curve_constant_b())
                    | self.infinity
            }

            fn to_projective(&self) -> Self::Projective {
                $name {
                    x: self.x,
                    y: self.y,
                    z: $base::conditional_select(&$base::one(), &$base::zero(), self.infinity),
                }
            }

            fn get_xy(&self) -> CtOption<(Self::Base, Self::Base)> {
                CtOption::new((self.x, self.y), !self.is_zero())
            }

            fn from_xy(x: Self::Base, y: Self::Base) -> CtOption<Self> {
                let p = $name_affine {
                    x, y, infinity: 0u8.into()
                };
                CtOption::new(p, p.is_on_curve())
            }

            fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
                let mut tmp = *bytes;
                let ysign = Choice::from(tmp[31] >> 7);
                tmp[31] &= 0b0111_1111;

                $base::from_bytes(&tmp).and_then(|x| {
                    CtOption::new(Self::zero(), x.ct_is_zero() & (!ysign)).or_else(|| {
                        let x3 = x.square() * x;
                        (x3 + $name::curve_constant_b()).sqrt().and_then(|y| {
                            let sign = Choice::from(y.to_bytes()[0] & 1);

                            let y = $base::conditional_select(&y, &-y, ysign ^ sign);

                            CtOption::new(
                                $name_affine {
                                    x,
                                    y,
                                    infinity: Choice::from(0u8),
                                },
                                Choice::from(1u8),
                            )
                        })
                    })
                })
            }

            fn to_bytes(&self) -> [u8; 32] {
                // TODO: not constant time
                if bool::from(self.is_zero()) {
                    [0; 32]
                } else {
                    let (x, y) = (self.x, self.y);
                    let sign = (y.to_bytes()[0] & 1) << 7;
                    let mut xbytes = x.to_bytes();
                    xbytes[31] |= sign;
                    xbytes
                }
            }

            fn from_bytes_wide(bytes: &[u8; 64]) -> CtOption<Self> {
                let mut xbytes = [0u8; 32];
                let mut ybytes = [0u8; 32];
                xbytes.copy_from_slice(&bytes[0..32]);
                ybytes.copy_from_slice(&bytes[32..64]);

                $base::from_bytes(&xbytes).and_then(|x| {
                    $base::from_bytes(&ybytes).and_then(|y| {
                        CtOption::new(Self::zero(), x.ct_is_zero() & y.ct_is_zero()).or_else(|| {
                            let on_curve =
                                (x * x.square() + $name::curve_constant_b()).ct_eq(&y.square());

                            CtOption::new(
                                $name_affine {
                                    x,
                                    y,
                                    infinity: Choice::from(0u8),
                                },
                                Choice::from(on_curve),
                            )
                        })
                    })
                })
            }

            fn to_bytes_wide(&self) -> [u8; 64] {
                // TODO: not constant time
                if bool::from(self.is_zero()) {
                    [0; 64]
                } else {
                    let mut out = [0u8; 64];
                    (&mut out[0..32]).copy_from_slice(&self.x.to_bytes());
                    (&mut out[32..64]).copy_from_slice(&self.y.to_bytes());

                    out
                }
            }

            fn a() -> Self::Base {
                $name::curve_constant_a()
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }
        }

        impl Default for $name_affine {
            fn default() -> $name_affine {
                $name_affine::zero()
            }
        }

        impl<'a> From<&'a $name> for $name_affine {
            fn from(p: &'a $name) -> $name_affine {
                p.to_affine()
            }
        }

        impl From<$name> for $name_affine {
            fn from(p: $name) -> $name_affine {
                p.to_affine()
            }
        }

        impl ConstantTimeEq for $name_affine {
            fn ct_eq(&self, other: &Self) -> Choice {
                let z1 = self.infinity;
                let z2 = other.infinity;

                (z1 & z2) | ((!z1) & (!z2) & (self.x.ct_eq(&other.x)) & (self.y.ct_eq(&other.y)))
            }
        }

        impl PartialEq for $name_affine {
            fn eq(&self, other: &Self) -> bool {
                self.ct_eq(other).into()
            }
        }

        impl cmp::Eq for $name_affine {}

        impl ConditionallySelectable for $name_affine {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $name_affine {
                    x: $base::conditional_select(&a.x, &b.x, choice),
                    y: $base::conditional_select(&a.y, &b.y, choice),
                    infinity: Choice::conditional_select(&a.infinity, &b.infinity, choice),
                }
            }
        }

        impl_binops_additive!($name, $name);
        impl_binops_additive!($name, $name_affine);
        impl_binops_additive_specify_output!($name_affine, $name_affine, $name);
        impl_binops_additive_specify_output!($name_affine, $name, $name);
        impl_binops_multiplicative!($name, $scalar);
        impl_binops_multiplicative_mixed!($name_affine, $scalar, $name);

        impl Group for $name {
            type Scalar = $scalar;

            fn group_zero() -> Self {
                Self::zero()
            }
            fn group_add(&mut self, rhs: &Self) {
                *self = *self + *rhs;
            }
            fn group_sub(&mut self, rhs: &Self) {
                *self = *self - *rhs;
            }
            fn group_scale(&mut self, by: &Self::Scalar) {
                *self = *self * (*by);
            }
        }
    };
}

macro_rules! impl_projective_curve_specific {
    ($name:ident, $name_affine:ident, $iso_affine:ident, $base:ident, special_a0_b5) => {
        fn hash_to_curve(domain_prefix: &str) -> Box<dyn Fn(&[u8]) -> Self + 'static> {
            use super::hashtocurve;

            let domain_separation_tag: String = format!(
                "{}-{}_{}_{}_RO_",
                domain_prefix,
                $name_affine::CURVE_ID,
                "XMD:BLAKE2b",
                "SSWU"
            );

            Box::new(move |message| {
                let mut us = [Field::zero(); 2];
                hashtocurve::hash_to_field(message, domain_separation_tag.as_bytes(), &mut us);
                let q0 = hashtocurve::map_to_curve_simple_swu::<$base, $name_affine, $iso_affine>(
                    &us[0],
                    $name::THETA,
                    $name::Z,
                );
                let q1 = hashtocurve::map_to_curve_simple_swu::<$base, $name_affine, $iso_affine>(
                    &us[1],
                    $name::THETA,
                    $name::Z,
                );
                let r = q0 + &q1;
                assert!(bool::from(r.is_on_curve()));
                hashtocurve::iso_map::<$base, $name_affine, $iso_affine>(
                    &r,
                    &$name::ISOGENY_CONSTANTS,
                )
            })
        }

        fn one() -> Self {
            // NOTE: This is specific to b = 5

            const NEGATIVE_ONE: $base = $base::neg(&$base::one());
            const TWO: $base = $base::from_raw([2, 0, 0, 0]);

            Self {
                x: NEGATIVE_ONE,
                y: TWO,
                z: $base::one(),
            }
        }

        /// Apply the curve endomorphism by multiplying the x-coordinate
        /// by an element of multiplicative order 3.
        fn endo(&self) -> Self {
            $name {
                x: self.x * $base::ZETA,
                y: self.y,
                z: self.z,
            }
        }

        fn double(&self) -> Self {
            // http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
            //
            // There are no points of order 2.

            let a = self.x.square();
            let b = self.y.square();
            let c = b.square();
            let d = self.x + b;
            let d = d.square();
            let d = d - a - c;
            let d = d + d;
            let e = a + a + a;
            let f = e.square();
            let z3 = self.z * self.y;
            let z3 = z3 + z3;
            let x3 = f - (d + d);
            let c = c + c;
            let c = c + c;
            let c = c + c;
            let y3 = e * (d - x3) - c;

            let tmp = $name {
                x: x3,
                y: y3,
                z: z3,
            };

            $name::conditional_select(&tmp, &$name::zero(), self.is_zero())
        }
    };
    ($name:ident, $name_affine:ident, $iso_affine:ident, $base:ident, general) => {
        /// Unimplemented: hashing to this curve is not supported
        fn hash_to_curve(_domain_prefix: &str) -> Box<dyn Fn(&[u8]) -> Self + 'static> {
            unimplemented!()
        }

        /// Unimplemented: there is no standard generator for this curve.
        fn one() -> Self {
            unimplemented!()
        }

        /// Unimplemented: no endomorphism is supported for this curve.
        fn endo(&self) -> Self {
            unimplemented!()
        }

        fn double(&self) -> Self {
            // http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian.html#doubling-dbl-2007-bl
            //
            // There are no points of order 2.

            let xx = self.x.square();
            let yy = self.y.square();
            let a = yy.square();
            let zz = self.z.square();
            let s = ((self.x + yy).square() - xx - a).double();
            let m = xx.double() + xx + $name::curve_constant_a() * zz.square();
            let x3 = m.square() - s.double();
            let a = a.double();
            let a = a.double();
            let a = a.double();
            let y3 = m * (s - x3) - a;
            let z3 = (self.y + self.z).square() - yy - zz;

            let tmp = $name {
                x: x3,
                y: y3,
                z: z3,
            };

            $name::conditional_select(&tmp, &$name::zero(), self.is_zero())
        }
    };
}

macro_rules! impl_affine_curve_specific {
    ($name:ident, $base:ident, special_a0_b5) => {
        fn one() -> Self {
            // NOTE: This is specific to b = 5

            const NEGATIVE_ONE: $base = $base::neg(&$base::from_raw([1, 0, 0, 0]));
            const TWO: $base = $base::from_raw([2, 0, 0, 0]);

            Self {
                x: NEGATIVE_ONE,
                y: TWO,
                infinity: Choice::from(0u8),
            }
        }
    };
    ($name:ident, $base:ident, general) => {
        /// Unimplemented: there is no standard generator for this curve.
        fn one() -> Self {
            unimplemented!()
        }
    };
}

new_curve_impl!(
    (pub),
    Ep,
    EpAffine,
    IsoEpAffine,
    Fp,
    Fq,
    b"halo2_____pallas",
    "pallas",
    [0, 0, 0, 0],
    [5, 0, 0, 0],
    special_a0_b5
);
new_curve_impl!(
    (pub),
    Eq,
    EqAffine,
    IsoEqAffine,
    Fq,
    Fp,
    b"halo2______vesta",
    "vesta",
    [0, 0, 0, 0],
    [5, 0, 0, 0],
    special_a0_b5
);
new_curve_impl!(
    (pub(crate)),
    IsoEp,
    IsoEpAffine,
    EpAffine,
    Fp,
    Fq,
    b"halo2_iso_pallas",
    "iso-pallas",
    [
        0x92bb4b0b657a014b,
        0xb74134581a27a59f,
        0x49be2d7258370742,
        0x18354a2eb0ea8c9c,
    ],
    [1265, 0, 0, 0],
    general
);
new_curve_impl!(
    (pub(crate)),
    IsoEq,
    IsoEqAffine,
    EqAffine,
    Fq,
    Fp,
    b"halo2__iso_vesta",
    "iso-vesta",
    [
        0xc515ad7242eaa6b1,
        0x9673928c7d01b212,
        0x81639c4d96f78773,
        0x267f9b2ee592271a,
    ],
    [1265, 0, 0, 0],
    general
);

impl Ep {
    /// Constants used for computing the isogeny from IsoEp to Ep.
    pub const ISOGENY_CONSTANTS: [Fp; 13] = [
        Fp::from_raw([
            0x775f6034aaaaaaab,
            0x4081775473d8375b,
            0xe38e38e38e38e38e,
            0x0e38e38e38e38e38,
        ]),
        Fp::from_raw([
            0x8cf863b02814fb76,
            0x0f93b82ee4b99495,
            0x267c7ffa51cf412a,
            0x3509afd51872d88e,
        ]),
        Fp::from_raw([
            0x0eb64faef37ea4f7,
            0x380af066cfeb6d69,
            0x98c7d7ac3d98fd13,
            0x17329b9ec5253753,
        ]),
        Fp::from_raw([
            0xeebec06955555580,
            0x8102eea8e7b06eb6,
            0xc71c71c71c71c71c,
            0x1c71c71c71c71c71,
        ]),
        Fp::from_raw([
            0xc47f2ab668bcd71f,
            0x9c434ac1c96b6980,
            0x5a607fcce0494a79,
            0x1d572e7ddc099cff,
        ]),
        Fp::from_raw([
            0x2aa3af1eae5b6604,
            0xb4abf9fb9a1fc81c,
            0x1d13bf2a7f22b105,
            0x325669becaecd5d1,
        ]),
        Fp::from_raw([
            0x5ad985b5e38e38e4,
            0x7642b01ad461bad2,
            0x4bda12f684bda12f,
            0x1a12f684bda12f68,
        ]),
        Fp::from_raw([
            0xc67c31d8140a7dbb,
            0x07c9dc17725cca4a,
            0x133e3ffd28e7a095,
            0x1a84d7ea8c396c47,
        ]),
        Fp::from_raw([
            0x02e2be87d225b234,
            0x1765e924f7459378,
            0x303216cce1db9ff1,
            0x3fb98ff0d2ddcadd,
        ]),
        Fp::from_raw([
            0x93e53ab371c71c4f,
            0x0ac03e8e134eb3e4,
            0x7b425ed097b425ed,
            0x025ed097b425ed09,
        ]),
        Fp::from_raw([
            0x5a28279b1d1b42ae,
            0x5941a3a4a97aa1b3,
            0x0790bfb3506defb6,
            0x0c02c5bcca0e6b7f,
        ]),
        Fp::from_raw([
            0x4d90ab820b12320a,
            0xd976bbfabbc5661d,
            0x573b3d7f7d681310,
            0x17033d3c60c68173,
        ]),
        Fp::from_raw([
            0x992d30ecfffffde5,
            0x224698fc094cf91b,
            0x0000000000000000,
            0x4000000000000000,
        ]),
    ];

    /// Z = -13
    pub const Z: Fp = Fp::from_raw([
        0x992d30ecfffffff4,
        0x224698fc094cf91b,
        0x0000000000000000,
        0x4000000000000000,
    ]);

    /// `(F::ROOT_OF_UNITY.invert().unwrap() * z).sqrt().unwrap()`
    pub const THETA: Fp = Fp::from_raw([
        0xca330bcc09ac318e,
        0x51f64fc4dc888857,
        0x4647aef782d5cdc8,
        0x0f7bdb65814179b4,
    ]);
}

impl Eq {
    /// Constants used for computing the isogeny from IsoEq to Eq.
    pub const ISOGENY_CONSTANTS: [Fq; 13] = [
        Fq::from_raw([
            0x43cd42c800000001,
            0x0205dd51cfa0961a,
            0x8e38e38e38e38e39,
            0x38e38e38e38e38e3,
        ]),
        Fq::from_raw([
            0x8b95c6aaf703bcc5,
            0x216b8861ec72bd5d,
            0xacecf10f5f7c09a2,
            0x1d935247b4473d17,
        ]),
        Fq::from_raw([
            0xaeac67bbeb586a3d,
            0xd59d03d23b39cb11,
            0xed7ee4a9cdf78f8f,
            0x18760c7f7a9ad20d,
        ]),
        Fq::from_raw([
            0xfb539a6f0000002b,
            0xe1c521a795ac8356,
            0x1c71c71c71c71c71,
            0x31c71c71c71c71c7,
        ]),
        Fq::from_raw([
            0xb7284f7eaf21a2e9,
            0xa3ad678129b604d3,
            0x1454798a5b5c56b2,
            0x0a2de485568125d5,
        ]),
        Fq::from_raw([
            0xf169c187d2533465,
            0x30cd6d53df49d235,
            0x0c621de8b91c242a,
            0x14735171ee542778,
        ]),
        Fq::from_raw([
            0x6bef1642aaaaaaab,
            0x5601f4709a8adcb3,
            0xda12f684bda12f68,
            0x12f684bda12f684b,
        ]),
        Fq::from_raw([
            0x8bee58e5fb81de63,
            0x21d910aefb03b31d,
            0xd6767887afbe04d1,
            0x2ec9a923da239e8b,
        ]),
        Fq::from_raw([
            0x4986913ab4443034,
            0x97a3ca5c24e9ea63,
            0x66d1466e9de10e64,
            0x19b0d87e16e25788,
        ]),
        Fq::from_raw([
            0x8f64842c55555533,
            0x8bc32d36fb21a6a3,
            0x425ed097b425ed09,
            0x1ed097b425ed097b,
        ]),
        Fq::from_raw([
            0x58dfecce86b2745e,
            0x06a767bfc35b5bac,
            0x9e7eb64f890a820c,
            0x2f44d6c801c1b8bf,
        ]),
        Fq::from_raw([
            0xd43d449776f99d2f,
            0x926847fb9ddd76a1,
            0x252659ba2b546c7e,
            0x3d59f455cafc7668,
        ]),
        Fq::from_raw([
            0x8c46eb20fffffde5,
            0x224698fc0994a8dd,
            0x0000000000000000,
            0x4000000000000000,
        ]),
    ];

    /// Z = -13
    pub const Z: Fq = Fq::from_raw([
        0x8c46eb20fffffff4,
        0x224698fc0994a8dd,
        0x0000000000000000,
        0x4000000000000000,
    ]);

    /// `(F::ROOT_OF_UNITY.invert().unwrap() * z).sqrt().unwrap()`
    pub const THETA: Fq = Fq::from_raw([
        0x632cae9872df1b5d,
        0x38578ccadf03ac27,
        0x53c3808d9e2f2357,
        0x2b3483a1ee9a382f,
    ]);
}
