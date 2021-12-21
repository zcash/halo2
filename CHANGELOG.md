# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `halo2::dev::FailureLocation` (used in `VerifyFailure::Lookup`)

### Changed
- `halo2::plonk::Error` has been overhauled:
  - `Error` now implements `std::fmt::Display` and `std::error::Error`.
  - `Error` no longer implements `PartialEq`. Tests can check for specific error
    cases with `assert!(matches!(..))`, or the `assert_matches` crate.
  - `Error::IncompatibleParams` is now `Error::InvalidInstances`.
  - `Error::NotEnoughRowsAvailable` now stores the current value of `k`.
  - `Error::OpeningError` is now `Error::Opening`.
  - `Error::SynthesisError` is now `Error::Synthesis`.
  - `Error::TranscriptError` is now `Error::Transcript`, and stores the
    underlying `io::Error`.
- `halo2::dev::CircuitLayout::render` now takes `k` as a `u32`, matching the
  regular parameter APIs.
- `halo2::dev::VerifyFailure` has been overhauled:
  - `VerifyFailure::ConstraintNotSatisfied` now has a `cell_values` field,
    storing the values of the cells used in the unsatisfied constraint.
  - The `row` field of `VerifyFailure::Lookup` has been replaced by a `location`
    field, which can now indicate whether the location falls within an assigned
    region.
- `halo2::plonk::ConstraintSystem::enable_equality` and 
  `halo2::plonk::ConstraintSystem::query_any` now take `Into<Column<Any>>` instead 
  of `Column<Any>` as a parameter to avoid excesive `.into()` usage.

### Removed
- `halo2::arithmetic::BatchInvert` (use `ff::BatchInvert` instead).
- `impl Default for halo2::poly::Rotation` (use `Rotation::cur()` instead).

## [0.1.0-beta.1] - 2021-09-24
Initial beta release!
