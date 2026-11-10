//! Benchmarks for the prover's parallel kernels.
//!
//! Each group measures one kernel against the sequential code it replaced,
//! which is kept inline here as the baseline. Run with
//!
//! ```text
//! cargo bench -p halo2_proofs --features unstable-bench-internals \
//!     --bench prover_parallelism
//! ```
//!
//! The sizes are extended-domain sizes for circuits of `k = 11` (Orchard),
//! `k = 14` and `k = 17`.

// The baseline copies of earlier prover code keep its operator style.
#![allow(clippy::op_ref)]

#[macro_use]
extern crate criterion;

use criterion::{black_box, BatchSize, BenchmarkId, Criterion};
use group::ff::{Field, PrimeField};
use group::{Curve, CurveAffine as _};
use halo2_proofs::arithmetic::{
    best_fft, best_multiexp, compute_inner_product, eval_polynomial, parallelize,
    VartimeBatchInvert,
};
use halo2_proofs::bench_internals as internals;
use halo2_proofs::pasta::{Eq, EqAffine, Fp};
use halo2_proofs::poly::commitment::{create_proof, Blind, Params};
use halo2_proofs::poly::{Coeff, EvaluationDomain, Polynomial};
use halo2_proofs::transcript::{Blake2bWrite, Challenge255, Transcript, TranscriptWrite};
use rand::rngs::SysRng;
use rand_core::{Rng, UnwrapErr};
use std::collections::BTreeMap;

/// Extended-domain sizes `log2(n)` used by the FFT benchmarks.
const EXTENDED_LOG_SIZES: [u32; 5] = [13, 14, 15, 16, 19];

fn random_vector(n: usize) -> Vec<Fp> {
    let mut rng = UnwrapErr(SysRng);
    (0..n).map(|_| Fp::random(&mut rng)).collect()
}

fn root_of_unity(log_n: u32) -> Fp {
    Fp::ROOT_OF_UNITY.pow_vartime([1u64 << (Fp::S - log_n)])
}

/// The FFT's setup steps: before, every transform built its twiddle table
/// with a sequential running product and bit-reversed its input with
/// sequential swaps.
fn fft_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_setup");
    group.sample_size(20);
    for log_n in EXTENDED_LOG_SIZES {
        let n = 1usize << log_n;
        let omega = root_of_unity(log_n);

        group.bench_function(BenchmarkId::new("twiddles/sequential", log_n), |b| {
            b.iter(|| {
                (0..n / 2)
                    .scan(Fp::ONE, |w, _| {
                        let tw = *w;
                        *w *= omega;
                        Some(tw)
                    })
                    .collect::<Vec<_>>()
            })
        });
        group.bench_function(BenchmarkId::new("twiddles/parallel", log_n), |b| {
            b.iter(|| internals::fft_twiddles(omega, n / 2))
        });

        let input = random_vector(n);
        group.bench_function(BenchmarkId::new("bitreverse/sequential", log_n), |b| {
            let mut a = input.clone();
            b.iter(|| {
                for i in 0..n {
                    let r = i.reverse_bits() >> (usize::BITS - log_n);
                    if i < r {
                        a.swap(i, r);
                    }
                }
            })
        });
        group.bench_function(BenchmarkId::new("bitreverse/parallel", log_n), |b| {
            let mut a = input.clone();
            b.iter(|| internals::bitreverse_permutation(&mut a, log_n))
        });

        // The whole transform, building the table on each call (as every
        // FFT did before) or reading the table an `EvaluationDomain` caches.
        group.bench_function(BenchmarkId::new("fft/table_per_call", log_n), |b| {
            let mut a = input.clone();
            b.iter(|| best_fft(&mut a, omega, log_n))
        });
        let twiddles = internals::fft_twiddles(omega, n / 2);
        group.bench_function(BenchmarkId::new("fft/cached_table", log_n), |b| {
            let mut a = input.clone();
            b.iter(|| internals::fft_with_twiddles(&mut a, &twiddles, log_n))
        });
    }
    group.finish();
}

/// Circuit sizes `log2(n)` used by the benchmarks of per-row vectors.
const ROW_LOG_SIZES: [u32; 3] = [11, 14, 17];

/// Number of chained sets in the permutation benchmark: Orchard's action
/// circuit has 12 permutation sets, so 4 is a conservative middle ground.
const PERMUTATION_SETS: usize = 4;

/// Montgomery batch inversion and grand products, the sequential scans of
/// the lookup and permutation arguments.
fn scans(c: &mut Criterion) {
    let mut group = c.benchmark_group("scans");
    group.sample_size(20);
    for log_n in ROW_LOG_SIZES {
        let n = 1usize << log_n;
        let input = random_vector(n);

        group.bench_function(BenchmarkId::new("batch_invert/sequential", log_n), |b| {
            b.iter_batched_ref(
                || input.clone(),
                |v| v.iter_mut().batch_invert_vartime(),
                BatchSize::LargeInput,
            )
        });
        group.bench_function(BenchmarkId::new("batch_invert/parallel", log_n), |b| {
            b.iter_batched_ref(
                || input.clone(),
                |v| internals::parallel_batch_invert(v),
                BatchSize::LargeInput,
            )
        });

        group.bench_function(BenchmarkId::new("grand_product/sequential", log_n), |b| {
            b.iter(|| {
                std::iter::once(Fp::ONE)
                    .chain(input.iter().copied())
                    .scan(Fp::ONE, |acc, v| {
                        *acc *= v;
                        Some(*acc)
                    })
                    .take(n)
                    .collect::<Vec<_>>()
            })
        });
        group.bench_function(BenchmarkId::new("grand_product/parallel", log_n), |b| {
            b.iter(|| internals::grand_product(&input, n))
        });

        // The permutation argument's sets, each starting from the last value
        // of the previous one: before, a single running product through all
        // of them.
        let sets: Vec<Vec<Fp>> = (0..PERMUTATION_SETS).map(|_| random_vector(n)).collect();
        group.bench_function(BenchmarkId::new("chained_sets/sequential", log_n), |b| {
            b.iter(|| {
                let mut carried = Fp::ONE;
                sets.iter()
                    .map(|factors| {
                        let mut z = Vec::with_capacity(n);
                        z.push(carried);
                        for factor in &factors[..n - 1] {
                            z.push(z[z.len() - 1] * factor);
                        }
                        carried = z[n - 1];
                        z
                    })
                    .collect::<Vec<_>>()
            })
        });
        group.bench_function(BenchmarkId::new("chained_sets/parallel", log_n), |b| {
            b.iter_batched(
                || sets.clone(),
                |sets| internals::chained_grand_products(sets, n),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

/// Size of the lookup table: a 10-bit range check, as in Orchard.
const LOOKUP_TABLE_SIZE: u64 = 1 << 10;

/// The permutation step of the lookup argument, `(A, S) -> (A', S')`: before,
/// a stable sort comparing field elements with `Ord` and a `BTreeMap` of the
/// table values.
fn lookup_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_sort");
    group.sample_size(20);
    for log_n in ROW_LOG_SIZES {
        let n = 1usize << log_n;
        let mut rng = UnwrapErr(SysRng);
        let input: Vec<Fp> = (0..n)
            .map(|_| Fp::from(rng.next_u64() % LOOKUP_TABLE_SIZE))
            .collect();
        let table: Vec<Fp> = (0..n as u64)
            .map(|i| Fp::from(i.min(LOOKUP_TABLE_SIZE - 1)))
            .collect();

        group.bench_function(BenchmarkId::new("ord_btreemap", log_n), |b| {
            b.iter(|| {
                let mut permuted_input = input.clone();
                permuted_input.sort();
                let mut leftover = BTreeMap::<Fp, u32>::new();
                for value in &table {
                    *leftover.entry(*value).or_insert(0) += 1;
                }
                let mut permuted_table = vec![Fp::ZERO; n];
                let mut repeated = vec![];
                for row in 0..n {
                    let value = permuted_input[row];
                    if row == 0 || value != permuted_input[row - 1] {
                        permuted_table[row] = value;
                        *leftover.get_mut(&value).unwrap() -= 1;
                    } else {
                        repeated.push(row);
                    }
                }
                for (value, count) in leftover {
                    for _ in 0..count {
                        permuted_table[repeated.pop().unwrap()] = value;
                    }
                }
                (permuted_input, permuted_table)
            })
        });
        group.bench_function(BenchmarkId::new("keyed_sort_merge", log_n), |b| {
            b.iter(|| internals::permute_lookup_values(&input, &table).unwrap())
        });
    }
    group.finish();
}

/// Horner's rule on one thread, as `eval_polynomial` was.
fn horner(poly: &[Fp], point: Fp) -> Fp {
    poly.iter()
        .rev()
        .fold(Fp::ZERO, |acc, coeff| acc * point + coeff)
}

/// The inner-product argument as it was: a materialised vector `b` of powers
/// of `x_3`, folded each round, sequential evaluations and folds, and the L
/// and R multiexps one after the other.
fn sequential_ipa(
    params: &Params<EqAffine>,
    k: u32,
    mut rng: impl Rng,
    transcript: &mut Blake2bWrite<Vec<u8>, EqAffine, Challenge255<EqAffine>>,
    p_poly: &Polynomial<Fp, Coeff>,
    p_blind: Blind<Fp>,
    x_3: Fp,
) {
    let (u, w) = internals::params_u_w(params);
    let mut s_poly = p_poly.clone();
    for coeff in s_poly.iter_mut() {
        *coeff = Fp::random(&mut rng);
    }
    let s_at_x3 = horner(&s_poly, x_3);
    s_poly[0] -= &s_at_x3;
    let s_poly_blind = Blind(Fp::random(&mut rng));
    let s_poly_commitment = params.commit(&s_poly, s_poly_blind).to_affine();
    transcript.write_point(s_poly_commitment).unwrap();
    let xi = *transcript.squeeze_challenge_scalar::<()>();
    let z = *transcript.squeeze_challenge_scalar::<()>();

    let mut p_prime_poly = s_poly * xi + p_poly;
    let v = horner(&p_prime_poly, x_3);
    p_prime_poly[0] -= &v;
    let p_prime_blind = s_poly_blind * Blind(xi) + p_blind;
    let mut f = p_prime_blind.0;
    let mut p_prime = p_prime_poly.to_vec();
    let mut b = Vec::with_capacity(1 << k);
    let mut cur = Fp::ONE;
    for _ in 0..(1 << k) {
        b.push(cur);
        cur *= &x_3;
    }
    let mut g_prime = params.get_g();

    for j in 0..k {
        let half = 1 << (k - j - 1);
        let l_j = best_multiexp(&p_prime[half..], &g_prime[0..half]);
        let r_j = best_multiexp(&p_prime[0..half], &g_prime[half..]);
        let value_l_j = compute_inner_product(&p_prime[half..], &b[0..half]);
        let value_r_j = compute_inner_product(&p_prime[0..half], &b[half..]);
        let l_j_randomness = Fp::random(&mut rng);
        let r_j_randomness = Fp::random(&mut rng);
        let l_j = l_j + &best_multiexp(&[value_l_j * &z, l_j_randomness], &[u, w]);
        let r_j = r_j + &best_multiexp(&[value_r_j * &z, r_j_randomness], &[u, w]);
        transcript.write_point(l_j.to_affine()).unwrap();
        transcript.write_point(r_j.to_affine()).unwrap();

        let u_j = *transcript.squeeze_challenge_scalar::<()>();
        let u_j_inv = u_j.invert().unwrap();
        for i in 0..half {
            p_prime[i] = p_prime[i] + &(p_prime[i + half] * &u_j_inv);
            b[i] = b[i] + &(b[i + half] * &u_j);
        }
        p_prime.truncate(half);
        b.truncate(half);

        let (g_lo, g_hi) = g_prime.split_at_mut(half);
        parallelize(g_lo, |g_lo, start| {
            let g_hi = &g_hi[start..];
            let tmp: Vec<Eq> = g_lo
                .iter()
                .zip(g_hi.iter())
                .map(|(g_lo, g_hi)| g_lo.to_curve() + &(*g_hi * u_j))
                .collect();
            Eq::batch_normalize(&tmp, g_lo);
        });
        g_prime.truncate(half);

        f += &(l_j_randomness * &u_j_inv);
        f += &(r_j_randomness * &u_j);
    }
    transcript.write_scalar(p_prime[0]).unwrap();
    transcript.write_scalar(f).unwrap();
}

/// The inner-product argument that opens the final polynomial commitment,
/// and the Horner evaluation it and every other opening relies on.
fn ipa(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipa");
    group.sample_size(10);
    for log_n in ROW_LOG_SIZES {
        let poly = random_vector(1 << log_n);
        let point = Fp::random(&mut UnwrapErr(SysRng));
        group.bench_function(BenchmarkId::new("eval_polynomial/sequential", log_n), |b| {
            b.iter(|| horner(black_box(&poly), black_box(point)))
        });
        group.bench_function(BenchmarkId::new("eval_polynomial/parallel", log_n), |b| {
            b.iter(|| eval_polynomial(black_box(&poly), black_box(point)))
        });
    }
    for k in [11, 14] {
        let params: Params<EqAffine> = Params::new(k);
        let domain = EvaluationDomain::<Fp>::new(1, k);
        let p_poly = domain.coeff_from_vec(random_vector(1 << k));
        let p_blind = Blind(Fp::random(&mut UnwrapErr(SysRng)));
        let x_3 = Fp::random(&mut UnwrapErr(SysRng));
        group.bench_function(BenchmarkId::new("create_proof/sequential", k), |b| {
            b.iter(|| {
                let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
                sequential_ipa(
                    &params,
                    k,
                    UnwrapErr(SysRng),
                    &mut transcript,
                    &p_poly,
                    p_blind,
                    x_3,
                );
                transcript.finalize()
            })
        });
        group.bench_function(BenchmarkId::new("create_proof/parallel", k), |b| {
            b.iter(|| {
                let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
                create_proof(
                    &params,
                    UnwrapErr(SysRng),
                    &mut transcript,
                    &p_poly,
                    p_blind,
                    x_3,
                )
                .unwrap();
                transcript.finalize()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, fft_setup, scans, lookup_sort, ipa);
criterion_main!(benches);
