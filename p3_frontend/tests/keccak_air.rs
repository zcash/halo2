use p3_keccak_air::{generate_trace_rows, KeccakAir, NUM_ROUNDS};
use p3_util::log2_ceil_usize;

use halo2curves::bn256::Fr;
use p3_frontend::{CompileParams, FWrap};
use rand::random;

mod common;

#[test]
fn test_keccak() {
    let num_hashes = 4;
    // TODO: Replace `random()` with a pseudorandom generator with known seed for deterministic
    // results.
    let inputs = (0..num_hashes).map(|_| random()).collect::<Vec<_>>();
    let size = inputs.len() * NUM_ROUNDS;
    // TODO: 6 must be bigger than unusable rows.  Add a helper function to calculate this
    let n = (size + 6).next_power_of_two();
    let k = log2_ceil_usize(n) as u32;
    let air = KeccakAir {};
    let num_public_values = 0;
    let params = CompileParams { disable_zk: false };
    let trace = generate_trace_rows::<FWrap<Fr>>(inputs);
    let (compiled_circuit, witness, pis) =
        common::compile_witgen(air, &params, k, size, num_public_values, trace);

    common::setup_prove_verify(&compiled_circuit, k, &pis, witness);
}
