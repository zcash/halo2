# Selector combining

Heavy use of custom gates can lead to a circuit defining many binary selectors, which
would increase proof size and verification time.

This section describes an optimization, applied automatically by halo2, that combines
binary selector columns into fewer fixed columns.

The basic idea is that if we have $\ell$ binary selectors labelled $1, \ldots, \ell$ that
are enabled on disjoint sets of rows, then under some additional conditions we can combine
them into a single fixed column, say $q$, such that:
$$
q = \begin{cases}
  k, &\text{if the selector labelled } k \text{ is } 1 \\
  0, &\text{if all these selectors are } 0.
\end{cases}
$$

However, the devil is in the detail.

The halo2 API allows defining some selectors to be "simple selectors", subject to the
following condition:

> Every polynomial constraint involving a simple selector $s$ must be of the form
> $s \cdot t = 0,$ where $t$ is a polynomial involving *no* simple selectors.

Suppose that $s$ has label $k$ in some set of $\ell$ simple selectors that are combined
into $q$ as above. Then this condition ensures that replacing $s$ by
$q \cdot \prod_{1 \leq h \leq \ell,\,h \neq k}\; (h - q)$ will not change the meaning of
any constraints.

> It would be possible to relax this condition by ensuring that every use of a binary
> selector is substituted by a precise interpolation of its value from the corresponding
> combined selector. However,
>
> * the restriction simplifies the implementation, developer tooling, and human
>   understanding and debugging of the resulting constraint system;
> * the scope to apply the optimization is not impeded very much by this restriction for
>   typical circuits.

Note that replacing $s$ by $q \cdot \prod_{1 \leq h \leq \ell,\,h \neq k}\; (h - q)$ will
increase the degree of constraints selected by $s$ by $\ell-1$, and so we must choose the
selectors that are combined in such a way that the maximum degree bound is not exceeded.

## Identifying selectors that can be combined

We need a partition of the overall set of selectors $s_0, \ldots, s_{m-1}$ into subsets
(called "combinations"), such that no two selectors in a combination are enabled on the
same row.

Labels must be unique within a combination, but they are not unique across combinations.
Do not confuse a selector's index with its label.

Suppose that we are given $\mathsf{max\_degree}$, the degree bound of the circuit.

We use the following algorithm:

1. Leave nonsimple selectors unoptimized, i.e. map each of them to a separate fixed
   column.
2. Check (or ensure by construction) that all polynomial constraints involving each simple
   selector $s_i$ are of the form $s_i \cdot t_{i,j} = 0$ where $t_{i,j}$ do not involve
   any simple selectors. For each $i$, record the maximum degree of any $t_{i,j}$ as
   $d^\mathsf{max}_i$.
3. Compute a binary "exclusion matrix" $X$ such that $X_{j,i}$ is $1$ whenever $i \neq j$
   and $s_i$ and $s_j$ are enabled on the same row; and $0$ otherwise.
   > Since $X$ is symmetric and is zero on the diagonal, we can represent it by either its
   > upper or lower triangular entries. The rest of the algorithm is guaranteed only to
   > access only the entries $X_{j,i}$ where $j > i$.
4. Initialize a boolean array $\mathsf{added}_{0..{k-1}}$ to all $\mathsf{false}$.
   > $\mathsf{added}_i$ will record whether $s_i$ has been included in any combination.
6. Iterate over the $s_i$ that have not yet been added to any combination:
   * a. Add $s_i$ to a fresh combination $c$, and set $\mathsf{added}_i = \mathsf{true}$.
   * b. Let mut $d := d^\mathsf{max}_i - 1$.
     > $d$ is used to keep track of the largest degree, *excluding* the selector
     > expression, of any gate involved in the combination $c$ so far.
   * c. Iterate over all the selectors $s_j$ for $j > i$ that can potentially join $c$,
     i.e. for which $\mathsf{added}_j$ is false:
     * i. (Optimization) If $d + \mathsf{len}(c) = \mathsf{max\_degree}$, break to the
       outer loop, since no more selectors can be added to $c$.
     * ii. Let $d^\mathsf{new} = \mathsf{max}(d, d^\mathsf{max}_j-1)$.
     * iii. If $X_{j,i'}$ is $\mathsf{true}$ for any $i'$ in $c$, or if
       $d^\mathsf{new} + (\mathsf{len}(c) + 1) > \mathsf{max\_degree}$, break to the outer
       loop.
       > $d^\mathsf{new} + (\mathsf{len}(c) + 1)$ is the maximum degree, *including* the
       > selector expression, of any constraint that would result from adding $s_j$ to the
       > combination $c$.
     * iv. Set $d := d^\mathsf{new}$.
     * v. Add $s_j$ to $c$ and set $\mathsf{added}_j := \mathsf{true}$.
   * d. Allocate a fixed column $q_c$, initialized to all-zeroes.
   * e. For each selector $s' \in c$:
     * i. Label $s'$ with a distinct index $k$ where $1 \leq k \leq \mathsf{len}(c)$.
     * ii. Record that $s'$ should be substituted with
       $q_c \cdot \prod_{1 \leq h \leq \mathsf{len}(c),\,h \neq k} (h-q_c)$ in all gate
       constraints.
     * iii. For each row $r$ such that $s'$ is enabled at $r$, assign the value $k$ to
       $q_c$ at row $r$.

The above algorithm is implemented in
[halo2_proofs/src/plonk/circuit/compress_selectors.rs](https://github.com/zcash/halo2/blob/main/halo2_proofs/src/plonk/circuit/compress_selectors.rs).
This is used by the `compress_selectors` function of
[halo2_proofs/src/plonk/circuit.rs](https://github.com/zcash/halo2/blob/main/halo2_proofs/src/plonk/circuit.rs)
which does the actual substitutions.

## Writing circuits to take best advantage of selector combining

For this optimization it is beneficial for a circuit to use simple selectors as far as
possible, rather than fixed columns. It is usually not beneficial to do manual combining
of selectors, because the resulting fixed columns cannot take part in the automatic
combining. That means that to get comparable results you would need to do a global
optimization manually, which would interfere with writing composable gadgets.

Whether two selectors are enabled on the same row (and so are inhibited from being
combined) depends on how regions are laid out by the floor planner. The currently
implemented floor planners do not attempt to take this into account. We suggest not
worrying about it too much — the gains that can be obtained by cajoling a floor planner to
shuffle around regions in order to improve combining are likely to be relatively small.
