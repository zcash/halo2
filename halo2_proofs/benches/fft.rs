#[macro_use]
extern crate criterion;

use crate::arithmetic::{best_fft, best_ifft};
use crate::pasta::Fp;
use group::ff::Field;
use halo2_proofs::*;

use criterion::{BenchmarkId, Criterion};
use rand_core::OsRng;

fn criterion_benchmark(c: &mut Criterion) {
    let mut fft_group = c.benchmark_group("fft");
    for k in 3..19 {
        fft_group.bench_function(BenchmarkId::new("k", k), |b| {
            let mut a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let omega = Fp::random(OsRng); // would be weird if this mattered
            b.iter(|| best_fft(&mut a, omega, k as u32));
        });
    }
    fft_group.finish();

    let mut ifft_group = c.benchmark_group("ifft");
    for k in 3..19 {
        ifft_group.bench_function(BenchmarkId::new("k", k), |b| {
            let mut a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let omega_inv = Fp::random(OsRng); // would be weird if this mattered
            let divisor = Fp::random(OsRng); // would be weird if this mattered
            b.iter(|| best_ifft(&mut a, omega_inv, k, divisor));
        });
    }
    ifft_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
