use super::multicore::{self, log_threads, prelude::*};
use crate::arithmetic::*;

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
    if n == 4 {
        self_swap_arithmetic(left, right, 0, 1);
        self_swap_arithmetic(left, right, 2, 3);
        self_swap_arithmetic(left, right, 0, 2);
        self_swap_arithmetic_with_twiddle(left, right, &twiddles[twiddle_chunk], 1, 3);
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

/// This performs permutation and butterfly arithmetic twiddle for `[G]`
pub(crate) fn self_swap_arithmetic_with_twiddle<G: Group>(
    a: &mut [G],
    b: &mut [G],
    twiddle: &G::Scalar,
    former: usize,
    latter: usize,
) {
    let mut t = a[latter];
    t.group_scale(twiddle);
    a[latter] = a[former];
    a[former].group_add(&t);
    a[latter].group_sub(&t);
    let mut t = b[latter];
    t.group_scale(twiddle);
    b[latter] = b[former];
    b[former].group_add(&t);
    b[latter].group_sub(&t);
}
