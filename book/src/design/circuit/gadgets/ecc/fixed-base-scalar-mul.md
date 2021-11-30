# Fixed-base scalar multiplication

There are $6$ fixed bases in the Orchard protocol:
- $\mathcal{K}^{\mathsf{Orchard}}$, used in deriving the nullifier;
- $\mathcal{G}^{\mathsf{Orchard}}$, used in spend authorization;
- $\mathcal{R}$ base for $\mathsf{NoteCommit}^{\mathsf{Orchard}}$;
- $\mathcal{V}$ and $\mathcal{R}$ bases for $\mathsf{ValueCommit}^{\mathsf{Orchard}}$; and
- $\mathcal{R}$ base for $\mathsf{Commit}^{\mathsf{ivk}}$.
## Decompose scalar
We support fixed-base scalar multiplication with three types of scalars:
### Full-width scalar
A $255$-bit scalar from $\mathbb{F}_q$. We decompose a full-width scalar $\alpha$ into $85$ $3$-bit windows:

$$\alpha = k_0 + k_1 \cdot (2^3)^1 + \cdots + k_{84} \cdot (2^3)^{84}, k_i \in [0..2^3).$$

The scalar multiplication will be computed correctly for $k_{0..84}$ representing any integer in the range $[0, 2^{255})$.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
9 & q_\text{scalar-fixed} \cdot \left(\sum\limits_{i=0}^7{w - i}\right) = 0 \\\hline
\end{array}
$$

We range-constrain each $3$-bit word of the scalar decomposition using a polynomial range-check constraint:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
9 & q_\text{decompose-base-field} \cdot \RangeCheck{\text{word}}{2^3} = 0 \\\hline
\end{array}
$$
where $\RangeCheck{\text{word}}{\texttt{range}} = \text{word} \cdot (1 - \text{word}) \cdots (\texttt{range} - 1 - \text{word}).$

### Base field element
We support using a base field element as the scalar in fixed-base multiplication. This occurs, for example, in the scalar multiplication for the nullifier computation of the Action circuit $\mathsf{DeriveNullifier_{nk}} = \mathsf{Extract}_\mathbb{P}\left(\left[(\mathsf{PRF_{nk}^{nfOrchard}}(\rho) + \psi) \bmod{q_\mathbb{P}}\right]\mathcal{K}^\mathsf{Orchard} + \mathsf{cm}\right)$: here, the scalar $$\left[(\mathsf{PRF_{nk}^{nfOrchard}}(\rho) + \psi) \bmod{q_\mathbb{P}}\right]$$ is the result of a base field addition.

Decompose the base field element $\alpha$ into three-bit windows, and range-constrain each window, using the [short range decomposition](../decomposition.md#short-range-decomposition) gadget in strict mode, with $W = 85, K = 3.$

If $k_{0..84}$ is witnessed directly then no issue of canonicity arises. However, because the scalar is given as a base field element here, care must be taken to ensure a canonical representation, since $2^{255} > p$. That is, we must check that $0 \leq \alpha < p,$ where $p$ the is Pallas base field modulus $$p = 2^{254} + t_p = 2^{254} + 45560315531419706090280762371685220353.$$ Note that $t_p < 2^{130}.$

To do this, we decompose $\alpha$ into three pieces: $$\alpha = \alpha_0 \text{ (252 bits) } \,||\, \alpha_1 \text{ (2 bits) } \,||\, \alpha_2 \text{ (1 bit) }.$$

We check the correctness of this decomposition by:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
5 & q_\text{canon-base-field} \cdot \RangeCheck{\alpha_1}{2^2} = 0 \\\hline
3 & q_\text{canon-base-field} \cdot \RangeCheck{\alpha_2}{2^1} = 0 \\\hline
2 & q_\text{canon-base-field} \cdot \left(z_{84} - (\alpha_1 + \alpha_2 \cdot 2^2)\right) = 0 \\\hline
\end{array}
$$
If the MSB $\alpha_2 = 0$ is not set, then $\alpha < 2^{254} < p.$ However, in the case where $\alpha_2 = 1$, we must check:
- $\alpha_2 = 1 \implies \alpha_1 = 0;$
- $\alpha_2 = 1 \implies \alpha_0 < t_p$:
  - $\alpha_2 = 1 \implies 0 \leq \alpha_0 < 2^{130}$,
  - $\alpha_2 = 1 \implies 0 \leq \alpha_0 + 2^{130} - t_p < 2^{130}$

To check that $0 \leq \alpha_0 < 2^{130},$ we make use of the three-bit running sum decomposition:
- Firstly, we constrain $\alpha_0$ to be a $132$-bit value by enforcing its high $120$ bits to be all-zero. We can get $\textsf{alpha\_0\_hi\_120}$ from the decomposition:
$$
\begin{aligned}
z_{44} &= k_{44} + 2^3 k_{45} + \cdots + 2^{3 \cdot (84 - 44)} k_{84}\\
\implies \textsf{alpha\_0\_hi\_120} &= z_{44} - 2^{3 \cdot (84 - 44)} k_{84}\\
&= z_{44} - 2^{3 \cdot (40)} z_{84}.
\end{aligned}
$$
- Then, we constrain bits $130..\!\!=\!\!131$ of $\alpha_0$ to be zeroes; in other words, we constrain the three-bit word $k_{43} = \alpha[129..\!\!=\!\!131] = \alpha_0[129..\!\!=\!\!131] \in \{0, 1\}.$ We make use of the running sum decomposition to obtain $k_{43} = z_{43} - z_{44} \cdot 2^3.$

Define $\alpha'_0 = \alpha_0 + 2^{130} - t_p$. To check that $0 \leq \alpha'_0 < 2^{130},$ we use 13 ten-bit [lookups](../decomposition.md#lookup-decomposition), where we constrain the $z_{13}$ running sum output of the lookup to be $0$ if $\alpha_2 = 1.$
$$
\begin{array}{|c|l|l|}
\hline
\text{Degree} & \text{Constraint} & \text{Comment} \\\hline
3 & q_\text{canon-base-field} \cdot \alpha_2 \cdot \alpha_1 = 0 & \alpha_2 = 1 \implies \alpha_1 = 0 \\\hline
3 & q_\text{canon-base-field} \cdot \alpha_2 \cdot \textsf{alpha\_0\_hi\_120} = 0 & \text{Constrain $\alpha_0$ to be a $132$-bit value} \\\hline
4 & q_\text{canon-base-field} \cdot \alpha_2 \cdot k_{43} \cdot (1 - k_{43}) = 0 & \text{Constrain $\alpha_0[130..\!\!=\!\!131]$ to $0$}  \\\hline
3 & q_\text{canon-base-field} \cdot \alpha_2 \cdot z_{13}(\texttt{lookup}(\alpha_0', 13)) = 0 & \alpha_2 = 1 \implies 0 \leq \alpha'_0 < 2^{130}\\\hline
\end{array}
$$
### Short signed scalar
A short signed scalar is witnessed as a magnitude $m$ and sign $s$ such that
$$
s \in \{-1, 1\} \\
m \in [0, 2^{64}) \\
\mathsf{v^{old}} - \mathsf{v^{new}} = s \cdot m.
$$

This is used for $\mathsf{ValueCommit^{Orchard}}$. We want to compute $\mathsf{ValueCommit^{Orchard}_{rcv}}(\mathsf{v^{old}} - \mathsf{v^{new}}) = [\mathsf{v^{old}} - \mathsf{v^{new}}] \mathcal{V} + [\mathsf{rcv}] \mathcal{R}$, where
$$
-(2^{64}-1) \leq \mathsf{v^{old}} - \mathsf{v^{new}} \leq 2^{64}-1
$$

$\mathsf{v^{old}}$ and $\mathsf{v^{new}}$ are each already constrained to $64$ bits (by their use as inputs to $\mathsf{NoteCommit^{Orchard}}$).

Decompose the magnitude $m$ into three-bit windows, and range-constrain each window, using the [short range decomposition](../decomposition.md#short-range-decomposition) gadget in strict mode, with $W = 22, K = 3.$

We have two additional constraints:
$$
\begin{array}{|c|l|l|}
\hline
\text{Degree} & \text{Constraint} & \text{Comment} \\\hline
3 & q_\text{scalar-fixed-short} \cdot \BoolCheck{k_{21}} = 0 & \text{The last window must be a single bit.}\\\hline
3 & q_\text{scalar-fixed-short} \cdot \left(s^2 - 1\right) = 0  &\text{The sign must be $1$ or $-1$.}\\\hline
\end{array}
$$
where $\BoolCheck{x} = x \cdot (1 - x)$.

## Load fixed base
Then, we precompute multiples of the fixed base $B$ for each window. This takes the form of a window table: $M[0..W)[0..8)$ such that:

- for the first (W-1) rows $M[0..(W-1))[0..8)$: $$M[w][k] = [(k+2) \cdot (2^3)^w]B$$
- in the last row $M[W-1][0..8)$: $$M[w][k] = [k \cdot (2^3)^w - \sum\limits_{j=0}^{83} 2^{3j+1}]B$$

The additional $(k + 2)$ term lets us avoid adding the point at infinity in the case $k = 0$. We offset these accumulated terms by subtracting them in the final window, i.e. we subtract $\sum\limits_{j=0}^{W-2} 2^{3j+1}$.

> Note: Although an offset of $(k + 1)$ would naively suffice, it introduces an edge case when $k_0 = 7, k_1= 0$.
> In this case, the window table entries evaluate to the same point:
> * $M[0][k_0] = [(7+1)*(2^3)^0]B = [8]B,$
> * $M[1][k_1] = [(0+1)*(2^3)^1]B = [8]B.$
>
> In fixed-base scalar multiplication, we sum the multiples of $B$ at each window (except the last) using incomplete addition.
> Since the point doubling case is not handled by incomplete addition, we avoid it by using an offset of $(k+2).$

For each window of fixed-base multiples $M[w] = (M[w][0], \cdots, M[w][7]), w \in [0..(W-1))$:
- Define a Lagrange interpolation polynomial $\mathcal{L}_x(k)$ that maps $k \in [0..8)$ to the $x$-coordinate of the multiple $M[w][k]$, i.e.
  $$
  \mathcal{L}_x(k) = \begin{cases}
    ([(k + 2) \cdot (2^3)^w] B)_x &\text{for } w \in [0..(W-1)); \\
    ([k \cdot (2^3)^w - \sum\limits_{j=0}^{83} 2^{3j+1}] B)_x &\text{for } w = 84; \text{ and}
  \end{cases}
  $$
- Find a value $z_w$ such that $z_w + (M[w][k])_y$ is a square $u^2$ in the field, but the wrong-sign $y$-coordinate $z_w - (M[w][k])_y$ does not produce a square.

Repeating this for all $W$ windows, we end up with:
- an $W \times 8$ table $\mathcal{L}_x$ storing $8$ coefficients interpolating the $x-$coordinate for each window. Each $x$-coordinate interpolation polynomial will be of the form
$$\mathcal{L}_x[w](k) = c_0 + c_1 \cdot k + c_2 \cdot k^2 + \cdots + c_7 \cdot k^7,$$
where $k \in [0..8), w \in [0..85)$ and $c_k$'s are the coefficients for each power of $k$; and
- a length-$W$ array $Z$ of $z_w$'s.

We load these precomputed values into fixed columns whenever we do fixed-base scalar multiplication in the circuit.

## Fixed-base scalar multiplication
Given a decomposed scalar $\alpha$ and a fixed base $B$, we compute $[\alpha]B$ as follows:

1. For each $k_w, w \in [0..85), k_w \in [0..8)$ in the scalar decomposition, witness the $x$- and $y$-coordinates $(x_w,y_w) = M[w][k_w].$
2. Check that $(x_w, y_w)$ is on the curve: $y_w^2 = x_w^3 + b$.
3. Witness $u_w$ such that $y_w + z_w = u_w^2$.
4. For all windows but the last, use [incomplete addition](./incomplete-add.md) to sum the $M[w][k_w]$'s, resulting in $[\alpha - k_{84} \cdot (2^3)^{84} + \sum\limits_{j=0}^{83} 2^{3j+1}]B$.
5. For the last window, use complete addition $M[83][k_{83}] + M[84][k_{84}]$ and return the final result.

> Note: complete addition is required in the final step to correctly map $[0]B$ to a representation of the point at infinity, $(0,0)$; and also to handle a corner case for which the last step is a doubling.

Constraints:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
8 & q_\text{mul-fixed} \cdot \left( \mathcal{L}_x[w](k_w) - x_w \right) = 0 \\\hline
4 & q_\text{mul-fixed} \cdot \left( y_w^2 - x_w^3 - b \right) = 0 \\\hline
3 & q_\text{mul-fixed} \cdot \left( u_w^2 - y_w - Z[w] \right) = 0 \\\hline
\end{array}
$$

where $b = 5$ (from the Pallas curve equation).

### Signed short exponent
Recall that the signed short exponent is witnessed as a $64-$bit magnitude $m$, and a sign $s \in {1, -1}.$ Using the above algorithm, we compute $P = [m] \mathcal{B}$. Then, to get the final result $P',$ we conditionally negate $P$ using $(x, y) \mapsto (x, s \cdot y)$.

Constraints:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_\text{mul-fixed-short} \cdot \left(s \cdot P_y - P'_y\right) = 0 \\\hline
\end{array}
$$

## Layout

$$
\begin{array}{|c|c|c|c|c|c|c|c|}
\hline
  x_P   &   y_P   &      x_{QR}       &        y_{QR}      &    u   & \text{window}   & L_{0..=7}   & \textsf{fixed\_z}   \\\hline
x_{P,0} & y_{P,0} &                   &                    &   u_0  & \text{window}_0 & L_{0..=7,0} & \textsf{fixed\_z}_0 \\\hline
x_{P,1} & y_{P,1} & x_{Q,1} = x_{P,0} & y_{Q,1} = y_{P,0}  &   u_1  & \text{window}_1 & L_{0..=7,1} & \textsf{fixed\_z}_1 \\\hline
x_{P,2} & y_{P,2} & x_{Q,2} = x_{R,1} & y_{Q,2} = y_{R,1}  &   u_2  & \text{window}_2 & L_{0..=7,1} & \textsf{fixed\_z}_2 \\\hline
\vdots  & \vdots  &      \vdots       &       \vdots       & \vdots &     \vdots      &    \vdots   &        \vdots       \\\hline
\end{array}
$$

Note: this doesn't include the last row that uses [complete addition](./addition.md#Complete-addition). In the implementation this is allocated in a different region.
