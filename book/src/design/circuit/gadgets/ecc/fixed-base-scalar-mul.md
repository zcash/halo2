# Fixed-base scalar multiplication

There are $5$ fixed bases in the Orchard protocol:
- $\mathcal{K}^{\mathsf{Orchard}}$, used in deriving the nullifier;
- $\mathcal{R}$ base for $\mathsf{NoteCommit}^{\mathsf{Orchard}}$;
- $\mathcal{V}$ and $\mathcal{R}$ bases for $\mathsf{ValueCommit}^{\mathsf{Orchard}}$; and
- $\mathcal{R}$ base for $\mathsf{Commit}^{\mathsf{ivk}}$.

## Witness scalar
In most cases, we multiply the fixed bases by $255-$bit scalars from $\mathbb{F}_q$. We decompose a full-width scalar $\alpha$ into $85$ $3$-bit windows:

$$\alpha = k_0 + k_1 \cdot (2^3)^1 + \cdots + k_{84} \cdot (2^3)^{84}, k_i \in [0..2^3).$$

## Load fixed base
Then, we precompute multiples of the fixed base $B$ for each window. This takes the form of a window table: $M[0..85)[0..8)$ such that:

- for the first 84 rows $M[0..84)[0..8)$: $$M[w][k] = [(k+1) \cdot (2^3)^w]B$$
- in the last row $M[84][0..8)$: $$M[w][k] = [k \cdot (2^3)^w - \sum\limits_{j=0}^{83} (2^3)^j]B$$

The additional $(k + 1)$ term lets us avoid adding the point at infinity in the case $k = 0$. We offset these accumulated terms by subtracting them in the final window, i.e. we subtract $\sum\limits_{j=0}^{83} (2^3)^j$.

For each window of fixed-base multiples $M[w] = (M[w][0], \cdots, M[w][7]), w \in [0..84)$:
- Define a Lagrange interpolation polynomial $\mathcal{L}_x(k)$ that maps $k \in [0..8)$ to the $x$-coordinate of the multiple $M[w][k]$, i.e.
  $$
  \mathcal{L}_x(k) = \begin{cases}
    ([(k + 1) \cdot 8^w] B)_x &\text{for } w \in [0..84); \\
    ([k \cdot (8)^w - \sum\limits_{j=0}^{83} (8)^j] B)_x &\text{for } w = 84; \text{ and}
  \end{cases}
  $$
- Find a value $z_w$ such that $z_w + (M[w][k])_y$ is a square $u^2$ in the field, but the wrong-sign $y$-coordinate $z_w - (M[w][k])_y$ does not produce a square.

Repeating this for all $85$ windows, we end up with:
- an $85 \times 8$ table $\mathcal{L}_x$ storing $8$ coefficients interpolating the $x-$coordinate for each window. Each $x$-coordinate interpolation polynomial will be of the form
$$\mathcal{L}_x[w](k) = c_0 + c_1 \cdot k + c_2 \cdot k^2 + \cdots + c_7 \cdot k^7,$$
where $k \in [0..8), w \in [0..85)$ and $c_k$'s are the coefficients for each power of $k$; and
- a length-$85$ array $Z$ of $z_w$'s.

We load these precomputed values into fixed columns whenever we do fixed-base scalar multiplication in the circuit.

## Fixed-base scalar multiplication
Given a decomposed scalar $\alpha$ and a fixed base $B$, we compute $[\alpha]B$ as such:

1. For each $k_w, w \in [0..85), k_w \in [0..8)$ in the scalar decomposition, witness the $x$- and $y$-coordinates $(x_w,y_w) = M[w][k_w].$
2. Check that $(x_w, y_w)$ is on the curve: $y_w^2 = x_w^3 + b$.
3. Witness $u_w$ such that $y_w + z_w = u_w^2$.
4. Use [incomplete addition](./incomplete-add.md) to sum the $M[w][k_w]$'s, resulting in $[\alpha]B$.

Constraints:
 - $x_w = \mathcal{L}_x[w](k_w)$;
 - $y_w^2 = x_w^3 + b,$ where $b = 5$ (from the Pallas curve equation);
 - $u_w^2 = y_w + Z[w].$

### Fixed-base scalar multiplication with signed short exponent
This is used for $\mathsf{ValueCommit^{Orchard}}$. We want to compute $\mathsf{ValueCommit^{Orchard}_{rcv}}(\mathsf{v^{old}} - \mathsf{v^{new}}) = [\mathsf{v^{old}} - \mathsf{v^{new}}] \mathcal{V} + [\mathsf{rcv}] \mathcal{R}$, where
$$
-(2^{64}-1) \leq \mathsf{v^{old}} - \mathsf{v^{new}} \leq 2^{64}-1
$$

$\mathsf{v^{old}}$ and $\mathsf{v^{new}}$ are each already constrained to $64$ bits (by their use as inputs to $\mathsf{NoteCommit^{Orchard}}$).

Witness the sign $s$ and magnitude $m$ such that
$$
s \in \{-1, 1\} \\
m \in [0, 2^{64}) \\
\mathsf{v^{old}} - \mathsf{v^{new}} = s \cdot m
$$

Then compute $P = [m] \mathcal{V}$, and conditionally negate $P$ using $(x, y) \mapsto (x, s \cdot y)$.

We can reuse the window table from full-width fixed-base scalar multiplication, but with only $\mathsf{ceil}(64 / 3) = 22$ windows.
