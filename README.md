# halo2 [![Crates.io](https://img.shields.io/crates/v/halo2.svg)](https://crates.io/crates/halo2) #

## [Documentation](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs)

This repository contains the [halo2](https://github.com/zcash/halo2) fork from
PSE and includes contributions from the community.

We use the `main` branch for development, which means it may contain
unstable/unfinished features.  For end-users we recomend using the tag releases
which can be seen as curated checkpoints with some level of guarantee of
stability.

For experimental features `privacy-scaling-explorations/halo2` fork adds, please refer to [`experimental-features.md`](./book/src/user/experimental-features.md).

## Minimum Supported Rust Version

Requires Rust **1.65.0** or higher.

Minimum supported Rust version can be changed in the future, but it will be done with a
minor version bump.

## Controlling parallelism

`halo2` currently uses [rayon](https://github.com/rayon-rs/rayon) for parallel computation. The `RAYON_NUM_THREADS` environment variable can be used to set the number of threads.

When compiling to WASM-targets, notice that since version `1.7`, `rayon` will fallback automatically (with no need to handle features) to require `getrandom` in order to be able to work. For more info related to WASM-compilation.

See: [Rayon: Usage with WebAssembly](https://github.com/rayon-rs/rayon#usage-with-webassembly) for more 

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
