use super::multicore::{self, log_threads, prelude::*};
pub use pasta_curves::arithmetic::*;

/// This performs serial butterfly arithmetic
pub(crate) fn serial_fft<G: Group>(a: &mut [G], n: usize, log_n: u32, twiddles: &[G::Scalar]) {
    let mut chunk = 2_usize;
    let mut twiddle_chunk = (n / 2) as usize;
    for _ in 0..log_n {
        a.chunks_mut(chunk).for_each(|coeffs| {
            let (left, right) = coeffs.split_at_mut(chunk / 2);
            serial_butterfly_arithmetic(left, right, twiddle_chunk, twiddles)
        });
        chunk *= 2;
        twiddle_chunk /= 2;
    }
}

/// This performs recursive butterfly arithmetic
pub(crate) fn parallel_fft<G: Group>(
    left: &mut [G],
    right: &mut [G],
    n: usize,
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    if n == 8 {
        left.chunks_mut(2)
            .zip(right.chunks_mut(2))
            .for_each(|(a, b)| self_swap_arithmetic(a, b, 0, 1));
        let (a, b) = left.split_at_mut(n / 2);
        let (c, d) = right.split_at_mut(n / 2);
        let (a1, a2) = a.split_at_mut(n / 4);
        let (b1, b2) = b.split_at_mut(n / 4);
        let (c1, c2) = c.split_at_mut(n / 4);
        let (d1, d2) = d.split_at_mut(n / 4);
        double_butterfly_arithmetic(a1, a2, b1, b2, twiddle_chunk * 2, twiddles);
        double_butterfly_arithmetic(c1, c2, d1, d2, twiddle_chunk * 2, twiddles);
        double_butterfly_arithmetic(a, b, c, d, twiddle_chunk, twiddles);
    } else {
        let (a, b) = left.split_at_mut(n / 2);
        let (c, d) = right.split_at_mut(n / 2);
        rayon::join(
            || parallel_fft(a, b, n / 2, twiddle_chunk * 2, twiddles),
            || parallel_fft(c, d, n / 2, twiddle_chunk * 2, twiddles),
        );
        double_butterfly_arithmetic(a, b, c, d, twiddle_chunk, twiddles);
    }
}

/// This performs butterfly arithmetic with two `G` array
pub(crate) fn serial_butterfly_arithmetic<G: Group>(
    left: &mut [G],
    right: &mut [G],
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    // case when twiddle factor is one
    swap_arithmetic(&mut left[0], &mut right[0]);

    left.iter_mut()
        .zip(right.iter_mut())
        .enumerate()
        .skip(1)
        .for_each(|(i, (a, b))| swap_arithmetic_with_twiddle(a, b, &twiddles[i * twiddle_chunk]));
}

/// This performs butterfly arithmetic with two `G` array
pub(crate) fn parallel_butterfly_arithmetic<G: Group>(
    left: &mut [G],
    right: &mut [G],
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    // case when twiddle factor is one
    swap_arithmetic(&mut left[0], &mut right[0]);

    left.par_iter_mut()
        .zip(right.par_iter_mut())
        .enumerate()
        .skip(1)
        .for_each(|(i, (a, b))| swap_arithmetic_with_twiddle(a, b, &twiddles[i * twiddle_chunk]));
}

/// This performs butterfly arithmetic with two `G` array and divisor
pub(crate) fn parallel_butterfly_arithmetic_with_divisor<G: Group>(
    left: &mut [G],
    right: &mut [G],
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
    divisor: G::Scalar,
) {
    // case when twiddle factor is one
    swap_arithmetic(&mut left[0], &mut right[0]);
    left[0].group_scale(&divisor);
    right[0].group_scale(&divisor);

    left.par_iter_mut()
        .zip(right.par_iter_mut())
        .enumerate()
        .skip(1)
        .for_each(|(i, (a, b))| {
            swap_arithmetic_with_twiddle(a, b, &twiddles[i * twiddle_chunk]);
            a.group_scale(&divisor);
            b.group_scale(&divisor)
        });
}

/// This performs butterfly arithmetic with four `G` array
pub(crate) fn double_butterfly_arithmetic<G: Group>(
    a: &mut [G],
    b: &mut [G],
    c: &mut [G],
    d: &mut [G],
    twiddle_chunk: usize,
    twiddles: &[G::Scalar],
) {
    // case when twiddle factor is one
    swap_arithmetic(&mut a[0], &mut b[0]);
    swap_arithmetic(&mut c[0], &mut d[0]);

    a.iter_mut()
        .zip(b.iter_mut())
        .zip(c.iter_mut())
        .zip(d.iter_mut())
        .enumerate()
        .skip(1)
        .for_each(|(i, (((a, b), c), d))| {
            swap_arithmetic_with_twiddle(a, b, &twiddles[i * twiddle_chunk]);
            swap_arithmetic_with_twiddle(c, d, &twiddles[i * twiddle_chunk]);
        });
}

/// This performs bit reverse permutation over `[G]`
pub(crate) fn swap_bit_reverse<G: Group>(a: &mut [G], n: usize, log_n: u32) {
    // sort by bit reverse
    let diff = 64 - log_n;
    for i in 0..n as u64 {
        let ri = i.reverse_bits() >> diff;
        if i < ri {
            a.swap(ri as usize, i as usize);
        }
    }
}

/// This performs permutation and butterfly arithmetic without twiddle for `G`
pub(crate) fn swap_arithmetic<G: Group>(a: &mut G, b: &mut G) {
    let t = *b;
    *b = *a;
    a.group_add(&t);
    b.group_sub(&t);
}

/// This performs permutation and butterfly arithmetic twiddle for `G`
pub(crate) fn swap_arithmetic_with_twiddle<G: Group>(a: &mut G, b: &mut G, twiddle: &G::Scalar) {
    let mut t = *b;
    t.group_scale(twiddle);
    *b = *a;
    a.group_add(&t);
    b.group_sub(&t);
}

/// This performs permutation and butterfly arithmetic without twiddle for `[G]`
pub(crate) fn self_swap_arithmetic<G: Group>(
    a: &mut [G],
    b: &mut [G],
    former: usize,
    latter: usize,
) {
    let t = a[latter];
    a[latter] = a[former];
    a[former].group_add(&t);
    a[latter].group_sub(&t);
    let t = b[latter];
    b[latter] = b[former];
    b[former].group_add(&t);
    b[latter].group_sub(&t);
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
