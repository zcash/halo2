# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- The minimum supported Rust version is now 1.88.
- Migrated to `ff 0.14`, `group 0.14`, `pasta_curves 0.6`.
- The following APIs now have a `pasta_curves::arithmetic::VartimeField` bound
  and make use of variable-time inversions for improved performance:
  - `halo2_poseidon::generate_constants`

## [0.1.0] - 2024-12-16
Initial release, extracted from `halo2_gadgets 0.3.0`. Includes minor changes
for `no-std` support.
