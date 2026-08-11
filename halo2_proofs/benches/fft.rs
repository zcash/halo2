#[macro_use]
extern crate criterion;

use crate::arithmetic::best_fft;
use crate::pasta::Fp;
use group::ff::Field;
use halo2_proofs::*;

use criterion::{BatchSize, BenchmarkId, Criterion};
use rand_core::OsRng;

const ORCHARD_K: u32 = 11;
const ORCHARD_EXTENDED_K: u32 = 14;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft");
    for k in 3..19 {
        group.bench_function(BenchmarkId::new("k", k), |b| {
            let mut a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let omega = Fp::random(OsRng); // would be weird if this mattered
            b.iter(|| {
                best_fft(&mut a, omega, k as u32);
            });
        });
    }

    let extension = 1 << (ORCHARD_EXTENDED_K - ORCHARD_K);
    let domain = poly::EvaluationDomain::<Fp>::new(extension + 1, ORCHARD_K);
    assert_eq!(domain.extended_len(), 1 << ORCHARD_EXTENDED_K);
    let coefficients = (0..(1 << ORCHARD_K))
        .map(|_| Fp::random(OsRng))
        .collect::<Vec<_>>();

    group.bench_function(
        BenchmarkId::new("coeff_to_extended", "orchard-k11-to-k14"),
        |b| {
            b.iter_batched(
                || domain.coeff_from_vec(coefficients.clone()),
                |coefficients| domain.coeff_to_extended(coefficients),
                BatchSize::LargeInput,
            );
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
