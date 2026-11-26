# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-11-26
### Added
- `halo2_poseidon::Spec::try_permute` defaulted hook allowing specifications
  with compile-time constants to evaluate permutations without allocating round
  constants on the heap during sponge hashing.

### Changed
- `halo2_poseidon::P128Pow5T3` now uses faster equivalent MDS constants,
  and avoids per-hash heap allocations during sponge hashing.
- `Absorbing::init_empty` and `Squeezing::init_full` no longer allocate
  temporary `Vec`s on the heap.
- For specifications using the default `Spec::try_permute`, hashing now
  initializes `Spec::constants()` lazily on the first permutation rather
  than during `Hash::init()`, and caches the constants for the sponge’s
  lifetime.

## [0.2.0] - 2026-09-29
### Changed
- The minimum supported Rust version is now 1.88.
- Migrated to `ff 0.14`, `group 0.14`, `pasta_curves 0.6`.

## [0.1.0] - 2024-12-16
Initial release, extracted from `halo2_gadgets 0.3.0`. Includes minor changes
for `no-std` support.
