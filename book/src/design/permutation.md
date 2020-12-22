# Permutation argument

Given that gates in halo2 circuits operate "locally" (on cells in the current row or
defined relative rows), it is common to need to copy a value from some arbitrary cell into
the current row for use in a gate. This is performed with an equality constraint, which
enforces that the source and destination cells contain the same value.

We implement these equality constraints by constructing a permutation that represents the
constraints, and then using a permutation argument within the proof to enforce them.

## Notation

A permutation is a one-to-one and onto mapping of a set onto itself. A permutation can be
factored uniquely into a composition of cycles (up to ordering of cycles, and rotation of
each cycle).

We sometimes use [cycle notation](https://en.wikipedia.org/wiki/Permutation#Cycle_notation)
to write permutations. Let $(a\ b\ c)$ denote a cycle where $a$ maps to $b$, $b$ maps to
$c$, and $c$ maps to $a$ (with the obvious generalisation to arbitrary-sized cycles).
Writing two or more cycles next to each other denotes a composition of the corresponding
permutations. For example, $(a\ b)\ (c\ d)$ denotes the permutation that maps $a$ to $b$,
$b$ to $a$, $c$ to $d$, and $d$ to $c$.

## Constructing the permutation

### Goal

We want to construct a permutation in which each subset of variables that are in a
equality-constraint set form a cycle. For example, suppose that we have a circuit that
defines the following equality constraints:

- $a \equiv b$
- $a \equiv c$
- $d \equiv e$

From this we have the equality-constraint sets $\{a, b, c\}$ and $\{d, e\}$. We want to
construct the permutation:

$$(a\ b\ c)\ (d\ e)$$

which defines the mapping of $[a, b, c, d, e]$ to $[b, c, a, e, d]$.

### Algorithm

We need to keep track of the set of cycles, which is a
[set of disjoint sets](https://en.wikipedia.org/wiki/Disjoint-set_data_structure).
Efficient data structures for this problem are known; for the sake of simplicity we choose
one that is not asymptotically optimal but is easy to implement.

We represent the current state as:

- an array $\mathsf{mapping}$ for the permutation itself;
- an auxiliary array $\mathsf{aux}$ that keeps track of a distinguished element of each
  cycle;
- another array $\mathsf{sizes}$ that keeps track of the size of each cycle.

We have the invariant that for each element $x$ in a given cycle $C$, $\mathsf{aux}(x)$
points to the same element $c \in C$. This allows us to quickly decide whether two given
elements $x$ and $y$ are in the same cycle, by checking whether
$\mathsf{aux}(x) = \mathsf{aux}(y)$. Also, $\mathsf{sizes}(\mathsf{aux}(x))$ gives the
size of the cycle containing $x$. (This is guaranteed only for
$\mathsf{sizes}(\mathsf{aux}(x)))$, not for $\mathsf{sizes}(x)$.)

The algorithm starts with a representation of the identity permutation:
for all $x$, we set $\mathsf{mapping}(x) = x$, $\mathsf{aux}(x) = x$, and
$\mathsf{sizes}(x) = 1$.

To add an equality constraint $\mathit{left} \equiv \mathit{right}$:

1. Check whether $\mathit{left}$ and $\mathit{right}$ are already in the same cycle, i.e.
   whether $\mathsf{aux}(\mathit{left}) = \mathsf{aux}(\mathit{right})$. If so, there is
   nothing to do.
2. Otherwise, $\mathit{left}$ and $\mathit{right}$ belong to different cycles. Make
   $\mathit{left}$ the larger cycle and $\mathit{right}$ the smaller one, by swapping them
   iff $\mathsf{sizes}(\mathsf{aux}(\mathit{left})) < \mathsf{sizes}(\mathsf{aux}(\mathit{right}))$.
3. Following the mapping around the right (smaller) cycle, for each element $x$ set
   $\mathsf{aux}(x) = \mathsf{aux}(\mathit{left})$.
4. Splice the smaller cycle into the larger one by swapping $\mathsf{mapping}(\mathit{left})$
   with $\mathsf{mapping}(\mathit{right})$.

For example, given two disjoint cycles $(A\ B\ C\ D)$ and $(E\ F\ G\ H)$:

```plaintext
A +---> B
^       +
|       |
+       v
D <---+ C       E +---> F
                ^       +
                |       |
                +       v
                H <---+ G
```

After adding constraint $B \equiv E$ the above algorithm produces the cycle:

```plaintext
A +---> B +-------------+
^                       |
|                       |
+                       v
D <---+ C <---+ E       F
                ^       +
                |       |
                +       v
                H <---+ G
```

### Broken alternatives

If we did not check whether $\mathit{left}$ and $\mathit{right}$ were already in the same
cycle, then we could end up undoing an equality constraint. For example, if we have the
following constraints:

- $a \equiv b$
- $b \equiv c$
- $c \equiv d$
- $b \equiv d$

and we tried to implement adding an equality constraint just using step 4 of the above
algorithm, then we would end up constructing the cycle $(a\ b)\ (c\ d)$, rather than the
correct $(a\ b\ c\ d)$.

## Argument specification

TODO: Document what we do with the permutation once we have it.
