use ff::PrimeField;
use halo2::arithmetic::{CurveAffine, FieldExt};

/// Decompose a scalar into `window_num_bits` bits (little-endian)
/// For a window size of `w`, this returns [k_0, ..., k_n] where each `k_i`
/// is a `w`-bit value, and `scalar = k_0 + k_1 * w + k_n * w^n`.
pub fn decompose_scalar_fixed<C: CurveAffine>(
    scalar: C::Scalar,
    scalar_num_bits: usize,
    window_num_bits: usize,
) -> Vec<u8> {
    // Pad bits to multiple of window_num_bits
    let padding = (window_num_bits - (scalar_num_bits % window_num_bits)) % window_num_bits;
    let bits: Vec<bool> = scalar
        .to_le_bits()
        .into_iter()
        .take(scalar_num_bits)
        .chain(std::iter::repeat(false).take(padding))
        .collect();
    assert_eq!(bits.len(), scalar_num_bits + padding);

    bits.chunks_exact(window_num_bits)
        .map(|chunk| chunk.iter().rev().fold(0, |acc, b| (acc << 1) + (*b as u8)))
        .collect()
}

/// Evaluate y = f(x) given the coefficients of f(x)
pub fn evaluate<C: CurveAffine>(x: u8, coeffs: &[C::Base]) -> C::Base {
    let x = C::Base::from_u64(x as u64);
    coeffs
        .iter()
        .rev()
        .fold(C::Base::default(), |acc, coeff| acc * x + coeff)
}
