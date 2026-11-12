//! End-to-end prover benchmark: whole proofs of a circuit that exercises
//! every prover component (a custom gate, a lookup, an instance column and a
//! permutation argument split into several sets), at Orchard's size `k = 11`
//! and larger, with one or several circuits per proof.
//!
//! To compare two revisions, run on the first with
//! `cargo bench --bench prover -- --save-baseline before`, then on the second
//! with `cargo bench --bench prover -- --baseline before`.

#[macro_use]
extern crate criterion;

#[path = "../tests/support/prover_circuit.rs"]
mod prover_circuit;

use criterion::{BenchmarkId, Criterion};
use prover_circuit::{circuits, prove, setup};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

/// `(k, circuits per proof)` pairs to benchmark.
const CASES: [(u32, usize); 5] = [(11, 1), (11, 4), (14, 1), (14, 4), (16, 1)];

fn prover(c: &mut Criterion) {
    let mut group = c.benchmark_group("prover");
    group.sample_size(10);
    for (k, count) in CASES {
        let (params, pk) = setup(k);
        let circuits = circuits(k, count);
        group.bench_function(
            BenchmarkId::new(format!("k{k}"), format!("{count}_circuits")),
            |b| b.iter(|| prove(&params, &pk, &circuits, UnwrapErr(SysRng))),
        );
    }
    group.finish();
}

criterion_group!(benches, prover);
criterion_main!(benches);
