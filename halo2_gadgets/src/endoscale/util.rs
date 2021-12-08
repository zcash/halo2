//! Primitives used in endoscaling.

use group::{Curve, Group};
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

use subtle::CtOption;

/// Maps a pair of bits to a multiple of a scalar using endoscaling.
pub(crate) fn compute_endoscalar_pair<F: FieldExt>(bits: [bool; 2]) -> F {
    // [2 * bits.0 - 1]
    let mut scalar = F::from(bits[0]).double() - F::one();

    if bits[1] {
        scalar *= F::ZETA;
    }

    scalar
}

/// Maps a K-bit bitstring to a scalar.
///
/// This corresponds to Algorithm 1 from [BGH2019], where `F` corresponds to $F_q$, the
/// scalar field of $P$. Where Algorithm 1 computes $Acc = [scalar] P$, this function
/// computes `scalar`.
///
/// [BGH2019]: https://eprint.iacr.org/2019/1021.pdf
#[allow(dead_code)]
pub(crate) fn compute_endoscalar<F: FieldExt>(bits: &[bool]) -> F {
    compute_endoscalar_with_acc(None, bits)
}

/// Maps a K-bit bitstring to a scalar.
///
/// This function takes an optional accumulator which can be initialised to some value.
/// This is convenient when chunking the bitstring being endoscaled is a partial chunk
/// in some larger bitstring.
///
/// # Panics
/// Panics if there is an odd number of bits.
pub(crate) fn compute_endoscalar_with_acc<F: FieldExt>(acc: Option<F>, bits: &[bool]) -> F {
    assert_eq!(bits.len() % 2, 0);

    let mut acc = acc.unwrap_or_else(|| (F::ZETA + F::one()).double());

    for j in (0..(bits.len() / 2)).rev() {
        let pair = [bits[2 * j], bits[2 * j + 1]];
        let endo = compute_endoscalar_pair::<F>(pair);
        acc = acc.double();
        acc += endo;
    }
    acc
}

/// Maps a pair of bits to a multiple of a base using endoscaling.
///
/// # Panics
/// Panics if the base is the identity.
pub(crate) fn endoscale_point_pair<C: CurveAffine>(bits: [bool; 2], base: C) -> CtOption<C> {
    assert!(!bool::from(base.to_curve().is_identity()));

    let mut base = {
        let base = base.coordinates();
        (*base.unwrap().x(), *base.unwrap().y())
    };

    if !bits[0] {
        base.1 = -base.1;
    }

    if bits[1] {
        base.0 *= C::Base::ZETA;
    }

    C::from_xy(base.0, base.1)
}

/// Maps a K-bit bitstring to a multiple of a given base.
///
/// This is Algorithm 1 from [BGH2019](https://eprint.iacr.org/2019/1021.pdf).
///
/// # Panics
/// Panics if the base is the identity.
/// Panics if there is an odd number of bits.
#[allow(dead_code)]
pub(crate) fn endoscale_point<C: CurveAffine>(bits: &[bool], base: C) -> C {
    assert_eq!(bits.len() % 2, 0);
    assert!(!bool::from(base.to_curve().is_identity()));

    // Initialise accumulator to [2](φ(P) + P)
    let mut acc = (base.to_curve() + base * C::Scalar::ZETA).double();

    for j in (0..(bits.len() / 2)).rev() {
        let pair = [bits[2 * j], bits[2 * j + 1]];
        let endo = endoscale_point_pair::<C>(pair, base);
        acc = acc.double();
        acc += endo.unwrap();
    }

    acc.to_affine()
}

#[cfg(test)]
mod tests {
    use super::*;
    use group::prime::PrimeCurveAffine;
    use pasta_curves::pallas;
    use rand::{random, rngs::OsRng};

    #[test]
    fn test_alg1_alg2() {
        let base = pallas::Point::random(OsRng);
        let num_bits = 128;
        let bits: Vec<_> = std::iter::repeat(random::<bool>()).take(num_bits).collect();

        let endoscalar: pallas::Scalar = compute_endoscalar(&bits);
        let endoscaled_base = endoscale_point(&bits, base.to_affine());

        assert_eq!(base * endoscalar, endoscaled_base.to_curve());
    }

    fn shift_padded_endo<F: FieldExt, const K: usize>(padded_endo: F, k_prime: usize) -> F {
        //   (1 - 2^{(K - K')/2}) * 2^{K'/2}
        // = 2^{K'/2} - 2^{K/2}
        let shift = F::from(1 << (k_prime / 2)) - F::from(1 << (K / 2));
        padded_endo - shift
    }

    /// Test that shifting the endoscalar of the padded chunk recovers the
    /// same result as directly endoscaling the original chunk.
    fn endo_partial_chunk<F: FieldExt, const K: usize>(k_prime: usize) {
        assert!(k_prime > 0);
        assert!(k_prime < K);
        let bits: Vec<_> = std::iter::repeat(random::<bool>()).take(k_prime).collect();

        let padding = std::iter::repeat(false).take(K - k_prime);
        let padded_bits: Vec<_> = bits.iter().copied().chain(padding).collect();
        let padded_endo = compute_endoscalar_with_acc(Some(F::zero()), &padded_bits);

        let endo = shift_padded_endo::<_, K>(padded_endo, k_prime);

        assert_eq!(endo, compute_endoscalar_with_acc(Some(F::zero()), &bits));
    }

    #[test]
    /// Test that shifting the endoscalar of the padded chunk recovers the
    /// same result as directly endoscaling the original chunk.
    fn test_endo_partial_chunk() {
        endo_partial_chunk::<pallas::Base, 10>(2);
        endo_partial_chunk::<pallas::Base, 10>(4);
        endo_partial_chunk::<pallas::Base, 10>(6);
        endo_partial_chunk::<pallas::Base, 10>(8);
    }

    fn endo_chunk<F: FieldExt, const K: usize>(num_bits: usize) {
        let bits: Vec<_> = std::iter::repeat(random::<bool>()).take(num_bits).collect();
        let endoscalar_by_pair = compute_endoscalar(&bits);

        let pad_len = (K - (num_bits % K)) % K;
        let two_pow_k_div_two = F::from(1u64 << (K / 2));

        // Pad bits from the right with `pad_len` zeros
        let bits: Vec<_> = bits
            .iter()
            .copied()
            .chain(std::iter::repeat(false).take(pad_len))
            .collect();

        // Initialise accumulator
        let mut acc = (F::ZETA + F::one()).double();

        let mut chunks = bits.chunks(K).rev();
        let last_chunk = chunks.next().unwrap();
        let last_endo = compute_endoscalar_with_acc(Some(F::zero()), last_chunk);

        // If the last chunk was padded, adjust it for a shift.
        if pad_len > 0 {
            let k_prime = K - pad_len;
            let two_pow_k_prime_div_two = F::from(1u64 << (k_prime / 2));
            let shifted_endo = shift_padded_endo::<_, K>(last_endo, k_prime);
            acc = acc * two_pow_k_prime_div_two + shifted_endo;
        } else {
            acc = acc * two_pow_k_div_two + last_endo;
        };

        for chunk in chunks.rev() {
            let endo = compute_endoscalar_with_acc(Some(F::zero()), chunk);
            acc = acc * two_pow_k_div_two + endo;
        }

        assert_eq!(acc, endoscalar_by_pair);
    }

    #[test]
    fn test_endo_chunk() {
        endo_chunk::<pallas::Base, 2>(8);
        endo_chunk::<pallas::Base, 4>(8);
        endo_chunk::<pallas::Base, 6>(8);
        endo_chunk::<pallas::Base, 8>(8);
        endo_chunk::<pallas::Base, 10>(8);
    }
}
