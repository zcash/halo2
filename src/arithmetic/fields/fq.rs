use super::{Field, Group};

use core::convert::TryInto;
use core::fmt;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use super::{adc, mac, sbb};

/// This represents an element of $\mathbb{F}_q$ where
///
/// `q = 0x40000000000000000000000000000000038aa127696286c9842cafd400000001`
///
/// is the base field of the Tweedledee curve.
// The internal representation of this type is four 64-bit unsigned
// integers in little-endian order. `Fq` values are always in
// Montgomery form; i.e., Fq(a) = aR mod q, with R = 2^256.
#[derive(Clone, Copy, Eq)]
pub struct Fq(pub(crate) [u64; 4]);

impl fmt::Debug for Fq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.to_bytes();
        write!(f, "0x")?;
        for &b in tmp.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl From<bool> for Fq {
    fn from(bit: bool) -> Fq {
        if bit {
            Fq::one()
        } else {
            Fq::zero()
        }
    }
}

impl From<u64> for Fq {
    fn from(val: u64) -> Fq {
        Fq([val, 0, 0, 0]) * R2
    }
}

impl ConstantTimeEq for Fq {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0[0].ct_eq(&other.0[0])
            & self.0[1].ct_eq(&other.0[1])
            & self.0[2].ct_eq(&other.0[2])
            & self.0[3].ct_eq(&other.0[3])
    }
}

impl PartialEq for Fq {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl ConditionallySelectable for Fq {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Fq([
            u64::conditional_select(&a.0[0], &b.0[0], choice),
            u64::conditional_select(&a.0[1], &b.0[1], choice),
            u64::conditional_select(&a.0[2], &b.0[2], choice),
            u64::conditional_select(&a.0[3], &b.0[3], choice),
        ])
    }
}

/// Constant representing the modulus
/// q = 0x40000000000000000000000000000000038aa127696286c9842cafd400000001
const MODULUS: Fq = Fq([
    0x842cafd400000001,
    0x38aa127696286c9,
    0x0,
    0x4000000000000000,
]);

impl<'a> Neg for &'a Fq {
    type Output = Fq;

    #[inline]
    fn neg(self) -> Fq {
        self.neg()
    }
}

impl Neg for Fq {
    type Output = Fq;

    #[inline]
    fn neg(self) -> Fq {
        -&self
    }
}

impl<'a, 'b> Sub<&'b Fq> for &'a Fq {
    type Output = Fq;

    #[inline]
    fn sub(self, rhs: &'b Fq) -> Fq {
        self.sub(rhs)
    }
}

impl<'a, 'b> Add<&'b Fq> for &'a Fq {
    type Output = Fq;

    #[inline]
    fn add(self, rhs: &'b Fq) -> Fq {
        self.add(rhs)
    }
}

impl<'a, 'b> Mul<&'b Fq> for &'a Fq {
    type Output = Fq;

    #[inline]
    fn mul(self, rhs: &'b Fq) -> Fq {
        self.mul(rhs)
    }
}

impl_binops_additive!(Fq, Fq);
impl_binops_multiplicative!(Fq, Fq);

/// INV = -(q^{-1} mod 2^64) mod 2^64
const INV: u64 = 0x842cafd3ffffffff;

/// R = 2^256 mod q
const R: Fq = Fq([
    0x7379f083fffffffd,
    0xf5601c89c3d86ba3,
    0xffffffffffffffff,
    0x3fffffffffffffff,
]);

/// R^2 = 2^512 mod q
const R2: Fq = Fq([
    0x8595fa8000000010,
    0x7e16a565c6895230,
    0xf4c0e6fcb03aa0a2,
    0xc8ad9106886013,
]);

/// R^3 = 2^768 mod q
const R3: Fq = Fq([
    0xa624f338075cdb5e,
    0x57964eacb8fe21f2,
    0xcb266d18c0413bc2,
    0xa42cdf95f959577,
]);

const S: u32 = 34;

/// GENERATOR^t where t * 2^s + 1 = q
/// with t odd. In other words, this
/// is a 2^s root of unity.
///
/// `GENERATOR = 5 mod q` is a generator
/// of the q - 1 order multiplicative
/// subgroup.
const ROOT_OF_UNITY: Fq = Fq::from_raw([
    0x1cbd3234869d57ec,
    0xa287dd1b8084fbf,
    0xf1dbcb645a987293,
    0x113efc510dc03c0b,
]);

impl Default for Fq {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl Fq {
    /// Returns zero, the additive identity.
    #[inline]
    pub const fn zero() -> Fq {
        Fq([0, 0, 0, 0])
    }

    /// Returns one, the multiplicative identity.
    #[inline]
    pub const fn one() -> Fq {
        R
    }

    /// Doubles this field element.
    #[inline]
    pub const fn double(&self) -> Fq {
        // TODO: This can be achieved more efficiently with a bitshift.
        self.add(self)
    }

    /// Converts a 512-bit little endian integer into
    /// a `Fq` by reducing by the modulus.
    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Fq {
        Fq::from_u512([
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

    fn from_u512(limbs: [u64; 8]) -> Fq {
        // We reduce an arbitrary 512-bit number by decomposing it into two 256-bit digits
        // with the higher bits multiplied by 2^256. Thus, we perform two reductions
        //
        // 1. the lower bits are multiplied by R^2, as normal
        // 2. the upper bits are multiplied by R^2 * 2^256 = R^3
        //
        // and computing their sum in the field. It remains to see that arbitrary 256-bit
        // numbers can be placed into Montgomery form safely using the reduction. The
        // reduction works so long as the product is less than R=2^256 multipled by
        // the modulus. This holds because for any `c` smaller than the modulus, we have
        // that (2^256 - 1)*c is an acceptable product for the reduction. Therefore, the
        // reduction always works so long as `c` is in the field; in this case it is either the
        // constant `R2` or `R3`.
        let d0 = Fq([limbs[0], limbs[1], limbs[2], limbs[3]]);
        let d1 = Fq([limbs[4], limbs[5], limbs[6], limbs[7]]);
        // Convert to Montgomery form
        d0 * R2 + d1 * R3
    }

    /// Converts from an integer represented in little endian
    /// into its (congruent) `Fq` representation.
    pub const fn from_raw(val: [u64; 4]) -> Self {
        (&Fq(val)).mul(&R2)
    }

    /// Squares this element.
    #[inline]
    pub const fn square(&self) -> Fq {
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

        Fq::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
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
        (&Fq([r4, r5, r6, r7])).sub(&MODULUS)
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

        Fq::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
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

        Fq([d0, d1, d2, d3])
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
        (&Fq([d0, d1, d2, d3])).sub(&MODULUS)
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

        Fq([d0 & mask, d1 & mask, d2 & mask, d3 & mask])
    }
}

impl<'a> From<&'a Fq> for [u8; 32] {
    fn from(value: &'a Fq) -> [u8; 32] {
        value.to_bytes()
    }
}

impl Group for Fq {
    type Scalar = Fq;

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

impl Field for Fq {
    const NUM_BITS: u32 = 255;
    const CAPACITY: u32 = 254;
    const S: u32 = S;
    const ROOT_OF_UNITY: Self = ROOT_OF_UNITY;
    const ROOT_OF_UNITY_INV: Self = Fq::from_raw([
        0x824860d3eb30de02,
        0xad9f0afd0ea63acc,
        0xd250318c11a16fe1,
        0x12a9f5e1dd62dabc,
    ]);
    const UNROLL_T_EXPONENT: [u64; 4] = [
        0x9b71de17e6d2d5a0,
        0x0000000000296ee0,
        0x8c00000000000000,
        0x2ecc05e,
    ];
    const T_EXPONENT: [u64; 4] = [
        0xda58a1b2610b2bf5,
        0x0000000000e2a849,
        0x0000000000000000,
        0x10000000,
    ];
    const UNROLL_S_EXPONENT: u64 = 0x344cfe85d;
    const TWO_INV: Self = Fq::from_raw([
        0xc21657ea00000001,
        0x01c55093b4b14364,
        0x0000000000000000,
        0x2000000000000000,
    ]);
    const RESCUE_ALPHA: u64 = 5;
    const RESCUE_INVALPHA: [u64; 4] = [
        0xd023bfdccccccccd,
        0x360880ec544ed23a,
        0x3333333333333333,
        0x3333333333333333,
    ];
    const ZETA: Self = Fq::from_raw([
        0x4394c2bd148fa4fd,
        0x69cf8de720e52ec1,
        0x87ad8b5ff9731ffe,
        0x36c66d3a1e049a58,
    ]);

    fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::zero())
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }

    fn from_u64(v: u64) -> Self {
        Fq::from_raw([v as u64, 0, 0, 0])
    }

    fn from_u128(v: u128) -> Self {
        Fq::from_raw([v as u64, (v >> 64) as u64, 0, 0])
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
        // Tonelli-Shank's algorithm for q mod 16 = 1
        // https://eprint.iacr.org/2012/685.pdf (page 12, algorithm 5)

        // w = self^((t - 1) // 2)
        let w = self.pow_vartime(&[0xed2c50d9308595fa, 0x715424, 0x0, 0x8000000]);

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
                let tmp_is_one = tmp.ct_eq(&Fq::one());
                let squared = Fq::conditional_select(&tmp, &z, tmp_is_one).square();
                tmp = Fq::conditional_select(&squared, &tmp, tmp_is_one);
                let new_z = Fq::conditional_select(&z, &squared, tmp_is_one);
                j_less_than_v &= !j.ct_eq(&v);
                k = u32::conditional_select(&j, &k, tmp_is_one);
                z = Fq::conditional_select(&z, &new_z, j_less_than_v);
            }

            let result = x * z;
            x = Fq::conditional_select(&result, &x, b.ct_eq(&Fq::one()));
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
            0x842cafd3ffffffff,
            0x38aa127696286c9,
            0x0,
            0x4000000000000000,
        ]);

        CtOption::new(tmp, !self.ct_eq(&Self::zero()))
    }

    /// Attempts to convert a little-endian byte representation of
    /// a scalar into a `Fq`, failing if the input is not canonical.
    fn from_bytes(bytes: &[u8; 32]) -> CtOption<Fq> {
        let mut tmp = Fq([0, 0, 0, 0]);

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

    /// Converts an element of `Fq` into a byte representation in
    /// little-endian byte order.
    fn to_bytes(&self) -> [u8; 32] {
        // Turn into canonical form by computing
        // (a.R) / R = a
        let tmp = Fq::montgomery_reduce(self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0);

        let mut res = [0; 32];
        res[0..8].copy_from_slice(&tmp.0[0].to_le_bytes());
        res[8..16].copy_from_slice(&tmp.0[1].to_le_bytes());
        res[16..24].copy_from_slice(&tmp.0[2].to_le_bytes());
        res[24..32].copy_from_slice(&tmp.0[3].to_le_bytes());

        res
    }

    /// Converts a 512-bit little endian integer into
    /// a `Fq` by reducing by the modulus.
    fn from_bytes_wide(bytes: &[u8; 64]) -> Fq {
        Fq::from_u512([
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
        let tmp = Fq::montgomery_reduce(self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0);

        u128::from(tmp.0[0]) | (u128::from(tmp.0[1]) << 64)
    }
}

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
        format!("{:?}", Fq::ZETA),
        "0x36c66d3a1e049a5887ad8b5ff9731ffe69cf8de720e52ec14394c2bd148fa4fd"
    );
    let a = Fq::ZETA;
    assert!(bool::from(a != Fq::one()));
    let b = a * a;
    assert!(bool::from(b != Fq::one()));
    let c = b * a;
    assert!(bool::from(c == Fq::one()));
}

#[test]
fn test_inv_root_of_unity() {
    assert_eq!(Fq::ROOT_OF_UNITY_INV, Fq::ROOT_OF_UNITY.invert().unwrap());
}

#[test]
fn test_inv_2() {
    assert_eq!(Fq::TWO_INV, Fq::from(2).invert().unwrap());
}
