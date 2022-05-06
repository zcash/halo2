# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `halo2_gadgets::sinsemilla::MessagePiece::from_subpieces`
- `halo2_gadgets::utilities`:
  - `FieldValue` trait.
  - `RangeConstrained` newtype wrapper.
- `halo2_gadgets::ecc`:
  - `EccInstructions::ScalarVar` is now treated as a full-width scalar, instead
    of being restricted to a base field element.
  - `EccInstructions::witness_scalar_var` API to witness a full-width scalar
    used in variable-base scalar multiplication.
  - `BaseFitsInScalarInstructions` trait that can be implemented for a curve
    whose base field fits into its scalar field. This provides a method
    `scalar_var_from_base` that converts a base field element that exists as
    a variable in the circuit, into a scalar to be used in variable-base
    scalar multiplication.
  - `ScalarVar::new` and `ScalarVar::from_base` gadget APIs.
- `halo2_gadgets::ecc::chip`:
  - `ScalarVar` enum with `BaseFieldElem` and `FullWidth` variants. `FullWidth`
    is unimplemented for `halo2_gadgets v0.1.0`.

### Changed
- `halo2_gadgets::ecc`:
  - `EccInstructions::mul` now takes a `Self::ScalarVar` as argument, instead
    of assuming that the scalar fits in a base field element `Self::Var`.
- `halo2_gadgets::ecc::chip`:
  - `ScalarKind` has been renamed to `FixedScalarKind`.

## [0.1.0-beta.3] - 2022-04-06
### Changed
- Migrated to `halo2_proofs 0.1.0-beta.4`.

## [0.1.0-beta.2] - 2022-03-22
### Changed
- Migrated to `halo2_proofs 0.1.0-beta.3`.

## [0.1.0-beta.1] - 2022-02-14
Initial release!
