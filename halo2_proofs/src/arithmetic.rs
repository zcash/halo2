//! This module provides common utilities, traits and structures for group,
//! field and polynomial arithmetic.

mod fft;

use super::multicore::{self, log_threads, prelude::*};
pub use ff::Field;
use fft::*;
use group::{
    ff::{BatchInvert, PrimeField},
    Group as _,
};
pub use pasta_curves::arithmetic::*;

fn multiexp_serial<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C], acc: &mut C::Curve) {
    let coeffs: Vec<_> = coeffs.iter().map(|a| a.to_repr()).collect();

    let c = if bases.len() < 4 {
        1
    } else if bases.len() < 32 {
        3
    } else {
        (f64::from(bases.len() as u32)).ln().ceil() as usize
    };

    fn get_at<F: PrimeField>(segment: usize, c: usize, bytes: &F::Repr) -> usize {
        let skip_bits = segment * c;
        let skip_bytes = skip_bits / 8;

        if skip_bytes >= 32 {
            return 0;
        }

        let mut v = [0; 8];
        for (v, o) in v.iter_mut().zip(bytes.as_ref()[skip_bytes..].iter()) {
            *v = *o;
        }

        let mut tmp = u64::from_le_bytes(v);
        tmp >>= skip_bits - (skip_bytes * 8);
        tmp = tmp % (1 << c);

        tmp as usize
    }

    let segments = (256 / c) + 1;

    for current_segment in (0..segments).rev() {
        for _ in 0..c {
            *acc = acc.double();
        }

        #[derive(Clone, Copy)]
        enum Bucket<C: CurveAffine> {
            None,
            Affine(C),
            Projective(C::Curve),
        }

        impl<C: CurveAffine> Bucket<C> {
            fn add_assign(&mut self, other: &C) {
                *self = match *self {
                    Bucket::None => Bucket::Affine(*other),
                    Bucket::Affine(a) => Bucket::Projective(a + *other),
                    Bucket::Projective(mut a) => {
                        a += *other;
                        Bucket::Projective(a)
                    }
                }
            }

            fn add(self, mut other: C::Curve) -> C::Curve {
                match self {
                    Bucket::None => other,
                    Bucket::Affine(a) => {
                        other += a;
                        other
                    }
                    Bucket::Projective(a) => other + &a,
                }
            }
        }

        let mut buckets: Vec<Bucket<C>> = vec![Bucket::None; (1 << c) - 1];

        for (coeff, base) in coeffs.iter().zip(bases.iter()) {
            let coeff = get_at::<C::Scalar>(current_segment, c, coeff);
            if coeff != 0 {
                buckets[coeff - 1].add_assign(base);
            }
        }

        // Summation by parts
        // e.g. 3a + 2b + 1c = a +
        //                    (a) + b +
        //                    ((a) + b) + c
        let mut running_sum = C::Curve::identity();
        for exp in buckets.into_iter().rev() {
            running_sum = exp.add(running_sum);
            *acc = *acc + &running_sum;
        }
    }
}

/// Performs a small multi-exponentiation operation.
/// Uses the double-and-add algorithm with doublings shared across points.
pub fn small_multiexp<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) -> C::Curve {
    let coeffs: Vec<_> = coeffs.iter().map(|a| a.to_repr()).collect();
    let mut acc = C::Curve::identity();

    // for byte idx
    for byte_idx in (0..32).rev() {
        // for bit idx
        for bit_idx in (0..8).rev() {
            acc = acc.double();
            // for each coeff
            for coeff_idx in 0..coeffs.len() {
                let byte = coeffs[coeff_idx].as_ref()[byte_idx];
                if ((byte >> bit_idx) & 1) != 0 {
                    acc += bases[coeff_idx];
                }
            }
        }
    }

    acc
}

/// Performs a multi-exponentiation operation.
///
/// This function will panic if coeffs and bases have a different length.
///
/// This will use multithreading if beneficial.
pub fn best_multiexp<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) -> C::Curve {
    assert_eq!(coeffs.len(), bases.len());

    let num_threads = multicore::current_num_threads();
    if coeffs.len() > num_threads {
        let chunk = coeffs.len() / num_threads;
        let num_chunks = coeffs.chunks(chunk).len();
        let mut results = vec![C::Curve::identity(); num_chunks];
        multicore::scope(|scope| {
            let chunk = coeffs.len() / num_threads;

            for ((coeffs, bases), acc) in coeffs
                .chunks(chunk)
                .zip(bases.chunks(chunk))
                .zip(results.iter_mut())
            {
                scope.spawn(move |_| {
                    multiexp_serial(coeffs, bases, acc);
                });
            }
        });
        results.iter().fold(C::Curve::identity(), |a, b| a + b)
    } else {
        let mut acc = C::Curve::identity();
        multiexp_serial(coeffs, bases, &mut acc);
        acc
    }
}

/// Performs a radix-$2$ Fast-Fourier Transformation (FFT) on a vector of size
/// $n = 2^k$, when provided `log_n` = $k$ and an element of multiplicative
/// order $n$ called `omega` ($\omega$). The result is that the vector `a`, when
/// interpreted as the coefficients of a polynomial of degree $n - 1$, is
/// transformed into the evaluations of this polynomial at each of the $n$
/// distinct powers of $\omega$. This transformation is invertible by providing
/// $\omega^{-1}$ in place of $\omega$ and dividing each resulting field element
/// by $n$.
pub fn best_fft<G: Group>(a: &mut [G], omega: G::Scalar, log_n: u32) {
    let n = a.len() as usize;
    assert_eq!(n, 1 << log_n);

    // bit reverse permutation
    swap_bit_reverse(a, n, log_n);

    // precompute twiddle factors
    let twiddles: Vec<_> = (0..(n / 2) as usize)
        .scan(G::Scalar::one(), |w, _| {
            let tw = *w;
            w.group_scale(&omega);
            Some(tw)
        })
        .collect();

    if log_n <= log_threads() {
        serial_fft(a, n, log_n, &twiddles)
    } else {
        let (left, right) = a.split_at_mut(n / 2);
        parallel_fft(left, right, n / 2, 2, &twiddles);
        parallel_butterfly_arithmetic(left, right, 1, &twiddles)
    }
}

/// Performs a radix-$2$ inverse Fast-Fourier Transformation (iFFT)
pub fn best_ifft<G: Group>(a: &mut [G], omega_inv: G::Scalar, log_n: u32, divisor: G::Scalar) {
    let n = a.len() as usize;
    assert_eq!(n, 1 << log_n);

    // bit reverse permutation
    swap_bit_reverse(a, n, log_n);

    // precompute twiddle factors
    let twiddles: Vec<_> = (0..(n / 2) as usize)
        .scan(G::Scalar::one(), |w, _| {
            let tw = *w;
            w.group_scale(&omega_inv);
            Some(tw)
        })
        .collect();

    if log_n <= log_threads() {
        serial_fft(a, n, log_n, &twiddles);
        a.iter_mut().for_each(|a| a.group_scale(&divisor))
    } else {
        let (left, right) = a.split_at_mut(n / 2);
        parallel_fft(left, right, n / 2, 2, &twiddles);
        parallel_butterfly_arithmetic_with_divisor(left, right, 1, &twiddles, divisor)
    }
}

/// This evaluates a provided polynomial (in coefficient form) at `point`.
pub fn eval_polynomial<F: Field>(poly: &[F], point: F) -> F {
    // TODO: parallelize?
    poly.iter()
        .rev()
        .fold(F::zero(), |acc, coeff| acc * point + coeff)
}

/// This computes the inner product of two vectors `a` and `b`.
///
/// This function will panic if the two vectors are not the same size.
pub fn compute_inner_product<F: Field>(a: &[F], b: &[F]) -> F {
    // TODO: parallelize?
    assert_eq!(a.len(), b.len());

    let mut acc = F::zero();
    for (a, b) in a.iter().zip(b.iter()) {
        acc += (*a) * (*b);
    }

    acc
}

/// Divides polynomial `a` in `X` by `X - b` with
/// no remainder.
pub fn kate_division<'a, F: Field, I: IntoIterator<Item = &'a F>>(a: I, mut b: F) -> Vec<F>
where
    I::IntoIter: DoubleEndedIterator + ExactSizeIterator,
{
    b = -b;
    let a = a.into_iter();

    let mut q = vec![F::zero(); a.len() - 1];

    let mut tmp = F::zero();
    for (q, r) in q.iter_mut().rev().zip(a.rev()) {
        let mut lead_coeff = *r;
        lead_coeff.sub_assign(&tmp);
        *q = lead_coeff;
        tmp = lead_coeff;
        tmp.mul_assign(&b);
    }

    q
}

/// This simple utility function will parallelize an operation that is to be
/// performed over a mutable slice.
pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Sync + Clone>(v: &mut [T], f: F) {
    let n = v.len();
    let num_threads = multicore::current_num_threads();
    let mut chunk = (n as usize) / num_threads;
    if chunk < num_threads {
        chunk = n as usize;
    }

    multicore::scope(|scope| {
        for (chunk_num, v) in v.chunks_mut(chunk).enumerate() {
            let f = f.clone();
            scope.spawn(move |_| {
                let start = chunk_num * chunk;
                f(v, start);
            });
        }
    });
}

/// Returns coefficients of an n - 1 degree polynomial given a set of n points
/// and their evaluations. This function will panic if two values in `points`
/// are the same.
pub fn lagrange_interpolate<F: FieldExt>(points: &[F], evals: &[F]) -> Vec<F> {
    assert_eq!(points.len(), evals.len());
    if points.len() == 1 {
        // Constant polynomial
        return vec![evals[0]];
    } else {
        let mut denoms = Vec::with_capacity(points.len());
        for (j, x_j) in points.iter().enumerate() {
            let mut denom = Vec::with_capacity(points.len() - 1);
            for x_k in points
                .iter()
                .enumerate()
                .filter(|&(k, _)| k != j)
                .map(|a| a.1)
            {
                denom.push(*x_j - x_k);
            }
            denoms.push(denom);
        }
        // Compute (x_j - x_k)^(-1) for each j != i
        denoms.iter_mut().flat_map(|v| v.iter_mut()).batch_invert();

        let mut final_poly = vec![F::zero(); points.len()];
        for (j, (denoms, eval)) in denoms.into_iter().zip(evals.iter()).enumerate() {
            let mut tmp: Vec<F> = Vec::with_capacity(points.len());
            let mut product = Vec::with_capacity(points.len() - 1);
            tmp.push(F::one());
            for (x_k, denom) in points
                .iter()
                .enumerate()
                .filter(|&(k, _)| k != j)
                .map(|a| a.1)
                .zip(denoms.into_iter())
            {
                product.resize(tmp.len() + 1, F::zero());
                for ((a, b), product) in tmp
                    .iter()
                    .chain(std::iter::once(&F::zero()))
                    .zip(std::iter::once(&F::zero()).chain(tmp.iter()))
                    .zip(product.iter_mut())
                {
                    *product = *a * (-denom * x_k) + *b * denom;
                }
                std::mem::swap(&mut tmp, &mut product);
            }
            assert_eq!(tmp.len(), points.len());
            assert_eq!(product.len(), points.len() - 1);
            for (final_coeff, interpolation_coeff) in final_poly.iter_mut().zip(tmp.into_iter()) {
                *final_coeff += interpolation_coeff * eval;
            }
        }
        final_poly
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::pasta::Fp;
    use crate::poly::EvaluationDomain;
    use rand_core::OsRng;

    #[test]
    fn test_bitreverse() {
        fn bitreverse(mut n: usize, l: usize) -> usize {
            let mut r = 0;
            for _ in 0..l {
                r = (r << 1) | (n & 1);
                n >>= 1;
            }
            r
        }

        for k in 3..10 {
            let n = 1 << k;
            for i in 0..n as u64 {
                assert_eq!(
                    bitreverse(i as usize, k),
                    (i.reverse_bits() >> (64 - k)) as usize
                )
            }
        }
    }

    #[test]
    fn test_fft() {
        fn prev_fft<G: Group>(a: &mut [G], omega: G::Scalar, log_n: u32) {
            let n = a.len() as u32;
            assert_eq!(n, 1 << log_n);

            swap_bit_reverse(a, n as usize, log_n);

            let mut m = 1;
            for _ in 0..log_n {
                let w_m = omega.pow_vartime(&[u64::from(n / (2 * m)), 0, 0, 0]);
                let mut k = 0;
                while k < n {
                    let mut w = G::Scalar::one();
                    for j in 0..m {
                        let mut t = a[(k + j + m) as usize];
                        t.group_scale(&w);
                        a[(k + j + m) as usize] = a[(k + j) as usize];
                        a[(k + j + m) as usize].group_sub(&t);
                        a[(k + j) as usize].group_add(&t);
                        w *= &w_m;
                    }
                    k += 2 * m;
                }
                m *= 2;
            }
        }

        // This checks whether fft algorithm is correct by comparing with previous `serial_fft`
        for k in 3..10 {
            let mut a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let mut b = a.clone();
            let omega = Fp::random(OsRng); // would be weird if this mattered
            prev_fft(&mut a, omega, k);
            best_fft(&mut b, omega, k);
            assert_eq!(a, b);
        }

        // This checks whether `best_ifft` is inverse operation of `best_fft`
        for k in 3..10 {
            let domain = EvaluationDomain::<Fp>::new(1, k);
            let mut a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let b = a.clone();
            best_fft(&mut a, domain.get_omega(), k);
            best_ifft(&mut a, domain.get_omega_inv(), k, domain.get_divisor());
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_lagrange_interpolate() {
        let rng = OsRng;

        let points = (0..5).map(|_| Fp::random(rng)).collect::<Vec<_>>();
        let evals = (0..5).map(|_| Fp::random(rng)).collect::<Vec<_>>();

        for coeffs in 0..5 {
            let points = &points[0..coeffs];
            let evals = &evals[0..coeffs];

            let poly = lagrange_interpolate(points, evals);
            assert_eq!(poly.len(), points.len());

            for (point, eval) in points.iter().zip(evals) {
                assert_eq!(eval_polynomial(&poly, *point), *eval);
            }
        }
    }
}
