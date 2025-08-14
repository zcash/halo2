#[macro_use]
extern crate criterion;

use crate::arithmetic::{compute_inner_product, eval_polynomial};
use crate::pasta::Fp;
use group::ff::Field;
use halo2_proofs::*;

use criterion::{BenchmarkId, Criterion};
use rand_core::OsRng;

fn criterion_benchmark(c: &mut Criterion) {
    let mut eval_polynomial_group = c.benchmark_group("poly-eval");
    for k in 3..19 {
        eval_polynomial_group.bench_function(BenchmarkId::new("k", k), |b| {
            let poly = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let point = Fp::random(OsRng);
            b.iter(|| eval_polynomial(&poly, point));
        });
    }
    eval_polynomial_group.finish();

    let mut compute_inner_product_group = c.benchmark_group("poly-inner-product");
    for k in 3..19 {
        compute_inner_product_group.bench_function(BenchmarkId::new("k", k), |b| {
            let a = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            let other = (0..(1 << k)).map(|_| Fp::random(OsRng)).collect::<Vec<_>>();
            b.iter(|| compute_inner_product(&a, &other));
        });
    }
    compute_inner_product_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
