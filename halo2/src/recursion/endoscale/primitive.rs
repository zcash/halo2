//! Primitives used in endoscaling.

use group::{Curve, Group};
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

use subtle::CtOption;

/// Maps a pair of bits to a multiple of a scalar using endoscaling.
pub(crate) fn endoscale_pair_scalar<F: FieldExt>(bits: [bool; 2]) -> F {
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
pub(crate) fn endoscale_scalar<F: FieldExt>(acc: Option<F>, bits: &[bool]) -> F {
    assert_eq!(bits.len() % 2, 0);

    let mut acc = if let Some(acc) = acc {
        acc
    } else {
        (F::ZETA + F::one()).double()
    };
    for j in (0..(bits.len() / 2)).rev() {
        let pair = [bits[2 * j], bits[2 * j + 1]];
        let endo = endoscale_pair_scalar::<F>(pair);
        acc = acc.double();
        acc += endo;
    }
    acc
}

/// Maps a pair of bits to a multiple of a base using endoscaling.
pub(crate) fn endoscale_pair<C: CurveAffine>(bits: [bool; 2], base: C) -> CtOption<C> {
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
#[allow(dead_code)]
pub(crate) fn endoscale<C: CurveAffine>(bits: &[bool], base: C) -> C {
    assert_eq!(bits.len() % 2, 0);

    // Initialise accumulator to [2](φ(P) + P)
    let mut acc = (base.to_curve() + base * C::Scalar::ZETA).double();

    for j in (0..(bits.len() / 2)).rev() {
        let pair = [bits[2 * j], bits[2 * j + 1]];
        let endo = endoscale_pair::<C>(pair, base);
        acc = acc.double();
        acc += endo.unwrap();
    }

    acc.to_affine()
}

#[allow(dead_code)]
pub(crate) fn i2lebsp<const NUM_BITS: usize>(int: u64) -> [bool; NUM_BITS] {
    assert!(NUM_BITS <= 64);

    fn gen_const_array<Output: Copy + Default, const LEN: usize>(
        closure: impl FnMut(usize) -> Output,
    ) -> [Output; LEN] {
        fn gen_const_array_with_default<Output: Copy, const LEN: usize>(
            default_value: Output,
            mut closure: impl FnMut(usize) -> Output,
        ) -> [Output; LEN] {
            let mut ret: [Output; LEN] = [default_value; LEN];
            for (bit, val) in ret.iter_mut().zip((0..LEN).map(|idx| closure(idx))) {
                *bit = val;
            }
            ret
        }
        gen_const_array_with_default(Default::default(), closure)
    }

    gen_const_array(|mask: usize| (int & (1 << mask)) != 0)
}

#[test]
fn test_endoscale_primitives() {
    use group::prime::PrimeCurveAffine;
    use pasta_curves::pallas;
    use rand::rngs::OsRng;

    let base = pallas::Point::random(OsRng);
    let bits = [true, false, true, false, false, false, true, true];

    let endoscalar: pallas::Scalar = endoscale_scalar(None, &bits);
    let endoscaled_base = endoscale(&bits, base.to_affine());

    assert_eq!(base * endoscalar, endoscaled_base.to_curve());

    fn endoscale_scalar_by_chunk<F: FieldExt, const K: usize>(bits: &[bool]) -> F {
        assert_eq!(bits.len() % K, 0);

        let mut acc = (F::ZETA + F::one()).double();
        for chunk_idx in (0..(bits.len() / K)).rev() {
            let idx = chunk_idx * K;
            acc = endoscale_scalar(Some(acc), &bits[idx..(idx + K)]);
        }
        acc
    }
    let endoscalar_by_chunk: pallas::Scalar = endoscale_scalar_by_chunk::<_, 4>(&bits);
    let endoscalar_by_pair: pallas::Scalar = endoscale_scalar(None, &bits);
    assert_eq!(endoscalar_by_chunk, endoscalar_by_pair);
}
