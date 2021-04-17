use super::super::{util::*, Gate};
use halo2::{arithmetic::FieldExt, plonk::Expression};

pub struct CompressionGate<F: FieldExt>(pub Expression<F>);

impl<F: FieldExt> CompressionGate<F> {
    fn ones() -> Expression<F> {
        Expression::Constant(F::one())
    }

    // Decompose `A,B,C,D` words
    // (2, 11, 9, 10)-bit chunks
    #[allow(clippy::too_many_arguments)]
    pub fn s_decompose_abcd(
        s_decompose_abcd: Expression<F>,
        a: Expression<F>,
        spread_a: Expression<F>,
        b: Expression<F>,
        spread_b: Expression<F>,
        tag_b: Expression<F>,
        c_lo: Expression<F>,
        spread_c_lo: Expression<F>,
        c_mid: Expression<F>,
        spread_c_mid: Expression<F>,
        c_hi: Expression<F>,
        spread_c_hi: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        tag_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Vec<Self> {
        let check_spread_and_range =
            Gate::three_bit_spread_and_range(c_lo.clone(), spread_c_lo.clone())
                + Gate::three_bit_spread_and_range(c_mid.clone(), spread_c_mid.clone())
                + Gate::three_bit_spread_and_range(c_hi.clone(), spread_c_hi.clone())
                + Gate::two_bit_spread_and_range(a.clone(), spread_a.clone());
        let range_check_tag_b = Gate::range_check(tag_b, 0, 2);
        let range_check_tag_d = Gate::range_check(tag_d, 0, 1);
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

        [
            range_check_tag_b,
            range_check_tag_d,
            dense_check,
            spread_check,
            check_spread_and_range,
        ]
        .iter()
        .map(|expr| CompressionGate(s_decompose_abcd.clone() * expr.clone()))
        .collect::<Vec<_>>()
    }

    // Decompose `E,F,G,H` words
    // (6, 5, 14, 7)-bit chunks
    #[allow(clippy::too_many_arguments)]
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
        tag_c: Expression<F>,
        d: Expression<F>,
        spread_d: Expression<F>,
        tag_d: Expression<F>,
        word_lo: Expression<F>,
        spread_word_lo: Expression<F>,
        word_hi: Expression<F>,
        spread_word_hi: Expression<F>,
    ) -> Vec<Self> {
        let check_spread_and_range =
            Gate::three_bit_spread_and_range(a_lo.clone(), spread_a_lo.clone())
                + Gate::three_bit_spread_and_range(a_hi.clone(), spread_a_hi.clone())
                + Gate::three_bit_spread_and_range(b_hi.clone(), spread_b_hi.clone())
                + Gate::two_bit_spread_and_range(b_lo.clone(), spread_b_lo.clone());
        let range_check_tag_c = Gate::range_check(tag_c, 0, 4);
        let range_check_tag_d = Gate::range_check(tag_d, 0, 0);
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

        [
            range_check_tag_c,
            range_check_tag_d,
            dense_check,
            spread_check,
            check_spread_and_range,
        ]
        .iter()
        .map(|expr| CompressionGate(s_decompose_efgh.clone() * expr.clone()))
        .collect::<Vec<_>>()
    }

    // s_upper_sigma_0 on abcd words
    // (2, 11, 9, 10)-bit chunks
    #[allow(clippy::too_many_arguments)]
    pub fn s_upper_sigma_0(
        s_upper_sigma_0: Expression<F>,
        spread_r0_even: Expression<F>,
        spread_r0_odd: Expression<F>,
        spread_r1_even: Expression<F>,
        spread_r1_odd: Expression<F>,
        spread_a: Expression<F>,
        spread_b: Expression<F>,
        spread_c_lo: Expression<F>,
        spread_c_mid: Expression<F>,
        spread_c_hi: Expression<F>,
        spread_d: Expression<F>,
    ) -> Self {
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

        CompressionGate(s_upper_sigma_0 * (spread_witness + (xor * -F::one())))
    }

    // s_upper_sigma_1 on efgh words
    // (6, 5, 14, 7)-bit chunks
    #[allow(clippy::too_many_arguments)]
    pub fn s_upper_sigma_1(
        s_upper_sigma_1: Expression<F>,
        spread_r0_even: Expression<F>,
        spread_r0_odd: Expression<F>,
        spread_r1_even: Expression<F>,
        spread_r1_odd: Expression<F>,
        spread_a_lo: Expression<F>,
        spread_a_hi: Expression<F>,
        spread_b_lo: Expression<F>,
        spread_b_hi: Expression<F>,
        spread_c: Expression<F>,
        spread_d: Expression<F>,
    ) -> Self {
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

        CompressionGate(s_upper_sigma_1 * (spread_witness + (xor * -F::one())))
    }

    // First part of choice gate on (E, F, G), E ∧ F
    #[allow(clippy::too_many_arguments)]
    pub fn s_ch(
        s_ch: Expression<F>,
        spread_p0_even: Expression<F>,
        spread_p0_odd: Expression<F>,
        spread_p1_even: Expression<F>,
        spread_p1_odd: Expression<F>,
        spread_e_lo: Expression<F>,
        spread_e_hi: Expression<F>,
        spread_f_lo: Expression<F>,
        spread_f_hi: Expression<F>,
    ) -> Self {
        let lhs_lo = spread_e_lo + spread_f_lo;
        let lhs_hi = spread_e_hi + spread_f_hi;
        let lhs = lhs_lo + lhs_hi * F::from_u64(1 << 32);

        let rhs_even = spread_p0_even + spread_p1_even * F::from_u64(1 << 32);
        let rhs_odd = spread_p0_odd + spread_p1_odd * F::from_u64(1 << 32);
        let rhs = rhs_even + rhs_odd * F::from_u64(2);

        CompressionGate(s_ch * (lhs + rhs * -F::one()))
    }

    // Second part of Choice gate on (E, F, G), ¬E ∧ G
    #[allow(clippy::too_many_arguments)]
    pub fn s_ch_neg(
        s_ch_neg: Expression<F>,
        spread_q0_even: Expression<F>,
        spread_q0_odd: Expression<F>,
        spread_q1_even: Expression<F>,
        spread_q1_odd: Expression<F>,
        spread_e_lo: Expression<F>,
        spread_e_hi: Expression<F>,
        spread_e_neg_lo: Expression<F>,
        spread_e_neg_hi: Expression<F>,
        spread_g_lo: Expression<F>,
        spread_g_hi: Expression<F>,
    ) -> Vec<Self> {
        let neg_check = Self::neg_check(
            spread_e_lo,
            spread_e_hi,
            spread_e_neg_lo.clone(),
            spread_e_neg_hi.clone(),
        );
        let lhs_lo = spread_e_neg_lo + spread_g_lo;
        let lhs_hi = spread_e_neg_hi + spread_g_hi;
        let lhs = lhs_lo + lhs_hi * F::from_u64(1 << 32);

        let rhs_even = spread_q0_even + spread_q1_even * F::from_u64(1 << 32);
        let rhs_odd = spread_q0_odd + spread_q1_odd * F::from_u64(1 << 32);
        let rhs = rhs_even + rhs_odd * F::from_u64(2);

        [neg_check, lhs + rhs * -F::one()]
            .iter()
            .map(|expr| CompressionGate(s_ch_neg.clone() * expr.clone()))
            .collect::<Vec<_>>()
    }

    // Majority gate on (A, B, C)
    #[allow(clippy::too_many_arguments)]
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

    // Negation gate, used in second part of Choice gate
    fn neg_check(
        word_lo: Expression<F>,
        word_hi: Expression<F>,
        neg_word_lo: Expression<F>,
        neg_word_hi: Expression<F>,
    ) -> Expression<F> {
        let evens = Self::ones() * F::from_u64(MASK_EVEN_32 as u64);
        // evens - word_lo = neg_word_lo
        let lo_check = neg_word_lo + word_lo + (evens.clone() * (-F::one()));
        // evens - word_hi = neg_word_hi
        let hi_check = neg_word_hi + word_hi + (evens * (-F::one()));

        lo_check + hi_check
    }

    // s_h_prime to get H' = H + Ch(E, F, G) + s_upper_sigma_1(E) + K + W
    #[allow(clippy::too_many_arguments)]
    pub fn s_h_prime(
        s_h_prime: Expression<F>,
        h_prime_lo: Expression<F>,
        h_prime_hi: Expression<F>,
        h_prime_carry: Expression<F>,
        sigma_e_lo: Expression<F>,
        sigma_e_hi: Expression<F>,
        ch_lo: Expression<F>,
        ch_hi: Expression<F>,
        ch_neg_lo: Expression<F>,
        ch_neg_hi: Expression<F>,
        h_lo: Expression<F>,
        h_hi: Expression<F>,
        k_lo: Expression<F>,
        k_hi: Expression<F>,
        w_lo: Expression<F>,
        w_hi: Expression<F>,
    ) -> Self {
        let lo = h_lo + ch_lo + ch_neg_lo + sigma_e_lo + k_lo + w_lo;
        let hi = h_hi + ch_hi + ch_neg_hi + sigma_e_hi + k_hi + w_hi;

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
    #[allow(clippy::too_many_arguments)]
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
    #[allow(clippy::too_many_arguments)]
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

    fn check_lo_hi(lo: Expression<F>, hi: Expression<F>, word: Expression<F>) -> Expression<F> {
        lo + hi * F::from_u64(1 << 16) + (word * (-F::one()))
    }

    // s_digest on final round
    #[allow(clippy::too_many_arguments)]
    pub fn s_digest(
        s_digest: Expression<F>,
        lo_0: Expression<F>,
        hi_0: Expression<F>,
        word_0: Expression<F>,
        lo_1: Expression<F>,
        hi_1: Expression<F>,
        word_1: Expression<F>,
        lo_2: Expression<F>,
        hi_2: Expression<F>,
        word_2: Expression<F>,
        lo_3: Expression<F>,
        hi_3: Expression<F>,
        word_3: Expression<F>,
    ) -> Vec<Self> {
        [
            Self::check_lo_hi(lo_0, hi_0, word_0),
            Self::check_lo_hi(lo_1, hi_1, word_1),
            Self::check_lo_hi(lo_2, hi_2, word_2),
            Self::check_lo_hi(lo_3, hi_3, word_3),
        ]
        .iter()
        .map(|expr| CompressionGate(s_digest.clone() * expr.clone()))
        .collect::<Vec<_>>()
    }
}
