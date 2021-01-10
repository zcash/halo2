//! This module contains the `Field` abstraction that allows us to write
//! code that generalizes over a pair of fields.

use core::mem::size_of;
use static_assertions::const_assert;
use std::assert;
use std::convert::TryInto;
use std::marker::PhantomData;
use subtle::{Choice, ConstantTimeEq, CtOption};

use super::Group;

const_assert!(size_of::<usize>() >= 4);

/// This trait is a common interface for dealing with elements of a finite
/// field.
pub trait FieldExt:
    ff::PrimeField + From<bool> + Ord + ConstantTimeEq + Group<Scalar = Self>
{
    /// Generator of the $2^S$ multiplicative subgroup
    const ROOT_OF_UNITY: Self;

    /// Inverse of `ROOT_OF_UNITY`
    const ROOT_OF_UNITY_INV: Self;

    /// The value $(T-1)/2$ such that $2^S \cdot T = p - 1$ with $T$ odd.
    const T_MINUS1_OVER2: [u64; 4];

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

    /// Computes:
    ///
    /// * (true,  sqrt(num/div)),                 if num and div are nonzero and num/div is a square in the field;
    /// * (true,  0),                             if num is zero;
    /// * (false, 0),                             if num is nonzero and div is zero;
    /// * (false, sqrt(ROOT_OF_UNITY * num/div)), if num and div are nonzero and num/div is a nonsquare in the field;
    ///
    /// where ROOT_OF_UNITY is a generator of the order 2^n subgroup (and therefore a nonsquare).
    ///
    /// The choice of root from sqrt is unspecified.
    fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self);

    /// This computes a random element of the field using system randomness.
    fn rand() -> Self {
        Self::random(rand::rngs::OsRng)
    }

    /// Returns whether or not this element is zero.
    fn ct_is_zero(&self) -> Choice;

    /// Obtains a field element congruent to the integer `v`.
    fn from_u64(v: u64) -> Self;

    /// Obtains a field element congruent to the integer `v`.
    fn from_u128(v: u128) -> Self;

    /// Converts this field element to its normalized, little endian byte
    /// representation.
    fn to_bytes(&self) -> [u8; 32];

    /// Attempts to obtain a field element from its normalized, little endian
    /// byte representation.
    fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self>;

    /// Obtains a field element that is congruent to the provided little endian
    /// byte representation of an integer.
    fn from_bytes_wide(bytes: &[u8; 64]) -> Self;

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

    /// Gets the lower 128 bits of this field element when expressed
    /// canonically.
    fn get_lower_128(&self) -> u128;

    /// Gets the lower 32 bits of this field element when expressed
    /// canonically.
    fn get_lower_32(&self) -> u32;

    /// Performs a batch inversion using Montgomery's trick, returns the product
    /// of every inverse. Zero inputs are ignored.
    fn batch_invert(v: &mut [Self]) -> Self {
        let mut tmp = Vec::with_capacity(v.len());

        let mut acc = Self::one();
        for p in v.iter() {
            tmp.push(acc);
            acc = Self::conditional_select(&(acc * p), &acc, p.ct_is_zero());
        }

        acc = acc.invert().unwrap();
        let allinv = acc;

        for (p, tmp) in v.iter_mut().rev().zip(tmp.into_iter().rev()) {
            let skip = p.ct_is_zero();

            let tmp = tmp * acc;
            acc = Self::conditional_select(&(acc * *p), &acc, skip);
            *p = Self::conditional_select(&tmp, p, skip);
        }

        allinv
    }
}

/// Parameters for a perfect hash function used in square root computation.
#[derive(Debug)]
struct SqrtHasher<F: FieldExt> {
    hash_xor: u32,
    hash_mod: usize,
    marker: PhantomData<F>,
}

impl<F: FieldExt> SqrtHasher<F> {
    /// Returns a perfect hash of x for use with SqrtTables::inv.
    fn hash(&self, x: &F) -> usize {
        ((x.get_lower_32() ^ self.hash_xor) as usize) % self.hash_mod
    }
}

/// Tables used for square root computation.
#[derive(Debug)]
pub struct SqrtTables<F: FieldExt> {
    hasher: SqrtHasher<F>,
    inv: Vec<u8>,
    g0: [F; 256],
    g1: [F; 256],
    g2: [F; 256],
    g3: [F; 129],
}

impl<F: FieldExt> SqrtTables<F> {
    /// Build tables given parameters for the perfect hash.
    pub fn new(hash_xor: u32, hash_mod: usize) -> Self {
        let hasher = SqrtHasher {
            hash_xor,
            hash_mod,
            marker: PhantomData,
        };

        let gtab: Vec<Vec<F>> = (0..4)
            .scan(F::ROOT_OF_UNITY, |gi, _| {
                // gi == ROOT_OF_UNITY^(256^i)
                let gtab_i: Vec<F> = (0..256)
                    .scan(F::one(), |acc, _| {
                        let res = *acc;
                        *acc *= *gi;
                        Some(res)
                    })
                    .collect();
                *gi = gtab_i[255] * *gi;
                Some(gtab_i)
            })
            .collect();

        // Now invert gtab[3].
        let mut inv: Vec<u8> = vec![1; hash_mod];
        for j in 0..256 {
            let hash = hasher.hash(&gtab[3][j]);
            // 1 is the last value to be assigned, so this ensures there are no collisions.
            assert!(inv[hash] == 1);
            inv[hash] = ((256 - j) & 0xFF) as u8;
        }

        SqrtTables::<F> {
            hasher,
            inv,
            g0: gtab[0][..].try_into().unwrap(),
            g1: gtab[1][..].try_into().unwrap(),
            g2: gtab[2][..].try_into().unwrap(),
            g3: gtab[3][0..129].try_into().unwrap(),
        }
    }

    /// Computes:
    ///
    /// * (true,  sqrt(num/div)),                 if num and div are nonzero and num/div is a square in the field;
    /// * (true,  0),                             if num is zero;
    /// * (false, 0),                             if num is nonzero and div is zero;
    /// * (false, sqrt(ROOT_OF_UNITY * num/div)), if num and div are nonzero and num/div is a nonsquare in the field;
    ///
    /// where ROOT_OF_UNITY is a generator of the order 2^n subgroup (and therefore a nonsquare).
    ///
    /// The choice of root from sqrt is unspecified.
    pub fn sqrt_ratio(&self, num: &F, div: &F) -> (Choice, F) {
        // Based on:
        // * [Sarkar2020](https://eprint.iacr.org/2020/1407)
        // * [BDLSY2012](https://cr.yp.to/papers.html#ed25519)
        //
        // We need to calculate uv and v, where v = u^((m-1)/2), u = num/div, and p-1 = T * 2^S.
        // We can rewrite as follows:
        //
        //      v = (num/div)^((T-1)/2)
        //        = num^((T-1)/2) * div^(p-1 - (T-1)/2)    [Fermat's Little Theorem]
        //        =       "       * div^(T * 2^S - (T-1)/2)
        //        =       "       * div^((2^(S+1) - 1)*(T-1)/2 + 2^S)
        //        = (num * div^(2^(S+1) - 1))^((T-1)/2) * div^(2^S)
        //
        // Let  w = (num * div^(2^(S+1) - 1))^((T-1)/2) * div^(2^S - 1).
        // Then v = w * div, and uv = num * v / div = num * w.
        //
        // We calculate:
        //
        //      s = div^(2^S - 1) using an addition chain
        //      t = div^(2^(S+1) - 1) = s^2 * div
        //      w = (num * t)^((T-1)/2) * s using another addition chain
        //
        // then u and uv as above. The addition chains are given in
        // https://github.com/zcash/pasta/blob/master/addchain_sqrt.py .
        // The overall cost of this part is similar to a single full-width exponentiation,
        // regardless of S.

        let sqr = |x: F, i: u32| (0..i).fold(x, |x, _| x.square());

        // s = div^(2^S - 1)
        let s = (0..5).fold(*div, |d: F, i| sqr(d, 1 << i) * d);

        // t == div^(2^(S+1) - 1)
        let t = s.square() * div;

        // TODO: replace this with an addition chain.
        let w = ff::Field::pow_vartime(&(t * num), &F::T_MINUS1_OVER2) * s;

        // v == u^((T-1)/2)
        let v = w * div;

        // uv = u * v
        let uv = w * num;

        self.sqrt_common(num, div, &uv, &v)
    }

    /// Same as sqrt_ratio but given num, div, v = u^((T-1)/2), and uv = u * v as input.
    ///
    /// The choice of root from sqrt is unspecified.
    fn sqrt_common(&self, num: &F, div: &F, uv: &F, v: &F) -> (Choice, F) {
        let sqr = |x: F, i: u32| (0..i).fold(x, |x, _| x.square());
        let inv = |x: F| self.inv[self.hasher.hash(&x)] as usize;

        let x3 = *uv * v;
        let x2 = sqr(x3, 8);
        let x1 = sqr(x2, 8);
        let x0 = sqr(x1, 8);

        // i = 0, 1
        let mut t_ = inv(x0); // = t >> 16
                              // 1 == x0 * ROOT_OF_UNITY^(t_ << 24)
        assert!(t_ < 0x100);
        let alpha = x1 * self.g2[t_];

        // i = 2
        t_ += inv(alpha) << 8; // = t >> 8
                               // 1 == x1 * ROOT_OF_UNITY^(t_ << 16)
        assert!(t_ < 0x10000);
        let alpha = x2 * self.g1[t_ & 0xFF] * self.g2[t_ >> 8];

        // i = 3
        t_ += inv(alpha) << 16; // = t
                                // 1 == x2 * ROOT_OF_UNITY^(t_ << 8)
        assert!(t_ < 0x1000000);
        let alpha = x3 * self.g0[t_ & 0xFF] * self.g1[(t_ >> 8) & 0xFF] * self.g2[t_ >> 16];

        t_ += inv(alpha) << 24; // = t << 1
                                // 1 == x3 * ROOT_OF_UNITY^t_
        t_ = (t_ + 1) >> 1;
        assert!(t_ <= 0x80000000);
        let res = *uv
            * self.g0[t_ & 0xFF]
            * self.g1[(t_ >> 8) & 0xFF]
            * self.g2[(t_ >> 16) & 0xFF]
            * self.g3[t_ >> 24];

        let sqdiv = res.square() * div;
        let is_square = (sqdiv - num).ct_is_zero();
        let is_nonsquare = (sqdiv - F::ROOT_OF_UNITY * num).ct_is_zero();
        assert!(bool::from(
            num.ct_is_zero() | div.ct_is_zero() | (is_square ^ is_nonsquare)
        ));

        (is_square, res)
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
