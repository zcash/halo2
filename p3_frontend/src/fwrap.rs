//! `FWrap` is a Wrapper type over `ff::Field` (halo2 compatible field type) that satisfies the
//! plonky3 field traits.

use halo2_middleware::ff::{Field, PrimeField};
use num_bigint::BigUint;
use p3_field::{
    AbstractField, Field as p3Field, Packable, PrimeField as p3PrimeField, PrimeField64,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::hash::Hash;
use std::iter::{Product, Sum};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FWrap<F: Field>(pub F);

unsafe impl<F: Field> Send for FWrap<F> {}
unsafe impl<F: Field> Sync for FWrap<F> {}

impl<F: PrimeField> Serialize for FWrap<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.to_repr().as_ref())
    }
}

struct FWrapVisitor<F>(PhantomData<F>);

impl<'de, F: PrimeField> de::Visitor<'de> for FWrapVisitor<F> {
    type Value = FWrap<F>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a field repr as bytes")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut repr = F::Repr::default();
        repr.as_mut().copy_from_slice(value);
        let v: Option<F> = F::from_repr(repr).into();
        v.map(|v| FWrap(v))
            .ok_or(E::custom("invalid field repr: {:value?}"))
    }
}

impl<'de, F: PrimeField> Deserialize<'de> for FWrap<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(FWrapVisitor::<F>(PhantomData {}))
    }
}

impl<F: Field> fmt::Display for FWrap<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<F: PrimeField + Hash> AbstractField for FWrap<F> {
    type F = Self;

    fn zero() -> Self {
        Self(F::ZERO)
    }
    fn one() -> Self {
        Self(F::ONE)
    }
    fn two() -> Self {
        Self(F::from(2u64))
    }
    fn neg_one() -> Self {
        Self(F::ZERO - F::ONE)
    }

    #[inline]
    fn from_f(f: Self::F) -> Self {
        f
    }

    fn from_bool(b: bool) -> Self {
        Self(F::from(u64::from(b)))
    }

    fn from_canonical_u8(n: u8) -> Self {
        Self(F::from(u64::from(n)))
    }

    fn from_canonical_u16(n: u16) -> Self {
        Self(F::from(u64::from(n)))
    }

    fn from_canonical_u32(n: u32) -> Self {
        Self(F::from(u64::from(n)))
    }

    fn from_canonical_u64(n: u64) -> Self {
        Self(F::from(n))
    }

    fn from_canonical_usize(n: usize) -> Self {
        Self(F::from(n as u64))
    }

    fn from_wrapped_u32(n: u32) -> Self {
        // A u32 must be canonical, plus we don't store canonical encodings anyway, so there's no
        // need for a reduction.
        Self(F::from(u64::from(n)))
    }

    fn from_wrapped_u64(n: u64) -> Self {
        // There's no need to reduce `n` to canonical form, as our internal encoding is
        // non-canonical, so there's no need for a reduction.
        Self(F::from(n))
    }

    fn generator() -> Self {
        Self(F::MULTIPLICATIVE_GENERATOR)
    }
}

impl<F: Field> Add for FWrap<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0.add(rhs.0))
    }
}

impl<F: Field> AddAssign for FWrap<F> {
    fn add_assign(&mut self, rhs: Self) {
        self.0.add_assign(rhs.0)
    }
}

impl<F: Field> Sum for FWrap<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        FWrap(iter.map(|x| x.0).sum::<F>())
    }
}

impl<F: Field> Sub for FWrap<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0.sub(rhs.0))
    }
}

impl<F: Field> SubAssign for FWrap<F> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0.sub_assign(rhs.0)
    }
}

impl<F: Field> Neg for FWrap<F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

impl<F: Field> Mul for FWrap<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0.mul(rhs.0))
    }
}

impl<F: Field> MulAssign for FWrap<F> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0.mul_assign(rhs.0)
    }
}

impl<F: Field> Product for FWrap<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|x, y| x * y).unwrap_or(Self(F::ONE))
    }
}

impl<F: Field> Div for FWrap<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let rhs_inv = rhs.0.invert().expect("division by 0");
        #[allow(clippy::suspicious_arithmetic_impl)]
        Self(self.0 * rhs_inv)
    }
}

impl<F: Field> Packable for FWrap<F> {}
impl<F: PrimeField + Hash> p3Field for FWrap<F> {
    type Packing = Self;

    fn is_zero(&self) -> bool {
        self.0.is_zero().into()
    }

    fn try_inverse(&self) -> Option<Self> {
        let inverse: Option<F> = self.0.invert().into();
        inverse.map(|u| FWrap(u))
    }

    #[allow(clippy::let_and_return)]
    fn order() -> BigUint {
        let minus_one = F::ZERO - F::ONE;
        let minus_one_repr = minus_one.to_repr();
        let le = F::ONE.to_repr().as_ref()[0] == 1;
        let mut minus_one = if le {
            BigUint::from_bytes_le(minus_one_repr.as_ref())
        } else {
            BigUint::from_bytes_be(minus_one_repr.as_ref())
        };
        minus_one += 1u64;
        let p = minus_one;
        p
    }
}

impl<F: PrimeField + Hash + Ord> p3PrimeField for FWrap<F> {
    fn as_canonical_biguint(&self) -> BigUint {
        let le = F::ONE.to_repr().as_ref()[0] == 1;
        if le {
            BigUint::from_bytes_le(self.0.to_repr().as_ref())
        } else {
            BigUint::from_bytes_be(self.0.to_repr().as_ref())
        }
    }
}

// HACK: In general an `FWrap` will need more than 64 bits.  This trait is only implemented in
// order to use `FWrap` with witness generation from plonky3 that requries this trait but doesn't
// use the order.  Do not use an `ff::PrimeField` on a circuit that requires a 64 bit prime field
// (i.e. relies on the `ORDER_U64` value), only use it on circuits that always assign less than 64
// bit values on the field elements.
impl<F: PrimeField + Hash + Ord> PrimeField64 for FWrap<F> {
    const ORDER_U64: u64 = 0;

    fn as_canonical_u64(&self) -> u64 {
        self.as_canonical_biguint()
            .try_into()
            .expect("field fits in u64")
    }
}

#[allow(clippy::bool_assert_comparison)]
#[cfg(test)]
mod test {
    use super::*;
    use halo2curves::bn256::Fr;
    use serde_test::{assert_tokens, Token};

    type F = FWrap<Fr>;

    #[test]
    fn test_fwrap() {
        // Serialize & Deserialize

        let v = F::two();
        assert_tokens(
            &v,
            &[Token::Bytes(&[
                2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ])],
        );

        // AbstractField

        assert_eq!(F::zero().as_canonical_u64(), 0);
        assert_eq!(F::one().as_canonical_u64(), 1);
        assert_eq!(F::two().as_canonical_u64(), 2);
        assert_eq!(
            format!("{}", F::neg_one()),
            "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000"
        );
        assert_eq!(F::from_f(F::two()).as_canonical_u64(), 2);
        assert_eq!(F::from_bool(true).as_canonical_u64(), 1);
        assert_eq!(F::from_canonical_u8(0x12).as_canonical_u64(), 0x12);
        assert_eq!(F::from_canonical_u16(0x1234).as_canonical_u64(), 0x1234);
        assert_eq!(F::from_canonical_u32(0x123456).as_canonical_u64(), 0x123456);
        assert_eq!(
            F::from_canonical_u64(0xfffffff12).as_canonical_u64(),
            0xfffffff12
        );
        assert_eq!(
            F::from_canonical_usize(0xfffffff12).as_canonical_u64(),
            0xfffffff12
        );
        assert_eq!(F::from_wrapped_u32(0x123456).as_canonical_u64(), 0x123456);
        assert_eq!(
            F::from_wrapped_u64(0xfffffff12).as_canonical_u64(),
            0xfffffff12
        );
        assert_eq!(
            format!("{}", F::generator()),
            "0x0000000000000000000000000000000000000000000000000000000000000007"
        );

        // Arithmetic operators

        assert_eq!(F::one() + F::one(), F::two());
        let mut v = F::one();
        v += F::one();
        assert_eq!(v, F::two());
        assert_eq!([F::one(), F::one()].into_iter().sum::<F>(), F::two());

        assert_eq!(F::two() - F::one(), F::one());
        let mut v = F::two();
        v -= F::one();
        assert_eq!(v, F::one());

        assert_eq!(-F::one(), F::neg_one());

        assert_eq!(F::two() + F::two(), F::from_canonical_u64(4));
        let mut v = F::two();
        v *= F::two();
        assert_eq!(v, F::from_canonical_u64(4));
        assert_eq!(
            [F::two(), F::two()].into_iter().product::<F>(),
            F::from_canonical_u64(4)
        );

        assert_eq!(
            F::from_canonical_u64(10) / F::from_canonical_u64(5),
            F::from_canonical_u64(2)
        );

        // p3Field
        assert_eq!(F::zero().is_zero(), true);
        assert_eq!(F::one().is_zero(), false);
        assert_eq!(
            format!("{}", F::two().try_inverse().unwrap()),
            "0x183227397098d014dc2822db40c0ac2e9419f4243cdcb848a1f0fac9f8000001"
        );
        assert_eq!(
            format!("{}", F::order()),
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        );

        // p3PrimeField

        assert_eq!(
            format!("{}", F::from_canonical_u64(1234).as_canonical_biguint()),
            "1234"
        );

        // PrimeField64

        assert_eq!(F::from_canonical_u64(1234).as_canonical_u64(), 1234);
    }
}
