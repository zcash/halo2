# Endoscaling

Often in proof systems, it is necessary to multiply a group element by a scalar that depends
on a challenge. Since the challenge is random, what matters is only that the scalar retains
that randomness; that is, it is acceptable to apply a 1-1 mapping to the scalar if that allows
the multiplication to be done more efficiently.

The Pasta curves we use for Halo 2 are equipped with an endomorphism that allows such
efficient multiplication. By allowing a 1-1 mapping as described above, we can avoid having
to "decompose" the input challenge using an algorithm such as
[[Pornin2020]](https://eprint.iacr.org/2020/454) that requires lattice basis reduction.

## Definitions

- The Lagrange basis polynomial $\ell_i(X)$ is such that $\ell_i(\omega^i) = 1$ and
  $\ell_i(\omega^j) = 0$ for $i \neq j$.

- We consider curves over a base field $\mathbb{F}_p$ with a "cubic endomorphism" $\phi$
  defined on $\mathbb{F}_p$-rational points by $\phi((x, y)) = (\zeta_p \cdot x, y)$ for
  $\zeta_p \in \mathbb{F}_p$. This is equivalent to $\phi(P) = [\zeta_q]P$ for some
  $\zeta_q \in \mathbb{F}_q$ of multiplicative order $3$.

## Endoscaling for public inputs

In the Halo 2 proof system, this technique can optionally be used to commit to an instance
column using bits that represent the public input. Each basis polynomial corresponds with a
cell in the column.

## Computing an endoscaling commitment

Let $N$ be the limit on the number of bits that can be input to endoscaling at once while
avoiding collisions. For CM curves that have a cubic endomorphism $\phi$, such as the
Pasta curves, this limit can be computed using the script
[checksumsets.py in zcash/pasta](https://github.com/zcash/pasta/blob/master/checksumsets.py).

Assume that $N$ is even. (For Pasta, $N = 248$.)

Let $\text{Endoscale}$ be Algorithm 1 in the [Halo paper](https://eprint.iacr.org/2019/1021.pdf):
$$
(\mathbf{r}, G) \mapsto [n(\mathbf{r})] G
$$

Given $G_i = \text{Comm}(\ell_i(X))$, we compute an endoscaling instance column commitment by
calculating the sum $P = \sum_{i = 0}^{m - 1} \text{Endoscale}(\mathbf{r}_i, G_i)$.

### Algorithm 1 (optimized)

The input bits to endoscaling are $\mathbf{r}$. Split $\mathbf{r}$ into $m$ chunks
$\mathbf{r}_0, \mathbf{r}_1, ..., \mathbf{r}_{m - 1} \in \{0, 1\}^N$. For now assume that all
the $\mathbf{r}_i$ are the same length.

let $S(i, j) = \begin{cases}
  [2\mathbf{r}_{i,2j} - 1] G_i,\text{ if } \mathbf{r}_{i,2j+1} = 0, \\
  \phi([2\mathbf{r}_{i,2j} - 1] G_i),\text{ otherwise}.
\end{cases}$

$P := [2] \sum_{i=0}^{m-1} (G_i + \phi(G_i))$

for $j$ in $0..N/2$:

$\hspace{2em} \mathrm{Inner} := S(0, j)$

$\hspace{2em}$  for $i$ in $1..m$:

$\hspace{4em}$    $\mathrm{Inner} := \mathrm{Inner} \;⸭\; S(i, j)$

$\hspace{2em}$  $P := (P \;⸭\; \mathrm{Inner}) \;⸭\; P$

which is equivalent to (using complete addition)

$P := \mathcal{O}$

for $i$ in $0..m$:

$\hspace{2em} P := [2] (G_i + \phi(G_i))$

$\hspace{2em}$for $j$ in $0..N/2$:

$\hspace{4em}$  $P := (P + S(i, j)) + P$


#### Circuit cost

Define a running sum $z_{i,j} = \sum\limits_{k=2j}^{t} (\mathbf{b}_{i,k} \cdot 2^{t−k})$

* $z_{i,t/2} = 0$
* $z_{i,t/2-1} = 2\mathbf{b}_{i,t-1} + \mathbf{b}_{i,t-2}$
* $z_{i,0} = b_i$

$z_{i,j} = 4 z_{i,j+1} + 2\mathbf{b}_{i,2j-1} + \mathbf{b}_{i,2j-2}$

$\mathbf{b}_{i,2j-2} = z_{i,j} - 4 z_{i,j+1} - 2\mathbf{b}_{i,2j-1}$

$\begin{array}{rl}
  (0, 0) &\rightarrow (G_x, -G_y) \\
  (0, 1) &\rightarrow (\zeta \cdot G_x, -G_y) \\
  (1, 0) &\rightarrow (G_x, G_y) \\
  (1, 1) &\rightarrow (\zeta \cdot G_x, G_y)
\end{array}$

let $S(i, j) = \begin{cases}
  [2\mathbf{b}_{i,2j} - 1] G_i, &\text{ if } \mathbf{b}_{i,2j+1} = 0 \\
  \phi([2 \cdot \mathbf{b}_{i,2j} - 1] G_i), &\text{ otherwise}
\end{cases}$

$S_x(i, j) = (\mathbf{b}_{i,2j+1} \cdot (\zeta - 1) + 1) \cdot G_{i,x}$
$S_y(i, j) = (2\mathbf{b}_{i,2j} - 1) \cdot G_{i,y}$


Let $r$ be the number of incomplete additions we're doing per row.
For $r = 2$:

$$
\begin{array}{|c|c|c|c|c|c|}
\hline
z_{i,j} & \mathbf{b}_{i,2j+1} & z_{i',j} & \mathbf{b}_{i',2j+1} & P_x & P_y & Inner_x & Inner_y & \lambda_1 & \lambda_2 & \lambda_3 & G_{i,x} & G_{i,y}  & G_{i',x} & G_{i',y}\\\hline
. & . & \\
\hline
\end{array}
$$

$Inner = S_{i,j}\;⸭\;S_{i',j}$
$P_{j+1} = (P_j\;⸭\;Inner)\;⸭\;P_j$

With inner loop optimization: $c = 3 + 6r$
Without inner loop optimization: $c = 8r$

### Algorithm 2

Split $\mathbf{r}$ into $K$-bit chunks $r_{0..=u-1}$.

$\mathsf{Acc} := 2(\zeta + 1)$

for $i$ from $N/K - 1$ down to $0$:

$\hspace{2em}$ look up $s = \mathsf{endoscale\_scalar}(r_i)$

$\hspace{2em}$ $\mathsf{Acc} := 2^{K/2} \cdot \mathsf{Acc} + s$

#### Handling partial chunks

Suppose that $\mathbf{r}$ is not a multiple of $K$ bits. In that case we will have a partial chunk $r_u$ of length $K' < K$ bits.
The unoptimized algorithm for computing the table is:

$(a, b) := (0, 0)$

for $i$ from $K/2 − 1$ down to $0$:

$\hspace{2em}$ let $(\mathbf{c}_i, \mathbf{d}_i) = \begin{cases}
(0, 2\mathbf{r}_{2i} − 1),&\text{if } \mathbf{r}_{2i+1} = 0 \\
(2\mathbf{r}_{2i} − 1, 0),&\text{otherwise}
\end{cases}$

$(a, b) := (2a + \mathbf{c}_i, 2b + \mathbf{d}_i)$

Output $[a \cdot \zeta_q + b]\, P$.

We want to derive the table output for $K'$ when $\mathbf{r} = r_u$ from the table output for $K$.
Pad $r_u$ to $K$ bits on the right (high-order bits) with zeros.

So the effect of running the above algorithm for the padding bits will be:

$(a, b) := (0, 0)$

for $i$ from $0$ up to $(K-K')/2 − 1$:

$\hspace{2em} b := 2b - 1$

(which is equivalent to $(a, b) := (0, 1 - 2^{(K-K')/2})$)

for $i$ from $(K-K')/2$ up to $K/2 − 1$:

$\hspace{2em}$ let $(\mathbf{c}_i, \mathbf{d}_i) = \begin{cases}
(0, 2\mathbf{r}_{2i} − 1),&\text{if } \mathbf{r}_{2i+1} = 0 \\
(2\mathbf{r}_{2i} − 1, 0),&\text{otherwise}
\end{cases}$

$\hspace{2em} (a, b) := (2a + \mathbf{c}_i, 2b + \mathbf{d}_i)$

Output $[a \cdot \zeta_q + b]\, P$.

So now we need to adjust the result of the table lookup to take account that we initialized $(a, b)$ to $(0, 1 - 2^{(K-K')/2})$ instead of $(0, 0)$.

The offset for $b$ will get multiplied by $2^{K'/2}$, which means that we need to subtract $(1 - 2^{(K-K')/2}) \cdot 2^{K'/2} = (2^{K'/2} - 2^{K/2})$.

#### Circuit costs

##### Initial chunk
In the case where the bitstring length is a multiple of $K$, we witness the first
full chunk like so:

$$
\begin{array}{|c|c|c|c|c|}
  \texttt{z}   & \texttt{acc} & \texttt{endoscalars\_copy} & \texttt{q\_init} & \texttt{q\_lookup} \\\hline
     z[u]      &     acc_1    &      \texttt{endo}(r_u)    &         1        &           1        \\\hline
     z[u-1]    &              &                            &         0        &           0        \\\hline
\end{array}
$$

with the following constraints:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_\text{init} \cdot [(\texttt{init\_acc} \cdot 2^{K / 2} + \texttt{endo}(r_u))  - acc_1] = 0 \\\hline
\end{array}
$$

where $\texttt{init\_acc} = 2 \cdot (\zeta + 1)$.
As before, $q_{lookup}$ looks up the tuple $(z[u-1] - z[u] * 2^K, \texttt{endo}(r_u)).$

If the first chunk is a $K'$-bit partial chunk, it has been right-padded with $K - K'$ zeros.
We constrain it in its own region:

$$
\begin{array}{|c|c|c|c|c|}
  \texttt{z}   & \texttt{acc} & \texttt{endoscalars\_copy} & \texttt{q\_partial} & \texttt{q\_lookup} & \texttt{q\_short\_range\_check} \\\hline
     z[u]      &      r_u     &      \texttt{endo}(r_u)    &          1          &           1        &                 1               \\\hline
     z[u-1]    &    acc_1     &          2^{K'/2}          &          0          &           0        &                 0               \\\hline
\end{array}
$$

with the following constraints:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_\text{partial} \cdot [(z[u-1] - z[u] \cdot 2^K) - r_u] = 0 \\\hline
2 & q_\text{partial} \cdot [(\texttt{init\_acc} \cdot 2^{K' / 2} + \texttt{shifted\_endo})  - acc_1] = 0 \\\hline
\end{array}
$$

where $\texttt{init\_acc} = 2 \cdot (\zeta + 1),$ and $\texttt{shifted\_endo} = \texttt{endo}(r_u) - (2^{K'/2} - 2^{K/2})$.

As before, $q_{lookup}$ looks up the tuple $(z[u-1] - z[u] * 2^K, \texttt{endo}(r_u)).$
Additionally, we do a $\texttt{q\_short\_range\_check}(r_u, K')$ to check that $r_u$ is
indeed a $K'$-bit value. (see [Lookup short range check](./decomposition.md#short-range-check).)

##### Steady state
After initializing the first chunk, we proceed with the remaining chunks in the steady state:

$$
\begin{array}{|c|c|c|c|c|}
  \texttt{z}   & \texttt{acc} & \texttt{endoscalars\_copy} & \texttt{q\_endoscale} & \texttt{q\_lookup} \\\hline
     z[i]      &  acc_{u-i+1} &      \texttt{endo}(r_i)    &           1           &           1        \\\hline
     z[i-1]    &  acc_{u-i}   &   \texttt{endo}(r_{i-1})   &           1           &           1        \\\hline
     z[i-2]    &              &                            &           0           &           0        \\\hline

\end{array}
$$

with the following constraints:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_\text{endoscale} \cdot [(acc_{u-i+1} \cdot 2^{K / 2} + \texttt{endo}(r_i))  - acc_{u-i}] = 0 \\\hline
\end{array}
$$

As before, $q_{lookup}$ looks up the tuple $(z[i-1] - z[i] * 2^K, \texttt{endo}(r_i)).$
