# An End-to-End example

This example builds off the previous example of the `simple-example`, where the `mock-prover` is replaced with an example of generating real proofs.

## Constructing the circuit using private inputs from the User

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/e2e-example.rs:construct-circuit}}
```

## Creating a proof

To generate a proof, we need to generate a `verification key` (`vk`) and a `proving key` (`ok`) as inputs for generating the proof using `create_proof`.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/e2e-example.rs:create-proof}}
```

## Write proof to a file

This is an example of how to write the proof to a file.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/e2e-example.rs:write-proof}}
```

## Verifying the proof

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/e2e-example.rs:verify-proof}}
```

## Running the example

To generate the proof and verify the proof in this example run: `cargo run --example e2e-example 2 3` where `2` and `3` are the values of the `a` and `b` variables in the proof.

## Full `Main` Function

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/e2e-example.rs:main}}
```