# Developer tools

The `halo2` crate includes several utilities to help you design and implement your
circuits.

## Mock prover

`halo2::dev::MockProver` is a tool for debugging circuits, as well as cheaply verifying
their correctness in unit tests. The private and public inputs to the circuit are
constructed as would normally be done to create a proof, but `MockProver::run` instead
creates an object that will test every constraint in the circuit directly. It returns
granular error messages that indicate which specific constraint (if any) is not satisfied.

## Circuit visualizations

The `dev-graph` feature flag exposes several helper methods for creating graphical
representations of circuits.

### Circuit layout

`halo2::dev::CircuitLayout` renders the circuit layout as a grid:

```rust,ignore,no_run
{{#include ../../../examples/circuit-layout.rs:dev-graph}}
```

- Columns are layed out from left to right as instance, advice, and fixed. The order of
  columns is otherwise without meaning.
  - Instance columns have a white background.
  - Advice columns have a red background.
  - Fixed columns have a blue background.
- Regions are shown as labelled green boxes (overlaying the background colour). A region
  may appear as multiple boxes if some of its columns happen to not be adjacent.
- Cells that have been assigned to by the circuit will be shaded in grey. If any cells are
  assigned to more than once (which is usually a mistake), they will be shaded darker than
  the surrounding cells.

### Circuit structure

`halo2::dev::circuit_dot_graph` builds a [DOT graph string] representing the given
circuit, which can then be rendered witha variety of [layout programs]. The graph is built
from calls to `Layouter::namespace` both within the circuit, and inside the gadgets and
chips that it uses.

[DOT graph string]: https://graphviz.org/doc/info/lang.html
[layout programs]: https://en.wikipedia.org/wiki/DOT_(graph_description_language)#Layout_programs

```rust,ignore,no_run
fn main() {
    // Prepare the circuit you want to render.
    // You don't need to include any witness variables.
    let a = Fp::rand();
    let instance = Fp::one() + Fp::one();
    let lookup_table = vec![instance, a, a, Fp::zero()];
    let circuit: MyCircuit<Fp> = MyCircuit {
        a: None,
        lookup_table,
    };

    // Generate the DOT graph string.
    let dot_string = halo2::dev::circuit_dot_graph(&circuit);

    // Now you can either handle it in Rust, or just
    // print it out to use with command-line tools.
    print!("{}", dot_string);
}
```

## Cost estimator

The `cost-model` binary takes high-level parameters for a circuit design, and estimates
the verification cost, as well as resulting proof size.

```plaintext
Usage: cargo run --example cost-model -- [OPTIONS] k

Positional arguments:
  k                       2^K bound on the number of rows.

Optional arguments:
  -h, --help              Print this message.
  -a, --advice R[,R..]    An advice column with the given rotations. May be repeated.
  -i, --instance R[,R..]  An instance column with the given rotations. May be repeated.
  -f, --fixed R[,R..]     A fixed column with the given rotations. May be repeated.
  -g, --gate-degree D     Maximum degree of the custom gates.
  -l, --lookup N,I,T      A lookup over N columns with max input degree I and max table degree T. May be repeated.
  -p, --permutation N     A permutation over N columns. May be repeated.
```

For example, to estimate the cost of a circuit with three advice columns and one fixed
column (with various rotations), and a maximum gate degree of 4:

```plaintext
> cargo run --example cost-model -- -a 0,1 -a 0 -a-0,-1,1 -f 0 -g 4 11
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/examples/cost-model -a 0,1 -a 0 -a 0,-1,1 -f 0 -g 4 11`
Circuit {
    k: 11,
    max_deg: 4,
    advice_columns: 3,
    lookups: 0,
    permutations: [],
    column_queries: 7,
    point_sets: 3,
    estimator: Estimator,
}
Proof size: 1440 bytes
Verification: at least 81.689ms
```
