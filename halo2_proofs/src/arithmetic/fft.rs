pub use pasta_curves::arithmetic::*;

/// This performs serial butterfly arithmetic
pub(crate) fn serial_fft<G: Group>(a: &mut [G], n: usize, log_n: u32, twiddles: &[G::Scalar]) {
    let mut chunk = 2_usize;
    let mut twiddle_chunk = (n / 2) as usize;
    for _ in 0..log_n {
        a.chunks_mut(chunk).for_each(|coeffs| {
            let (left, right) = coeffs.split_at_mut(chunk / 2);
            butterfly_arithmetic(left, right, twiddle_chunk, twiddles)
        });
        chunk *= 2;
        twiddle_chunk /= 2;
    }
}

/// This perform recursive butterfly arithmetic
pub(crate) fn parallel_fft<G: Group>(
    a: &mut [G],
    n: usize,
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    if n == 2 {
        let t = a[1];
        a[1] = a[0];
        a[0].group_add(&t);
        a[1].group_sub(&t);
    } else {
        let (left, right) = a.split_at_mut(n / 2);
        rayon::join(
            || parallel_fft(left, n / 2, twiddle_chunk * 2, twiddles),
            || parallel_fft(right, n / 2, twiddle_chunk * 2, twiddles),
        );

        butterfly_arithmetic(left, right, twiddle_chunk, twiddles)
    }
}

/// This performs bit reverse permutation over `[G]`
pub(crate) fn swap_bit_reverse<G: Group>(a: &mut [G], n: usize, log_n: u32) {
    assert!(log_n <= 64);
    let diff = 64 - log_n;
    for i in 0..n as u64 {
        let ri = i.reverse_bits() >> diff;
        if i < ri {
            a.swap(ri as usize, i as usize);
        }
    }
}

/// This performs butterfly arithmetic with two `G` array
fn butterfly_arithmetic<G: Group>(
    left: &mut [G],
    right: &mut [G],
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    // case when twiddle factor is one
    let t = right[0];
    right[0] = left[0];
    left[0].group_add(&t);
    right[0].group_sub(&t);

    left.iter_mut()
        .zip(right.iter_mut())
        .enumerate()
        .skip(1)
        .for_each(|(i, (a, b))| {
            let mut t = *b;
            t.group_scale(&twiddles[i * twiddle_chunk]);
            *b = *a;
            a.group_add(&t);
            b.group_sub(&t);
        });
}

#[cfg(test)]
mod tests {
    use super::{swap_bit_reverse, FieldExt};
    use crate::pasta::Fp;
    use ff::Field;
    use proptest::{collection::vec, prelude::*};
    use rand_core::OsRng;

    fn bitreverse(mut n: usize, l: usize) -> usize {
        let mut r = 0;
        for _ in 0..l {
            r = (r << 1) | (n & 1);
            n >>= 1;
        }
        r
    }

    fn arb_poly(k: u32, rng: OsRng) -> Vec<Fp> {
        (0..(1 << k)).map(|_| Fp::random(rng)).collect::<Vec<_>>()
    }

    proptest! {
        #[test]
        fn test_swap_bit_reverse(k in 3u32..10) {
            let mut a = arb_poly(k, OsRng);
            let b = a.clone();
            swap_bit_reverse(&mut a, 1 << k, k);
            for (i, a) in a.iter().enumerate() {
                assert_eq!(*a, b[bitreverse(i, k as usize)]);
            }
        }
    }
}
