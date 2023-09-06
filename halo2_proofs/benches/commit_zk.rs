extern crate criterion;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use group::ff::Field;
use halo2_proofs::arithmetic::parallelize;
use halo2curves::pasta::pallas::Scalar;
use rand_chacha::rand_core::RngCore;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::{collections::HashMap, iter};

#[cfg(feature = "multicore")]
use maybe_rayon::current_num_threads;

#[cfg(not(feature = "multicore"))]
fn current_num_threads() -> usize {
    1
}

fn rand_poly_serial(mut rng: ChaCha20Rng, domain: usize) -> Vec<Scalar> {
    // Sample a random polynomial of degree n - 1
    let mut random_poly = vec![Scalar::zero(); 1 << domain];
    for coeff in random_poly.iter_mut() {
        *coeff = Scalar::random(&mut rng);
    }

    random_poly
}

fn rand_poly_par(mut rng: ChaCha20Rng, domain: usize) -> Vec<Scalar> {
    // Sample a random polynomial of degree n - 1
    let n = 1usize << domain as usize;
    let mut random_poly = vec![Scalar::ZERO; n];

    let num_threads = current_num_threads();
    let chunk_size = n / num_threads;
    let thread_seeds = (0..)
        .step_by(chunk_size + 1)
        .take(n % num_threads)
        .chain(
            (chunk_size != 0)
                .then(|| ((n % num_threads) * (chunk_size + 1)..).step_by(chunk_size))
                .into_iter()
                .flatten(),
        )
        .take(num_threads)
        .zip(iter::repeat_with(|| {
            let mut seed = [0u8; 32];
            rng.fill_bytes(&mut seed);
            ChaCha20Rng::from_seed(seed)
        }))
        .collect::<HashMap<_, _>>();

    parallelize(&mut random_poly, |chunk, offset| {
        let mut rng = thread_seeds[&offset].clone();
        chunk.iter_mut().for_each(|v| *v = Scalar::random(&mut rng));
    });
    random_poly
}

fn bench_commit(c: &mut Criterion) {
    let mut group = c.benchmark_group("Blinder_poly");
    let rand = ChaCha20Rng::from_seed([1u8; 32]);
    for i in [
        18usize, 19usize, 20usize, 21usize, 22usize, 23usize, 24usize, 25usize,
    ]
    .iter()
    {
        group.bench_with_input(BenchmarkId::new("serial", i), i, |b, i| {
            b.iter(|| rand_poly_serial(rand.clone(), *i))
        });
        group.bench_with_input(BenchmarkId::new("parallel", i), i, |b, i| {
            b.iter(|| rand_poly_par(rand.clone(), *i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_commit);
criterion_main!(benches);
