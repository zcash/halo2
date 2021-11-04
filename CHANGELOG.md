# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- `halo2::plonk::Error` has been overhauled:
  - `Error` now implements `std::fmt::Display` and `std::error::Error`.
  - `Error` no longer implements `PartialEq`. Tests can check for specific error
    cases with `assert!(matches!(..))`.
  - `Error::IncompatibleParams` is now `Error::InvalidInstances`.
  - `Error::OpeningError` is now `Error::Opening`.
  - `Error::SynthesisError` is now `Error::Synthesis`.
  - `Error::TranscriptError` is now `Error::Transcript`, and stores the
    underlying `io::Error`.

### Removed
- `halo2::arithmetic::BatchInvert` (use `ff::BatchInvert` instead).

## [0.1.0-beta.1] - 2021-09-24
Initial beta release!
