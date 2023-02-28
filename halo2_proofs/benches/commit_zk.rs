extern crate criterion;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use group::ff::Field;
use halo2_proofs::*;
use halo2curves::pasta::pallas::Scalar;
use rand_chacha::rand_core::RngCore;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use rayon::{current_num_threads, prelude::*};

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
    let n_threads = current_num_threads();
    let n = 1usize << domain;
    let n_chunks = n_threads + if n % n_threads != 0 { 1 } else { 0 };
    let mut rand_vec = vec![Scalar::zero(); n];

    let mut thread_seeds: Vec<ChaCha20Rng> = (0..n_chunks)
        .into_iter()
        .map(|_| {
            let mut seed = [0u8; 32];
            rng.fill_bytes(&mut seed);
            ChaCha20Rng::from_seed(seed)
        })
        .collect();

    thread_seeds
        .par_iter_mut()
        .zip_eq(rand_vec.par_chunks_mut(n / n_threads))
        .for_each(|(mut rng, chunk)| chunk.iter_mut().for_each(|v| *v = Scalar::random(&mut rng)));

    rand_vec
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
