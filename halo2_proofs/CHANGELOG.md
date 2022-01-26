# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
(relative to `halo2 0.1.0-beta.1`)

### Added
- `halo2_proofs::plonk`:
  - `VerificationStrategy`
  - `SingleVerifier`, an implementation of `VerificationStrategy` for verifying
    proofs individually.
  - `BatchVerifier`, an implementation of `VerificationStrategy` for verifying
    multiple proofs in a batch.
- `halo2_proofs::dev::FailureLocation` (used in `VerifyFailure::Lookup`)

### Changed
- `halo2_proofs::plonk::verify_proof` now takes a `VerificationStrategy` instead
  of an `MSM` directly.
- `halo2_proofs` now depends on `rand_core` instead of `rand`.
- `halo2_proofs::plonk::create_proof` now take an argument `R: rand_core::RngCore`.
- `halo2_proofs::plonk::Error` has been overhauled:
  - `Error` now implements `std::fmt::Display` and `std::error::Error`.
  - `Error` no longer implements `PartialEq`. Tests can check for specific error
    cases with `assert!(matches!(..))`, or the `assert_matches` crate.
  - `Error::IncompatibleParams` is now `Error::InvalidInstances`.
  - `Error::NotEnoughRowsAvailable` now stores the current value of `k`.
  - `Error::OpeningError` is now `Error::Opening`.
  - `Error::SynthesisError` is now `Error::Synthesis`.
  - `Error::TranscriptError` is now `Error::Transcript`, and stores the
    underlying `io::Error`.
- `halo2_proofs::dev::CircuitLayout::render` now takes `k` as a `u32`, matching
  the regular parameter APIs.
- `halo2_proofs::dev::VerifyFailure` has been overhauled:
  - `VerifyFailure::Cell` has been renamed to `VerifyFailure::CellNotAssigned`.
  - `VerifyFailure::ConstraintNotSatisfied` now has a `cell_values` field,
    storing the values of the cells used in the unsatisfied constraint.
  - The `row` fields of `VerifyFailure::{ConstraintNotSatisfied, Lookup}` have
    been replaced by `location` fields, which can now indicate whether the
    location falls within an assigned region.
- `halo2_proofs::plonk::ConstraintSystem::enable_equality` and 
  `halo2_proofs::plonk::ConstraintSystem::query_any` now take `Into<Column<Any>>`
  instead of `Column<Any>` as a parameter to avoid excesive `.into()` usage.

### Removed
- `halo2_proofs::arithmetic::BatchInvert` (use `ff::BatchInvert` instead).
- `impl Default for halo2_proofs::poly::Rotation` (use `Rotation::cur()` instead).
- `halo2_proofs::poly`:
  - `EvaluationDomain::{add_extended, sub_extended, mul_extended}`
  - `Polynomial::one_minus`
  - `impl Neg, Sub for Polynomial`
  - `impl Mul for Polynomial<_, ExtendedLagrangeCoeff>`
