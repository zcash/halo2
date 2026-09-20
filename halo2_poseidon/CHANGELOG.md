# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- The minimum supported Rust version is now 1.88.

### Fixed
- `halo2_poseidon::Hash::<_, _, ConstantLength<0>, _, _>::hash` no longer
  panics: `ConstantLength<0>` now pads the empty message to a single all-zero
  block, so the sponge is permuted once before squeezing. Padding for `L > 0`
  is unchanged.

## [0.1.0] - 2024-12-16
Initial release, extracted from `halo2_gadgets 0.3.0`. Includes minor changes
for `no-std` support.
