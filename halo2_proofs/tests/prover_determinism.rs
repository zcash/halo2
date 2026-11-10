//! Pins the exact bytes the prover writes for a fixed circuit and a fixed RNG.
//!
//! The prover is a deterministic function of its inputs and the RNG stream, so
//! any change to how its work is scheduled (sequential or parallel, in which
//! order, on how many threads) must leave the proof unchanged. A change that
//! alters these bytes either changed the protocol, changed the order in which
//! randomness is drawn, or introduced nondeterminism; all three need a
//! deliberate decision rather than an accidental one.
//!
//! The circuit (in `support/prover_circuit.rs`) exercises every prover
//! component. It is pinned at three sizes because the prover switches
//! between sequential and parallel code paths by vector length: K = 12 is the
//! smallest size here at which every parallel path is taken.

#[path = "support/prover_circuit.rs"]
mod prover_circuit;

use prover_circuit::{circuits, setup, verify, TestRng};

/// Proves two instances of the test circuit with a seeded RNG and returns
/// the proof bytes, after checking that the proof verifies.
fn prove(k: u32) -> Vec<u8> {
    let (params, pk) = setup(k);
    let circuits = circuits(k, 2);
    let proof = prover_circuit::prove(&params, &pk, &circuits, TestRng(0x9e37_79b9_7f4a_7c15));
    assert!(verify(&params, &pk, &circuits, &proof));
    proof
}

fn digest(proof: &[u8]) -> String {
    blake2b_simd::Params::new()
        .hash_length(32)
        .hash(proof)
        .to_hex()
        .to_string()
}

#[test]
fn proof_bytes_are_pinned_k5() {
    assert_eq!(digest(&prove(5)), EXPECTED_K5);
}

#[test]
fn proof_bytes_are_pinned_k10() {
    assert_eq!(digest(&prove(10)), EXPECTED_K10);
}

#[test]
fn proof_bytes_are_pinned_k12() {
    assert_eq!(digest(&prove(12)), EXPECTED_K12);
}

#[test]
fn proving_is_deterministic() {
    assert_eq!(prove(6), prove(6));
}

const EXPECTED_K5: &str = "bfe78e5ff358190e7d917100393c899406b86a8ccbd03c61f3c3dc53b4c86c55";
const EXPECTED_K10: &str = "ad9f1c6cf228c62c5ac0e48093efbeb277fbb89919dedf9ba26bb13958b23c3d";
const EXPECTED_K12: &str = "ba8ff157c532f7df3b854426e0db1565eca6c3b041a3988422a02ec3fcfdffcb";
