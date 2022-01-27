# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- pass in an additional `rng: impl RngCore` argument to `builder::InProgress::create_proof`, `builder::Bundle::create_proof`, `circuit::Proof::create`.
### Removed
- `orchard::value::ValueSum::from_raw`

## [0.1.0-beta.1] - 2021-12-17
Initial release!
