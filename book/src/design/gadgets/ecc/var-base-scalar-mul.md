# Variable-base scalar multiplication

In the Orchard circuit we need to check $\mathsf{pk_d} = [\mathsf{ivk}] \mathsf{g_d}$ where $\mathsf{ivk} \in [0, p)$ and the scalar field is $\mathbb{F}_q$ with $p < q$.

We have $p = 2^{254} + t_p$ and $q = 2^{254} + t_q$, for $t_p, t_q < 2^{128}$.

## Witness scalar
We're trying to compute $[\alpha] T$ for $\alpha \in [0, q)$. Set $k = \alpha + t_q$ and $n = 254$. Then we can compute

$\begin{array}{cl}
[2^{254} + (\alpha + t_q)] T &= [2^{254} + (\alpha + t_q) - (2^{254} + t_q)] T \\
                             &= [\alpha] T
\end{array}$

provided that $\alpha + t_q \in [0, 2^{n+1})$, i.e. $\alpha < 2^{n+1} - t_q$ which covers the whole range we need because in fact $2^{255} - t_q > q$.

Thus, given a scalar $\alpha$, we witness the boolean decomposition of $k = \alpha + t_q.$ (We use big-endian bit order for convenient input into the variable-base scalar multiplication algorithm.)

$$k = k_{254} \cdot 2^{254} + k_{253} \cdot 2^{253} + \cdots + k_0.$$

## Variable-base scalar multiplication
We use an optimized double-and-add algorithm, copied from ["Faster variable-base scalar multiplication in zk-SNARK circuits"](https://github.com/zcash/zcash/issues/3924) with some variable name changes:

```ignore
Acc := [2] T
for i from n-1 down to 0 {
    P := k_{i+1} ? T : −T
    Acc := (Acc + P) + Acc
}
return (k_0 = 0) ? (Acc - T) : Acc
```

It remains to check that the x-coordinates of each pair of points to be added are distinct.

When adding points in a prime-order group, we can rely on Theorem 3 from Appendix C of the [Halo paper](https://eprint.iacr.org/2019/1021.pdf), which says that if we have two such points with nonzero indices wrt a given odd-prime order base, where the indices taken in the range $-(q-1)/2..(q-1)/2$ are distinct disregarding sign, then they have different x-coordinates. This is helpful, because it is easier to reason about the indices of points occurring in the scalar multiplication algorithm than it is to reason about their x-coordinates directly.

So, the required check is equivalent to saying that the following "indexed version" of the above algorithm never asserts:

```ignore
acc := 2
for i from n-1 down to 0 {
    p = k_{i+1} ? 1 : −1
    assert acc ≠ ± p
    assert (acc + p) ≠ acc    // X
    acc := (acc + p) + acc
    assert 0 < acc ≤ (q-1)/2
}
if k_0 = 0 {
    assert acc ≠ 1
    acc := acc - 1
}
```

The maximum value of `acc` is:
```ignore
    <--- n 1s --->
  1011111...111111
= 1100000...000000 - 1
```
= $2^{n+1} + 2^n - 1$

> The assertion labelled X obviously cannot fail because $p \neq 0$. It is possible to see that acc is monotonically increasing except in the last conditional. It reaches its largest value when $k$ is maximal, i.e. $2^{n+1} + 2^n - 1$.

So to entirely avoid exceptional cases, we would need $2^{n+1} + 2^n - 1 < (q-1)/2$. But we can use $n$ larger by $c$ if the last $c$ iterations use [complete addition](./addition.md#Complete-addition).

The first $i$ for which the algorithm using **only** incomplete addition fails is going to be $252$, since $2^{252+1} + 2^{252} - 1 > (q - 1)/2$. We need $n = 254$ to make the wraparound technique above work.

```python
sage: q = 0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001
sage: 2^253 + 2^252 - 1 < (q-1)//2
False
sage: 2^252 + 2^251 - 1 < (q-1)//2
True
```

So the last three iterations of the loop ($i = 2..0$) need to use [complete addition](./addition.md#Complete-addition), as does the conditional subtraction at the end. Writing this out using ⸭ for incomplete addition (as we do in the spec), we have:

```ignore
Acc := [2] T
for i from 253 down to 3 {
    P := k_{i+1} ? T : −T
    Acc := (Acc ⸭ P) ⸭ Acc
}
for i from 2 down to 0 {
    P := k_{i+1} ? T : −T
    Acc := (Acc + P) + Acc  // complete addition
}
return (k_0 = 0) ? (Acc + (-T)) : Acc  // complete addition
```

## Constraint program for optimized double-and-add
Define a running sum $\mathbf{z_j} = \sum_{i=j}^{n} (\mathbf{k}_{i} \cdot 2^{i-j})$, where $n = 254$ and:

$$
\begin{aligned}
    &\mathbf{z}_{n+1} = 0,\\
    &\mathbf{z}_{n} = \mathbf{k}_{n}, \hspace{2em}\text{(most significant bit)}\\
	&\mathbf{z}_0 = k.\\
\end{aligned}
$$

$\begin{array}{l}
\text{Initialize } A_{254} = [2] T. \\
\\
\text{for } i \text{ from } 254 \text{ down to } 4: \\
\hspace{1.5em} \BoolCheck{\mathbf{k}_i} = 0 \\
\hspace{1.5em} \mathbf{z}_{i} = 2\mathbf{z}_{i+1} + \mathbf{k}_{i} \\
\hspace{1.5em} x_{P,i} = x_T \\
\hspace{1.5em} y_{P,i} = (2 \mathbf{k}_i - 1) \cdot y_T  \hspace{2em}\text{(conditionally negate)} \\
\hspace{1.5em} \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) = y_{A,i} - y_{P,i} \\
\hspace{1.5em} x_{R,i} = \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
\hspace{1.5em} (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) = 2 y_{\mathsf{A},i} \\
\hspace{1.5em} \lambda_{2,i}^2 = x_{A,i-1} + x_{R,i} + x_{A,i} \\
\hspace{1.5em} \lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) = y_{A,i} + y_{A,i-1}. \\
\end{array}$

The helper $\BoolCheck{x} = x \cdot (1 - x)$.
After substitution of $x_{P,i}, y_{P,i}, x_{R,i}, y_{A,i}$, and $y_{A,i-1}$, this becomes:

$\begin{array}{l}
\text{Initialize } A_{254} = [2] T. \\
\\
\text{for } i \text{ from } 254 \text{ down to } 4: \\
\hspace{1.5em} \text{// let } \mathbf{k}_{i} = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\
\hspace{1.5em} \text{// let } y_{A,i} = \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2} \\[2ex]
\hspace{1.5em} \BoolCheck{\mathbf{k}_i} = 0 \\
\hspace{1.5em} \lambda_{1,i} \cdot (x_{A,i} - x_T) = y_{A,i} - (2 \mathbf{k}_i - 1) \cdot y_T \\
\hspace{1.5em} \lambda_{2,i}^2 = x_{A,i-1} + \lambda_{1,i}^2 - x_T \\[1ex]
\hspace{1.5em} \begin{cases}
                 \lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) = y_{A,i} + y_{A, i-1}, &\text{if } i > 4 \\[0.5ex]
                 \lambda_{2,4} \cdot (x_{A,4} - x_{A,3}) = y_{A,4} + y_{A,3}^\text{witnessed}, &\text{if } i = 4.
               \end{cases}
\end{array}$

Here, $y_{A,3}^\text{witnessed}$ is assigned to a cell. This is unlike previous $y_{A,i}$'s, which were implicitly derived from $\lambda_{1,i}, \lambda_{2,i}, x_{A,i}, x_T$, but never actually assigned.

The bits $\mathbf{k}_{3 \dots 1}$ are used in three further steps, using [complete addition](./addition.md#Complete-addition):

$\begin{array}{l}
\text{for } i \text{ from } 3 \text{ down to } 1: \\
\hspace{1.5em} \text{// let } \mathbf{k}_{i} = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\[0.5ex]
\hspace{1.5em} \BoolCheck{\mathbf{k}_i} = 0 \\
\hspace{1.5em} (x_{A,i-1}, y_{A,i-1}) = \left((x_{A,i}, y_{A,i}) + (x_T, y_T)\right) + (x_{A,i}, y_{A,i})
\end{array}$

If the least significant bit $\mathbf{k_0} = 1,$ we set $B = \mathcal{O},$ otherwise we set ${B = -T}$. Then we return ${A + B}$ using complete addition.

Let $B = \begin{cases}
(0, 0), &\text{ if } \mathbf{k_0} = 1, \\
(x_T, -y_T), &\text{ otherwise.}
\end{cases}$

Output $(x_{A,0}, y_{A,0}) + B$.

(Note that $(0, 0)$ represents $\mathcal{O}$.)

## Incomplete addition

We need six advice columns to witness $(x_T, y_T, \lambda_1, \lambda_2, x_{A,i}, \mathbf{z}_i)$. However, since $(x_T, y_T)$ are the same, we can perform two incomplete additions in a single row, reusing the same $(x_T, y_T)$. We split the scalar bits used in incomplete addition into $hi$ and $lo$ halves and process them in parallel. This means that we effectively have two for loops:
- the first, covering the $hi$ half for $i$ from $254$ down to $130$, with a special case at $i = 130$; and
- the second, covering the $lo$ half for the remaining $i$ from $129$ down to $4$, with a special case at $i = 4$.

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
    x_T     &    y_T      &          z^{hi}           &    x_A^{hi}        &  \lambda_1^{hi}  &  \lambda_2^{hi}  &  q_1^{hi}  &  q_2^{hi}   &  q_3^{hi}  &         z^{lo}        &  x_A^{lo}   &  \lambda_1^{lo}     &  \lambda_2^{lo}   &  q_1^{lo}  &  q_2^{lo}   &  q_3^{lo}  \\\hline
            &             &  \mathbf{z}_{255} = 0     &                    & y_{A,254}=2[T]_y &                  &     1      &     0       &     0      &   \mathbf{z}_{130}    &             &   y_{A,129}         &                   &     1      &     0       &     0      \\\hline
    x_T     &    y_T      &    \mathbf{z}_{254}       & x_{A,254} = 2[T]_x & \lambda_{1,254}  & \lambda_{2,254}  &     0      &     1       &     0      &   \mathbf{z}_{129}    & x_{A,129}   & \lambda_{1,129}     & \lambda_{2,129}   &     0      &     1       &     0      \\\hline
    x_T     &    y_T      &    \mathbf{z}_{253}       &     x_{A,253}      & \lambda_{1,253}  & \lambda_{2,253}  &     0      &     1       &     0      &   \mathbf{z}_{128}    & x_{A,128}   & \lambda_{1,128}     & \lambda_{2,128}   &     0      &     1       &     0      \\\hline
   \vdots   &   \vdots    &         \vdots            &      \vdots        &      \vdots      &      \vdots      &   \vdots   &   \vdots    &   \vdots   &        \vdots         &  \vdots     &      \vdots         &      \vdots       &   \vdots   &   \vdots    &   \vdots   \\\hline
    x_T     &    y_T      &    \mathbf{z}_{130}       &     x_{A,130}      & \lambda_{1,130}  & \lambda_{2,130}  &     0      &     0       &     1      &   \mathbf{z}_5        & x_{A,5}     & \lambda_{1,5}       & \lambda_{2,5}     &     0      &     1       &     0      \\\hline
    x_T     &    y_T      &                           &     x_{A,129}      &    y_{A,129}     &                  &            &             &            &   \mathbf{z}_4        & x_{A,4}     & \lambda_{1,4}       & \lambda_{2,4}     &     0      &     0       &     1      \\\hline
            &             &                           &                    &                  &                  &            &             &            &                       & x_{A,3}     &     y_{A,3}         &                   &            &             &            \\\hline

\end{array}
$$

For each $hi$ and $lo$ half, we have three sets of gates. Note that $i$ is going from $255..=3$; $i$ is NOT indexing the rows.

### $q_1 = 1$ <a name="incomplete-first-row-gate">
This gate is only used on the first row (before the for loop). We check that $\lambda_1, \lambda_2$ are initialized to values consistent with the initial $y_A.$
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
4 & q_1 \cdot \left(y_{A,n}^\text{witnessed} - y_{A,n}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
y_{A,n} &= \frac{(\lambda_{1,n} + \lambda_{2,n}) \cdot (x_{A,n} - (\lambda_{1,n}^2 - x_{A,n} - x_T))}{2},\\
y_{A,n}^\text{witnessed} &\text{ is witnessed.}
\end{aligned}
$$

### $q_2 = 1$ <a name="incomplete-main-loop-gate">
This gate is used on all rows corresponding to the for loop except the last.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_2 \cdot \left(x_{T,cur} - x_{T,next}\right) = 0 \\\hline
2 & q_2 \cdot \left(y_{T,cur} - y_{T,next}\right) = 0 \\\hline
3 & q_2 \cdot \BoolCheck{\mathbf{k}_i} = 0, \text{ where } \mathbf{k}_i = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\\hline
4 & q_2 \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{T,i}) - y_{A,i} + (2\mathbf{k}_i - 1) \cdot y_{T,i}\right) = 0 \\\hline
3 & q_2 \cdot \left(\lambda_{2,i}^2 - x_{A,i-1} - x_{R,i} - x_{A,i}\right) = 0 \\\hline
3 & q_2 \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) - y_{A,i} - y_{A,i-1}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
x_{R,i} &= \lambda_{1,i}^2 - x_{A,i} - x_T, \\
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2}, \\
y_{A,i-1} &= \frac{(\lambda_{1,i-1} + \lambda_{2,i-1}) \cdot (x_{A,i-1} - (\lambda_{1,i-1}^2 - x_{A,i-1} - x_T))}{2}, \\
\end{aligned}
$$

### $q_3 = 1$ <a name="incomplete-last-row-gate">
This gate is used on the final iteration of the for loop, handling the special case where we check that the output $y_A$ has been witnessed correctly.
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_3 \cdot \BoolCheck{\mathbf{k}_i} = 0, \text{ where } \mathbf{k}_i = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\\hline
4 & q_3 \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{T,i}) - y_{A,i} + (2\mathbf{k}_i - 1) \cdot y_{T,i}\right) = 0 \\\hline
3 & q_3 \cdot \left(\lambda_{2,i}^2 - x_{A,i-1} - x_{R,i} - x_{A,i}\right) = 0 \\\hline
3 & q_3 \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) - y_{A,i} - y_{A,i-1}^\text{witnessed}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
x_{R,i} &= \lambda_{1,i}^2 - x_{A,i} - x_T, \\
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2},\\
y_{A,i-1}^\text{witnessed} &\text{ is witnessed.}
\end{aligned}
$$

## Complete addition

We reuse the [complete addition](addition.md#complete-addition) constraints to implement
the final $c$ rounds of double-and-add. This requires two rows per round because we need
9 advice columns for each complete addition. In the 10th advice column we stash the other
cells that we need to correctly implement the double-and-add:

- The base $y$ coordinate, so we can conditionally negate it as input to one of the
  complete additions.
- The running sum, which we constrain over two rows instead of sequentially.

### Layout

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
a_0 & a_1 & a_2 & a_3 &    a_4    &   a_5    &   a_6   &   a_7    &   a_8    & a_9     & q_\texttt{mul\_decompose\_var} \\\hline
x_T & y_p & x_A & y_A & \lambda_1 & \alpha_1 & \beta_1 & \gamma_1 & \delta_1 & z_{i+1} & 0 \\\hline
x_A & y_A & x_q & y_q & \lambda_2 & \alpha_2 & \beta_2 & \gamma_2 & \delta_2 & y_T     & 1 \\\hline
    &     & x_r & y_r &           &          &         &          &          & z_i     & 0 \\\hline
\end{array}
$$

### Constraints <a name="complete-gate">

In addition to the complete addition constraints, we define the following gate:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
  & q_\texttt{mul\_decompose\_var} \cdot \BoolCheck{\mathbf{k}_i} = 0 \\\hline
  & q_\texttt{mul\_decompose\_var} \cdot \Ternary{\mathbf{k}_i}{y_T - y_p}{y_T + y_p} = 0 \\\hline
\end{array}
$$
where $\mathbf{k}_i = \mathbf{z}_{i} - 2\mathbf{z}_{i+1}$.

## LSB

### Layout

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
a_0 & a_1 & a_2 & a_3 &   a_4   &  a_5   &  a_6  &  a_7   &  a_8   & a_9 & q_\texttt{mul\_lsb} \\\hline
x_p & y_p & x_A & y_A & \lambda & \alpha & \beta & \gamma & \delta & z_1 & 1 \\\hline
x_T & y_T & x_r & y_r &         &        &       &        &        & z_0 & 0 \\\hline
\end{array}
$$

### Constraints <a name="lsb-gate">

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
  & q_\texttt{mul\_lsb} \cdot \BoolCheck{\mathbf{k}_0} = 0 \\\hline
  & q_\texttt{mul\_lsb} \cdot \Ternary{\mathbf{k}_0}{x_p}{x_p - x_T} = 0 \\\hline
  & q_\texttt{mul\_lsb} \cdot \Ternary{\mathbf{k}_0}{y_p}{y_p + y_T} = 0 \\\hline
\end{array}
$$
where $\mathbf{k}_0 = \mathbf{z}_0 - 2\mathbf{z}_1$.

# Overflow check
Recall that we defined $\mathbf{z}_j = \sum_{i=j}^n (\mathbf{k}_i \cdot 2^{i−j})$, where $n = 254$.

$\mathbf{z}_j$ cannot overflow for any $j \geq 1$, because it is a weighted sum of bits only up to $2^{n-j}$, which can be at most $2^{254} - 1$ when $j = 1$; this is smaller than $p$ (and also $q$).

However, for full-width scalar mul, it may not be possible to represent $\mathbf{z}_0$ in the base field (e.g. when the base field is Pasta's $\mathbb{F}_p$ and $p < q$). In that case $\mathbf{z}_0 = \alpha + t_q$ *can* overflow $[0, p)$.

So, we need to special-case the row that would mention $\mathbf{z}_0$ so that it is correct for whatever representation we use for a full-width scalar.

Our representation for $k$ will be the pair $(\mathbf{k}_{254}, k' = k - 2^{254} \cdot \mathbf{k}_{254})$. We'll use $k'$ in place of $\alpha + t_q$ for $\mathbf{z}_0$, constraining $k'$ to 254 bits so that it fits in an $\mathbb{F}_p$ element.

Then we just have to generalize the [overflow check used for variable-base scalar mul in the Orchard circuit](https://zcash.github.io/halo2/design/gadgets/ecc/var-base-scalar-mul.html#overflow-check) to work for $k' \in [0, 2 \cdot t_q)$ (because the maximum value of $\alpha + t_q$ is $q - 1 + t_q = 2^{254} + t_q - 1 + t_q$).


> Note: the bits $\mathbf{k}_{254..0}$ do not represent a value reduced modulo $q$, but rather a representation of the unreduced $\alpha + t_q$.

Overflow can only occur in the final step that constrains $\mathbf{z}_0 = 2 \cdot \mathbf{z}_1 + \mathbf{k}_0$, and only if $\mathbf{z}_1$ has the bit with weight $2^{253}$ set (i.e. if $\mathbf{k}_{254} = 1$). If we instead set $\mathbf{z}_0 = 2 \cdot \mathbf{z}_1 - 2^{254} \cdot \mathbf{k}_{254} + \mathbf{k}_0$, now $\mathbf{z}_0$ cannot overflow and should be equal to $k'$.

It is then sufficient to also check that $\mathbf{z}_0 + 2^{254} \cdot \mathbf{k}_{254} = \alpha + t_q$ as an integer where $\alpha \in [0, q)$.

Represent $\alpha$ as $2^{254} \cdot \boldsymbol\alpha_{254} + 2^{253} \cdot \boldsymbol\alpha_{253} + \alpha''$ where we constrain $\alpha'' \in [0, 2^{253})$ and $\boldsymbol\alpha_{253}$ and $\boldsymbol\alpha_{254}$ to boolean. For this to be a canonical representation we also need $\boldsymbol \alpha_{254} = 1 \implies (\boldsymbol \alpha_{253} = 0 \;\wedge\; \alpha'' \in [0, t_q))$.

Let $\alpha' = 2^{253} \cdot \boldsymbol\alpha_{253} + \alpha''$.

If $\boldsymbol\alpha_{254} = 1$:
* constrain $\mathbf{k}_{254} = 1$ and $\mathbf{z}_0 = \alpha' + t_q$. This cannot overflow because in this case $\alpha' \in [0, t_q)$ and so $\mathbf{z}_0 \in [0, 2 \cdot t_q)$.

If $\boldsymbol\alpha_{254} = 0$:
* we should have $\mathbf{k}_{254} = 1$ iff $\alpha' \in [2^{254} - t_q, 2^{254})$, i.e. witness $\mathbf{k}_{254}$ as boolean and then
  * If $\mathbf{k}_{254} = 0$ then constrain $\alpha' \not\in [2^{254} - t_q, 2^{254})$.
    * This can be done by constraining either $\boldsymbol\alpha_{253} = 0$ or $\alpha'' + t_q \in [0, 2^{253})$. ($\alpha'' + t_q$ cannot overflow.)
  * If $\mathbf{k}_{254} = 1$ then constrain $\alpha' \in [2^{254} - t_q, 2^{254})$.
    * This can be done by constraining $\alpha' - (2^{254} - t_q) \in [0, 2^{130})$ and $\alpha' - 2^{254} + 2^{130} \in [0, 2^{130})$.

### Overflow check constraints

Represent $\alpha$ as $2^{254} \cdot \boldsymbol\alpha_{254} + 2^{253} \cdot \boldsymbol\alpha_{253} + \alpha''$ as above.

The constraints for the overflow check are:

$$
\begin{aligned}
&\alpha'' \in [0, 2^{253}) \\
&\boldsymbol\alpha_{253} \in \{0,1\} \\
&\boldsymbol\alpha_{254} \in \{0,1\} \\
&\mathbf{k}_{254} \in \{0,1\} \\
\boldsymbol \alpha_{254} = 1 \implies &\boldsymbol \alpha_{253} = 0 \\
\boldsymbol \alpha_{254} = 1 \implies &\alpha'' \in [0, 2^{130}) \;❁ \\
\boldsymbol \alpha_{254} = 1 \implies &\alpha'' + 2^{130} - t_q \in [0, 2^{130}) \;❁ \\
\boldsymbol\alpha_{254} = 1 \implies &\mathbf{k}_{254} = 1 \\
\boldsymbol\alpha_{254} = 1 \implies &\mathbf{z}_0 = \alpha' + t_q \\
\boldsymbol\alpha_{254} = 0 \;\wedge\; \boldsymbol\alpha_{253} = 1 \;\wedge\; \mathbf{k}_{254} = 0 &\implies \alpha'' + t_q \in [0, 2^{253}) \\
\boldsymbol\alpha_{254} = 0 \;\wedge\; \mathbf{k}_{254} = 1 &\implies \alpha' - 2^{254} + t_q \in [0, 2^{130}) \;❁ \\
\boldsymbol\alpha_{254} = 0 \;\wedge\; \mathbf{k}_{254} = 1 &\implies \alpha' - 2^{254} + 2^{130} \in [0, 2^{130}) \;❁ \\
\end{aligned}
$$

Note that the four 130-bit constraints marked $❁$ are in two pairs that occur in disjoint cases. We can therefore combine them into two 130-bit constraints using a new witness variable $u$; the other constraint always being on $u + 2^{130} - t_q$:

$$
\begin{aligned}
\boldsymbol \alpha_{254} = 1 \implies &u = \alpha'' \\
\boldsymbol\alpha_{254} = 0 \;\wedge\; \mathbf{k}_{254} = 1 \implies &u = \alpha' - 2^{254} + t_q \\
&u \in [0, 2^{130}) \\
&u + 2^{130} - t_q \in [0, 2^{130}) \\
\end{aligned}
$$

($u$ is unconstrained and can be witnessed as $0$ in the case $\boldsymbol\alpha_{254} = 0 \;\wedge\; \mathbf{k}_{254} = 0$.)

### Cost

* 25 10-bit and one 3-bit range check, to constrain $\alpha''$ to 253 bits;
* 25 10-bit and one 3-bit range check, to constrain $\alpha'' + t_q$ to 253 bits when $\boldsymbol\alpha_{254} = 0 \;\wedge\; \boldsymbol\alpha_{253} = 1 \;\wedge\; \mathbf{k}_{254} = 0$;
