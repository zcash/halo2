use ff::PrimeField;
use halo2::arithmetic::{CurveAffine, FieldExt};

/// Decompose a scalar into `window_num_bits` bits (little-endian)
/// For a window size of `w`, this returns [k_0, ..., k_n] where each `k_i`
/// is a `w`-bit value, and `scalar = k_0 + k_1 * w + k_n * w^n`.
/// Note that we are returning a `Vec<u8>` which means the window size is
/// limited to <= 8 bits.
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

#[cfg(test)]
mod tests {
    use super::decompose_scalar_fixed;
    use ff::PrimeField;
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use proptest::prelude::*;
    use std::convert::TryInto;

    prop_compose! {
        fn arb_scalar()(bytes in prop::array::uniform32(0u8..)) -> pallas::Scalar {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Scalar::from_bytes_wide(&buf)
        }
    }

    proptest! {
        #[test]
        fn test_decompose_scalar_fixed(
            scalar in arb_scalar(),
            window_num_bits in 1u8..9
        ) {
            // Get decomposition into `window_num_bits` bits
            let decomposed = decompose_scalar_fixed::<pallas::Affine>(scalar, pallas::Scalar::NUM_BITS as usize, window_num_bits as usize);

            // Flatten bits
            let mut bits: Vec<bool> = decomposed.iter().map(|window| (0..window_num_bits).map(|mask| (window & (1 << mask)) != 0).collect::<Vec<bool>>()
            ).flatten().collect();

            // Pad or truncate bits to 32 bytes
            if bits.len() >= 32 * 8 {
                for bit in bits[32*8..].iter() {
                    assert!(!bit);
                }
                bits = bits[0..32*8].to_vec()
            } else {
                let padding = 32 * 8 - bits.len();
                bits.extend_from_slice(&vec![false; padding]);
            }
            let bytes: Vec<u8> = bits.chunks_exact(8).map(|chunk| chunk.iter().rev().fold(0, |acc, b| (acc << 1) + (*b as u8))).collect();

            // Check that original scalar is recovered from decomposition
            assert_eq!(scalar, pallas::Scalar::from_bytes(&bytes.try_into().unwrap()).unwrap());
        }
    }
}
