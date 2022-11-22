# BN256 pairing

BN256 pairing library that implements original traits from `zkcrypto`,

* [`zkcrypto/ff`](https://github.com/zkcrypto/ff)
* [`zkcrypto/group`](https://github.com/zkcrypto/group)
* [`zkcrypto/pairing`](https://github.com/zkcrypto/pairing)

and plus

`FieldExt`, `CurveExt` [traits](https://github.com/zcash/pasta_curves/tree/main/src/arithmetic) that are used in `halo2` library.

This implementation is mostly ported from [matterlabs/pairing](https://github.com/matter-labs/pairing/tree/master/src/bn256) and [zkcrypto/bls12-381](https://github.com/zkcrypto/bls12_381).

## Bench

None Assembly
```
$ cargo test --profile bench test_field -- --nocapture
```

Assembly
```
$ cargo test --profile bench test_field --features asm -- --nocapture
```
