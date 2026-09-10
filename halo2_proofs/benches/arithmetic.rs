#[macro_use]
extern crate criterion;

use crate::arithmetic::small_multiexp;
use crate::pasta::{EqAffine, Fp};
use crate::poly::commitment::Params;
use group::ff::Field;
use halo2_proofs::*;

use criterion::{black_box, Criterion};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = UnwrapErr(SysRng);

    // small multiexp
    {
        let params: Params<EqAffine> = Params::new(5);
        let g = &mut params.get_g();
        let len = g.len() / 2;
        let (g_lo, g_hi) = g.split_at_mut(len);

        let coeff_1 = Fp::random(&mut rng);
        let coeff_2 = Fp::random(&mut rng);

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
