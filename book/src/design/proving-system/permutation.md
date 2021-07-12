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
to write permutations. Let $(a\ b\ c)$ denote a cycle where $a$ maps to $b,$ $b$ maps to
$c,$ and $c$ maps to $a$ (with the obvious generalization to arbitrary-sized cycles).
Writing two or more cycles next to each other denotes a composition of the corresponding
permutations. For example, $(a\ b)\ (c\ d)$ denotes the permutation that maps $a$ to $b,$
$b$ to $a,$ $c$ to $d,$ and $d$ to $c.$

## Constructing the permutation

### Goal

We want to construct a permutation in which each subset of variables that are in a
equality-constraint set form a cycle. For example, suppose that we have a circuit that
defines the following equality constraints:

- $a \equiv b$
- $a \equiv c$
- $d \equiv e$

From this we have the equality-constraint sets $\{a, b, c\}$ and $\{d, e\}.$ We want to
construct the permutation:

$$(a\ b\ c)\ (d\ e)$$

which defines the mapping of $[a, b, c, d, e]$ to $[b, c, a, e, d].$

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

We have the invariant that for each element $x$ in a given cycle $C,$ $\mathsf{aux}(x)$
points to the same element $c \in C.$ This allows us to quickly decide whether two given
elements $x$ and $y$ are in the same cycle, by checking whether
$\mathsf{aux}(x) = \mathsf{aux}(y).$ Also, $\mathsf{sizes}(\mathsf{aux}(x))$ gives the
size of the cycle containing $x.$ (This is guaranteed only for
$\mathsf{sizes}(\mathsf{aux}(x)),$ not for $\mathsf{sizes}(x).$)

The algorithm starts with a representation of the identity permutation:
for all $x,$ we set $\mathsf{mapping}(x) = x,$ $\mathsf{aux}(x) = x,$ and
$\mathsf{sizes}(x) = 1.$

To add an equality constraint $\mathit{left} \equiv \mathit{right}$:

1. Check whether $\mathit{left}$ and $\mathit{right}$ are already in the same cycle, i.e.
   whether $\mathsf{aux}(\mathit{left}) = \mathsf{aux}(\mathit{right}).$ If so, there is
   nothing to do.
2. Otherwise, $\mathit{left}$ and $\mathit{right}$ belong to different cycles. Make
   $\mathit{left}$ the larger cycle and $\mathit{right}$ the smaller one, by swapping them
   iff $\mathsf{sizes}(\mathsf{aux}(\mathit{left})) < \mathsf{sizes}(\mathsf{aux}(\mathit{right})).$
3. Set $\mathsf{sizes}(\mathsf{aux}(\mathit{left})) :=
        \mathsf{sizes}(\mathsf{aux}(\mathit{left})) + \mathsf{sizes}(\mathsf{aux}(\mathit{right})).$
4. Following the mapping around the right (smaller) cycle, for each element $x$ set
   $\mathsf{aux}(x) := \mathsf{aux}(\mathit{left}).$
5. Splice the smaller cycle into the larger one by swapping $\mathsf{mapping}(\mathit{left})$
   with $\mathsf{mapping}(\mathit{right}).$

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

and we tried to implement adding an equality constraint just using step 5 of the above
algorithm, then we would end up constructing the cycle $(a\ b)\ (c\ d),$ rather than the
correct $(a\ b\ c\ d).$

## Argument specification

We need to check a permutation of cells in $m$ columns, represented in Lagrange basis by
polynomials $v_0, \ldots, v_{m-1}.$

We will label *each cell* in those $m$ columns with a unique element of $\mathbb{F}^\times.$

Suppose that we have a permutation on these labels,
$$
\sigma(\mathsf{column}: i, \mathsf{row}: j) = (\mathsf{column}: i', \mathsf{row}: j').
$$
in which the cycles correspond to equality-constraint sets.

> If we consider the set of pairs $\{(\mathit{label}, \mathit{value})\}$, then the values within
> each cycle are equal if and only if permuting the label in each pair by $\sigma$ yields the
> same set:
>
> ![An example for a cycle (A B C D). The set before permuting the labels is {(A, 7), (B, 7), (C, 7), (D, 7)}, and the set after is {(D, 7), (A, 7), (B, 7), (C, 7)} which is the same. If one of the 7s is replaced by 3, then the set after permuting labels is not the same.](./permutation-diagram.png)
>
> Since the labels are distinct, set equality is the same as multiset equality, which we can
> check using a product argument.

Let $\omega$ be a $2^k$ root of unity and let $\delta$ be a $T$ root of unity, where
${T \cdot 2^S + 1 = p}$ with $T$ odd and ${k \leq S.}$
We will use $\delta^i \cdot \omega^j \in \mathbb{F}^\times$ as the label for the
cell in the $j$th row of the $i$th column of the permutation argument.

We represent $\sigma$ by a vector of $m$ polynomials $s_i(X)$ such that
$s_i(\omega^j) = \delta^{i'} \cdot \omega^{j'}.$

Notice that the identity permutation can be represented by the vector of $m$ polynomials
$\mathsf{ID}_i(\omega^j)$ such that $\mathsf{ID}_i(\omega^j) = \delta^i \cdot \omega^j.$

We will use a challenge $\beta$ to compress each ${(\mathit{label}, \mathit{value})}$ pair
to $\mathit{value} + \beta \cdot \mathit{label}.$ Just as in the product argument we used
for [lookups](lookup.md), we also use a challenge $\gamma$ to randomize each term of the
product.

Now given our permutation represented by $s_0, \ldots, s_{m-1}$ over columns represented by
$v_0, \ldots, v_{m-1},$ we want to ensure that:
$$
\prod\limits_{i=0}^{m-1} \prod\limits_{j=0}^{n-1} \left(\frac{v_i(\omega^j) + \beta \cdot \delta^i \cdot \omega^j + \gamma}{v_i(\omega^j) + \beta \cdot s_i(\omega^j) + \gamma}\right) = 1
$$

> Here ${v_i(\omega^j) + \beta \cdot \delta^i \cdot \omega^j}$ represents the unpermuted
> $(\mathit{label}, value)$ pair, and ${v_i(\omega^j) + \beta \cdot s_i(\omega^j)}$
> represents the permuted $(\sigma(\mathit{label}), value)$ pair.

Let $Z_P$ be such that $Z_P(\omega^0) = Z_P(\omega^n) = 1$ and for $0 \leq j < n$:
$$\begin{array}{rl}
Z_P(\omega^{j+1}) &= \prod\limits_{h=0}^{j} \prod\limits_{i=0}^{m-1} \frac{v_i(\omega^h) + \beta \cdot \delta^i \cdot \omega^h + \gamma}{v_i(\omega^h) + \beta \cdot s_i(\omega^h) + \gamma} \\
                  &= Z_P(\omega^j) \prod\limits_{i=0}^{m-1} \frac{v_i(\omega^j) + \beta \cdot \delta^i \cdot \omega^j + \gamma}{v_i(\omega^j) + \beta \cdot s_i(\omega^j) + \gamma}
\end{array}$$

Then it is sufficient to enforce the rules:
$$
Z_P(\omega X) \cdot \prod\limits_{i=0}^{m-1} \left(v_i(X) + \beta \cdot s_i(X) + \gamma\right) - Z_P(X) \cdot \prod\limits_{i=0}^{m-1} \left(v_i(X) + \beta \cdot \delta^i \cdot X + \gamma\right) = 0 \\
\ell_0 \cdot (1 - Z_P(X)) = 0
$$

This assumes that the number of columns $m$ is such that the polynomial in the first
rule above fits within the degree bound of the PLONK configuration. We will see
[below](#spanning-a-large-number-of-columns) how to handle a larger number of columns.

> The optimization used to obtain the simple representation of the identity permutation was suggested
> by Vitalik Buterin for PLONK, and is described at the end of section 8 of the PLONK paper. Note that
> the $\delta^i$ are all distinct quadratic non-residues, provided that the number of columns that
> are enabled for equality is no more than $T$, which always holds in practice for the curves used in
> Halo 2.

## Zero-knowledge adjustment

Similarly to the [lookup argument](lookup.md#zero-knowledge-adjustment), we need an
adjustment to the above argument to account for the last $t$ rows of each column being
filled with random values.

We limit the number of usable rows to $u = 2^k - t - 1.$ We add two selectors,
defined in the same way as for the lookup argument:

* $q_\mathit{blind}$ is set to $1$ on the last $t$ rows, and $0$ elsewhere;
* $q_\mathit{last}$ is set to $1$ only on row $u,$ and $0$ elsewhere (i.e. it is set on
  the row in between the usable rows and the blinding rows).

We enable the product rule from above only for the usable rows:

$\big(1 - (q_\mathit{last}(X) + q_\mathit{blind}(X))\big) \cdot$
$\hspace{1em}\left(Z_P(\omega X) \cdot \prod\limits_{i=0}^{m-1} \left(v_i(X) + \beta \cdot s_i(X) + \gamma\right) - Z_P(X) \cdot \prod\limits_{i=0}^{m-1} \left(v_i(X) + \beta \cdot \delta^i \cdot X + \gamma\right)\right) = 0$

The rule that is enabled on row $0$ remains the same:

$$
\ell_0(X) \cdot (1 - Z_P(X)) = 0
$$

Since we can no longer rely on the wraparound to ensure that each product $Z_P$ becomes
$1$ again at $\omega^{2^k},$ we would instead need to constrain $Z(\omega^u) = 1.$ This
raises the same problem that was described for the lookup argument. So we allow
$Z(\omega^u)$ to be either zero or one:

$$
q_\mathit{last}(X) \cdot (Z_P(X)^2 - Z_P(X)) = 0
$$

which gives perfect completeness and zero knowledge.

## Spanning a large number of columns

The halo2 implementation does not in practice limit the number of columns for which
equality constraints can be enabled. Therefore, it must solve the problem that the
above approach might yield a product rule with a polynomial that exceeds the PLONK
configuration's degree bound. The degree bound could be raised, but this would be
inefficient if no other rules require a larger degree.

Instead, we split the product across $b$ sets of $m$ columns, using product columns
$Z_{P,0}, \ldots Z_{P,b-1},$ and we use another rule to copy the product from the end
of one column set to the beginning of the next.

That is, for $0 \leq a < b$ we have:

$\big(1 - (q_\mathit{last}(X) + q_\mathit{blind}(X))\big) \cdot$
$\hspace{1em}\left(Z_{P,a}(\omega X) \cdot \!\prod\limits_{i=am}^{(a+1)m-1}\! \left(v_i(X) + \beta \cdot s_i(X) + \gamma\right) - Z_P(X) \cdot \!\prod\limits_{i=am}^{(a+1)m-1}\! \left(v_i(X) + \beta \cdot \delta^i \cdot X + \gamma\right)\right)$
$\hspace{2em}= 0$

> For simplicity this is written assuming that the number of columns enabled for
> equality constraints is a multiple of $m$; if not then the products for the last
> column set will have fewer than $m$ terms.

For the first column set we have:

$$
\ell_0 \cdot (1 - Z_{P,0}(X)) = 0
$$

For each subsequent column set, $0 < a < b,$ we use the following rule to copy
$Z_{P,a-1}(\omega^u)$ to the start of the next column set, $Z_{P,a}(\omega^0)$:

$$
\ell_0 \cdot \left(Z_{P,a}(X) - Z_{P,a-1}(\omega^u X)\right) = 0
$$

For the last column set, we allow $Z_{P,b-1}(\omega^u)$ to be either zero or one:

$$
q_\mathit{last}(X) \cdot \left(Z_{P,b-1}(X)^2 - Z_{P,b-1}(X)\right) = 0
$$

which gives perfect completeness and zero knowledge as before.
