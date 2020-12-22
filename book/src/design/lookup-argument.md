# Lookup argument

halo2 uses the following lookup technique, which allows for lookups in arbitrary sets, and
is arguably simpler than Plookup.

## Note on Language

In addition to the [general notes on language](../design.md#note-on-language):

- We call the $Z(X)$ polynomial (the grand product argument polynomial for the permutation
  argument) the "permutation product" column.

## Technique Description

We express lookups in terms of a "subset argument" over a table with $2^k$ rows (numbered
from 0), and columns $A$ and $S$.

The goal of the subset argument is to enforce that every cell in $A$ is equal to _some_
cell in $S$. This means that more than one cell in $A$ can be equal to the _same_ cell in
$S$, and some cells in $S$ don't need to be equal to any of the cells in $A$.

- $S$ might be fixed, but it doesn't need to be. That is, we can support looking up values
  in either fixed or variable tables (where the latter includes advice columns).
- $A$ and $S$ can contain duplicates. If the sets represented by $A$ and/or $S$ are not
  naturally of size $2^k$, we extend $S$ with duplicates and $A$ with dummy values known
  to be in $S$.
  - Alternatively we could add a "lookup selector" that controls which elements of the $A$
    column participate in lookups. This would modify the occurrence of $A(X)$ in the
    permutation rule below to replace $A$ with, say, $S_0$ if a lookup is not selected.

Let $\ell_i$ be the Lagrange basis polynomial that evaluates to $1$ at row $i$, and $0$
otherwise.

We start by allowing the prover to supply permutation columns of $A$ and $S$. Let's call
these $A'$ and $S'$, respectively. We can enforce that they are permutations using a
permutation argument with product column $Z$ with the rules:

$$
Z(X) (A(X) + \beta) (S(X) + \gamma) - Z(\omega^{-1} X) (A'(X) + \beta) (S'(X) + \gamma) = 0
$$$$
\ell_0(X) (Z(X) - 1) = 0
$$

This is a version of the permutation argument which allows $A'$ and $S'$ to be
permutations of $A$ and $S$, respectively, but doesn't specify the exact permutations.
$\beta$ and $\gamma$ are separate challenges so that we can combine these two permutation
arguments into one without worrying that they might interfere with each other.

The goal of these permutations is to allow $A'$ and $S'$ to be arranged by the prover in a
particular way:

1. All the cells of column $A'$ are arranged so that like-valued cells are vertically
   adjacent to each other. This could be done by some kind of sorting algorithm, but all
   that matters is that like-valued cells are on consecutive rows in column $A'$, and that
   $A'$ is a permutation of $A$.
2. The first row in a sequence of like values in $A'$ is the row that has the
   corresponding value in $S'.$ Apart from this constraint, $S'$ is any arbitrary
   permutation of $S$.

Now, we'll enforce that either $A'_i = S'_i$ or that $A'_i = A'_{i-1}$, using the rule

$$
(A'(X) - S'(X)) \cdot (A'(X) - A'(\omega^{-1} X)) = 0
$$

In addition, we enforce $A'_0 = S'_0$ using the rule

$$
\ell_0(X) \cdot (A'(X) - S'(X)) = 0
$$

Together these constraints effectively force every element in $A'$ (and thus $A$) to equal
at least one element in $S'$ (and thus $S$). Proof: by induction on prefixes of the rows.

## Cost

* There is the original column $A$ and the fixed column $S$.
* There is a permutation product column $Z$.
* There are the two permutations $A'$ and $S'$.
* The gates are all of low degree.

## Generalizations

halo2's lookup argument implementation generalizes the above technique in the following
ways:

- $A$ and $S$ can be extended to multiple columns, combined using a random challenge. $A'$
  and $S'$ stay as single columns.
  - The commitments to the columns of $S$ can be precomputed, then combined cheaply once
    the challenge is known by taking advantage of the homomorphic property of Pedersen
    commitments.
- Then, a lookup argument for an arbitrary-width relation can be implemented in terms of a
  subset argument, i.e. to constrain $\mathcal{R}(x, y, ...)$ in each row, consider
  $\mathcal{R}$ as a set of tuples $S$ (using the method of the previous point), and check
  that $(x, y, ...) \in \mathcal{R}$.
  - In the case where $\mathcal{R}$ represents a function, this implicitly also checks
    that the inputs are in the domain. This is typically what we want, and often saves an
    additional range check.
- We can support multiple tables in the same circuit, by combining them into a single
  table that includes a tag column to identify the original table.
  - The tag column could be merged with the "lookup selector" mentioned earlier, if this
    were implemented.

These generalizations are similar to those in sections 4 and 5 of the
[Plookup paper](https://eprint.iacr.org/2020/315.pdf) That is, the differences from
Plookup are in the subset argument. This argument can then be used in all the same ways;
for instance, the optimized range check technique in section 5 of the Plookup paper can
also be used with this subset argument.
