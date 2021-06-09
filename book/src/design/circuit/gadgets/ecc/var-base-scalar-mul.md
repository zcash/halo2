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

When adding points in the large prime-order subgroup, we can rely on Theorem 3 from Appendix C of the [Halo paper](https://eprint.iacr.org/2019/1021.pdf), which says that if we have two such points with nonzero indices wrt a given odd-prime order base, where the indices taken in the range $-(q-1)/2..(q-1)/2$ are distinct disregarding sign, then they have different x-coordinates. This is helpful, because it is easier to reason about the indices of points occurring in the scalar multiplication algorithm than it is to reason about their x-coordinates directly.

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

> The assertion labelled X obviously cannot fail because $u \neq 0$. It is possible to see that acc is monotonically increasing except in the last conditional. It reaches its largest value when $k$ is maximal, i.e. $2^{n+1} + 2^n - 1$.

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

## Constraint program for optimized double-and-add (incomplete addition)
Define a running sum $\mathbf{z_j} = \sum_{i=j}^{n} (\mathbf{k}_{i} \cdot 2^{i-j})$, where $n = 254$ and:

$$
\begin{aligned}
    &\mathbf{z}_{n+1} = 0,\\
    &\mathbf{z}_{n} = \mathbf{k}_{n}, \hspace{2em}\text{(most significant bit)}\\
	&\mathbf{z}_0 = k.\\
\end{aligned}
$$

Initialize $A_{254} = [2] T$

for $i$ from $254$ down to $4$:
$$
\begin{aligned}
    &(\mathbf{k}_i)(\mathbf{k}_i-1) = 0\\
    &\mathbf{z}_{i} = 2\mathbf{z}_{i+1} + \mathbf{k}_{i}\\
    & x_{U,i} = x_T\\
    & y_{U,i} = (2 \mathbf{k}_i - 1) \cdot y_T  \hspace{2em}\text{(conditionally negate)}\\
    & \lambda_{1,i} \cdot (x_{A,i} - x_{U,i}) = y_{A,i} - y_{U,i}\\
    & \lambda_{1,i}^2 = x_{R,i} + x_{A,i} + x_{U,i}\\
    & (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) = 2 y_{\mathsf{A},i}\\
    & \lambda_{2,i}^2 = x_{A,i-1} + x_{R,i} + x_{A,i}\\
    & \lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) = y_{A,i} + y_{A,i-1}\\

\end{aligned}
$$

After substitution of $y_{P,i}$, $x_{R,i}$, $y_{A,i}$, and $y_{A,i+1}$, this becomes:

Initialize $A_{254} = [2] T$


for $i$ from $254$ down to $4$:

$$
\begin{aligned}
    &\texttt{// let } \mathbf{k}_i = \mathbf{z}_{i+1} - 2\mathbf{z}_i\\
    &\texttt{// let } x_{R,i} = (\lambda_{1,i}^2 - x_{A,i} - x_T)\\
    &\texttt{// let } y_{A,i} = \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2}\\
    &(\mathbf{z}_{i+1} - 2\mathbf{z}_i)(\mathbf{z}_{i+1} - 2\mathbf{z}_i - 1) = 0\\
    &\lambda_{1,i} \cdot (x_{A,i} - x_T) = \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2} - (2 \cdot (\mathbf{z}_{i+1} - 2\mathbf{z}_i) - 1) \cdot y_T\\
    &\lambda_{2,i}^2 = x_{A,i-1} + (\lambda_{1,i}^2 - x_{A,i} - x_T) + x_{A,i}\\
    & \\
    &\texttt{if } i > 3 \texttt{ then } 2 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) =\\
        &\hspace{2em}(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T)) +\\
        &\hspace{2em}(\lambda_{1,i-1} + \lambda_{2,i-1}) \cdot (x_{A,i-1} - (\lambda_{1,i-1}^2 - x_{A,i-1} - x_T))\\
\end{aligned}
$$

$\lambda_{2,3} \cdot (x_{A,3} - x_{A,2}) = \frac{(\lambda_{1,3} + \lambda_{2,3}) \cdot (x_{A,3} - (\lambda_{1,3}^2 - x_{A,3} - x_T))\hspace{2em}}{2} + y_{A,2}$

The bits $\mathbf{k}_{3 \dots 1}$ are used in double-and-add using [complete addition](./addition.md#Complete-addition).

If the least significant bit is set $\mathbf{k_0} = 1,$ we return the accumulator $A$. Else, if $\mathbf{k_0} = 0,$ we return $A - T$ (also using complete addition).

Output $(x_{A,0}, y_{A,0})$

### Circuit design
We need six advice columns to witness $(x_T, y_T, \lambda_1, \lambda_2, x_{A,i}, \mathbf{z}_i)$. However, since $(x_T, y_T)$ are the same, we can perform two incomplete additions in a single row, reusing the same $(x_T, y_T)$. We split the scalar bits used in incomplete addition into $hi$ and $lo$ halves and process them in parallel:

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|}
\hline
    x_T   &  y_T   &          z^{hi}           &    x_A^{hi}   &  \lambda_1^{hi}  &  \lambda_2^{hi}      &         z^{lo}        &  x_A^{lo}   &  \lambda_1^{lo}     &  \lambda_2^{lo}       \\\hline
          &        &           0               &   2[T]_x      &                  &                      &   \mathbf{z}_{129}    & x_{A,129}   &                     &                       \\\hline
    x_T   &  y_T   &    \mathbf{z}_{254}       &   x_{A,254}   & \lambda_{1,254}  & \lambda_{2,254}      &   \mathbf{z}_{128}    & x_{A,128}   & \lambda_{1,128}     & \lambda_{2,128}     \\\hline
    x_T   &  y_T   &    \mathbf{z}_{253}       &   x_{A,253}   & \lambda_{1,253}  & \lambda_{2,253}      &   \mathbf{z}_{127}    & x_{A,127}   & \lambda_{1,127}     & \lambda_{2,127}       \\\hline
   \vdots & \vdots &         \vdots            &    \vdots     &      \vdots      &      \vdots          &        \vdots         &  \vdots     &      \vdots         &      \vdots           \\\hline
    x_T   &  y_T   &    \mathbf{z}_{130}       &   x_{A,130}   & \lambda_{1,130}  & \lambda_{2,130}      &   \mathbf{z}_4        & x_{A,4}     & \lambda_{1,4}       & \lambda_{2,4}         \\\hline
    x_T   &  y_T   &    \mathbf{z}_{129}       &   x_{A,129}   & \lambda_{1,129}  & \lambda_{2,129}      &   \mathbf{z}_3        & x_{A,3}     & \lambda_{1,3}       & \lambda_{2,3}         \\\hline
\end{array}
$$

For each $hi$ and $lo$ half, the following constraints are enforced:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2   & q_{ECC,i} \cdot (\mathbf{z}_{i+1} - 2\mathbf{z}_i - \mathbf{k}_i) = 0 \\\hline
3 & q_{ECC,i} \cdot \mathbf{k}_i \cdot (\mathbf{k}_i - 1) = 0 \\\hline
4 & q_{ECC,i} \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) - y_{A,i} + (2\mathbf{k}_i - 1) \cdot y_{P,i}\right) = 0 \\\hline
4 & q_{ECC,i} \cdot \left((\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i})) - 2 y_{A,i}\right) = 0 \\\hline
3 & q_{ECC,i} \cdot \left(\lambda_{2,i}^2 - x_{A,i+1} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) - x_{A,i}\right) = 0 \\\hline
3   & q_{ECC,i} \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - y_{A,i} - y_{A,i+1}\right) = 0 \\\hline
2   & q_{ECC,i} \cdot \left(x_{P,i} - x_{P,i-1}\right) = 0 \\\hline
2   & q_{ECC,i} \cdot \left(y_{P,i} - y_{P,i-1}\right) = 0 \\\hline
\end{array}
$$
