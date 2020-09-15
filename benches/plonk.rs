#[macro_use]
extern crate criterion;

extern crate halo2;
use crate::arithmetic::{small_multiexp, EqAffine, Field, Fp, Fq};
use crate::poly::commitment::Params;
use crate::transcript::DummyHash;
use halo2::*;

use criterion::{black_box, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    // small multiexp
    {
        let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(5);
        let g = &mut params.get_g();
        let len = g.len() / 2;
        let (g_lo, g_hi) = g.split_at_mut(len);

        let coeff_1 = Fp::random();
        let coeff_2 = Fp::random();

        c.bench_function("double-and-add", |b| {
            b.iter(|| {
                for (g_lo, g_hi) in g_lo.iter().zip(g_hi.iter()) {
                    small_multiexp(&[black_box(coeff_1), black_box(coeff_2)], &[*g_lo, *g_hi]);
                }
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
