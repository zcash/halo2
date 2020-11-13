use bitvec::{array::BitArray, order::Lsb0};
use core::convert::TryInto;
use core::fmt;
use core::ops::{Add, Mul, Neg, Sub};
use rand::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::arithmetic::{adc, mac, sbb, FieldExt, Group};

/// This represents an element of $\mathbb{F}_p$ where
///
/// `p = 0x40000000000000000000000000000000038aa1276c3f59b9a14064e200000001`
///
/// is the base field of the Tweedledum curve.
// The internal representation of this type is four 64-bit unsigned
// integers in little-endian order. `Fp` values are always in
// Montgomery form; i.e., Fp(a) = aR mod p, with R = 2^256.
#[derive(Clone, Copy, Eq)]
pub struct Fp(pub(crate) [u64; 4]);

impl fmt::Debug for Fp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.to_bytes();
        write!(f, "0x")?;
        for &b in tmp.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl From<bool> for Fp {
    fn from(bit: bool) -> Fp {
        if bit {
            Fp::one()
        } else {
            Fp::zero()
        }
    }
}

impl From<u64> for Fp {
    fn from(val: u64) -> Fp {
        Fp([val, 0, 0, 0]) * R2
    }
}

impl ConstantTimeEq for Fp {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0[0].ct_eq(&other.0[0])
            & self.0[1].ct_eq(&other.0[1])
            & self.0[2].ct_eq(&other.0[2])
            & self.0[3].ct_eq(&other.0[3])
    }
}

impl PartialEq for Fp {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl std::cmp::Ord for Fp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let left = self.to_bytes();
        let right = other.to_bytes();
        left.iter()
            .zip(right.iter())
            .rev()
            .find_map(|(left_byte, right_byte)| match left_byte.cmp(right_byte) {
                std::cmp::Ordering::Equal => None,
                res => Some(res),
            })
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl std::cmp::PartialOrd for Fp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ConditionallySelectable for Fp {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Fp([
            u64::conditional_select(&a.0[0], &b.0[0], choice),
            u64::conditional_select(&a.0[1], &b.0[1], choice),
            u64::conditional_select(&a.0[2], &b.0[2], choice),
            u64::conditional_select(&a.0[3], &b.0[3], choice),
        ])
    }
}

/// Constant representing the modulus
/// p = 0x40000000000000000000000000000000038aa1276c3f59b9a14064e200000001
const MODULUS: Fp = Fp([
    0xa14064e200000001,
    0x38aa1276c3f59b9,
    0x0,
    0x4000000000000000,
]);

/// The modulus as u32 limbs.
#[cfg(not(target_pointer_width = "64"))]
const MODULUS_LIMBS_32: [u32; 8] = [
    0x0000_0001,
    0xa140_64e2,
    0x6c3f_59b9,
    0x038a_a127,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x4000_0000,
];

impl<'a> Neg for &'a Fp {
    type Output = Fp;

    #[inline]
    fn neg(self) -> Fp {
        self.neg()
    }
}

impl Neg for Fp {
    type Output = Fp;

    #[inline]
    fn neg(self) -> Fp {
        -&self
    }
}

impl<'a, 'b> Sub<&'b Fp> for &'a Fp {
    type Output = Fp;

    #[inline]
    fn sub(self, rhs: &'b Fp) -> Fp {
        self.sub(rhs)
    }
}

impl<'a, 'b> Add<&'b Fp> for &'a Fp {
    type Output = Fp;

    #[inline]
    fn add(self, rhs: &'b Fp) -> Fp {
        self.add(rhs)
    }
}

impl<'a, 'b> Mul<&'b Fp> for &'a Fp {
    type Output = Fp;

    #[inline]
    fn mul(self, rhs: &'b Fp) -> Fp {
        self.mul(rhs)
    }
}

impl_binops_additive!(Fp, Fp);
impl_binops_multiplicative!(Fp, Fp);

/// INV = -(p^{-1} mod 2^64) mod 2^64
const INV: u64 = 0xa14064e1ffffffff;

/// R = 2^256 mod p
const R: Fp = Fp([
    0x1c3ed159fffffffd,
    0xf5601c89bb41f2d3,
    0xffffffffffffffff,
    0x3fffffffffffffff,
]);

/// R^2 = 2^512 mod p
const R2: Fp = Fp([
    0x280c9c4000000010,
    0x91a4409b5400af74,
    0xdd7b28e19094c659,
    0xc8ad9107ccca0e,
]);

/// R^3 = 2^768 mod p
const R3: Fp = Fp([
    0x98fb3d144380a737,
    0xf9fdbeb55b7eb87c,
    0x63f75cb999eafa89,
    0x217cb214ebb8fc72,
]);

/// `GENERATOR = 5 mod p` is a generator of the `p - 1` order multiplicative subgroup, and
/// is also a quadratic non-residue.
const GENERATOR: Fp = Fp::from_raw([
    0x0000_0000_0000_0005,
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0000,
]);

const S: u32 = 33;

/// GENERATOR^t where t * 2^s + 1 = p
/// with t odd. In other words, this
/// is a 2^s root of unity.
///
/// `GENERATOR = 5 mod p` is a generator
/// of the p - 1 order multiplicative
/// subgroup.
const ROOT_OF_UNITY: Fp = Fp::from_raw([
    0x53de9f31b88837ce,
    0xff46e8f3f3ea99d6,
    0xf624f2eaaf8c2d57,
    0x2ae45117890ee2fc,
]);

/// GENERATOR^{2^s} where t * 2^s + 1 = p
/// with t odd. In other words, this
/// is a t root of unity.
///
/// `GENERATOR = 5 mod p` is a generator
/// of the p - 1 order multiplicative
/// subgroup.
const DELTA: Fp = Fp::from_raw([
    0x48796f6fde98a425,
    0xa99d8b67e918805e,
    0x671383de08b5fe3c,
    0x1e9372724e80300d,
]);

impl Default for Fp {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl Fp {
    /// Returns zero, the additive identity.
    #[inline]
    pub const fn zero() -> Fp {
        Fp([0, 0, 0, 0])
    }

    /// Returns one, the multiplicative identity.
    #[inline]
    pub const fn one() -> Fp {
        R
    }

    /// Doubles this field element.
    #[inline]
    pub const fn double(&self) -> Fp {
        // TODO: This can be achieved more efficiently with a bitshift.
        self.add(self)
    }

    fn from_u512(limbs: [u64; 8]) -> Fp {
        // We reduce an arbitrary 512-bit number by decomposing it into two 256-bit digits
        // with the higher bits multiplied by 2^256. Thus, we perform two reductions
        //
        // 1. the lower bits are multiplied by R^2, as normal
        // 2. the upper bits are multiplied by R^2 * 2^256 = R^3
        //
        // and computing their sum in the field. It remains to see that arbitrary 256-bit
        // numbers can be placed into Montgomery form safely using the reduction. The
        // reduction works so long as the product is less than R=2^256 multiplied by
        // the modulus. This holds because for any `c` smaller than the modulus, we have
        // that (2^256 - 1)*c is an acceptable product for the reduction. Therefore, the
        // reduction always works so long as `c` is in the field; in this case it is either the
        // constant `R2` or `R3`.
        let d0 = Fp([limbs[0], limbs[1], limbs[2], limbs[3]]);
        let d1 = Fp([limbs[4], limbs[5], limbs[6], limbs[7]]);
        // Convert to Montgomery form
        d0 * R2 + d1 * R3
    }

    /// Converts from an integer represented in little endian
    /// into its (congruent) `Fp` representation.
    pub const fn from_raw(val: [u64; 4]) -> Self {
        (&Fp(val)).mul(&R2)
    }

    /// Squares this element.
    #[inline]
    pub const fn square(&self) -> Fp {
        let (r1, carry) = mac(0, self.0[0], self.0[1], 0);
        let (r2, carry) = mac(0, self.0[0], self.0[2], carry);
        let (r3, r4) = mac(0, self.0[0], self.0[3], carry);

        let (r3, carry) = mac(r3, self.0[1], self.0[2], 0);
        let (r4, r5) = mac(r4, self.0[1], self.0[3], carry);

        let (r5, r6) = mac(r5, self.0[2], self.0[3], 0);

        let r7 = r6 >> 63;
        let r6 = (r6 << 1) | (r5 >> 63);
        let r5 = (r5 << 1) | (r4 >> 63);
        let r4 = (r4 << 1) | (r3 >> 63);
        let r3 = (r3 << 1) | (r2 >> 63);
        let r2 = (r2 << 1) | (r1 >> 63);
        let r1 = r1 << 1;

        let (r0, carry) = mac(0, self.0[0], self.0[0], 0);
        let (r1, carry) = adc(0, r1, carry);
        let (r2, carry) = mac(r2, self.0[1], self.0[1], carry);
        let (r3, carry) = adc(0, r3, carry);
        let (r4, carry) = mac(r4, self.0[2], self.0[2], carry);
        let (r5, carry) = adc(0, r5, carry);
        let (r6, carry) = mac(r6, self.0[3], self.0[3], carry);
        let (r7, _) = adc(0, r7, carry);

        Fp::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
    }

    #[inline(always)]
    const fn montgomery_reduce(
        r0: u64,
        r1: u64,
        r2: u64,
        r3: u64,
        r4: u64,
        r5: u64,
        r6: u64,
        r7: u64,
    ) -> Self {
        // The Montgomery reduction here is based on Algorithm 14.32 in
        // Handbook of Applied Cryptography
        // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

        let k = r0.wrapping_mul(INV);
        let (_, carry) = mac(r0, k, MODULUS.0[0], 0);
        let (r1, carry) = mac(r1, k, MODULUS.0[1], carry);
        let (r2, carry) = mac(r2, k, MODULUS.0[2], carry);
        let (r3, carry) = mac(r3, k, MODULUS.0[3], carry);
        let (r4, carry2) = adc(r4, 0, carry);

        let k = r1.wrapping_mul(INV);
        let (_, carry) = mac(r1, k, MODULUS.0[0], 0);
        let (r2, carry) = mac(r2, k, MODULUS.0[1], carry);
        let (r3, carry) = mac(r3, k, MODULUS.0[2], carry);
        let (r4, carry) = mac(r4, k, MODULUS.0[3], carry);
        let (r5, carry2) = adc(r5, carry2, carry);

        let k = r2.wrapping_mul(INV);
        let (_, carry) = mac(r2, k, MODULUS.0[0], 0);
        let (r3, carry) = mac(r3, k, MODULUS.0[1], carry);
        let (r4, carry) = mac(r4, k, MODULUS.0[2], carry);
        let (r5, carry) = mac(r5, k, MODULUS.0[3], carry);
        let (r6, carry2) = adc(r6, carry2, carry);

        let k = r3.wrapping_mul(INV);
        let (_, carry) = mac(r3, k, MODULUS.0[0], 0);
        let (r4, carry) = mac(r4, k, MODULUS.0[1], carry);
        let (r5, carry) = mac(r5, k, MODULUS.0[2], carry);
        let (r6, carry) = mac(r6, k, MODULUS.0[3], carry);
        let (r7, _) = adc(r7, carry2, carry);

        // Result may be within MODULUS of the correct value
        (&Fp([r4, r5, r6, r7])).sub(&MODULUS)
    }

    /// Multiplies `rhs` by `self`, returning the result.
    #[inline]
    pub const fn mul(&self, rhs: &Self) -> Self {
        // Schoolbook multiplication

        let (r0, carry) = mac(0, self.0[0], rhs.0[0], 0);
        let (r1, carry) = mac(0, self.0[0], rhs.0[1], carry);
        let (r2, carry) = mac(0, self.0[0], rhs.0[2], carry);
        let (r3, r4) = mac(0, self.0[0], rhs.0[3], carry);

        let (r1, carry) = mac(r1, self.0[1], rhs.0[0], 0);
        let (r2, carry) = mac(r2, self.0[1], rhs.0[1], carry);
        let (r3, carry) = mac(r3, self.0[1], rhs.0[2], carry);
        let (r4, r5) = mac(r4, self.0[1], rhs.0[3], carry);

        let (r2, carry) = mac(r2, self.0[2], rhs.0[0], 0);
        let (r3, carry) = mac(r3, self.0[2], rhs.0[1], carry);
        let (r4, carry) = mac(r4, self.0[2], rhs.0[2], carry);
        let (r5, r6) = mac(r5, self.0[2], rhs.0[3], carry);

        let (r3, carry) = mac(r3, self.0[3], rhs.0[0], 0);
        let (r4, carry) = mac(r4, self.0[3], rhs.0[1], carry);
        let (r5, carry) = mac(r5, self.0[3], rhs.0[2], carry);
        let (r6, r7) = mac(r6, self.0[3], rhs.0[3], carry);

        Fp::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
    }

    /// Subtracts `rhs` from `self`, returning the result.
    #[inline]
    pub const fn sub(&self, rhs: &Self) -> Self {
        let (d0, borrow) = sbb(self.0[0], rhs.0[0], 0);
        let (d1, borrow) = sbb(self.0[1], rhs.0[1], borrow);
        let (d2, borrow) = sbb(self.0[2], rhs.0[2], borrow);
        let (d3, borrow) = sbb(self.0[3], rhs.0[3], borrow);

        // If underflow occurred on the final limb, borrow = 0xfff...fff, otherwise
        // borrow = 0x000...000. Thus, we use it as a mask to conditionally add the modulus.
        let (d0, carry) = adc(d0, MODULUS.0[0] & borrow, 0);
        let (d1, carry) = adc(d1, MODULUS.0[1] & borrow, carry);
        let (d2, carry) = adc(d2, MODULUS.0[2] & borrow, carry);
        let (d3, _) = adc(d3, MODULUS.0[3] & borrow, carry);

        Fp([d0, d1, d2, d3])
    }

    /// Adds `rhs` to `self`, returning the result.
    #[inline]
    pub const fn add(&self, rhs: &Self) -> Self {
        let (d0, carry) = adc(self.0[0], rhs.0[0], 0);
        let (d1, carry) = adc(self.0[1], rhs.0[1], carry);
        let (d2, carry) = adc(self.0[2], rhs.0[2], carry);
        let (d3, _) = adc(self.0[3], rhs.0[3], carry);

        // Attempt to subtract the modulus, to ensure the value
        // is smaller than the modulus.
        (&Fp([d0, d1, d2, d3])).sub(&MODULUS)
    }

    /// Negates `self`.
    #[inline]
    pub const fn neg(&self) -> Self {
        // Subtract `self` from `MODULUS` to negate. Ignore the final
        // borrow because it cannot underflow; self is guaranteed to
        // be in the field.
        let (d0, borrow) = sbb(MODULUS.0[0], self.0[0], 0);
        let (d1, borrow) = sbb(MODULUS.0[1], self.0[1], borrow);
        let (d2, borrow) = sbb(MODULUS.0[2], self.0[2], borrow);
        let (d3, _) = sbb(MODULUS.0[3], self.0[3], borrow);

        // `tmp` could be `MODULUS` if `self` was zero. Create a mask that is
        // zero if `self` was zero, and `u64::max_value()` if self was nonzero.
        let mask = (((self.0[0] | self.0[1] | self.0[2] | self.0[3]) == 0) as u64).wrapping_sub(1);

        Fp([d0 & mask, d1 & mask, d2 & mask, d3 & mask])
    }
}

impl From<Fp> for [u8; 32] {
    fn from(value: Fp) -> [u8; 32] {
        value.to_bytes()
    }
}

impl<'a> From<&'a Fp> for [u8; 32] {
    fn from(value: &'a Fp) -> [u8; 32] {
        value.to_bytes()
    }
}

impl Group for Fp {
    type Scalar = Fp;

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

impl ff::Field for Fp {
    fn random(mut rng: impl RngCore) -> Self {
        let mut random_bytes = [0; 64];
        rng.fill_bytes(&mut random_bytes[..]);

        Self::from_bytes_wide(&random_bytes)
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }

    fn is_zero(&self) -> bool {
        self.ct_is_zero().into()
    }

    fn double(&self) -> Self {
        self.double()
    }

    #[inline(always)]
    fn square(&self) -> Self {
        self.square()
    }

    /// Computes the square root of this element, if it exists.
    fn sqrt(&self) -> CtOption<Self> {
        // Tonelli-Shank's algorithm for p mod 16 = 1
        // https://eprint.iacr.org/2012/685.pdf (page 12, algorithm 5)

        // w = self^((t - 1) // 2)
        let w = self.pow_vartime(&[0xdb0fd66e68501938, 0xe2a849, 0x0, 0x10000000]);

        let mut v = S;
        let mut x = self * w;
        let mut b = x * w;

        // Initialize z as the 2^S root of unity.
        let mut z = ROOT_OF_UNITY;

        for max_v in (1..=S).rev() {
            let mut k = 1;
            let mut tmp = b.square();
            let mut j_less_than_v: Choice = 1.into();

            for j in 2..max_v {
                let tmp_is_one = tmp.ct_eq(&Fp::one());
                let squared = Fp::conditional_select(&tmp, &z, tmp_is_one).square();
                tmp = Fp::conditional_select(&squared, &tmp, tmp_is_one);
                let new_z = Fp::conditional_select(&z, &squared, tmp_is_one);
                j_less_than_v &= !j.ct_eq(&v);
                k = u32::conditional_select(&j, &k, tmp_is_one);
                z = Fp::conditional_select(&z, &new_z, j_less_than_v);
            }

            let result = x * z;
            x = Fp::conditional_select(&result, &x, b.ct_eq(&Fp::one()));
            z = z.square();
            b *= z;
            v = k;
        }

        CtOption::new(
            x,
            (x * x).ct_eq(self), // Only return Some if it's the square root.
        )
    }

    /// Computes the multiplicative inverse of this element,
    /// failing if the element is zero.
    fn invert(&self) -> CtOption<Self> {
        let tmp = self.pow_vartime(&[
            0xa14064e1ffffffff,
            0x38aa1276c3f59b9,
            0x0,
            0x4000000000000000,
        ]);

        CtOption::new(tmp, !self.ct_eq(&Self::zero()))
    }

    fn pow_vartime<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::one();
        let mut found_one = false;
        for e in exp.as_ref().iter().rev() {
            for i in (0..64).rev() {
                if found_one {
                    res = res.square();
                }

                if ((*e >> i) & 1) == 1 {
                    found_one = true;
                    res *= self;
                }
            }
        }
        res
    }
}

#[cfg(not(target_pointer_width = "64"))]
type ReprBits = [u32; 8];

#[cfg(target_pointer_width = "64")]
type ReprBits = [u64; 4];

impl ff::PrimeField for Fp {
    type Repr = [u8; 32];
    type ReprBits = ReprBits;

    const NUM_BITS: u32 = 255;
    const CAPACITY: u32 = 254;
    const S: u32 = S;

    fn from_repr(repr: Self::Repr) -> Option<Self> {
        Self::from_bytes(&repr).into()
    }

    fn to_repr(&self) -> Self::Repr {
        self.to_bytes()
    }

    fn to_le_bits(&self) -> BitArray<Lsb0, Self::ReprBits> {
        let bytes = self.to_bytes();

        #[cfg(not(target_pointer_width = "64"))]
        let limbs = [
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            u32::from_le_bytes(bytes[20..24].try_into().unwrap()),
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
        ];

        #[cfg(target_pointer_width = "64")]
        let limbs = [
            u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
        ];

        BitArray::new(limbs)
    }

    fn is_odd(&self) -> bool {
        self.to_bytes()[0] & 1 == 1
    }

    fn char_le_bits() -> BitArray<Lsb0, Self::ReprBits> {
        #[cfg(not(target_pointer_width = "64"))]
        {
            BitArray::new(MODULUS_LIMBS_32)
        }

        #[cfg(target_pointer_width = "64")]
        BitArray::new(MODULUS.0)
    }

    fn multiplicative_generator() -> Self {
        GENERATOR
    }

    fn root_of_unity() -> Self {
        Self::ROOT_OF_UNITY
    }
}

impl FieldExt for Fp {
    const ROOT_OF_UNITY: Self = ROOT_OF_UNITY;
    const ROOT_OF_UNITY_INV: Self = Fp::from_raw([
        0x9246674078fa45bb,
        0xd822ebd60888c5ea,
        0x56d579133a11731f,
        0x1c88fa9e942120bb,
    ]);
    const UNROLL_T_EXPONENT: [u64; 4] = [
        0x3b3a6633d1897d83,
        0x0000000000c93d5b,
        0xf000000000000000,
        0xe34ab16,
    ];
    const T_EXPONENT: [u64; 4] = [
        0xb61facdcd0a03271,
        0x0000000001c55093,
        0x0000000000000000,
        0x20000000,
    ];
    const DELTA: Self = DELTA;
    const UNROLL_S_EXPONENT: u64 = 0x11cb54e91;
    const TWO_INV: Self = Fp::from_raw([
        0xd0a0327100000001,
        0x01c55093b61facdc,
        0x0000000000000000,
        0x2000000000000000,
    ]);
    const RESCUE_ALPHA: u64 = 5;
    const RESCUE_INVALPHA: [u64; 4] = [
        0x810050b4cccccccd,
        0x360880ec56991494,
        0x3333333333333333,
        0x3333333333333333,
    ];
    const ZETA: Self = Fp::from_raw([
        0x8598abb3a410c9c8,
        0x7881fb239ba41a26,
        0x9bebc9146ef83d9a,
        0x1508415ab5e97c94,
    ]);

    fn ct_is_zero(&self) -> Choice {
        self.ct_eq(&Self::zero())
    }

    fn from_u64(v: u64) -> Self {
        Fp::from_raw([v as u64, 0, 0, 0])
    }

    fn from_u128(v: u128) -> Self {
        Fp::from_raw([v as u64, (v >> 64) as u64, 0, 0])
    }

    /// Attempts to convert a little-endian byte representation of
    /// a scalar into a `Fp`, failing if the input is not canonical.
    fn from_bytes(bytes: &[u8; 32]) -> CtOption<Fp> {
        let mut tmp = Fp([0, 0, 0, 0]);

        tmp.0[0] = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        tmp.0[1] = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        tmp.0[2] = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        tmp.0[3] = u64::from_le_bytes(bytes[24..32].try_into().unwrap());

        // Try to subtract the modulus
        let (_, borrow) = sbb(tmp.0[0], MODULUS.0[0], 0);
        let (_, borrow) = sbb(tmp.0[1], MODULUS.0[1], borrow);
        let (_, borrow) = sbb(tmp.0[2], MODULUS.0[2], borrow);
        let (_, borrow) = sbb(tmp.0[3], MODULUS.0[3], borrow);

        // If the element is smaller than MODULUS then the
        // subtraction will underflow, producing a borrow value
        // of 0xffff...ffff. Otherwise, it'll be zero.
        let is_some = (borrow as u8) & 1;

        // Convert to Montgomery form by computing
        // (a.R^0 * R^2) / R = a.R
        tmp *= &R2;

        CtOption::new(tmp, Choice::from(is_some))
    }

    /// Converts an element of `Fp` into a byte representation in
    /// little-endian byte order.
    fn to_bytes(&self) -> [u8; 32] {
        // Turn into canonical form by computing
        // (a.R) / R = a
        let tmp = Fp::montgomery_reduce(self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0);

        let mut res = [0; 32];
        res[0..8].copy_from_slice(&tmp.0[0].to_le_bytes());
        res[8..16].copy_from_slice(&tmp.0[1].to_le_bytes());
        res[16..24].copy_from_slice(&tmp.0[2].to_le_bytes());
        res[24..32].copy_from_slice(&tmp.0[3].to_le_bytes());

        res
    }

    /// Converts a 512-bit little endian integer into
    /// a `Fp` by reducing by the modulus.
    fn from_bytes_wide(bytes: &[u8; 64]) -> Fp {
        Fp::from_u512([
            u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
        ])
    }

    fn get_lower_128(&self) -> u128 {
        let tmp = Fp::montgomery_reduce(self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0);

        u128::from(tmp.0[0]) | (u128::from(tmp.0[1]) << 64)
    }
}

#[cfg(test)]
use ff::{Field, PrimeField};

#[test]
fn test_inv() {
    // Compute -(r^{-1} mod 2^64) mod 2^64 by exponentiating
    // by totient(2**64) - 1

    let mut inv = 1u64;
    for _ in 0..63 {
        inv = inv.wrapping_mul(inv);
        inv = inv.wrapping_mul(MODULUS.0[0]);
    }
    inv = inv.wrapping_neg();

    assert_eq!(inv, INV);
}

#[test]
fn test_zeta() {
    assert_eq!(
        format!("{:?}", Fp::ZETA),
        "0x1508415ab5e97c949bebc9146ef83d9a7881fb239ba41a268598abb3a410c9c8"
    );

    let a = Fp::ZETA;
    assert!(a != Fp::one());
    let b = a * a;
    assert!(b != Fp::one());
    let c = b * a;
    assert!(c == Fp::one());
}

#[test]
fn test_inv_root_of_unity() {
    assert_eq!(Fp::ROOT_OF_UNITY_INV, Fp::ROOT_OF_UNITY.invert().unwrap());
}

#[test]
fn test_inv_2() {
    assert_eq!(Fp::TWO_INV, Fp::from(2).invert().unwrap());
}

#[test]
fn test_delta() {
    assert_eq!(Fp::DELTA, Fp::from(5).pow(&[1u64 << Fp::S, 0, 0, 0]));
}
