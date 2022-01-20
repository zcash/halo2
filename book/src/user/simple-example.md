# A simple example

Let's start with a simple circuit, to introduce you to the common APIs and how they are
used. The circuit will take a public input $c$, and will prove knowledge of two private
inputs $a$ and $b$ such that

$$a^2 \cdot b^2 = c.$$

## Define instructions

Firstly, we need to define the instructions that our circuit will rely on. Instructions
are the boundary between high-level [gadgets](../concepts/gadgets.md) and the low-level
circuit operations. Instructions may be as coarse or as granular as desired, but in
practice you want to strike a balance between an instruction being large enough to
effectively optimize its implementation, and small enough that it is meaningfully
reusable.

For our circuit, we will use three instructions:
- Load a private number into the circuit.
- Multiply two numbers.
- Expose a number as a public input to the circuit.

We also need a type for a variable representing a number. Instruction interfaces provide
associated types for their inputs and outputs, to allow the implementations to represent
these in a way that makes the most sense for their optimization goals.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:instructions}}
```

## Define a chip implementation

For our circuit, we will build a [chip](../concepts/chips.md) that provides the above
numeric instructions for a finite field.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:chip}}
```

Every chip needs to implement the `Chip` trait. This defines the properties of the chip
that a `Layouter` may rely on when synthesizing a circuit, as well as enabling any initial
state that the chip requires to be loaded into the circuit.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:chip-impl}}
```

## Configure the chip

The chip needs to be configured with the columns, permutations, and gates that will be
required to implement all of the desired instructions.

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:chip-config}}
```

## Implement chip traits

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:instructions-impl}}
```

## Build the circuit

Now that we have the instructions we need, and a chip that implements them, we can finally
build our circuit!

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:circuit}}
```

## Testing the circuit

`halo2_proofs::dev::MockProver` can be used to test that the circuit is working correctly. The
private and public inputs to the circuit are constructed as we will do to create a proof,
but by passing them to `MockProver::run` we get an object that can test every constraint
in the circuit, and tell us exactly what is failing (if anything).

```rust,ignore,no_run
{{#include ../../../halo2_proofs/examples/simple-example.rs:test-circuit}}
```

## Full example

You can find the source code for this example
[here](https://github.com/zcash/halo2/tree/main/halo2_proofs/examples/simple-example.rs).
