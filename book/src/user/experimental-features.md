# Experimental features

In `privacy-scaling-explorations/halo2` fork we have implemented many experimental features to cover different use cases, especially for those with huge circuits. We collect them in this page for easier tracking and referencing.

## Commitment scheme abstraction

To support different kinds of polynomial commitment schemes, we've added a trait `CommitmentScheme` to allow create/verify proofs with different commitment scheme implementations, currently there are 2 available implementations in this fork:

- [`IPACommitmentScheme`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/poly/ipa/commitment/struct.IPACommitmentScheme.html)

  The original implementation from `zcash/halo2` with the original multi-open strategy `{Prover,Verifier}IPA`

- [`KZGCommitmentScheme`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/poly/kzg/commitment/struct.KZGCommitmentScheme.html)

  KZG commitment scheme as in [GWC19](https://eprint.iacr.org/2019/953), which doesn't commit the instance columns, with 2 multi-open strategies available:

  - `{Prover,Verifier}GWC` - The original strategy in [GWC19](https://eprint.iacr.org/2019/953)
  - `{Prover,Verifier}SHPLONK` - The strategy proposed in [BDFG20](https://eprint.iacr.org/2020/081)

When using `create_proof` and `verify_proof`, we need to specify the commitment scheme and multi-open strategy like:

```rust
// Using IPA
create_proof<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _, _>
verify_proof<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _>

// Using KZG with GWC19 mutli-open strategy
create_proof<KZGCommitmentScheme<_>, ProverGWC<_>, _, _, _, _>
verify_proof<KZGCommitmentScheme<_>, ProverGWC<_>, _, _, _>

// Using KZG with BDFG20 mutli-open strategy
create_proof<KZGCommitmentScheme<_>, ProverSHPLONK<_>, _, _, _, _>
verify_proof<KZGCommitmentScheme<_>, ProverSHPLONK<_>, _, _, _>
```

## `ConstraintSystem` extension

### Dynamic lookup

[`ConstraintSystem::lookup_any`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.lookup_any) is added for use cases that need to lookup dynamic table instead of fixed table.

Unlike `ConstraintSystem::lookup` which only allows `TableColumn`(s) as table, it allows any `Expression`(s) without simple selector.

### Shuffle

[`ConstraintSystem::shuffle`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.shuffle) is added for use cases that only need shuffle without pre-defined mapping.

It allows us to prove any `Expression`(s) without simple selector is shuffled from the other. For example `halo2_proofs/examples/shuffle_api.rs` shows how to prove two lists of 2-tuples are shuffled of each other.

### Multi-phase

[`ConstraintSystem::advice_column_in`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.advice_column_in) and [`ConstraintSystem::challenge_usable_after`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.challenge_usable_after) are added for use cases that build PIOP sub-routine themselves, currently it supports up-to 3 phases as `{First,Second,Third}Phase`.

It allows us to allocate advice column in different interactive phases with extra challenges squeezed in-between. For example in `halo2_proofs/examples/shuffle.rs` it shows how to build a customized shuffle argument with such API.

### Unblinded advice column

[`ConstraintSystem::unblinded_advice_column`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.unblinded_advice_column) is added for use cases that want to reuse advice column commitment among different proofs. For example in `halo2_proofs/examples/vector-ops-unblinded.rs` it shows with this API and same assignment, two advice commitment from different proof can be same.

Worth mentioning, re-using advice column commitment in different proofs will need more blinding factors than the amount that prover adds, otherwise some information will be leaked and it's no longer perfect zero-knowledge.

## [`Expression`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/enum.Expression.html) extension

- [`Expression::Challenge`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/enum.Expression.html#variant.Challenge)

  A variant added for multi-phase. It can be obtained by [`ConstraintSystem::challenge_usable_after`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ConstraintSystem.html#method.challenge_usable_after) and used as a pseudo-random constant.

- [`Expression::identifier`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/enum.Expression.html#method.identifier)

  It prints `Expression` in a less verbose and human-readable way.

## [`Circuit`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/trait.Circuit.html) extension

- [`Circuit::Params`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/trait.Circuit.html#associatedtype.Params)

  To use this, feature `circuit-params` needs to be turned on.

  A associated type added for configuring circuit with runtime parameter. 
  
  It allows us to implement [`Circuit::params`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/trait.Circuit.html#method.params) to return the parameter of a circuit, and implement [`Circuit::configure_with_params`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/trait.Circuit.html#method.configure_with_params) to configure circuit with runtime parameter retrieved earlier.

## [`ProvingKey`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.ProvingKey.html) & [`VerifyingKey`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/struct.VerifyingKey.html) de/serialization and [`SerdeFormat`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/enum.SerdeFormat.html)

`ProvingKey::{read,write}` and `VerifyingKey::{read,write}` is added to serialize proving key and verifying key.

For field elements and elliptic curve points inside pk and vk, currently we have 3 different de/serialization format:

- [`SerdeFormat::Processed`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/enum.SerdeFormat.html#variant.Processed)

  It de/serializes them as `PrimeField::Repr` and `GroupEncoding::Repr` respectively, and checks all elements are valid.

- [`SerdeFormat::RawBytes`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/enum.SerdeFormat.html#variant.RawBytes)

  It de/serializes them as `SerdeObject::{from_raw_bytes,to_raw_bytes}` respectively, and checks all elements are valid.

- [`SerdeFormat::RawBytesUnchecked`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/enum.SerdeFormat.html#variant.RawBytesUnchecked)

  It de/serializes them as `SerdeObject::{from_raw_bytes,to_raw_bytes}` respectively, without checking if elements are valid.

Also `ParamsKZG::{read_custom,write_custom}` follows the same rule, and by default `ParamsKZG::{read,write}` uses `SerdeFormat::RawBytes` for efficiency.

## Thread safe [`Region`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/circuit/struct.Region.html)

To use this, feature `thread-safe-region` needs to be turned on.

It constrains the `RegionLayouter` to be `Send` so we can have a `Region` in different threads. It's useful when we want to arrange witness computation and assignment in the same place, but still keep the function `Send` so the caller can parallelize multiple of them.

For example `halo2_proofs/examples/vector-mul.rs` shows how to parallelize region computation and assignment.

## Optional selector compression

Currently [`keygen_vk`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/fn.keygen_vk.html) changes configured `ConstraintSystem` to compresses simple selectors into smaller set of fixed columns to reduce cost.

For some use cases that want to keep configured `ConstraintSystem` unchanged they can do the verifying key generation by calling [`keygen_vk_custom`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/plonk/fn.keygen_vk_custom.html) with second argument as `false` instead, which disables the selector compression.

## [`MockProver`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html) improvement

- [`MockProver::verify_par`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html#method.verify_par)

  Same checks as `MockProver::verify`, but parallelized.

- [`MockProver::verify_at_rows`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html#method.verify_at_rows)

  Same checks as `MockProver::verify`, but only on specified rows.

- [`MockProver::verify_at_rows_par`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html#method.verify_at_rows_par)

  Same checks as `MockProver::verify_at_rows`, but parallelized.

- [`MockProver::assert_satisfied_par`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html#method.assert_satisfied_par)

  Same assertions as `MockProver::assert_satisfied`, but parallelized.

- [`MockProver::assert_satisfied_at_rows_par`](https://privacy-scaling-explorations.github.io/halo2/halo2_proofs/dev/struct.MockProver.html#method.assert_satisfied_at_rows_par)

  Same assertions as `MockProver::assert_satisfied_par`, but only on specified rows.

## `Evaluator` and `evaluate_h`

They are introduced to improve quotient computation speed and memory usage for circuit with complicated `Expression`.
