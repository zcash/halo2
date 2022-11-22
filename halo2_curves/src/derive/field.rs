#[macro_export]
macro_rules! field_common {
    (
        $field:ident,
        $modulus:ident,
        $inv:ident,
        $modulus_str:ident,
        $two_inv:ident,
        $root_of_unity_inv:ident,
        $delta:ident,
        $zeta:ident,
        $r:ident,
        $r2:ident,
        $r3:ident
    ) => {
        impl $field {
            /// Returns zero, the additive identity.
            #[inline]
            pub const fn zero() -> $field {
                $field([0, 0, 0, 0])
            }

            /// Returns one, the multiplicative identity.
            #[inline]
            pub const fn one() -> $field {
                $r
            }

            fn from_u512(limbs: [u64; 8]) -> $field {
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
                let d0 = $field([limbs[0], limbs[1], limbs[2], limbs[3]]);
                let d1 = $field([limbs[4], limbs[5], limbs[6], limbs[7]]);
                // Convert to Montgomery form
                d0 * $r2 + d1 * $r3
            }

            /// Converts from an integer represented in little endian
            /// into its (congruent) `$field` representation.
            pub const fn from_raw(val: [u64; 4]) -> Self {
                (&$field(val)).mul(&$r2)
            }

            /// Attempts to convert a little-endian byte representation of
            /// a scalar into a `Fr`, failing if the input is not canonical.
            pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<$field> {
                <Self as ff::PrimeField>::from_repr(*bytes)
            }

            /// Converts an element of `Fr` into a byte representation in
            /// little-endian byte order.
            pub fn to_bytes(&self) -> [u8; 32] {
                <Self as ff::PrimeField>::to_repr(self)
            }
        }

        impl Group for $field {
            type Scalar = Self;

            fn group_zero() -> Self {
                Self::zero()
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

        impl fmt::Debug for $field {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let tmp = self.to_repr();
                write!(f, "0x")?;
                for &b in tmp.iter().rev() {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }
        }

        impl Default for $field {
            #[inline]
            fn default() -> Self {
                Self::zero()
            }
        }

        impl From<bool> for $field {
            fn from(bit: bool) -> $field {
                if bit {
                    $field::one()
                } else {
                    $field::zero()
                }
            }
        }

        impl From<u64> for $field {
            fn from(val: u64) -> $field {
                $field([val, 0, 0, 0]) * $r2
            }
        }

        impl ConstantTimeEq for $field {
            fn ct_eq(&self, other: &Self) -> Choice {
                self.0[0].ct_eq(&other.0[0])
                    & self.0[1].ct_eq(&other.0[1])
                    & self.0[2].ct_eq(&other.0[2])
                    & self.0[3].ct_eq(&other.0[3])
            }
        }

        impl PartialEq for $field {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.ct_eq(other).unwrap_u8() == 1
            }
        }

        impl core::cmp::Ord for $field {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                let left = self.to_repr();
                let right = other.to_repr();
                left.iter()
                    .zip(right.iter())
                    .rev()
                    .find_map(|(left_byte, right_byte)| match left_byte.cmp(right_byte) {
                        core::cmp::Ordering::Equal => None,
                        res => Some(res),
                    })
                    .unwrap_or(core::cmp::Ordering::Equal)
            }
        }

        impl core::cmp::PartialOrd for $field {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl ConditionallySelectable for $field {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $field([
                    u64::conditional_select(&a.0[0], &b.0[0], choice),
                    u64::conditional_select(&a.0[1], &b.0[1], choice),
                    u64::conditional_select(&a.0[2], &b.0[2], choice),
                    u64::conditional_select(&a.0[3], &b.0[3], choice),
                ])
            }
        }

        impl<'a> Neg for &'a $field {
            type Output = $field;

            #[inline]
            fn neg(self) -> $field {
                self.neg()
            }
        }

        impl Neg for $field {
            type Output = $field;

            #[inline]
            fn neg(self) -> $field {
                -&self
            }
        }

        impl<'a, 'b> Sub<&'b $field> for &'a $field {
            type Output = $field;

            #[inline]
            fn sub(self, rhs: &'b $field) -> $field {
                self.sub(rhs)
            }
        }

        impl<'a, 'b> Add<&'b $field> for &'a $field {
            type Output = $field;

            #[inline]
            fn add(self, rhs: &'b $field) -> $field {
                self.add(rhs)
            }
        }

        impl<'a, 'b> Mul<&'b $field> for &'a $field {
            type Output = $field;

            #[inline]
            fn mul(self, rhs: &'b $field) -> $field {
                self.mul(rhs)
            }
        }

        impl From<$field> for [u8; 32] {
            fn from(value: $field) -> [u8; 32] {
                value.to_repr()
            }
        }

        impl<'a> From<&'a $field> for [u8; 32] {
            fn from(value: &'a $field) -> [u8; 32] {
                value.to_repr()
            }
        }

        impl FieldExt for $field {
            const MODULUS: &'static str = $modulus_str;
            const TWO_INV: Self = $two_inv;
            const ROOT_OF_UNITY_INV: Self = $root_of_unity_inv;
            const DELTA: Self = $delta;
            const ZETA: Self = $zeta;

            fn from_u128(v: u128) -> Self {
                $field::from_raw([v as u64, (v >> 64) as u64, 0, 0])
            }

            /// Converts a 512-bit little endian integer into
            /// a `$field` by reducing by the modulus.
            fn from_bytes_wide(bytes: &[u8; 64]) -> $field {
                $field::from_u512([
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
                let tmp = $field::montgomery_reduce(
                    self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0,
                );

                u128::from(tmp.0[0]) | (u128::from(tmp.0[1]) << 64)
            }
        }
    };
}

#[macro_export]
macro_rules! field_arithmetic {
    ($field:ident, $modulus:ident, $inv:ident, $field_type:ident) => {
        field_specific!($field, $modulus, $inv, $field_type);
        impl $field {
            /// Doubles this field element.
            #[inline]
            pub const fn double(&self) -> $field {
                self.add(self)
            }

            /// Squares this element.
            #[inline]
            pub const fn square(&self) -> $field {
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

                $field::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
            }

            /// Multiplies `rhs` by `self`, returning the result.
            #[inline]
            pub const fn mul(&self, rhs: &Self) -> $field {
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

                $field::montgomery_reduce(r0, r1, r2, r3, r4, r5, r6, r7)
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
                let (d0, carry) = adc(d0, $modulus.0[0] & borrow, 0);
                let (d1, carry) = adc(d1, $modulus.0[1] & borrow, carry);
                let (d2, carry) = adc(d2, $modulus.0[2] & borrow, carry);
                let (d3, _) = adc(d3, $modulus.0[3] & borrow, carry);

                $field([d0, d1, d2, d3])
            }

            /// Negates `self`.
            #[inline]
            pub const fn neg(&self) -> Self {
                // Subtract `self` from `MODULUS` to negate. Ignore the final
                // borrow because it cannot underflow; self is guaranteed to
                // be in the field.
                let (d0, borrow) = sbb($modulus.0[0], self.0[0], 0);
                let (d1, borrow) = sbb($modulus.0[1], self.0[1], borrow);
                let (d2, borrow) = sbb($modulus.0[2], self.0[2], borrow);
                let (d3, _) = sbb($modulus.0[3], self.0[3], borrow);

                // `tmp` could be `MODULUS` if `self` was zero. Create a mask that is
                // zero if `self` was zero, and `u64::max_value()` if self was nonzero.
                let mask =
                    (((self.0[0] | self.0[1] | self.0[2] | self.0[3]) == 0) as u64).wrapping_sub(1);

                $field([d0 & mask, d1 & mask, d2 & mask, d3 & mask])
            }
        }
    };
}

#[macro_export]
macro_rules! field_specific {
    ($field:ident, $modulus:ident, $inv:ident, sparse) => {
        impl $field {
            /// Adds `rhs` to `self`, returning the result.
            #[inline]
            pub const fn add(&self, rhs: &Self) -> Self {
                let (d0, carry) = adc(self.0[0], rhs.0[0], 0);
                let (d1, carry) = adc(self.0[1], rhs.0[1], carry);
                let (d2, carry) = adc(self.0[2], rhs.0[2], carry);
                let (d3, _) = adc(self.0[3], rhs.0[3], carry);

                // Attempt to subtract the modulus, to ensure the value
                // is smaller than the modulus.
                (&$field([d0, d1, d2, d3])).sub(&$modulus)
            }

            #[allow(clippy::too_many_arguments)]
            #[inline(always)]
            pub(crate) const fn montgomery_reduce(
                r0: u64,
                r1: u64,
                r2: u64,
                r3: u64,
                r4: u64,
                r5: u64,
                r6: u64,
                r7: u64,
            ) -> $field {
                // The Montgomery reduction here is based on Algorithm 14.32 in
                // Handbook of Applied Cryptography
                // <http://cacr.uwaterloo.ca/hac/about/chap14.pdf>.

                let k = r0.wrapping_mul($inv);
                let (_, carry) = mac(r0, k, $modulus.0[0], 0);
                let (r1, carry) = mac(r1, k, $modulus.0[1], carry);
                let (r2, carry) = mac(r2, k, $modulus.0[2], carry);
                let (r3, carry) = mac(r3, k, $modulus.0[3], carry);
                let (r4, carry2) = adc(r4, 0, carry);

                let k = r1.wrapping_mul($inv);
                let (_, carry) = mac(r1, k, $modulus.0[0], 0);
                let (r2, carry) = mac(r2, k, $modulus.0[1], carry);
                let (r3, carry) = mac(r3, k, $modulus.0[2], carry);
                let (r4, carry) = mac(r4, k, $modulus.0[3], carry);
                let (r5, carry2) = adc(r5, carry2, carry);

                let k = r2.wrapping_mul($inv);
                let (_, carry) = mac(r2, k, $modulus.0[0], 0);
                let (r3, carry) = mac(r3, k, $modulus.0[1], carry);
                let (r4, carry) = mac(r4, k, $modulus.0[2], carry);
                let (r5, carry) = mac(r5, k, $modulus.0[3], carry);
                let (r6, carry2) = adc(r6, carry2, carry);

                let k = r3.wrapping_mul($inv);
                let (_, carry) = mac(r3, k, $modulus.0[0], 0);
                let (r4, carry) = mac(r4, k, $modulus.0[1], carry);
                let (r5, carry) = mac(r5, k, $modulus.0[2], carry);
                let (r6, carry) = mac(r6, k, $modulus.0[3], carry);
                let (r7, _) = adc(r7, carry2, carry);

                // Result may be within MODULUS of the correct value
                (&$field([r4, r5, r6, r7])).sub(&$modulus)
            }
        }
    };
    ($field:ident, $modulus:ident, $inv:ident, dense) => {
        impl $field {
            /// Adds `rhs` to `self`, returning the result.
            #[inline]
            pub const fn add(&self, rhs: &Self) -> Self {
                let (d0, carry) = adc(self.0[0], rhs.0[0], 0);
                let (d1, carry) = adc(self.0[1], rhs.0[1], carry);
                let (d2, carry) = adc(self.0[2], rhs.0[2], carry);
                let (d3, carry) = adc(self.0[3], rhs.0[3], carry);

                // Attempt to subtract the modulus, to ensure the value
                // is smaller than the modulus.
                let (d0, borrow) = sbb(d0, $modulus.0[0], 0);
                let (d1, borrow) = sbb(d1, $modulus.0[1], borrow);
                let (d2, borrow) = sbb(d2, $modulus.0[2], borrow);
                let (d3, borrow) = sbb(d3, $modulus.0[3], borrow);
                let (_, borrow) = sbb(carry, 0, borrow);

                let (d0, carry) = adc(d0, $modulus.0[0] & borrow, 0);
                let (d1, carry) = adc(d1, $modulus.0[1] & borrow, carry);
                let (d2, carry) = adc(d2, $modulus.0[2] & borrow, carry);
                let (d3, _) = adc(d3, $modulus.0[3] & borrow, carry);

                $field([d0, d1, d2, d3])
            }

            #[allow(clippy::too_many_arguments)]
            #[inline(always)]
            pub(crate) const fn montgomery_reduce(
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

                let k = r0.wrapping_mul($inv);
                let (_, carry) = mac(r0, k, $modulus.0[0], 0);
                let (r1, carry) = mac(r1, k, $modulus.0[1], carry);
                let (r2, carry) = mac(r2, k, $modulus.0[2], carry);
                let (r3, carry) = mac(r3, k, $modulus.0[3], carry);
                let (r4, carry2) = adc(r4, 0, carry);

                let k = r1.wrapping_mul($inv);
                let (_, carry) = mac(r1, k, $modulus.0[0], 0);
                let (r2, carry) = mac(r2, k, $modulus.0[1], carry);
                let (r3, carry) = mac(r3, k, $modulus.0[2], carry);
                let (r4, carry) = mac(r4, k, $modulus.0[3], carry);
                let (r5, carry2) = adc(r5, carry2, carry);

                let k = r2.wrapping_mul($inv);
                let (_, carry) = mac(r2, k, $modulus.0[0], 0);
                let (r3, carry) = mac(r3, k, $modulus.0[1], carry);
                let (r4, carry) = mac(r4, k, $modulus.0[2], carry);
                let (r5, carry) = mac(r5, k, $modulus.0[3], carry);
                let (r6, carry2) = adc(r6, carry2, carry);

                let k = r3.wrapping_mul($inv);
                let (_, carry) = mac(r3, k, $modulus.0[0], 0);
                let (r4, carry) = mac(r4, k, $modulus.0[1], carry);
                let (r5, carry) = mac(r5, k, $modulus.0[2], carry);
                let (r6, carry) = mac(r6, k, $modulus.0[3], carry);
                let (r7, carry2) = adc(r7, carry2, carry);

                // Result may be within MODULUS of the correct value
                let (d0, borrow) = sbb(r4, $modulus.0[0], 0);
                let (d1, borrow) = sbb(r5, $modulus.0[1], borrow);
                let (d2, borrow) = sbb(r6, $modulus.0[2], borrow);
                let (d3, borrow) = sbb(r7, $modulus.0[3], borrow);
                let (_, borrow) = sbb(carry2, 0, borrow);

                let (d0, carry) = adc(d0, $modulus.0[0] & borrow, 0);
                let (d1, carry) = adc(d1, $modulus.0[1] & borrow, carry);
                let (d2, carry) = adc(d2, $modulus.0[2] & borrow, carry);
                let (d3, _) = adc(d3, $modulus.0[3] & borrow, carry);

                $field([d0, d1, d2, d3])
            }
        }
    };
}
