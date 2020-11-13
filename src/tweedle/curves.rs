//! This module contains implementations for the Tweedledum and Tweedledee
//! elliptic curve groups.

use core::cmp;
use core::fmt::Debug;
use core::ops::{Add, Mul, Neg, Sub};
use ff::Field;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::{Fp, Fq};
use crate::arithmetic::{Curve, CurveAffine, FieldExt, Group};

macro_rules! new_curve_impl {
    ($name:ident, $name_affine:ident, $base:ident, $scalar:ident) => {
        /// Represents a point in the projective coordinate space.
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            x: $base,
            y: $base,
            z: $base,
        }

        impl $name {
            const fn curve_constant_b() -> $base {
                // NOTE: this is specific to b = 5
                $base::from_raw([5, 0, 0, 0])
            }
        }

        /// Represents a point in the affine coordinate space (or the point at
        /// infinity).
        #[derive(Copy, Clone, Debug)]
        pub struct $name_affine {
            x: $base,
            y: $base,
            infinity: Choice,
        }

        impl Curve for $name {
            type Affine = $name_affine;
            type Scalar = $scalar;
            type Base = $base;

            fn zero() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    z: $base::zero(),
                }
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

            /// Apply the curve endomorphism by multiplying the x-coordinate
            /// by an element of multiplicative order 3.
            fn endo(&self) -> Self {
                $name {
                    x: self.x * $base::ZETA,
                    y: self.y,
                    z: self.z,
                }
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }

            fn is_on_curve(&self) -> Choice {
                // Y^2 - X^3 = 5(Z^6)

                (self.y.square() - (self.x.square() * self.x))
                    .ct_eq(&((self.z.square() * self.z).square() * $name::curve_constant_b()))
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

            fn zero() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    infinity: Choice::from(1u8),
                }
            }

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

            fn is_zero(&self) -> Choice {
                self.infinity
            }

            fn is_on_curve(&self) -> Choice {
                // y^2 - x^3 ?= b
                (self.y.square() - (self.x.square() * self.x)).ct_eq(&$name::curve_constant_b())
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

new_curve_impl!(Ep, EpAffine, Fp, Fq);
new_curve_impl!(Eq, EqAffine, Fq, Fp);
