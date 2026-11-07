# halo2_proofs [![Crates.io](https://img.shields.io/crates/v/halo2_proofs.svg)](https://crates.io/crates/halo2_proofs) #

## [Documentation](https://docs.rs/halo2_proofs)

## Minimum Supported Rust Version

Requires Rust **1.88** or higher.

Minimum supported Rust version can be changed in the future, but it will be done with a
minor version bump.

## Controlling parallelism

`halo2_proofs` currently uses [rayon](https://github.com/rayon-rs/rayon) for parallel
computation. The `RAYON_NUM_THREADS` environment variable can be used to set the number of
threads.
The `"multicore"` feature also enables parallel Pasta curve multi-scalar multiplication.

You can disable `rayon` by disabling the `"multicore"` feature.
Warning! Halo2 will lose access to parallelism if you disable the `"multicore"` feature.
This will significantly degrade performance.

## Multi-scalar multiplication

Verifier MSM evaluation and public instance commitments try the Pasta curves'
GLV variable-time MSM backend and fall back to the generic signed-Booth
implementation when the curve or input size is not supported. Witness-bearing
prover commitments and IPA operations retain `arithmetic::best_multiexp`'s
existing implementation. Ordinary scalar multiplication is unchanged.

The `msm` Criterion benchmark compares the generic and hooked paths for Pallas and
Vesta at verifier sizes of 1, 2, 8, 16, 32 and 64 terms, and at 2048 terms for
Orchard-sized workloads. Run `cargo bench -p halo2_proofs --bench msm`; use
`RAYON_NUM_THREADS=1` or `--no-default-features` to compare single-worker or
non-multicore configurations.

## Verifier performance

Verification batches multi-opening denominators into one inversion and expands
IPA generator coefficients directly into their owned MSM allocation. Batch
verification applies each proof's random weight during coefficient generation
and coalesces buffered commitment terms once before the final MSM.

The `plonk_verifier` Criterion benchmark measures single and batch verification
with an Orchard-shaped instance layout (not the full Orchard circuit):

```sh
RAYON_NUM_THREADS=4 cargo bench -p halo2_proofs --bench plonk_verifier
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
