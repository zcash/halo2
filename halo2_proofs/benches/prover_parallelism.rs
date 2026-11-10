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

#[macro_use]
extern crate criterion;

use criterion::{BatchSize, BenchmarkId, Criterion};
use group::ff::{Field, PrimeField};
use halo2_proofs::arithmetic::{best_fft, VartimeBatchInvert};
use halo2_proofs::bench_internals as internals;
use halo2_proofs::pasta::Fp;
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

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

criterion_group!(benches, fft_setup, scans);
criterion_main!(benches);
