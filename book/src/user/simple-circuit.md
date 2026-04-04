# Building a simple circuit

This example walks through how to build a circuit by composing an existing gadget.
We use the [Poseidon hash](../design/gadgets.md) gadget from `halo2_gadgets` to hash
two private inputs and expose the digest as a public input.

If you want to see how to build a custom gadget and chip from scratch instead, see
[Building a simple gadget](simple-gadget.md).

## Overview

The circuit proves knowledge of two private field elements whose Poseidon hash equals a
given public value. It demonstrates:

- Allocating columns and configuring an existing chip (`Pow5Chip`).
- Loading private witnesses into advice cells.
- Using the `Hash` gadget to compute a Poseidon hash in-circuit.
- Exposing the result through an instance column.

## Define the circuit

We store the two private inputs as `Value<Fp>`, which is `Value::unknown()` during
key generation and `Value::known(...)` during proving.

```rust,ignore,no_run
{{#include ../../../halo2_gadgets/examples/simple-circuit.rs:circuit}}
```

## Configure the circuit

Configuration allocates all the columns the Poseidon chip needs, plus an instance
column for our public output. The key call is `Pow5Chip::configure`, which sets up
the gates for the Poseidon permutation.

```rust,ignore,no_run
{{#include ../../../halo2_gadgets/examples/simple-circuit.rs:config}}
```

## Implement the `Circuit` trait

In `synthesize`, we load our private inputs, construct the chip, call the `Hash`
gadget, and constrain the output to equal the instance column.

```rust,ignore,no_run
{{#include ../../../halo2_gadgets/examples/simple-circuit.rs:circuit-impl}}
```

## Testing the circuit

We compute the expected Poseidon hash outside the circuit using the primitives API,
then verify that `MockProver` accepts the correct public input and rejects a wrong one.

```rust,ignore,no_run
{{#include ../../../halo2_gadgets/examples/simple-circuit.rs:main}}
```

## Full example

You can find the source code for this example
[here](https://github.com/zcash/halo2/tree/main/halo2_gadgets/examples/simple-circuit.rs).
