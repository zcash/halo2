use core::cmp;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, Mul, Neg, Sub};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use ff::{Field, PrimeField};
use group::{
    cofactor::{CofactorCurve, CofactorGroup},
    prime::{PrimeCurve, PrimeCurveAffine, PrimeGroup},
    Curve as _, Group as _, GroupEncoding,
};
use rand::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::fields::{Fp, Fq};
use pasta_curves::arithmetic::{FieldExt, Group};

#[cfg(feature = "alloc")]
use pasta_curves::arithmetic::{Coordinates, CurveAffine, CurveExt};

// Reference: https://neuromancer.sk/std/secg/secp256k1
macro_rules! new_curve_impl {
    (($($privacy:tt)*), $name:ident, $name_affine:ident, $name_compressed:ident, $iso:ident, $base:ident, $scalar:ident,
     $curve_id:literal, $a_raw:expr, $b_raw:expr, $curve_type:ident) => {
        /// Represents a point in the projective coordinate space.
        #[derive(Copy, Clone, Debug)]
        #[cfg_attr(feature = "repr-c", repr(C))]
        $($privacy)* struct $name {
            $($privacy)* x: $base,
            $($privacy)* y: $base,
            $($privacy)* z: $base,
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
        #[cfg_attr(feature = "repr-c", repr(C))]
        $($privacy)* struct $name_affine {
            x: $base,
            y: $base,
        }

        #[derive(Copy, Clone)]
        $($privacy)* struct $name_compressed([u8; 33]);

        impl fmt::Debug for $name_affine {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                if self.is_identity().into() {
                    write!(f, "Infinity")
                } else {
                    write!(f, "({:?}, {:?})", self.x, self.y)
                }
            }
        }

        impl group::Group for $name {
            type Scalar = $scalar;

            fn random(mut rng: impl RngCore) -> Self {
                loop {
                    let x = $base::random(&mut rng);
                    let ysign = (rng.next_u32() % 2) as u8;

                    let x3 = x.square() * x;
                    let y = (x3 + $name::curve_constant_b()).sqrt();
                    if let Some(y) = Option::<$base>::from(y) {
                        let sign = y.is_odd().unwrap_u8();
                        let y = if ysign ^ sign == 0 { y } else { -y };

                        let p = $name_affine {
                            x,
                            y,
                        };
                        break p.to_curve();
                    }
                }
            }

            impl_projective_curve_specific!($name, $base, $curve_type);

            fn identity() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    z: $base::zero(),
                }
            }

            fn is_identity(&self) -> Choice {
                self.z.is_zero()
            }
        }

        impl std::fmt::Debug for $name_compressed {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0[..].fmt(f)
            }
        }

        impl Default for $name_compressed {
            fn default() -> Self {
                $name_compressed([0; 33])
            }
        }

        impl AsRef<[u8]> for $name_compressed {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsMut<[u8]> for $name_compressed {
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }


        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        impl CurveExt for $name {
            type ScalarExt = $scalar;
            type Base = $base;
            type AffineExt = $name_affine;

            const CURVE_ID: &'static str = $curve_id;

            impl_projective_curve_ext!($name, $iso, $base, $curve_type);

            fn a() -> Self::Base {
                $name::curve_constant_a()
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }

            fn new_jacobian(x: Self::Base, y: Self::Base, z: Self::Base) -> CtOption<Self> {
                let p = $name { x, y, z };
                CtOption::new(p, p.is_on_curve())
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
                    | self.z.is_zero()
            }
        }

        impl group::Curve for $name {
            type AffineRepr = $name_affine;

            fn batch_normalize(p: &[Self], q: &mut [Self::AffineRepr]) {
                assert_eq!(p.len(), q.len());

                let mut acc = $base::one();
                for (p, q) in p.iter().zip(q.iter_mut()) {
                    // We use the `x` field of $name_affine to store the product
                    // of previous z-coordinates seen.
                    q.x = acc;

                    // We will end up skipping all identities in p
                    acc = $base::conditional_select(&(acc * p.z), &acc, p.is_identity());
                }

                // This is the inverse, as all z-coordinates are nonzero and the ones
                // that are not are skipped.
                acc = acc.invert().unwrap();

                for (p, q) in p.iter().rev().zip(q.iter_mut().rev()) {
                    let skip = p.is_identity();

                    // Compute tmp = 1/z
                    let tmp = q.x * acc;

                    // Cancel out z-coordinate in denominator of `acc`
                    acc = $base::conditional_select(&(acc * p.z), &acc, skip);

                    // Set the coordinates to the correct value
                    let tmp2 = tmp.square();
                    let tmp3 = tmp2 * tmp;

                    q.x = p.x * tmp2;
                    q.y = p.y * tmp3;

                    *q = $name_affine::conditional_select(&q, &$name_affine::identity(), skip);
                }
            }

            fn to_affine(&self) -> Self::AffineRepr {
                let zinv = self.z.invert().unwrap_or($base::zero());
                let zinv2 = zinv.square();
                let x = self.x * zinv2;
                let zinv3 = zinv2 * zinv;
                let y = self.y * zinv3;

                let tmp = $name_affine {
                    x,
                    y,
                };

                $name_affine::conditional_select(&tmp, &$name_affine::identity(), zinv.is_zero())
            }
        }

        impl PrimeGroup for $name {}

        impl CofactorGroup for $name {
            type Subgroup = $name;

            fn clear_cofactor(&self) -> Self {
                // This is a prime-order group, with a cofactor of 1.
                *self
            }

            fn into_subgroup(self) -> CtOption<Self::Subgroup> {
                // Nothing to do here.
                CtOption::new(self, 1.into())
            }

            fn is_torsion_free(&self) -> Choice {
                // Shortcut: all points in a prime-order group are torsion free.
                1.into()
            }
        }

        impl PrimeCurve for $name {
            type Affine = $name_affine;
        }

        impl CofactorCurve for $name {
            type Affine = $name_affine;
        }

        impl GroupEncoding for $name {
            type Repr = $name_compressed;

            fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
                $name_affine::from_bytes(bytes).map(Self::from)
            }

            fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
                // We can't avoid curve checks when parsing a compressed encoding.
                $name_affine::from_bytes(bytes).map(Self::from)
            }

            fn to_bytes(&self) -> Self::Repr {
                $name_affine::from(self).to_bytes()
            }
        }

        impl<'a> From<&'a $name_affine> for $name {
            fn from(p: &'a $name_affine) -> $name {
                p.to_curve()
            }
        }

        impl From<$name_affine> for $name {
            fn from(p: $name_affine) -> $name {
                p.to_curve()
            }
        }

        impl Default for $name {
            fn default() -> $name {
                $name::identity()
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

                let self_is_zero = self.is_identity();
                let other_is_zero = other.is_identity();

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

        impl<T> Sum<T> for $name
        where
            T: core::borrow::Borrow<$name>,
        {
            fn sum<I>(iter: I) -> Self
            where
                I: Iterator<Item = T>,
            {
                iter.fold(Self::identity(), |acc, item| acc + item.borrow())
            }
        }

        impl<'a, 'b> Add<&'a $name> for &'b $name {
            type Output = $name;

            fn add(self, rhs: &'a $name) -> $name {
                if bool::from(self.is_identity()) {
                    *rhs
                } else if bool::from(rhs.is_identity()) {
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
                            $name::identity()
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
                if bool::from(self.is_identity()) {
                    rhs.to_curve()
                } else if bool::from(rhs.is_identity()) {
                    *self
                } else {
                    let z1z1 = self.z.square();
                    let u2 = rhs.x * z1z1;
                    let s2 = rhs.y * z1z1 * self.z;

                    if self.x == u2 {
                        if self.y == s2 {
                            self.double()
                        } else {
                            $name::identity()
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

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a, 'b> Mul<&'b $scalar> for &'a $name {
            type Output = $name;

            fn mul(self, other: &'b $scalar) -> Self::Output {
                // TODO: make this faster

                let mut acc = $name::identity();

                // This is a simple double-and-add implementation of point
                // multiplication, moving from most significant to least
                // significant bit of the scalar.
                //
                // We don't use `PrimeFieldBits::.to_le_bits` here, because that would
                // force users of this crate to depend on `bitvec` where they otherwise
                // might not need to.
                //
                // NOTE: We skip the leading bit because it's always unset (we are turning
                // the 32-byte repr into 256 bits, and $scalar::NUM_BITS = 255).
                for bit in other
                    .to_repr()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
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
                if bool::from(self.is_identity()) {
                    rhs.to_curve()
                } else if bool::from(rhs.is_identity()) {
                    self.to_curve()
                } else {
                    if self.x == rhs.x {
                        if self.y == rhs.y {
                            self.to_curve().double()
                        } else {
                            $name::identity()
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

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a, 'b> Mul<&'b $scalar> for &'a $name_affine {
            type Output = $name;

            fn mul(self, other: &'b $scalar) -> Self::Output {
                // TODO: make this faster

                let mut acc = $name::identity();

                // This is a simple double-and-add implementation of point
                // multiplication, moving from most significant to least
                // significant bit of the scalar.
                //
                // We don't use `PrimeFieldBits::.to_le_bits` here, because that would
                // force users of this crate to depend on `bitvec` where they otherwise
                // might not need to.
                //
                // NOTE: We skip the leading bit because it's always unset (we are turning
                // the 32-byte repr into 256 bits, and $scalar::NUM_BITS = 255).
                for bit in other
                    .to_repr()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
                {
                    acc = acc.double();
                    acc = $name::conditional_select(&acc, &(acc + self), bit);
                }

                acc
            }
        }

        impl PrimeCurveAffine for $name_affine {
            type Curve = $name;
            type Scalar = $scalar;

            impl_affine_curve_specific!($name, $base, $curve_type);

            fn identity() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                }
            }

            fn is_identity(&self) -> Choice {
                self.x.is_zero() & self.y.is_zero()
            }

            fn to_curve(&self) -> Self::Curve {
                $name {
                    x: self.x,
                    y: self.y,
                    z: $base::conditional_select(&$base::one(), &$base::zero(), self.is_identity()),
                }
            }
        }

        impl group::cofactor::CofactorCurveAffine for $name_affine {
            type Curve = $name;
            type Scalar = $scalar;

            fn identity() -> Self {
                <Self as PrimeCurveAffine>::identity()
            }

            fn generator() -> Self {
                <Self as PrimeCurveAffine>::generator()
            }

            fn is_identity(&self) -> Choice {
                <Self as PrimeCurveAffine>::is_identity(self)
            }

            fn to_curve(&self) -> Self::Curve {
                <Self as PrimeCurveAffine>::to_curve(self)
            }
        }

        impl GroupEncoding for $name_affine {
            type Repr = $name_compressed;

            fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
                let bytes = &bytes.0;
                let mut tmp = *bytes;
                let ysign = Choice::from(tmp[32]);
                let mut x_repr = [0; 32];
                x_repr.copy_from_slice(&tmp[..32]);

                $base::from_repr(x_repr).and_then(|x| {
                    CtOption::new(Self::identity(), x.is_zero() & (!ysign)).or_else(|| {
                        let x3 = x.square() * x;
                        (x3 + $name::curve_constant_b()).sqrt().and_then(|y| {
                            let sign = y.is_odd();

                            let y = $base::conditional_select(&y, &-y, ysign ^ sign);

                            CtOption::new(
                                $name_affine {
                                    x,
                                    y,
                                },
                                Choice::from(1u8),
                            )
                        })
                    })
                })
            }

            fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
                // We can't avoid curve checks when parsing a compressed encoding.
                Self::from_bytes(bytes)
            }

            fn to_bytes(&self) -> Self::Repr {
                // TODO: not constant time
                if bool::from(self.is_identity()) {
                    $name_compressed::default()
                } else {
                    let (x, y) = (self.x, self.y);
                    let sign = y.is_odd().unwrap_u8();
                    let mut dest = [0; 33];
                    dest[..32].copy_from_slice(&x.to_repr());
                    dest[32] = sign;
                    $name_compressed(dest)
                }
            }
        }

        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        impl CurveAffine for $name_affine {
            type ScalarExt = $scalar;
            type Base = $base;
            type CurveExt = $name;

            fn is_on_curve(&self) -> Choice {
                // y^2 - x^3 - ax ?= b
                (self.y.square() - (self.x.square() + &$name::curve_constant_a()) * self.x).ct_eq(&$name::curve_constant_b())
                    | self.is_identity()
            }

            fn coordinates(&self) -> CtOption<Coordinates<Self>> {
                Coordinates::from_xy( self.x, self.y )
            }

            fn from_xy(x: Self::Base, y: Self::Base) -> CtOption<Self> {
                let p = $name_affine {
                    x, y,
                };
                CtOption::new(p, p.is_on_curve())
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
                $name_affine::identity()
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
                self.x.ct_eq(&other.x) & self.y.ct_eq(&other.y)
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
                Self::identity()
            }
            fn group_add(&mut self, rhs: &Self) {
                *self += *rhs;
            }
            fn group_sub(&mut self, rhs: &Self) {
                *self -= *rhs;
            }
            fn group_scale(&mut self, by: &Self::Scalar) {
                *self *= *by;
            }
        }

        #[cfg(feature = "gpu")]
        impl ec_gpu::GpuName for $name_affine {
            fn name() -> alloc::string::String {
                ec_gpu::name!()
            }
        }
    };
}

macro_rules! impl_projective_curve_specific {
    ($name:ident, $base:ident, secp256k1) => {
        fn generator() -> Self {
            const SECP_GENERATOR_X: $base = $base::from_raw([
                0x59F2815B16F81798,
                0x029BFCDB2DCE28D9,
                0x55A06295CE870B07,
                0x79BE667EF9DCBBAC,
            ]);
            const SECP_GENERATOR_Y: $base = $base::from_raw([
                0x9C47D08FFB10D4B8,
                0xFD17B448A6855419,
                0x5DA4FBFC0E1108A8,
                0x483ADA7726A3C465,
            ]);

            Self {
                x: SECP_GENERATOR_X,
                y: SECP_GENERATOR_Y,
                z: $base::one(),
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

            $name::conditional_select(&tmp, &$name::identity(), self.is_identity())
        }
    };
    ($name:ident, $base:ident, secq256k1) => {
        fn generator() -> Self {
            const SECQ_GENERATOR_X: $base = $base::from_raw([
                0x860fee175831bb20,
                0x2cabb9347a25101b,
                0xe7590cbef17c26fc,
                0x9214b8774eb1412b,
            ]);
            const SECQ_GENERATOR_Y: $base = $base::from_raw([
                0x14a1bc519466eb6b,
                0x836a6e341a88892a,
                0xecc5b53440a7598a,
                0x28cb5b51a30b5532,
            ]);

            Self {
                x: SECQ_GENERATOR_X,
                y: SECQ_GENERATOR_Y,
                z: $base::one(),
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

            $name::conditional_select(&tmp, &$name::identity(), self.is_identity())
        }
    };
    ($name:ident, $base:ident, general) => {
        /// Unimplemented: there is no standard generator for this curve.
        fn generator() -> Self {
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

            $name::conditional_select(&tmp, &$name::identity(), self.is_identity())
        }
    };
}

#[cfg(feature = "alloc")]
macro_rules! impl_projective_curve_ext {
    ($name:ident, $iso:ident, $base:ident, secp256k1) => {
        fn hash_to_curve<'a>(domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a> {
            use super::hashtocurve;

            Box::new(move |message| {
                let mut us = [Field::zero(); 2];
                hashtocurve::hash_to_field($name::CURVE_ID, domain_prefix, message, &mut us);
                let q0 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                    &us[0],
                    $name::ROOT_OF_UNITY_INV,
                    $name::Z,
                );
                let q1 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                    &us[1],
                    $name::ROOT_OF_UNITY_INV,
                    $name::Z,
                );
                let r = q0 + &q1;
                debug_assert!(bool::from(r.is_on_curve()));
                hashtocurve::iso_map::<$base, $name, $iso>(&r, &$name::ISOGENY_CONSTANTS)
            })
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
    };
    ($name:ident, $iso:ident, $base:ident, secq256k1) => {
        fn hash_to_curve<'a>(domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a> {
            use super::hashtocurve;

            Box::new(move |message| {
                let mut us = [Field::zero(); 2];
                hashtocurve::hash_to_field($name::CURVE_ID, domain_prefix, message, &mut us);
                let q0 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                    &us[0],
                    $name::ROOT_OF_UNITY_INV,
                    $name::Z,
                );
                let q1 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                    &us[1],
                    $name::ROOT_OF_UNITY_INV,
                    $name::Z,
                );
                let r = q0 + &q1;
                debug_assert!(bool::from(r.is_on_curve()));
                hashtocurve::iso_map::<$base, $name, $iso>(&r, &$name::ISOGENY_CONSTANTS)
            })
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
    };
    ($name:ident, $iso:ident, $base:ident, general) => {
        /// Unimplemented: hashing to this curve is not supported
        fn hash_to_curve<'a>(_domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a> {
            unimplemented!()
        }

        /// Unimplemented: no endomorphism is supported for this curve.
        fn endo(&self) -> Self {
            unimplemented!()
        }
    };
}

macro_rules! impl_affine_curve_specific {
    ($name:ident, $base:ident, secp256k1) => {
        fn generator() -> Self {
            const SECP_GENERATOR_X: $base = $base::from_raw([
                0x59F2815B16F81798,
                0x029BFCDB2DCE28D9,
                0x55A06295CE870B07,
                0x79BE667EF9DCBBAC,
            ]);
            const SECP_GENERATOR_Y: $base = $base::from_raw([
                0x9C47D08FFB10D4B8,
                0xFD17B448A6855419,
                0x5DA4FBFC0E1108A8,
                0x483ADA7726A3C465,
            ]);

            Self {
                x: SECP_GENERATOR_X,
                y: SECP_GENERATOR_Y,
            }
        }
    };
    ($name:ident, $base:ident, secq256k1) => {
        fn generator() -> Self {
            const SECQ_GENERATOR_X: $base = $base::from_raw([
                0x860fee175831bb20,
                0x2cabb9347a25101b,
                0xe7590cbef17c26fc,
                0x9214b8774eb1412b,
            ]);
            const SECQ_GENERATOR_Y: $base = $base::from_raw([
                0x14a1bc519466eb6b,
                0x836a6e341a88892a,
                0xecc5b53440a7598a,
                0x28cb5b51a30b5532,
            ]);

            Self {
                x: SECQ_GENERATOR_X,
                y: SECQ_GENERATOR_Y,
            }
        }
    };
    ($name:ident, $base:ident, general) => {
        /// Unimplemented: there is no standard generator for this curve.
        fn generator() -> Self {
            unimplemented!()
        }
    };
}

new_curve_impl!(
    (pub),
    Secp256k1,
    Secp256k1Affine,
    Secp256k1Compressed,
    IsoSecp256k1,
    Fp,
    Fq,
    "secp256k1",
    [0, 0, 0, 0],
    [7, 0, 0, 0],
    secp256k1
);
new_curve_impl!(
    (pub),
    Secq256k1,
    Secq256k1Affine,
    Secq256k1Compressed,
    IsoSecq256k1,
    Fq,
    Fp,
    "secq256k1",
    [0, 0, 0, 0],
    [7, 0, 0, 0],
    secq256k1
);
new_curve_impl!(
    (pub(crate)),
    IsoSecp256k1,
    IsoSecp256k1Affine,
    IsoSecp256k1Compressed,
    Secp256k1,
    Fp,
    Fq,
    "iso-secp256k1",
    [
        0x405447c01a444533,
        0xe953d363cb6f0e5d,
        0xa08a5558f0f5d272,
        0x3f8731abdd661adc,
    ],
    [1771, 0, 0, 0],
    general
);
new_curve_impl!(
    (pub(crate)),
    IsoSecq256k1,
    IsoSecq256k1Affine,
    IsoSecq256k1Compressed,
    Secq256k1,
    Fq,
    Fp,
    "iso-secq256k1",
    [
        0xd902b503abde6324,
        0x3620be3ee2b4e7bc,
        0xfc9f5a3f6ede4d3c,
        0x080ddcd71c081be2,
    ],
    [1771, 0, 0, 0],
    general
);

// From https://github.com/geometryresearch/secp256k1_hash_to_curve/blob/main/circuits/circom/constants.circom
impl Secp256k1 {
    /// Constants used for computing the isogeny from IsoSecp to Secp.
    pub const ISOGENY_CONSTANTS: [Fp; 13] = [
        Fp::from_raw([
            10248191149674768524,
            4099276460824344803,
            16397105843297379214,
            10248191152060862008,
        ]),
        Fp::from_raw([
            5677861232072053346,
            16451756383528566833,
            16331199996347402988,
            6002227985152881894,
        ]),
        Fp::from_raw([
            16140637477814429057,
            15390439281582816146,
            13399077293683197125,
            564028334007329237,
        ]),
        Fp::from_raw([
            10248191149674768583,
            4099276460824344803,
            16397105843297379214,
            10248191152060862008,
        ]),
        Fp::from_raw([
            14207262949819313428,
            491854862080688571,
            17853591451159765588,
            17126563718956833821,
        ]),
        Fp::from_raw([
            11522098205669897371,
            9713490981125900413,
            11286949528964841693,
            15228765018197889418,
        ]),
        Fp::from_raw([
            9564978407794773380,
            13664254869414482678,
            11614616639002310276,
            3416063717353620669,
        ]),
        Fp::from_raw([
            12062302652890802481,
            8225878191764283416,
            8165599998173701494,
            3001113992576440947,
        ]),
        Fp::from_raw([
            16139934577133973923,
            7240293169244854895,
            12236461929419286229,
            14365933273833241615,
        ]),
        Fp::from_raw([
            11614616637729727036,
            3416063717353620669,
            7515340178177965473,
            5465701947765793071,
        ]),
        Fp::from_raw([
            12087522392169162607,
            737782293121032857,
            17557015139884872574,
            7243101504725699116,
        ]),
        Fp::from_raw([
            16119550551890077043,
            10693728869668149624,
            15414104513184973464,
            8792806907174565023,
        ]),
        Fp::from_raw([
            18446744069414582587,
            18446744073709551615,
            18446744073709551615,
            18446744073709551615,
        ]),
    ];

    /// Z = -11
    pub const Z: Fp = Fp::from_raw([
        18446744069414583332,
        18446744073709551615,
        18446744073709551615,
        18446744073709551615,
    ]);

    pub const ROOT_OF_UNITY_INV: Fp = Fp::ROOT_OF_UNITY_INV;
}

impl Secq256k1 {
    /// Constants used for computing the isogeny from IsoSecq to Secq.
    pub const ISOGENY_CONSTANTS: [Fq; 13] = [
        Fq::from_raw([
            7679007869575068054,
            9522933797269734319,
            16397105843297379213,
            10248191152060862008,
        ]),
        Fq::from_raw([
            9826996953646961554,
            15182850926035153421,
            14578491762904662818,
            12647934416601614380,
        ]),
        Fq::from_raw([
            12837744973953074055,
            3022921441994356503,
            9226076221592167090,
            5322610924144458968,
        ]),
        Fq::from_raw([
            7679007869575068113,
            9522933797269734319,
            16397105843297379213,
            10248191152060862008,
        ]),
        Fq::from_raw([
            5509687591411919004,
            593833991126057235,
            2079217350175104065,
            3150945307157219731,
        ]),
        Fq::from_raw([
            10055942181862970998,
            5902098865151897053,
            9296385024764340435,
            14583286435933530837,
        ]),
        Fq::from_raw([
            1018159320366879645,
            7658288605871115257,
            17763531330238827481,
            9564978408590137874,
        ]),
        Fq::from_raw([
            14136870513678256585,
            7591425463017576710,
            7289245881452331409,
            6323967208300807190,
        ]),
        Fq::from_raw([
            14802773332216597422,
            16078857340678580677,
            3084372689655359971,
            1069495981486797935,
        ]),
        Fq::from_raw([
            2553960894281893207,
            13252224180066972444,
            13664254869414482677,
            11614616639002310276,
        ]),
        Fq::from_raw([
            17487903423972654314,
            10114123023543861660,
            12342198062117431905,
            4726417960735829596,
        ]),
        Fq::from_raw([
            2523398215118668000,
            9249176628478019873,
            9442411000583469692,
            6856371160381489280,
        ]),
        Fq::from_raw([
            13822214165235121741,
            13451932020343611451,
            18446744073709551614,
            18446744073709551615,
        ]),
    ];

    /// Z = -14
    pub const Z: Fq = Fq::from_raw([
        13822214165235122483,
        13451932020343611451,
        18446744073709551614,
        18446744073709551615,
    ]);

    pub const ROOT_OF_UNITY_INV: Fq = Fq::ROOT_OF_UNITY_INV;
}

#[test]
fn test_curve() {
    crate::tests::curve::curve_tests::<Secq256k1>();
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
