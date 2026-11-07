#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use group::{ff::Field, Group};
use halo2_proofs::{
    arithmetic::{best_multiexp, CurveAffine, CurveExt},
    pasta,
    poly::commitment::Params,
};
use pasta::{EpAffine, EqAffine, Fp};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
use std::hint::black_box;

fn compare_msm<C: CurveAffine>(c: &mut Criterion, curve: &str) {
    let mut rng = UnwrapErr(SysRng);
    let coeffs = (0..2048)
        .map(|_| C::Scalar::random(&mut rng))
        .collect::<Vec<_>>();
    let bases = (0..coeffs.len())
        .map(|_| C::Curve::random(&mut rng).to_affine_vartime())
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group(format!("msm/{curve}"));
    group.sample_size(30);
    for len in [1, 2, 8, 16, 32, 64, 2048] {
        let coeffs = &coeffs[..len];
        let bases = &bases[..len];
        group.bench_function(BenchmarkId::new("generic", len), |b| {
            b.iter(|| best_multiexp(black_box(coeffs), black_box(bases)))
        });
        group.bench_function(BenchmarkId::new("hooked", len), |b| {
            // Same dispatch as the crate-private verifier entry point.
            b.iter(|| {
                C::CurveExt::try_multiexp_vartime(black_box(coeffs), black_box(bases))
                    .unwrap_or_else(|| best_multiexp(coeffs, bases))
            })
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    compare_msm::<EpAffine>(c, "pallas");
    compare_msm::<EqAffine>(c, "vesta");

    let mut group = c.benchmark_group("msm");
    for k in 8..16 {
        group
            .bench_function(BenchmarkId::new("k", k), |b| {
                let coeffs = (0..(1 << k))
                    .map(|_| Fp::random(&mut UnwrapErr(SysRng)))
                    .collect::<Vec<_>>();
                let bases = Params::<EqAffine>::new(k).get_g();

                b.iter(|| best_multiexp(&coeffs, &bases))
            })
            .sample_size(30);
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
