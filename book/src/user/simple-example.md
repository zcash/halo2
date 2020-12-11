# A simple example

Let's start with a simple circuit, to introduce you to the common APIs and how they are
used. The circuit will take a public input $c$, and will prove knowledge of two private
inputs $a$ and $b$ such that

$$a \cdot b = c.$$

```rust
# extern crate halo2;
use halo2::arithmetic::FieldExt;

struct MyCircuit<F: FieldExt> {
    a: F,
    b: F,
}
```

TODO
