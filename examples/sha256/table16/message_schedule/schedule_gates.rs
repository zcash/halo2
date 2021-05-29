use super::super::gates;
use halo2::{arithmetic::FieldExt, plonk::Expression};

/// s_word for W_16 to W_63
#[allow(clippy::too_many_arguments)]
pub fn s_word<F: FieldExt>(
    s_word: Expression<F>,
    sigma_0_lo: Expression<F>,
    sigma_0_hi: Expression<F>,
    sigma_1_lo: Expression<F>,
    sigma_1_hi: Expression<F>,
    w_minus_9_lo: Expression<F>,
    w_minus_9_hi: Expression<F>,
    w_minus_16_lo: Expression<F>,
    w_minus_16_hi: Expression<F>,
    word: Expression<F>,
    carry: Expression<F>,
) -> Vec<Expression<F>> {
    let lo = sigma_0_lo + sigma_1_lo + w_minus_9_lo + w_minus_16_lo;
    let hi = sigma_0_hi + sigma_1_hi + w_minus_9_hi + w_minus_16_hi;

    let word_check = lo
        + hi * F::from_u64(1 << 16)
        + (carry.clone() * F::from_u64(1 << 32) * (-F::one()))
        + (word * (-F::one()));
    let carry_check = gates::range_check(carry, 0, 3);

    [word_check, carry_check]
        .iter()
        .map(|expr| s_word.clone() * expr.clone())
        .collect()
}

/// s_decompose_0 for all words
pub fn s_decompose_0<F: FieldExt>(
    s_decompose_0: Expression<F>,
    lo: Expression<F>,
    hi: Expression<F>,
    word: Expression<F>,
) -> Vec<Expression<F>> {
    vec![s_decompose_0 * (lo + hi * F::from_u64(1 << 16) + word * (-F::one()))]
}

/// s_decompose_1 for W_1 to W_13
/// (3, 4, 11, 14)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_decompose_1<F: FieldExt>(
    s_decompose_1: Expression<F>,
    a: Expression<F>,
    b: Expression<F>,
    c: Expression<F>,
    tag_c: Expression<F>,
    d: Expression<F>,
    tag_d: Expression<F>,
    word: Expression<F>,
) -> Vec<Expression<F>> {
    let decompose_check = a
        + b * F::from_u64(1 << 3)
        + c * F::from_u64(1 << 7)
        + d * F::from_u64(1 << 18)
        + word * (-F::one());
    let range_check_tag_c = gates::range_check(tag_c, 0, 2);
    let range_check_tag_d = gates::range_check(tag_d, 0, 4);

    [decompose_check, range_check_tag_c, range_check_tag_d]
        .iter()
        .map(|expr| s_decompose_1.clone() * expr.clone())
        .collect()
}

/// s_decompose_2 for W_14 to W_48
/// (3, 4, 3, 7, 1, 1, 13)-bit chunks
#[allow(clippy::many_single_char_names)]
#[allow(clippy::too_many_arguments)]
pub fn s_decompose_2<F: FieldExt>(
    s_decompose_2: Expression<F>,
    a: Expression<F>,
    b: Expression<F>,
    c: Expression<F>,
    d: Expression<F>,
    tag_d: Expression<F>,
    e: Expression<F>,
    f: Expression<F>,
    g: Expression<F>,
    tag_g: Expression<F>,
    word: Expression<F>,
) -> Vec<Expression<F>> {
    let decompose_check = a
        + b * F::from_u64(1 << 3)
        + c * F::from_u64(1 << 7)
        + d * F::from_u64(1 << 10)
        + e * F::from_u64(1 << 17)
        + f * F::from_u64(1 << 18)
        + g * F::from_u64(1 << 19)
        + word * (-F::one());
    let range_check_tag_d = gates::range_check(tag_d, 0, 0);
    let range_check_tag_g = gates::range_check(tag_g, 0, 3);

    [decompose_check, range_check_tag_g, range_check_tag_d]
        .iter()
        .map(|expr| s_decompose_2.clone() * expr.clone())
        .collect()
}

/// s_decompose_3 for W_49 to W_61
/// (10, 7, 2, 13)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_decompose_3<F: FieldExt>(
    s_decompose_3: Expression<F>,
    a: Expression<F>,
    tag_a: Expression<F>,
    b: Expression<F>,
    c: Expression<F>,
    d: Expression<F>,
    tag_d: Expression<F>,
    word: Expression<F>,
) -> Vec<Expression<F>> {
    let decompose_check = a
        + b * F::from_u64(1 << 10)
        + c * F::from_u64(1 << 17)
        + d * F::from_u64(1 << 19)
        + word * (-F::one());
    let range_check_tag_a = gates::range_check(tag_a, 0, 1);
    let range_check_tag_d = gates::range_check(tag_d, 0, 3);

    [decompose_check, range_check_tag_a, range_check_tag_d]
        .iter()
        .map(|expr| s_decompose_3.clone() * expr.clone())
        .collect()
}

/// b_lo + 2^2 * b_mid = b, on W_[1..49]
fn check_b<F: FieldExt>(
    b: Expression<F>,
    b_lo: Expression<F>,
    b_hi: Expression<F>,
) -> Expression<F> {
    let expected_b = b_lo + b_hi * F::from_u64(1 << 2);
    expected_b + (b * -F::one())
}

/// b_lo + 2^2 * b_mid + 2^4 * b_hi = b, on W_[49..62]
fn check_b1<F: FieldExt>(
    b: Expression<F>,
    b_lo: Expression<F>,
    b_mid: Expression<F>,
    b_hi: Expression<F>,
) -> Expression<F> {
    let expected_b = b_lo + b_mid * F::from_u64(1 << 2) + b_hi * F::from_u64(1 << 4);
    expected_b + (b * -F::one())
}

/// sigma_0 v1 on W_1 to W_13
/// (3, 4, 11, 14)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_lower_sigma_0<F: FieldExt>(
    s_lower_sigma_0: Expression<F>,
    spread_r0_even: Expression<F>,
    spread_r0_odd: Expression<F>,
    spread_r1_even: Expression<F>,
    spread_r1_odd: Expression<F>,
    a: Expression<F>,
    spread_a: Expression<F>,
    b: Expression<F>,
    b_lo: Expression<F>,
    spread_b_lo: Expression<F>,
    b_hi: Expression<F>,
    spread_b_hi: Expression<F>,
    spread_c: Expression<F>,
    spread_d: Expression<F>,
) -> Vec<Expression<F>> {
    let check_b = check_b(b, b_lo.clone(), b_hi.clone());
    let spread_witness = spread_r0_even
        + spread_r0_odd * F::from_u64(2)
        + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);
    let xor_0 = spread_b_lo.clone()
        + spread_b_hi.clone() * F::from_u64(1 << 4)
        + spread_c.clone() * F::from_u64(1 << 8)
        + spread_d.clone() * F::from_u64(1 << 30);
    let xor_1 = spread_c.clone()
        + spread_d.clone() * F::from_u64(1 << 22)
        + spread_a.clone() * F::from_u64(1 << 50)
        + spread_b_lo.clone() * F::from_u64(1 << 56)
        + spread_b_hi.clone() * F::from_u64(1 << 60);
    let xor_2 = spread_d
        + spread_a.clone() * F::from_u64(1 << 28)
        + spread_b_lo.clone() * F::from_u64(1 << 34)
        + spread_b_hi.clone() * F::from_u64(1 << 38)
        + spread_c.clone() * F::from_u64(1 << 42);
    let xor = xor_0 + xor_1 + xor_2;

    [check_b, spread_witness + (xor * -F::one())]
        .iter()
        .chain(gates::two_bit_spread_and_range(b_lo, spread_b_lo).iter())
        .chain(gates::two_bit_spread_and_range(b_hi, spread_b_hi).iter())
        .chain(gates::three_bit_spread_and_range(a, spread_a).iter())
        .map(|expr| s_lower_sigma_0.clone() * expr.clone())
        .collect()
}

/// sigma_1 v1 on W_49 to W_61
/// (10, 7, 2, 13)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_lower_sigma_1<F: FieldExt>(
    s_lower_sigma_1: Expression<F>,
    spread_r0_even: Expression<F>,
    spread_r0_odd: Expression<F>,
    spread_r1_even: Expression<F>,
    spread_r1_odd: Expression<F>,
    spread_a: Expression<F>,
    b: Expression<F>,
    b_lo: Expression<F>,
    spread_b_lo: Expression<F>,
    b_mid: Expression<F>,
    spread_b_mid: Expression<F>,
    b_hi: Expression<F>,
    spread_b_hi: Expression<F>,
    c: Expression<F>,
    spread_c: Expression<F>,
    spread_d: Expression<F>,
) -> Vec<Expression<F>> {
    let check_b1 = check_b1(b, b_lo.clone(), b_mid.clone(), b_hi.clone());
    let spread_witness = spread_r0_even
        + spread_r0_odd * F::from_u64(2)
        + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);
    let xor_0 = spread_b_lo.clone()
        + spread_b_mid.clone() * F::from_u64(1 << 4)
        + spread_b_hi.clone() * F::from_u64(1 << 8)
        + spread_c.clone() * F::from_u64(1 << 14)
        + spread_d.clone() * F::from_u64(1 << 18);
    let xor_1 = spread_c.clone()
        + spread_d.clone() * F::from_u64(1 << 4)
        + spread_a.clone() * F::from_u64(1 << 30)
        + spread_b_lo.clone() * F::from_u64(1 << 50)
        + spread_b_mid.clone() * F::from_u64(1 << 54)
        + spread_b_hi.clone() * F::from_u64(1 << 58);
    let xor_2 = spread_d
        + spread_a.clone() * F::from_u64(1 << 26)
        + spread_b_lo.clone() * F::from_u64(1 << 46)
        + spread_b_mid.clone() * F::from_u64(1 << 50)
        + spread_b_hi.clone() * F::from_u64(1 << 54)
        + spread_c.clone() * F::from_u64(1 << 60);
    let xor = xor_0 + xor_1 + xor_2;

    [check_b1, spread_witness + (xor * -F::one())]
        .iter()
        .chain(gates::two_bit_spread_and_range(b_lo, spread_b_lo).iter())
        .chain(gates::two_bit_spread_and_range(b_mid, spread_b_mid).iter())
        .chain(gates::two_bit_spread_and_range(c, spread_c).iter())
        .chain(gates::three_bit_spread_and_range(b_hi, spread_b_hi).iter())
        .map(|expr| s_lower_sigma_1.clone() * expr.clone())
        .collect()
}

/// sigma_0 v2 on W_14 to W_48
/// (3, 4, 3, 7, 1, 1, 13)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_lower_sigma_0_v2<F: FieldExt>(
    s_lower_sigma_0_v2: Expression<F>,
    spread_r0_even: Expression<F>,
    spread_r0_odd: Expression<F>,
    spread_r1_even: Expression<F>,
    spread_r1_odd: Expression<F>,
    a: Expression<F>,
    spread_a: Expression<F>,
    b: Expression<F>,
    b_lo: Expression<F>,
    spread_b_lo: Expression<F>,
    b_hi: Expression<F>,
    spread_b_hi: Expression<F>,
    c: Expression<F>,
    spread_c: Expression<F>,
    spread_d: Expression<F>,
    spread_e: Expression<F>,
    spread_f: Expression<F>,
    spread_g: Expression<F>,
) -> Vec<Expression<F>> {
    let check_b = check_b(b, b_lo.clone(), b_hi.clone());
    let spread_witness = spread_r0_even
        + spread_r0_odd * F::from_u64(2)
        + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);
    let xor_0 = spread_b_lo.clone()
        + spread_b_hi.clone() * F::from_u64(1 << 4)
        + spread_c.clone() * F::from_u64(1 << 8)
        + spread_d.clone() * F::from_u64(1 << 14)
        + spread_e.clone() * F::from_u64(1 << 28)
        + spread_f.clone() * F::from_u64(1 << 30)
        + spread_g.clone() * F::from_u64(1 << 32);
    let xor_1 = spread_c.clone()
        + spread_d.clone() * F::from_u64(1 << 6)
        + spread_e.clone() * F::from_u64(1 << 20)
        + spread_f.clone() * F::from_u64(1 << 22)
        + spread_g.clone() * F::from_u64(1 << 24)
        + spread_a.clone() * F::from_u64(1 << 50)
        + spread_b_lo.clone() * F::from_u64(1 << 56)
        + spread_b_hi.clone() * F::from_u64(1 << 60);
    let xor_2 = spread_f
        + spread_g * F::from_u64(1 << 2)
        + spread_a.clone() * F::from_u64(1 << 28)
        + spread_b_lo.clone() * F::from_u64(1 << 34)
        + spread_b_hi.clone() * F::from_u64(1 << 38)
        + spread_c.clone() * F::from_u64(1 << 42)
        + spread_d * F::from_u64(1 << 48)
        + spread_e * F::from_u64(1 << 62);
    let xor = xor_0 + xor_1 + xor_2;

    [check_b, spread_witness + (xor * -F::one())]
        .iter()
        .chain(gates::two_bit_spread_and_range(b_lo, spread_b_lo).iter())
        .chain(gates::two_bit_spread_and_range(b_hi, spread_b_hi).iter())
        .chain(gates::three_bit_spread_and_range(a, spread_a).iter())
        .chain(gates::three_bit_spread_and_range(c, spread_c).iter())
        .map(|expr| s_lower_sigma_0_v2.clone() * expr.clone())
        .collect()
}

/// sigma_1 v2 on W_14 to W_48
/// (3, 4, 3, 7, 1, 1, 13)-bit chunks
#[allow(clippy::too_many_arguments)]
pub fn s_lower_sigma_1_v2<F: FieldExt>(
    s_lower_sigma_1_v2: Expression<F>,
    spread_r0_even: Expression<F>,
    spread_r0_odd: Expression<F>,
    spread_r1_even: Expression<F>,
    spread_r1_odd: Expression<F>,
    a: Expression<F>,
    spread_a: Expression<F>,
    b: Expression<F>,
    b_lo: Expression<F>,
    spread_b_lo: Expression<F>,
    b_hi: Expression<F>,
    spread_b_hi: Expression<F>,
    c: Expression<F>,
    spread_c: Expression<F>,
    spread_d: Expression<F>,
    spread_e: Expression<F>,
    spread_f: Expression<F>,
    spread_g: Expression<F>,
) -> Vec<Expression<F>> {
    let check_b = check_b(b, b_lo.clone(), b_hi.clone());
    let spread_witness = spread_r0_even
        + spread_r0_odd * F::from_u64(2)
        + (spread_r1_even + spread_r1_odd * F::from_u64(2)) * F::from_u64(1 << 32);
    let xor_0 = spread_d.clone()
        + spread_e.clone() * F::from_u64(1 << 14)
        + spread_f.clone() * F::from_u64(1 << 16)
        + spread_g.clone() * F::from_u64(1 << 18);
    let xor_1 = spread_e.clone()
        + spread_f.clone() * F::from_u64(1 << 2)
        + spread_g.clone() * F::from_u64(1 << 4)
        + spread_a.clone() * F::from_u64(1 << 30)
        + spread_b_lo.clone() * F::from_u64(1 << 36)
        + spread_b_hi.clone() * F::from_u64(1 << 40)
        + spread_c.clone() * F::from_u64(1 << 44)
        + spread_d.clone() * F::from_u64(1 << 50);
    let xor_2 = spread_g
        + spread_a.clone() * F::from_u64(1 << 26)
        + spread_b_lo.clone() * F::from_u64(1 << 32)
        + spread_b_hi.clone() * F::from_u64(1 << 36)
        + spread_c.clone() * F::from_u64(1 << 40)
        + spread_d * F::from_u64(1 << 46)
        + spread_e * F::from_u64(1 << 60)
        + spread_f * F::from_u64(1 << 62);
    let xor = xor_0 + xor_1 + xor_2;

    [check_b, spread_witness + (xor * -F::one())]
        .iter()
        .chain(gates::two_bit_spread_and_range(b_lo, spread_b_lo).iter())
        .chain(gates::two_bit_spread_and_range(b_hi, spread_b_hi).iter())
        .chain(gates::three_bit_spread_and_range(a, spread_a).iter())
        .chain(gates::three_bit_spread_and_range(c, spread_c).iter())
        .map(|expr| s_lower_sigma_1_v2.clone() * expr.clone())
        .collect()
}
