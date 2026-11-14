//! This module provides common utilities, traits and structures for group,
//! field and polynomial arithmetic.

pub use ff::Field;
use group::{ff::PrimeField, Group as _, GroupOpsOwned, ScalarMulOwned};
use maybe_rayon::prelude::*;
pub use pasta_curves::arithmetic::*;

use crate::multicore::{self, TheBestReduce};

/// This represents an element of a group with basic operations that can be
/// performed. This allows an FFT implementation (for example) to operate
/// generically over either a field or elliptic curve group.
pub trait FftGroup<Scalar: Field>:
    Copy + Send + Sync + 'static + GroupOpsOwned + ScalarMulOwned<Scalar>
{
}

impl<T, Scalar> FftGroup<Scalar> for T
where
    Scalar: Field,
    T: Copy + Send + Sync + 'static + GroupOpsOwned + ScalarMulOwned<Scalar>,
{
}

#[derive(Clone, Copy)]
enum Bucket<C: CurveAffine> {
    None,
    Affine(C),
    Projective(C::Curve),
}

impl<C: CurveAffine> Bucket<C> {
    fn add_assign(&mut self, other: C) {
        *self = match *self {
            Bucket::None => Bucket::Affine(other),
            Bucket::Affine(a) => Bucket::Projective(a + other),
            Bucket::Projective(mut a) => {
                a += other;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BoothDigit {
    magnitude: usize,
    negative: bool,
}

fn booth_digit(bytes: &[u8], window_bits: usize, window: usize) -> BoothDigit {
    debug_assert!(window_bits > 0);
    debug_assert!(window_bits < u64::BITS as usize);

    let window_start = window * window_bits;
    let byte_start = window_start / 8;
    let mut encoded = [0; 8];
    if byte_start < bytes.len() {
        for (slot, byte) in encoded.iter_mut().zip(&bytes[byte_start..]) {
            *slot = *byte;
        }
    }

    let bit_offset = window_start - byte_start * 8;
    let radix = 1usize << window_bits;
    let value = ((u64::from_le_bytes(encoded) >> bit_offset) as usize) & (radix - 1);
    let overlap = if window_start == 0 {
        0
    } else {
        let bit = window_start - 1;
        bytes
            .get(bit / 8)
            .map_or(0, |byte| usize::from((byte >> (bit % 8)) & 1))
    };

    // The bit below each window is its carry-in, while the window's high bit
    // is its carry-out. These terms cancel between adjacent windows, leaving
    // a signed digit whose magnitude is at most half the radix.
    if value < radix / 2 {
        BoothDigit {
            magnitude: value + overlap,
            negative: false,
        }
    } else {
        let magnitude = radix - value - overlap;
        BoothDigit {
            magnitude,
            negative: magnitude != 0,
        }
    }
}

fn booth_window_count<R: AsRef<[u8]>>(reprs: &[R], window_bits: usize) -> usize {
    debug_assert!(window_bits > 0);

    let repr_bytes = reprs
        .iter()
        .map(|repr| repr.as_ref().len())
        .max()
        .unwrap_or(0);
    let repr_bits = repr_bytes
        .checked_mul(u8::BITS as usize)
        .expect("scalar representation is too large");

    // Integer division reserves an additional carry window exactly when the
    // representation ends on a window boundary. Otherwise, the last partial
    // window cannot produce a carry.
    repr_bits / window_bits + 1
}

#[derive(Clone)]
struct BoothBuckets<C: CurveAffine> {
    c: usize,
    coeffs: Vec<Bucket<C>>,
}

impl<C: CurveAffine> BoothBuckets<C> {
    fn new(c: usize) -> Self {
        Self {
            c,
            coeffs: vec![Bucket::None; 1 << (c - 1)],
        }
    }

    fn sum(
        &mut self,
        coeffs: &[<C::Scalar as PrimeField>::Repr],
        bases: &[C],
        i: usize,
    ) -> C::Curve {
        // Recode each scalar window into a signed digit and add its base to
        // the bucket for the digit's magnitude.
        for (coeff, base) in coeffs.iter().zip(bases.iter()) {
            let digit = booth_digit(coeff.as_ref(), self.c, i);
            if digit.magnitude != 0 {
                let base = if digit.negative { -*base } else { *base };
                self.coeffs[digit.magnitude - 1].add_assign(base);
            }
        }
        // Summation by parts
        // e.g. 3a + 2b + 1c = a +
        //                    (a) + b +
        //                    ((a) + b) + c
        let mut acc = C::Curve::identity();
        let mut sum = C::Curve::identity();
        self.coeffs.iter().rev().for_each(|b| {
            sum = b.add(sum);
            acc += sum;
        });
        acc
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

    // Convert to canonical representations once instead of once per window.
    let coeffs = coeffs.iter().map(PrimeField::to_repr).collect::<Vec<_>>();

    let c = if bases.len() < 4 {
        1
    } else if bases.len() < 32 {
        3
    } else {
        (f64::from(bases.len() as u32)).ln().ceil() as usize
    };

    let mut multi_buckets: Vec<BoothBuckets<C>> =
        vec![BoothBuckets::new(c); booth_window_count(&coeffs, c)];
    let num_threads = multicore::current_num_threads();
    if should_parallelize_multiexp(coeffs.len(), num_threads) {
        multi_buckets
            .par_iter_mut()
            .enumerate()
            .rev()
            .map(|(i, buckets)| {
                let mut acc = buckets.sum(&coeffs, bases, i);
                (0..c * i).for_each(|_| acc = acc.double());
                acc
            })
            .the_best_reduce(C::Curve::identity, |a, b| a + b)
            .expect("multi_buckets always contains at least 1 bucket")
    } else {
        multi_buckets
            .iter_mut()
            .enumerate()
            .rev()
            .map(|(i, buckets)| buckets.sum(&coeffs, bases, i))
            .fold(C::Curve::identity(), |mut sum, bucket| {
                // restore original evaluation point
                (0..c).for_each(|_| sum = sum.double());
                sum + bucket
            })
    }
}

fn should_parallelize_multiexp(num_coeffs: usize, num_threads: usize) -> bool {
    // The parallel algorithm shifts each window result independently. With a
    // single worker, this only adds doublings compared to serial evaluation.
    num_threads > 1 && num_coeffs > num_threads
}

/// Performs a radix-$2$ Fast-Fourier Transformation (FFT) on a vector of size
/// $n = 2^k$, when provided `log_n` = $k$ and an element of multiplicative
/// order $n$ called `omega` ($\omega$). The result is that the vector `a`, when
/// interpreted as the coefficients of a polynomial of degree $n - 1$, is
/// transformed into the evaluations of this polynomial at each of the $n$
/// distinct powers of $\omega$. This transformation is invertible by providing
/// $\omega^{-1}$ in place of $\omega$ and dividing each resulting field element
/// by $n$.
///
/// This will use multithreading if beneficial.
pub fn best_fft<Scalar: Field, G: FftGroup<Scalar>>(a: &mut [G], omega: Scalar, log_n: u32) {
    let twiddles = fft_twiddles(omega, a.len() / 2);
    fft_with_twiddles(a, &twiddles, log_n);
}

/// Below this many elements, the FFT twiddle table is built sequentially:
/// splitting it across threads costs more than it saves.
const TWIDDLES_PARALLEL_THRESHOLD: usize = 1 << 13;

/// Returns `[1, omega, omega^2, ..., omega^(len - 1)]`, the twiddle factors
/// used by [`best_fft`] for a transform of size `2 * len`.
///
/// Each element is an exact field value, so splitting the sequence into
/// chunks that each start from `omega^start` gives the same table as the
/// sequential running product.
pub(crate) fn fft_twiddles<Scalar: Field>(omega: Scalar, len: usize) -> Vec<Scalar> {
    let mut twiddles = vec![Scalar::ONE; len];
    let fill = |chunk: &mut [Scalar], start: usize| {
        let mut w = omega.pow_vartime([start as u64]);
        for tw in chunk {
            *tw = w;
            w *= &omega;
        }
    };
    if len < TWIDDLES_PARALLEL_THRESHOLD {
        fill(&mut twiddles, 0);
    } else {
        parallelize(&mut twiddles, fill);
    }
    twiddles
}

/// Below this many elements, the bit-reversal permutation swaps in place
/// sequentially. The parallel version gathers from a copy, which only pays
/// off once the vector no longer fits in cache: measured with 14 threads, it
/// is slower at 2^13 elements (35 us against 20 us) and faster at 2^15
/// (67 us against 122 us).
const BITREVERSE_PARALLEL_THRESHOLD: usize = 1 << 15;

/// Permutes `a` (of length `2^log_n`) so that index `i` holds the element
/// that was at the bit reversal of `i`.
pub(crate) fn bitreverse_permutation<T: Copy + Send + Sync>(a: &mut [T], log_n: u32) {
    if log_n == 0 {
        return;
    }
    let shift = usize::BITS - log_n;
    let reverse = |i: usize| i.reverse_bits() >> shift;
    if a.len() < BITREVERSE_PARALLEL_THRESHOLD {
        for i in 0..a.len() {
            let r = reverse(i);
            if i < r {
                a.swap(r, i);
            }
        }
    } else {
        // A bit reversal is an involution made of disjoint swaps, but the
        // swaps of one chunk reach into every other chunk, so the parallel
        // version gathers from a copy instead of swapping in place.
        let source = a.to_vec();
        parallelize(a, |chunk, start| {
            for (offset, value) in chunk.iter_mut().enumerate() {
                *value = source[reverse(start + offset)];
            }
        });
    }
}

/// Performs the same transformation as [`best_fft`], given the twiddle table
/// `[1, omega, ..., omega^(n/2 - 1)]` for `n = 2^log_n`.
///
/// Passing the table lets a caller that transforms many vectors over the
/// same domain compute it once (see [`fft_twiddles`]).
pub(crate) fn fft_with_twiddles<Scalar: Field, G: FftGroup<Scalar>>(
    a: &mut [G],
    twiddles: &[Scalar],
    log_n: u32,
) {
    let threads = multicore::current_num_threads();
    let log_threads = log2_floor(threads);
    let n = a.len();
    assert_eq!(n, 1 << log_n);
    assert_eq!(twiddles.len(), n / 2);

    bitreverse_permutation(a, log_n);

    if log_n <= log_threads {
        let mut chunk = 2_usize;
        let mut twiddle_chunk = n / 2;
        for _ in 0..log_n {
            a.chunks_mut(chunk).for_each(|coeffs| {
                let (left, right) = coeffs.split_at_mut(chunk / 2);

                // case when twiddle factor is one
                let (a, left) = left.split_at_mut(1);
                let (b, right) = right.split_at_mut(1);
                let t = b[0];
                b[0] = a[0];
                a[0] += &t;
                b[0] -= &t;

                left.iter_mut()
                    .zip(right.iter_mut())
                    .enumerate()
                    .for_each(|(i, (a, b))| {
                        let mut t = *b;
                        t *= &twiddles[(i + 1) * twiddle_chunk];
                        *b = *a;
                        *a += &t;
                        *b -= &t;
                    });
            });
            chunk *= 2;
            twiddle_chunk /= 2;
        }
    } else {
        recursive_butterfly_arithmetic(a, n, 1, twiddles)
    }
}

/// This perform recursive butterfly arithmetic
pub fn recursive_butterfly_arithmetic<Scalar: Field, G: FftGroup<Scalar>>(
    a: &mut [G],
    n: usize,
    twiddle_chunk: usize,
    twiddles: &[Scalar],
) {
    if n == 2 {
        let t = a[1];
        a[1] = a[0];
        a[0] += &t;
        a[1] -= &t;
    } else {
        let (left, right) = a.split_at_mut(n / 2);
        multicore::join(
            || recursive_butterfly_arithmetic(left, n / 2, twiddle_chunk * 2, twiddles),
            || recursive_butterfly_arithmetic(right, n / 2, twiddle_chunk * 2, twiddles),
        );

        // case when twiddle factor is one
        let (a, left) = left.split_at_mut(1);
        let (b, right) = right.split_at_mut(1);
        let t = b[0];
        b[0] = a[0];
        a[0] += &t;
        b[0] -= &t;

        left.iter_mut()
            .zip(right.iter_mut())
            .enumerate()
            .for_each(|(i, (a, b))| {
                let mut t = *b;
                t *= &twiddles[(i + 1) * twiddle_chunk];
                *b = *a;
                *a += &t;
                *b -= &t;
            });
    }
}

/// This evaluates a provided polynomial (in coefficient form) at `point`.
///
/// Horner's rule is a fold of the affine maps `t -> t * point + coeff`. The
/// maps compose associatively, so long polynomials are split into chunks
/// that are evaluated in parallel and combined with a second, short Horner
/// pass in `point^chunk_len`. The result is exactly the sequential value.
// Without the inline hint, the dispatch below stops this function from being
// inlined into its callers, which made the sequential path 25% slower (46 us
// against 36 us at 2^11 coefficients).
#[inline]
pub fn eval_polynomial<F: Field>(poly: &[F], point: F) -> F {
    // Measured with 14 threads, splitting does not pay at 2^11 coefficients
    // and is 3.2x faster at 2^14.
    if poly.len() > EVAL_PARALLEL_THRESHOLD && multicore::current_num_threads() > 1 {
        eval_polynomial_parallel(poly, point)
    } else {
        horner(poly, point)
    }
}

/// Horner's rule on one thread.
#[inline]
fn horner<F: Field>(poly: &[F], point: F) -> F {
    poly.iter()
        .rev()
        .fold(F::ZERO, |acc, coeff| acc * point + coeff)
}

/// The parallel path of [`eval_polynomial`].
#[inline(never)]
fn eval_polynomial_parallel<F: Field>(poly: &[F], point: F) -> F {
    let chunk_len = poly.len().div_ceil(multicore::current_num_threads());
    let chunk_evals: Vec<F> = poly
        .chunks(chunk_len)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|chunk| horner(chunk, point))
        .collect();
    let shift = point.pow_vartime([chunk_len as u64]);
    chunk_evals
        .iter()
        .rev()
        .fold(F::ZERO, |acc, chunk_eval| acc * shift + chunk_eval)
}

/// This computes the inner product of two vectors `a` and `b`.
///
/// This function will panic if the two vectors are not the same size.
pub fn compute_inner_product<F: Field>(a: &[F], b: &[F]) -> F {
    // TODO: parallelize?
    assert_eq!(a.len(), b.len());

    let mut acc = F::ZERO;
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

    let mut q = vec![F::ZERO; a.len() - 1];

    let mut tmp = F::ZERO;
    for (q, r) in q.iter_mut().rev().zip(a.rev()) {
        let mut lead_coeff = *r;
        lead_coeff.sub_assign(&tmp);
        *q = lead_coeff;
        tmp = lead_coeff;
        tmp.mul_assign(&b);
    }

    q
}

/// Evaluates each `(polynomial, point)` pair, in parallel across pairs, and
/// returns the values in the order given.
///
/// After the challenge `x`, the prover evaluates every committed polynomial
/// at one or more rotations of `x`; the evaluations are independent, and
/// for small circuits each one is too short to split on its own.
pub(crate) fn evaluate_batch<F: Field>(queries: &[(&[F], F)]) -> Vec<F> {
    queries
        .into_par_iter()
        .map(|(poly, point)| eval_polynomial(poly, *point))
        .collect()
}

/// Divides the polynomial `a` by `X - b`, returning the quotient; the same
/// as [`kate_division`], in parallel for long polynomials.
///
/// The quotient satisfies `q[i] = a[i + 1] + b q[i + 1]` from the top
/// (`q[m] = 0` for `a` of length `m + 1`), a linear recurrence. Within a
/// chunk `[s, e)` of indices, `q[i] = l[i] + b^(e - i) q[e]` where `l` is
/// the recurrence run from zero at `e`. So chunks run the recurrence locally
/// in parallel, a short pass from the top carries `q[e]` across chunks, and
/// each chunk adds its correction in parallel. The result is exact.
pub(crate) fn kate_division_parallel<F: Field>(a: &[F], b: F) -> Vec<F> {
    if a.len() <= 1 {
        return vec![];
    }
    let mut q = vec![F::ZERO; a.len() - 1];
    let Some(chunk_len) = scan_chunk_len(q.len()) else {
        let mut carry = F::ZERO;
        for (i, q) in q.iter_mut().enumerate().rev() {
            carry = a[i + 1] + b * carry;
            *q = carry;
        }
        return q;
    };

    for_each_chunk(&mut q, chunk_len, |index, chunk| {
        let start = index * chunk_len;
        let mut carry = F::ZERO;
        for (offset, q) in chunk.iter_mut().enumerate().rev() {
            carry = a[start + offset + 1] + b * carry;
            *q = carry;
        }
    });

    // `carries[c]` is the true `q[e]` for chunk `c` ending at `e`. The last
    // chunk ends at `q.len()`, where `q` is zero.
    let num_chunks = q.len().div_ceil(chunk_len);
    let mut carries = vec![F::ZERO; num_chunks];
    for c in (0..num_chunks - 1).rev() {
        let next_start = (c + 1) * chunk_len;
        let next_len = chunk_len.min(q.len() - next_start);
        carries[c] = q[next_start] + b.pow_vartime([next_len as u64]) * carries[c + 1];
    }

    for_each_chunk(&mut q, chunk_len, |index, chunk| {
        let carry = carries[index];
        if carry.is_zero_vartime() {
            return;
        }
        let mut correction = carry;
        for q in chunk.iter_mut().rev() {
            correction *= b;
            *q += correction;
        }
    });
    q
}

/// This simple utility function will parallelize an operation that is to be
/// performed over a mutable slice.
pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Sync + Clone>(v: &mut [T], f: F) {
    let n = v.len();
    let num_threads = multicore::current_num_threads();
    let mut chunk = n / num_threads;
    if chunk < num_threads {
        chunk = n;
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

/// Below this many elements, the scans below run sequentially: the extra
/// pass and the thread synchronisation cost more than they save.
const SCAN_PARALLEL_THRESHOLD: usize = 1 << 11;

/// At or below this many coefficients, `eval_polynomial` runs Horner's rule
/// on one thread.
const EVAL_PARALLEL_THRESHOLD: usize = 1 << 11;

/// Returns the chunk length for a parallel scan over `n` elements, or `None`
/// if the scan should run sequentially.
fn scan_chunk_len(n: usize) -> Option<usize> {
    let threads = multicore::current_num_threads();
    if threads < 2 || n < SCAN_PARALLEL_THRESHOLD {
        None
    } else {
        Some(n.div_ceil(threads))
    }
}

/// Runs `f(chunk_index, chunk)` on consecutive chunks of `chunk_len`
/// elements of `v`, in parallel.
fn for_each_chunk<T: Send, F: Fn(usize, &mut [T]) + Send + Sync>(
    v: &mut [T],
    chunk_len: usize,
    f: F,
) {
    let f = &f;
    multicore::scope(|scope| {
        for (index, chunk) in v.chunks_mut(chunk_len).enumerate() {
            scope.spawn(move |_| f(index, chunk));
        }
    });
}

/// Inverts every nonzero element of `v` in place, leaving zeros unchanged.
///
/// This is Montgomery's trick (a product scan, one inversion, and a backward
/// scan). The scans are sequential, so the parallel version splits `v` into
/// chunks and runs the trick on each one: it costs one field inversion per
/// chunk and produces exactly the same values.
pub(crate) fn parallel_batch_invert<F: VartimeField>(v: &mut [F]) {
    match scan_chunk_len(v.len()) {
        None => {
            v.iter_mut().batch_invert_vartime();
        }
        Some(chunk_len) => for_each_chunk(v, chunk_len, |_, chunk| {
            chunk.iter_mut().batch_invert_vartime();
        }),
    }
}

/// Returns the `len` running products `[1, v_0, v_0 v_1, ...,
/// v_0 ... v_{len-2}]` of `values`, the evaluations of a grand-product
/// polynomial starting from one.
///
/// A prefix product only needs associativity, so it parallelises as a
/// two-level scan: each chunk computes its local running products, a
/// sequential pass multiplies the chunk totals into chunk offsets, and each
/// chunk is then scaled by its offset. Field arithmetic is exact, so the
/// result equals the sequential one element for element.
///
/// Panics if `len` is zero or `values` has fewer than `len - 1` elements.
pub(crate) fn grand_product<F: Field>(values: &[F], len: usize) -> Vec<F> {
    assert!(len >= 1 && values.len() >= len - 1);
    let mut products = vec![F::ONE; len];
    let (_, running) = products.split_at_mut(1);
    let factors = &values[..len - 1];

    let Some(chunk_len) = scan_chunk_len(running.len()) else {
        let mut acc = F::ONE;
        for (product, factor) in running.iter_mut().zip(factors) {
            acc *= factor;
            *product = acc;
        }
        return products;
    };

    for_each_chunk(running, chunk_len, |index, chunk| {
        let mut acc = F::ONE;
        for (product, factor) in chunk.iter_mut().zip(&factors[index * chunk_len..]) {
            acc *= factor;
            *product = acc;
        }
    });
    let offsets: Vec<F> = running
        .chunks(chunk_len)
        .scan(F::ONE, |acc, chunk| {
            let offset = *acc;
            *acc *= chunk[chunk.len() - 1];
            Some(offset)
        })
        .collect();
    for_each_chunk(running, chunk_len, |index, chunk| {
        if index > 0 {
            for product in chunk {
                *product *= offsets[index];
            }
        }
    });
    products
}

/// Returns the grand products of several chained sets: set `j` starts from
/// the last value of set `j - 1` instead of from one, as the sets of the
/// permutation argument do. Each output has `len` elements, and the value
/// carried into the next set is the last of them.
///
/// The chain looks sequential, but set `j` is just the grand product of its
/// own factors starting from one, scaled by the carried-in scalar. So every
/// set is computed independently and in parallel, and only the rescaling is
/// chained. That costs an extra multiplication per element, so short sets
/// (measured: four sets of 2^11 elements, 208 us in parallel against 153 us
/// sequentially) run the plain sequential chain instead.
pub(crate) fn chained_grand_products<F: Field>(sets: Vec<Vec<F>>, len: usize) -> Vec<Vec<F>> {
    assert!(len >= 1);
    // Same criterion as `grand_product`, which scans `len - 1` factors.
    if scan_chunk_len(len - 1).is_none() {
        let mut carried = F::ONE;
        return sets
            .iter()
            .map(|factors| {
                let mut products = Vec::with_capacity(len);
                products.push(carried);
                for factor in &factors[..len - 1] {
                    products.push(products[products.len() - 1] * factor);
                }
                carried = products[len - 1];
                products
            })
            .collect();
    }
    let mut products: Vec<Vec<F>> = sets
        .into_par_iter()
        .map(|factors| grand_product(&factors, len))
        .collect();
    let mut carried = F::ONE;
    for set in products.iter_mut() {
        if carried != F::ONE {
            parallelize(set, |set, _| {
                for value in set {
                    *value *= carried;
                }
            });
        }
        carried = set[len - 1];
    }
    products
}

fn log2_floor(num: usize) -> u32 {
    assert!(num > 0);

    let mut pow = 0;

    while (1 << (pow + 1)) <= num {
        pow += 1;
    }

    pow
}

/// Returns coefficients of an n - 1 degree polynomial given a set of n points
/// and their evaluations. This function will panic if two values in `points`
/// are the same.
pub fn lagrange_interpolate<F: VartimeField>(points: &[F], evals: &[F]) -> Vec<F> {
    assert_eq!(points.len(), evals.len());
    if points.len() == 1 {
        // Constant polynomial
        vec![evals[0]]
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
        denoms
            .iter_mut()
            .flat_map(|v| v.iter_mut())
            .batch_invert_vartime();

        let mut final_poly = vec![F::ZERO; points.len()];
        for (j, (denoms, eval)) in denoms.into_iter().zip(evals.iter()).enumerate() {
            let mut tmp: Vec<F> = Vec::with_capacity(points.len());
            let mut product = Vec::with_capacity(points.len() - 1);
            tmp.push(F::ONE);
            for (x_k, denom) in points
                .iter()
                .enumerate()
                .filter(|&(k, _)| k != j)
                .map(|a| a.1)
                .zip(denoms)
            {
                product.resize(tmp.len() + 1, F::ZERO);
                for ((a, b), product) in tmp
                    .iter()
                    .chain(std::iter::once(&F::ZERO))
                    .zip(std::iter::once(&F::ZERO).chain(tmp.iter()))
                    .zip(product.iter_mut())
                {
                    *product = *a * (-denom * x_k) + *b * denom;
                }
                std::mem::swap(&mut tmp, &mut product);
            }
            assert_eq!(tmp.len(), points.len());
            assert_eq!(product.len(), points.len() - 1);
            for (final_coeff, interpolation_coeff) in final_poly.iter_mut().zip(tmp) {
                *final_coeff += interpolation_coeff * eval;
            }
        }
        final_poly
    }
}

#[cfg(test)]
use rand::rngs::SysRng;
#[cfg(test)]
use rand_core::UnwrapErr;

#[cfg(test)]
use crate::pasta::{Ep, EpAffine, Eq, EqAffine, Fp, Fq};

#[cfg(test)]
fn assert_multiexp_matches_naive<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) {
    let expected = coeffs
        .iter()
        .zip(bases)
        .map(|(coeff, base)| *base * coeff)
        .fold(C::Curve::identity(), |acc, val| acc + val);

    assert_eq!(best_multiexp(coeffs, bases), expected);
}

#[cfg(test)]
fn evaluate_le_bytes(bytes: &[u8]) -> Fp {
    bytes.iter().rev().fold(Fp::ZERO, |mut acc, byte| {
        for bit in (0..u8::BITS).rev() {
            acc = acc.double();
            if (byte >> bit) & 1 != 0 {
                acc += Fp::ONE;
            }
        }
        acc
    })
}

#[cfg(test)]
fn evaluate_booth_digits(bytes: &[u8], window_bits: usize) -> Fp {
    (0..booth_window_count(&[bytes], window_bits))
        .rev()
        .fold(Fp::ZERO, |mut acc, window| {
            for _ in 0..window_bits {
                acc = acc.double();
            }

            let digit = booth_digit(bytes, window_bits, window);
            let magnitude = Fp::from(digit.magnitude as u64);
            if digit.negative {
                acc -= magnitude;
            } else {
                acc += magnitude;
            }
            acc
        })
}

#[test]
fn test_multiexp() {
    let mut rng = UnwrapErr(SysRng);
    for len in [0, 1, 2, 3, 4, 31, 32, 33, 255, 256, 257] {
        let coeffs = (0..len).map(|_| Fp::random(&mut rng)).collect::<Vec<_>>();
        let bases = (0..len)
            .map(|_| EqAffine::from(Eq::random(&mut rng)))
            .collect::<Vec<_>>();

        assert_multiexp_matches_naive(&coeffs, &bases);
    }
}

#[test]
fn test_booth_digit_boundaries() {
    let assert_digit = |bytes: &[u8], window_bits, window, magnitude, negative| {
        assert_eq!(
            booth_digit(bytes, window_bits, window),
            BoothDigit {
                magnitude,
                negative,
            }
        );
    };

    let repr_bytes = <Fp as PrimeField>::Repr::default().as_ref().len();
    let mut bytes = vec![0; repr_bytes];
    assert_digit(&bytes, 3, 0, 0, false);

    bytes[0] = 3;
    assert_digit(&bytes, 3, 0, 3, false);

    bytes[0] = 4;
    assert_digit(&bytes, 3, 0, 4, true);
    assert_digit(&bytes, 3, 1, 1, false);

    bytes[0] = 7;
    assert_digit(&bytes, 3, 0, 1, true);
    assert_digit(&bytes, 3, 1, 1, false);

    let bytes = vec![u8::MAX; repr_bytes];
    assert_digit(&bytes, 8, 31, 0, false);
    assert_digit(&bytes, 8, 32, 1, false);
}

#[test]
fn test_booth_digit_wide_representation() {
    const WIDE_REPR_BYTES: usize = 40;
    const FIRST_WIDE_BYTE: usize = 32;
    const WINDOW_BITS: usize = 8;
    const HALF_RADIX: usize = 1 << (WINDOW_BITS - 1);

    // A high-half value in the first byte above 256 bits produces a negative
    // digit whose carry must be consumed by the following window.
    let mut bytes = [0; WIDE_REPR_BYTES];
    let short_bytes = [0; FIRST_WIDE_BYTE];
    assert_eq!(
        booth_window_count(&[short_bytes.as_slice(), bytes.as_slice()], WINDOW_BITS),
        WIDE_REPR_BYTES + 1
    );

    bytes[FIRST_WIDE_BYTE] = HALF_RADIX as u8;
    assert_eq!(
        booth_digit(&bytes, WINDOW_BITS, FIRST_WIDE_BYTE),
        BoothDigit {
            magnitude: HALF_RADIX,
            negative: true,
        }
    );
    assert_eq!(
        booth_digit(&bytes, WINDOW_BITS, FIRST_WIDE_BYTE + 1),
        BoothDigit {
            magnitude: 1,
            negative: false,
        }
    );

    // The final data window likewise has a following carry-only window.
    bytes = [0; WIDE_REPR_BYTES];
    bytes[WIDE_REPR_BYTES - 1] = HALF_RADIX as u8;
    assert_eq!(
        booth_digit(&bytes, WINDOW_BITS, WIDE_REPR_BYTES - 1),
        BoothDigit {
            magnitude: HALF_RADIX,
            negative: true,
        }
    );
    assert_eq!(
        booth_digit(&bytes, WINDOW_BITS, WIDE_REPR_BYTES),
        BoothDigit {
            magnitude: 1,
            negative: false,
        }
    );

    let first_high_bit = {
        let mut bytes = [0; WIDE_REPR_BYTES];
        bytes[FIRST_WIDE_BYTE + 1] = 1;
        bytes
    };
    let all_ones = [u8::MAX; WIDE_REPR_BYTES];
    // Folding into `Fp` avoids a big-integer test dependency while exercising
    // the complete 320-bit recoding schedule.
    for bytes in [&bytes, &first_high_bit, &all_ones] {
        for window_bits in [1, 3, 5, 8, 9] {
            assert_eq!(
                evaluate_booth_digits(bytes, window_bits),
                evaluate_le_bytes(bytes)
            );
        }
    }
}

#[test]
fn test_multiexp_booth_boundaries() {
    let mut rng = UnwrapErr(SysRng);
    let mut fp_high_bit = Fp::ONE;
    for _ in 0..254 {
        fp_high_bit = fp_high_bit.double();
    }
    let fp_scalars = [
        Fp::ZERO,
        Fp::ONE,
        -Fp::ONE,
        Fp::from(3),
        Fp::from(4),
        Fp::from(7),
        Fp::from(8),
        fp_high_bit,
    ];
    let fp_coeffs = fp_scalars.into_iter().cycle().take(32).collect::<Vec<_>>();
    let fp_bases = (0..fp_coeffs.len())
        .map(|_| EqAffine::from(Eq::random(&mut rng)))
        .collect::<Vec<_>>();
    assert_multiexp_matches_naive(&fp_coeffs, &fp_bases);

    let mut fq_high_bit = Fq::ONE;
    for _ in 0..254 {
        fq_high_bit = fq_high_bit.double();
    }
    let fq_scalars = [
        Fq::ZERO,
        Fq::ONE,
        -Fq::ONE,
        Fq::from(3),
        Fq::from(4),
        Fq::from(7),
        Fq::from(8),
        fq_high_bit,
    ];
    let fq_coeffs = fq_scalars.into_iter().cycle().take(32).collect::<Vec<_>>();
    let fq_bases = (0..fq_coeffs.len())
        .map(|_| EpAffine::from(Ep::random(&mut rng)))
        .collect::<Vec<_>>();
    assert_multiexp_matches_naive(&fq_coeffs, &fq_bases);
}

#[test]
fn test_multiexp_algorithm_selection() {
    assert!(!should_parallelize_multiexp(usize::MAX, 1));
    assert!(!should_parallelize_multiexp(2, 2));
    assert!(should_parallelize_multiexp(3, 2));
}

#[test]
fn test_lagrange_interpolate() {
    let mut rng = UnwrapErr(SysRng);

    let points = (0..5).map(|_| Fp::random(&mut rng)).collect::<Vec<_>>();
    let evals = (0..5).map(|_| Fp::random(&mut rng)).collect::<Vec<_>>();

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

#[cfg(test)]
fn random_vector(n: usize, zero_every: usize) -> Vec<Fp> {
    let mut rng = UnwrapErr(SysRng);
    (0..n)
        .map(|i| {
            if zero_every > 0 && i % zero_every == 0 {
                Fp::ZERO
            } else {
                Fp::random(&mut rng)
            }
        })
        .collect()
}

#[test]
fn test_parallel_batch_invert_matches_sequential() {
    for n in [
        0,
        1,
        7,
        SCAN_PARALLEL_THRESHOLD - 1,
        3 * SCAN_PARALLEL_THRESHOLD + 5,
    ] {
        let original = random_vector(n, 97);
        let mut expected = original.clone();
        expected.iter_mut().batch_invert_vartime();
        let mut actual = original.clone();
        parallel_batch_invert(&mut actual);
        assert_eq!(actual, expected, "n = {n}");
        for (x, inv) in original.iter().zip(&actual) {
            assert!(bool::from(x.is_zero()) && bool::from(inv.is_zero()) || *x * inv == Fp::ONE);
        }
    }
}

#[test]
fn test_grand_product_matches_sequential() {
    for n in [
        1,
        2,
        9,
        SCAN_PARALLEL_THRESHOLD + 1,
        4 * SCAN_PARALLEL_THRESHOLD + 3,
    ] {
        let values = random_vector(n + 5, 0);
        let expected: Vec<Fp> = std::iter::once(Fp::ONE)
            .chain(values.iter().scan(Fp::ONE, |acc, v| {
                *acc *= v;
                Some(*acc)
            }))
            .take(n)
            .collect();
        assert_eq!(grand_product(&values, n), expected, "n = {n}");
        assert_eq!(grand_product(&values[..n - 1], n), expected, "n = {n}");
    }
}

#[test]
fn test_chained_grand_products_match_sequential_chain() {
    for len in [1, 5, 2 * SCAN_PARALLEL_THRESHOLD + 1] {
        check_chained_grand_products(len);
    }
}

#[cfg(test)]
fn check_chained_grand_products(len: usize) {
    let sets: Vec<Vec<Fp>> = (0..4).map(|_| random_vector(len, 0)).collect();
    let mut carried = Fp::ONE;
    let expected: Vec<Vec<Fp>> = sets
        .iter()
        .map(|factors| {
            let mut z = vec![carried];
            for factor in &factors[..len - 1] {
                z.push(z[z.len() - 1] * factor);
            }
            carried = z[len - 1];
            z
        })
        .collect();
    assert_eq!(chained_grand_products(sets, len), expected);
}

#[test]
fn test_eval_polynomial_matches_horner() {
    let point = Fp::random(&mut UnwrapErr(SysRng));
    for n in [
        0,
        1,
        3,
        EVAL_PARALLEL_THRESHOLD,
        3 * EVAL_PARALLEL_THRESHOLD + 11,
    ] {
        let poly = random_vector(n, 0);
        let expected = poly
            .iter()
            .rev()
            .fold(Fp::ZERO, |acc, coeff| acc * point + coeff);
        assert_eq!(eval_polynomial(&poly, point), expected, "n = {n}");
    }
}

#[test]
fn test_kate_division_parallel_matches_sequential() {
    let b = Fp::random(&mut UnwrapErr(SysRng));
    for n in [
        0,
        1,
        2,
        9,
        SCAN_PARALLEL_THRESHOLD + 1,
        3 * SCAN_PARALLEL_THRESHOLD + 7,
    ] {
        let a = random_vector(n, 0);
        let expected = if n == 0 { vec![] } else { kate_division(&a, b) };
        assert_eq!(kate_division_parallel(&a, b), expected, "n = {n}");
    }
    // A polynomial with a root at `b` divides exactly.
    let mut a = random_vector(2 * SCAN_PARALLEL_THRESHOLD, 0);
    let remainder = eval_polynomial(&a, b);
    a[0] -= remainder;
    let q = kate_division_parallel(&a, b);
    let x = Fp::random(&mut UnwrapErr(SysRng));
    assert_eq!(eval_polynomial(&q, x) * (x - b), eval_polynomial(&a, x));
}

#[test]
fn test_evaluate_batch_matches_individual_evaluations() {
    let polys: Vec<Vec<Fp>> = [0, 1, 17, 4 * EVAL_PARALLEL_THRESHOLD]
        .iter()
        .map(|&n| random_vector(n, 0))
        .collect();
    let mut rng = UnwrapErr(SysRng);
    let queries: Vec<(&[Fp], Fp)> = polys
        .iter()
        .map(|p| (&p[..], Fp::random(&mut rng)))
        .collect();
    let expected: Vec<Fp> = queries
        .iter()
        .map(|(p, x)| p.iter().rev().fold(Fp::ZERO, |acc, c| acc * x + c))
        .collect();
    assert_eq!(evaluate_batch(&queries), expected);
}

#[test]
fn test_fft_twiddles_match_running_product() {
    let omega = Fp::random(&mut UnwrapErr(SysRng));
    // Odd length above the threshold, so the parallel chunks are uneven.
    let len = TWIDDLES_PARALLEL_THRESHOLD * 2 + 3;
    let expected: Vec<Fp> = (0..len)
        .scan(Fp::ONE, |w, _| {
            let tw = *w;
            *w *= omega;
            Some(tw)
        })
        .collect();
    assert_eq!(fft_twiddles(omega, len), expected);
    assert_eq!(fft_twiddles(omega, 5), expected[..5]);
    assert!(fft_twiddles(omega, 0).is_empty());
}

#[test]
fn test_bitreverse_permutation_is_the_bit_reversal() {
    for log_n in [0, 1, 3, 16] {
        let n = 1usize << log_n;
        let mut a: Vec<usize> = (0..n).collect();
        bitreverse_permutation(&mut a, log_n);
        for (i, &v) in a.iter().enumerate() {
            let expected = (0..log_n).fold(0, |r, b| (r << 1) | ((i >> b) & 1));
            assert_eq!(v, expected, "log_n = {log_n}, i = {i}");
        }
    }
}

#[test]
fn test_fft_matches_naive_dft() {
    let mut rng = UnwrapErr(SysRng);
    for log_n in [0u32, 1, 4, 14] {
        let n = 1usize << log_n;
        let omega = Fp::ROOT_OF_UNITY.pow_vartime([1u64 << (Fp::S - log_n)]);
        let coeffs: Vec<Fp> = (0..n).map(|_| Fp::random(&mut rng)).collect();
        let mut evals = coeffs.clone();
        best_fft(&mut evals, omega, log_n);
        // Spot-check a few evaluation points; a full naive DFT at 2^14 is
        // quadratic.
        for i in [0, 1, n / 2, n - 1].into_iter().filter(|&i| i < n) {
            assert_eq!(
                evals[i],
                eval_polynomial(&coeffs, omega.pow_vartime([i as u64]))
            );
        }
    }
}
