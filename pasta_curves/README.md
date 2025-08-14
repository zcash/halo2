# `secpq_curves`

This crate provides an implementation of the Secpq elliptic curve constructions,
Secp256k1 (used in Ethereum/Bitcoin ECDSA) and Secq256k1. More details about these curves
can be found at https://hackmd.io/@dJO3Nbl4RTirkR2uDM6eOA/Bk0NvC8Vo.

## [Documentation](https://docs.rs/pasta_curves)

## Minimum Supported Rust Version

Requires Rust **1.56** or higher.

Minimum supported Rust version can be changed in the future, but it will be done with a
minor version bump.

## Curve Descriptions

- Secp256k1: y<sup>2</sup> = x<sup>3</sup> + 7 over
  `GF(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F)`.

- Secq256k1: y<sup>2</sup> = x<sup>3</sup> + 7 over
  `GF(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141)`.
