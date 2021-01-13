//! Benchmarks for hashing to the Pasta curves.

use criterion::{criterion_group, criterion_main, Criterion};

use halo2::arithmetic::{HashToCurve, Shake128};
use halo2::pasta::{pallas, vesta};

fn criterion_benchmark(c: &mut Criterion) {
    bench_hash_to_curve(c);
    bench_encode_to_curve(c);
    bench_map_to_curve(c);
}

fn bench_hash_to_curve(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash-to-curve");

    let hash_pallas = pallas::MAP.hash_to_curve("z.cash:test", Shake128::default());
    group.bench_function("Pallas", |b| b.iter(|| hash_pallas(b"benchmark")));

    let hash_vesta = vesta::MAP.hash_to_curve("z.cash:test", Shake128::default());
    group.bench_function("Vesta", |b| b.iter(|| hash_vesta(b"benchmark")));
}

fn bench_encode_to_curve(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode-to-curve");

    let encode_pallas = pallas::MAP.encode_to_curve("z.cash:test", Shake128::default());
    group.bench_function("Pallas", |b| b.iter(|| encode_pallas(b"benchmark")));

    let encode_vesta = vesta::MAP.encode_to_curve("z.cash:test", Shake128::default());
    group.bench_function("Vesta", |b| b.iter(|| encode_vesta(b"benchmark")));
}

fn bench_map_to_curve(c: &mut Criterion) {
    let mut group = c.benchmark_group("map-to-curve");

    let pallas_input = &pallas::Base::one();
    group.bench_function("Pallas", |b| {
        b.iter(|| pallas::MAP.map_to_curve(pallas_input))
    });

    let vesta_input = &vesta::Base::one();
    group.bench_function("Vesta", |b| b.iter(|| vesta::MAP.map_to_curve(vesta_input)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
