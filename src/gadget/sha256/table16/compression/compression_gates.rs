use super::super::{util::*, Gate};
use crate::arithmetic::FieldExt;
use crate::plonk::Expression;

pub struct CompressionGate<F: FieldExt>(pub Expression<F>);

impl<F: FieldExt> CompressionGate<F> {
    const ONES: Expression<F> = Expression::Ones();

    // Decompose `A,B,C,D` words
    // (2, 11, 9, 10)-bit chunks
    fn decompose_abcd(
        a: Expression<F>,
        spread_a: Expression<F>,
        b: Expression<F>,
        spread_b: Expression<F>,
        c_lo: Expression<F>,
        spread_c_lo: Expression<F>,
        c_mid: Expression<F>,
        spread_c_mid: Expression<F>,
        c_hi: Expression<F>,
        spread_c_hi: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Expression<F> {
        let check_spread_and_range =
            Gate::three_bit_spread_and_range(c_lo.clone(), spread_c_lo.clone())
                + Gate::three_bit_spread_and_range(c_mid.clone(), spread_c_mid.clone())
                + Gate::three_bit_spread_and_range(c_hi.clone(), spread_c_hi.clone())
                + Gate::two_bit_spread_and_range(a.clone(), spread_a.clone());
        let dense_check = a
            + b * F::from_u64(1 << 2)
            + c_lo * F::from_u64(1 << 13)
            + c_mid * F::from_u64(1 << 16)
            + c_hi * F::from_u64(1 << 19)
            + d * F::from_u64(1 << 22)
            + word_lo * (-F::one())
            + word_hi * F::from_u64(1 << 16) * (-F::one());
        let spread_check = spread_a
            + spread_b * F::from_u64(1 << 4)
            + spread_c_lo * F::from_u64(1 << 26)
            + spread_c_mid * F::from_u64(1 << 32)
            + spread_c_hi * F::from_u64(1 << 38)
            + spread_d * F::from_u64(1 << 44)
            + spread_word_lo * (-F::one())
            + spread_word_hi * F::from_u64(1 << 32) * (-F::one());

        dense_check + spread_check + check_spread_and_range
    }

    pub fn s_decompose_abcd(
        s_decompose_abcd: Expression<F>,
        a: Expression<F>,
        spread_a: Expression<F>,
        b: Expression<F>,
        spread_b: Expression<F>,
        c_lo: Expression<F>,
        spread_c_lo: Expression<F>,
        c_mid: Expression<F>,
        spread_c_mid: Expression<F>,
        c_hi: Expression<F>,
        spread_c_hi: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Self {
        CompressionGate(
            s_decompose_abcd
                * Self::decompose_abcd(
                    a,
                    spread_a,
                    b,
                    spread_b,
                    c_lo,
                    spread_c_lo,
                    c_mid,
                    spread_c_mid,
                    c_hi,
                    spread_c_hi,
                    d,
                    spread_d,
                    word_lo,
                    spread_word_lo,
                    word_hi,
                    spread_word_hi,
                ),
        )
    }

    // Decompose `E,F,G,H` words
    // (6, 5, 14, 7)-bit chunks
    fn decompose_efgh(
        a_lo: Expression<F>,
        spread_a_lo: Expression<F>,
        a_hi: Expression<F>,
        spread_a_hi: Expression<F>,
        b_lo: Expression<F>,
        spread_b_lo: Expression<F>,
        b_hi: Expression<F>,
        spread_b_hi: Expression<F>,
        c: Expression<F>,
        spread_c: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Expression<F> {
        let check_spread_and_range =
            Gate::three_bit_spread_and_range(a_lo.clone(), spread_a_lo.clone())
                + Gate::three_bit_spread_and_range(a_hi.clone(), spread_a_hi.clone())
                + Gate::three_bit_spread_and_range(b_hi.clone(), spread_b_hi.clone())
                + Gate::two_bit_spread_and_range(b_lo.clone(), spread_b_lo.clone());
        let dense_check = a_lo
            + a_hi * F::from_u64(1 << 3)
            + b_lo * F::from_u64(1 << 6)
            + b_hi * F::from_u64(1 << 8)
            + c * F::from_u64(1 << 11)
            + d * F::from_u64(1 << 25)
            + word_lo * (-F::one())
            + word_hi * F::from_u64(1 << 16) * (-F::one());
        let spread_check = spread_a_lo
            + spread_a_hi * F::from_u64(1 << 6)
            + spread_b_lo * F::from_u64(1 << 12)
            + spread_b_hi * F::from_u64(1 << 16)
            + spread_c * F::from_u64(1 << 22)
            + spread_d * F::from_u64(1 << 50)
            + spread_word_lo * (-F::one())
            + spread_word_hi * F::from_u64(1 << 32) * (-F::one());

        dense_check + spread_check + check_spread_and_range
    }

    pub fn s_decompose_efgh(
        s_decompose_efgh: Expression<F>,
        a_lo: Expression<F>,
        spread_a_lo: Expression<F>,
        a_hi: Expression<F>,
        spread_a_hi: Expression<F>,
        b_lo: Expression<F>,
        spread_b_lo: Expression<F>,
        b_hi: Expression<F>,
        spread_b_hi: Expression<F>,
        c: Expression<F>,
        spread_c: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Self {
        CompressionGate(
            s_decompose_efgh
                * Self::decompose_efgh(
                    a_lo,
                    spread_a_lo,
                    a_hi,
                    spread_a_hi,
                    b_lo,
                    spread_b_lo,
                    b_hi,
                    spread_b_hi,
                    c,
                    spread_c,
                    d,
                    spread_d,
                    word_lo,
                    spread_word_lo,
                    word_hi,
                    spread_word_hi,
                ),
        )
    }

    // s_upper_sigma_0 on abcd words
    // (2, 11, 9, 10)-bit chunks
    pub fn s_upper_sigma_0(
        s_upper_sigma_0: Expression<F>,
        spread_r0_even: Expression<F>,
        spread_r0_odd: Expression<F>,
        spread_r1_even: Expression<F>,
        spread_r1_odd: Expression<F>,
        a: Expression<F>,
        spread_a: Expression<F>,
        b: Expression<F>,
        spread_b: Expression<F>,
        c_lo: Expression<F>,
        spread_c_lo: Expression<F>,
        c_mid: Expression<F>,
        spread_c_mid: Expression<F>,
        c_hi: Expression<F>,
        spread_c_hi: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Self {
        let decompose_check = Self::decompose_abcd(
            a.clone(),
            spread_a.clone(),
            b.clone(),
            spread_b.clone(),
            c_lo.clone(),
            spread_c_lo.clone(),
            c_mid.clone(),
            spread_c_mid.clone(),
            c_hi.clone(),
            spread_c_hi.clone(),
            d.clone(),
            spread_d.clone(),
            word_lo.clone(),
            spread_word_lo.clone(),
            word_hi.clone(),
            spread_word_hi.clone(),
        );
        let spread_witness = spread_r0_even
            + spread_r0_odd * F::from_u64(2)
            + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);
        let xor_0 = spread_b.clone()
            + spread_c_lo.clone() * F::from_u64(1 << 22)
            + spread_c_mid.clone() * F::from_u64(1 << 28)
            + spread_c_hi.clone() * F::from_u64(1 << 34)
            + spread_d.clone() * F::from_u64(1 << 40)
            + spread_a.clone() * F::from_u64(1 << 60);
        let xor_1 = spread_c_lo.clone()
            + spread_c_mid.clone() * F::from_u64(1 << 6)
            + spread_c_hi.clone() * F::from_u64(1 << 12)
            + spread_d.clone() * F::from_u64(1 << 18)
            + spread_a.clone() * F::from_u64(1 << 38)
            + spread_b.clone() * F::from_u64(1 << 42);
        let xor_2 = spread_d
            + spread_a * F::from_u64(1 << 20)
            + spread_b * F::from_u64(1 << 24)
            + spread_c_lo * F::from_u64(1 << 46)
            + spread_c_mid * F::from_u64(1 << 52)
            + spread_c_hi * F::from_u64(1 << 58);
        let xor = xor_0 + xor_1 + xor_2;

        CompressionGate(s_upper_sigma_0 * (decompose_check + spread_witness + (xor * -F::one())))
    }

    // s_upper_sigma_1 on efgh words
    // (6, 5, 14, 7)-bit chunks
    pub fn s_upper_sigma_1(
        s_upper_sigma_1: Expression<F>,
        spread_r0_even: Expression<F>,
        spread_r0_odd: Expression<F>,
        spread_r1_even: Expression<F>,
        spread_r1_odd: Expression<F>,
        a_lo: Expression<F>,
        spread_a_lo: Expression<F>,
        a_hi: Expression<F>,
        spread_a_hi: Expression<F>,
        b_lo: Expression<F>,
        spread_b_lo: Expression<F>,
        b_hi: Expression<F>,
        spread_b_hi: Expression<F>,
        c: Expression<F>,
        spread_c: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Self {
        let decompose_check = Self::decompose_efgh(
            a_lo.clone(),
            spread_a_lo.clone(),
            a_hi.clone(),
            spread_a_hi.clone(),
            b_lo.clone(),
            spread_b_lo.clone(),
            b_hi.clone(),
            spread_b_hi.clone(),
            c.clone(),
            spread_c.clone(),
            d.clone(),
            spread_d.clone(),
            word_lo.clone(),
            spread_word_lo.clone(),
            word_hi.clone(),
            spread_word_hi.clone(),
        );
        let spread_witness = spread_r0_even
            + spread_r0_odd * F::from_u64(2)
            + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);

        let xor_0 = spread_b_lo.clone()
            + spread_b_hi.clone() * F::from_u64(1 << 4)
            + spread_c.clone() * F::from_u64(1 << 10)
            + spread_d.clone() * F::from_u64(1 << 38)
            + spread_a_lo.clone() * F::from_u64(1 << 52)
            + spread_a_hi.clone() * F::from_u64(1 << 58);
        let xor_1 = spread_c.clone()
            + spread_d.clone() * F::from_u64(1 << 28)
            + spread_a_lo.clone() * F::from_u64(1 << 42)
            + spread_a_hi.clone() * F::from_u64(1 << 48)
            + spread_b_lo.clone() * F::from_u64(1 << 54)
            + spread_b_hi.clone() * F::from_u64(1 << 58);
        let xor_2 = spread_d
            + spread_a_lo * F::from_u64(1 << 14)
            + spread_a_hi * F::from_u64(1 << 20)
            + spread_b_lo * F::from_u64(1 << 26)
            + spread_b_hi * F::from_u64(1 << 30)
            + spread_c * F::from_u64(1 << 36);
        let xor = xor_0 + xor_1 + xor_2;

        CompressionGate(s_upper_sigma_1 * (decompose_check + spread_witness + (xor * -F::one())))
    }

    // Choice gate on (E, F, G)
    pub fn s_ch(
        s_ch: Expression<F>,
        spread_pq_0_even: Expression<F>,
        spread_pq_0_odd: Expression<F>,
        spread_pq_1_even: Expression<F>,
        spread_pq_1_odd: Expression<F>,
        spread_e_lo: Expression<F>,
        spread_e_hi: Expression<F>,
        spread_fg_lo: Expression<F>,
        spread_fg_hi: Expression<F>,
    ) -> Self {
        let lhs_lo = spread_e_lo + spread_fg_lo;
        let lhs_hi = spread_e_hi + spread_fg_hi;
        let lhs = lhs_lo + lhs_hi * F::from_u64(1 << 32);

        let rhs_even = spread_pq_0_even + spread_pq_1_even * F::from_u64(1 << 32);
        let rhs_odd = spread_pq_0_odd + spread_pq_1_odd * F::from_u64(1 << 32);
        let rhs = rhs_even + rhs_odd * F::from_u64(2);

        CompressionGate(s_ch * (lhs + rhs * -F::one()))
    }

    // Majority gate on (A, B, C)
    pub fn s_maj(
        s_maj: Expression<F>,
        spread_m_0_even: Expression<F>,
        spread_m_0_odd: Expression<F>,
        spread_m_1_even: Expression<F>,
        spread_m_1_odd: Expression<F>,
        spread_a_lo: Expression<F>,
        spread_a_hi: Expression<F>,
        spread_b_lo: Expression<F>,
        spread_b_hi: Expression<F>,
        spread_c_lo: Expression<F>,
        spread_c_hi: Expression<F>,
    ) -> Self {
        let maj_even = spread_m_0_even + spread_m_1_even * F::from_u64(1 << 32);
        let maj_odd = spread_m_0_odd + spread_m_1_odd * F::from_u64(1 << 32);
        let maj = maj_even + maj_odd * F::from_u64(2);

        let a = spread_a_lo + spread_a_hi * F::from_u64(1 << 32);
        let b = spread_b_lo + spread_b_hi * F::from_u64(1 << 32);
        let c = spread_c_lo + spread_c_hi * F::from_u64(1 << 32);
        let sum = a + b + c;

        CompressionGate(s_maj * (sum + maj * -F::one()))
    }

    // Negation gate, used in Choice
    pub fn s_neg(
        s_neg: Expression<F>,
        word_lo: Expression<F>,
        word_hi: Expression<F>,
        neg_word_lo: Expression<F>,
        neg_word_hi: Expression<F>,
    ) -> Self {
        let evens = Self::ONES * F::from_u64(MASK_EVEN_32 as u64);
        // evens - word_lo = neg_word_lo
        let lo_check = neg_word_lo + word_lo + (evens.clone() * (-F::one()));
        // evens - word_hi = neg_word_hi
        let hi_check = neg_word_hi + word_hi + (evens * (-F::one()));

        CompressionGate(s_neg * (lo_check + hi_check))
    }

    // s_h_prime to get H' = H + Ch(E, F, G) + s_upper_sigma_1(E) + K + W
    pub fn s_h_prime(
        s_h_prime: Expression<F>,
        h_prime_lo: Expression<F>,
        h_prime_hi: Expression<F>,
        h_prime_carry: Expression<F>,
        sigma_e_lo: Expression<F>,
        sigma_e_hi: Expression<F>,
        ch_efg_lo: Expression<F>,
        ch_efg_hi: Expression<F>,
        h_lo: Expression<F>,
        h_hi: Expression<F>,
        k_lo: Expression<F>,
        k_hi: Expression<F>,
        w_lo: Expression<F>,
        w_hi: Expression<F>,
    ) -> Self {
        let lo = h_lo + ch_efg_lo + sigma_e_lo + k_lo + w_lo;
        let hi = h_hi + ch_efg_hi + sigma_e_hi + k_hi + w_hi;
        let sum = lo + hi * F::from_u64(1 << 16);
        let h_prime = h_prime_lo + h_prime_hi * F::from_u64(1 << 16);

        CompressionGate(
            s_h_prime
                * (sum
                    + h_prime_carry * F::from_u64(1 << 32) * (-F::one())
                    + h_prime * (-F::one())),
        )
    }

    // s_a_new to get A_new = H' + Maj(A, B, C) + s_upper_sigma_0(A)
    pub fn s_a_new(
        s_a_new: Expression<F>,
        a_new_lo: Expression<F>,
        a_new_hi: Expression<F>,
        a_new_carry: Expression<F>,
        sigma_a_lo: Expression<F>,
        sigma_a_hi: Expression<F>,
        maj_abc_lo: Expression<F>,
        maj_abc_hi: Expression<F>,
        h_prime_lo: Expression<F>,
        h_prime_hi: Expression<F>,
    ) -> Self {
        let lo = sigma_a_lo + maj_abc_lo + h_prime_lo;
        let hi = sigma_a_hi + maj_abc_hi + h_prime_hi;
        let sum = lo + hi * F::from_u64(1 << 16);
        let a_new = a_new_lo + a_new_hi * F::from_u64(1 << 16);

        CompressionGate(
            s_a_new
                * (sum + a_new_carry * F::from_u64(1 << 32) * (-F::one()) + a_new * (-F::one())),
        )
    }

    // s_e_new to get E_new = H' + D
    pub fn s_e_new(
        s_e_new: Expression<F>,
        e_new_lo: Expression<F>,
        e_new_hi: Expression<F>,
        e_new_carry: Expression<F>,
        d_lo: Expression<F>,
        d_hi: Expression<F>,
        h_prime_lo: Expression<F>,
        h_prime_hi: Expression<F>,
    ) -> Self {
        let lo = h_prime_lo + d_lo;
        let hi = h_prime_hi + d_hi;
        let sum = lo + hi * F::from_u64(1 << 16);
        let e_new = e_new_lo + e_new_hi * F::from_u64(1 << 16);

        CompressionGate(
            s_e_new
                * (sum + e_new_carry * F::from_u64(1 << 32) * (-F::one()) + e_new * (-F::one())),
        )
    }
}
