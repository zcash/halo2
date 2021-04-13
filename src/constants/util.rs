use ff::PrimeField;
use halo2::arithmetic::{CurveAffine, FieldExt};

/// Decompose a scalar into FIXED_BASE_WINDOW_SIZE bits (little-endian)
/// For a window size of `w`, this returns [k_0, ..., k_n] where each `k_i`
/// is a `w`-bit value, and `scalar = k_0 + k_1 * w + k_n * w^n`.
pub fn decompose_scalar_fixed<C: CurveAffine>(
    scalar: C::Scalar,
    scalar_num_bits: usize,
    window_num_bits: usize,
) -> Vec<u8> {
    let mut bits: Vec<bool> = scalar
        .to_le_bits()
        .into_iter()
        .take(scalar_num_bits)
        .collect();

    assert_eq!(bits.len(), scalar_num_bits);

    // Pad bits to multiple of window_num_bits
    bits.append(&mut vec![
        false;
        (window_num_bits
            - (scalar_num_bits % window_num_bits))
            % window_num_bits
    ]);

    bits.chunks_exact(window_num_bits)
        .map(|chunk| {
            let mut chunk = chunk.iter();
            *(chunk.next().unwrap()) as u8
                + ((*(chunk.next().unwrap()) as u8) << 1)
                + ((*(chunk.next().unwrap()) as u8) << 2)
        })
        .collect()
}

/// Evaluate y = f(x) given the coefficients of f(x)
pub fn evaluate<C: CurveAffine>(x: u8, coeffs: &[C::Base]) -> C::Base {
    coeffs
        .iter()
        .enumerate()
        .fold(C::Base::default(), |acc, (pow, coeff)| {
            acc + (*coeff) * C::Base::from_u64(x as u64).pow(&[pow as u64, 0, 0, 0])
        })
}
