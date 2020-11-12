//! This module contains the `Field` abstraction that allows us to write
//! code that generalizes over a pair of fields.

use std::fmt::Debug;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::Group;

/// This trait is a common interface for dealing with elements of a finite
/// field.
pub trait Field:
    Sized
    + Default
    + Copy
    + Clone
    + Send
    + Sync
    + 'static
    + Debug
    + From<bool>
    + From<u64>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + MulAssign
    + AddAssign
    + SubAssign
    + for<'a> MulAssign<&'a Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + ConditionallySelectable
    + ConstantTimeEq
    + Group<Scalar = Self>
{
    /// How many bits needed to express the modulus $p$?
    const NUM_BITS: u32;

    /// How many bits of information can be stored reliably?
    const CAPACITY: u32;

    /// Represents $S$ where $p - 1 = 2^S \cdot t$ with $t$ odd.
    const S: u32;

    /// Generator of the $2^S$ multiplicative subgroup
    const ROOT_OF_UNITY: Self;

    /// Inverse of `ROOT_OF_UNITY`
    const ROOT_OF_UNITY_INV: Self;

    /// The value $(2^S)^{-1} \mod t$.
    const UNROLL_T_EXPONENT: [u64; 4];

    /// Represents $t$ where $2^S \cdot t = p - 1$ with $t$ odd.
    const T_EXPONENT: [u64; 4];

    /// The value $t^{-1} \mod 2^S$.
    const UNROLL_S_EXPONENT: u64;

    /// Generator of the $t-order$ multiplicative subgroup
    const DELTA: Self;

    /// Inverse of $2$ in the field.
    const TWO_INV: Self;

    /// Ideally the smallest prime $\alpha$ such that gcd($p - 1$, $\alpha$) = $1$
    const RESCUE_ALPHA: u64;

    /// $RESCUE_INVALPHA \cdot RESCUE_ALPHA = 1 \mod p - 1$ such that
    /// `(a^RESCUE_ALPHA)^RESCUE_INVALPHA = a`.
    const RESCUE_INVALPHA: [u64; 4];

    /// Element of multiplicative order $3$.
    const ZETA: Self;

    /// This computes a random element of the field using system randomness.
    fn random() -> Self {
        use rand::{thread_rng, RngCore};

        let mut random_bytes = [0; 64];
        thread_rng().fill_bytes(&mut random_bytes[..]);

        Self::from_bytes_wide(&random_bytes)
    }

    /// Returns whether or not this element is zero.
    fn is_zero(&self) -> Choice;

    /// Doubles this element in the field.
    fn double(&self) -> Self;

    /// Obtains a field element congruent to the integer `v`.
    fn from_u64(v: u64) -> Self;

    /// Obtains a field element congruent to the integer `v`.
    fn from_u128(v: u128) -> Self;

    /// Attempts to obtain the square root of this field element.
    fn sqrt(&self) -> CtOption<Self>;

    /// Attempts to find the multiplicative inverse of this field element.
    fn invert(&self) -> CtOption<Self>;

    /// Returns zero, the additive identity.
    fn zero() -> Self;

    /// Returns one, the multiplicative identity.
    fn one() -> Self;

    /// Squares this element in the field.
    fn square(&self) -> Self;

    /// Converts this field element to its normalized, little endian byte
    /// representation.
    fn to_bytes(&self) -> [u8; 32];

    /// Attempts to obtain a field element from its normalized, little endian
    /// byte representation.
    fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self>;

    /// Obtains a field element that is congruent to the provided little endian
    /// byte representation of an integer.
    fn from_bytes_wide(bytes: &[u8; 64]) -> Self;

    /// Returns a square root of this element, if it exists and this element is
    /// nonzero. Always returns the same square root, and it is efficient to
    /// check that it has done so using `extract_radix2_vartime`.
    fn deterministic_sqrt(&self) -> Option<Self> {
        let sqrt = self.sqrt();
        if bool::from(sqrt.is_none()) {
            return None;
        }
        let sqrt = sqrt.unwrap();
        let extracted = sqrt.extract_radix2_vartime()?;

        if extracted.1 >> (Self::S - 1) == 1 {
            Some(-sqrt)
        } else {
            Some(sqrt)
        }
    }

    /// Returns an element $a$ of multiplicative order $t$ together with an
    /// integer `s` such that `self` is the square of $a \cdot \omega^{s}$ if
    /// indeed `self` is a square.
    fn extract_radix2_vartime(&self) -> Option<(Self, u64)> {
        if bool::from(self.is_zero()) {
            return None;
        }

        // TODO: these can probably be simplified
        let t = self.pow_vartime(&[1 << Self::S, 0, 0, 0]);
        let t = t.pow_vartime(&Self::UNROLL_T_EXPONENT);
        let t = t.pow_vartime(&Self::UNROLL_T_EXPONENT);
        let s = self.pow_vartime(&Self::T_EXPONENT);
        let mut s = s.pow_vartime(&[Self::UNROLL_S_EXPONENT, 0, 0, 0]);

        let mut m = Self::S;
        let mut c = Self::ROOT_OF_UNITY_INV;

        let mut extract: u64 = 0;

        let mut cur = 1;
        while s != Self::one() {
            let mut i = 1;
            {
                let mut s2i = s;
                s2i = s2i.square();
                while s2i != Self::one() {
                    i += 1;
                    s2i = s2i.square();
                }
            }

            for _ in 0..(m - i) {
                c = c.square();
                cur <<= 1;
            }
            extract |= cur;
            s *= c;
            m = i;
        }

        Some((t, extract))
    }

    /// Exponentiates `self` by `by`, where `by` is a little-endian order
    /// integer exponent.
    fn pow(&self, by: &[u64; 4]) -> Self {
        let mut res = Self::one();
        for e in by.iter().rev() {
            for i in (0..64).rev() {
                res = res.square();
                let mut tmp = res;
                tmp *= self;
                res.conditional_assign(&tmp, (((*e >> i) & 0x1) as u8).into());
            }
        }
        res
    }

    /// Exponentiates `self` by `by`, where `by` is a little-endian order
    /// integer exponent.
    ///
    /// **This operation is variable time with respect to the exponent.** If the
    /// exponent is fixed, this operation is effectively constant time.
    fn pow_vartime(&self, by: &[u64; 4]) -> Self {
        let mut res = Self::one();
        let mut found_one = false;
        for e in by.iter().rev() {
            for i in (0..64).rev() {
                if found_one {
                    res = res.square();
                }

                if ((*e >> i) & 1) == 1 {
                    found_one = true;
                    res.mul_assign(self);
                }
            }
        }
        res
    }

    /// Gets the lower 128 bits of this field element when expressed
    /// canonically.
    fn get_lower_128(&self) -> u128;

    /// Performs a batch inversion using Montgomery's trick, returns the product
    /// of every inverse. Zero inputs are ignored.
    fn batch_invert(v: &mut [Self]) -> Self {
        let mut tmp = Vec::with_capacity(v.len());

        let mut acc = Self::one();
        for p in v.iter() {
            tmp.push(acc);
            acc = Self::conditional_select(&(acc * p), &acc, p.is_zero());
        }

        acc = acc.invert().unwrap();
        let allinv = acc;

        for (p, tmp) in v.iter_mut().rev().zip(tmp.into_iter().rev()) {
            let skip = p.is_zero();

            let tmp = tmp * acc;
            acc = Self::conditional_select(&(acc * *p), &acc, skip);
            *p = Self::conditional_select(&tmp, p, skip);
        }

        allinv
    }
}

/// Compute a + b + carry, returning the result and the new carry over.
#[inline(always)]
pub(crate) const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + (b as u128) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}

/// Compute a - (b + borrow), returning the result and the new borrow.
#[inline(always)]
pub(crate) const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let ret = (a as u128).wrapping_sub((b as u128) + ((borrow >> 63) as u128));
    (ret as u64, (ret >> 64) as u64)
}

/// Compute a + (b * c) + carry, returning the result and the new carry over.
#[inline(always)]
pub(crate) const fn mac(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + ((b as u128) * (c as u128)) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}
