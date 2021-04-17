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
We use an optimized double-and-add algorithm is (copied from ["Faster variable-base scalar multiplication in zk-SNARK circuits"](https://github.com/zcash/zcash/issues/3924), with some variable name changes):
> 
>     Acc := [2] T
>     for i from n-1 down to 0 {
>         P := k_{i+1} ? T : −T
>         Acc := (Acc + P) + Acc
>     }
>     return (k_0 = 0) ? (Acc - T) : Acc

> It remains to check that the x-coordinates of each pair of points to be added are distinct.
> 
> When adding points in the large prime-order subgroup, we can rely on Theorem 3 from Appendix C of the [Halo paper](https://eprint.iacr.org/2019/1021.pdf), which says that if we have two such points with nonzero indices wrt a given odd-prime order base, where the indices taken in the range $-(q-1)/2..(q-1)/2$ are distinct disregarding sign, then they have different x-coordinates. This is helpful, because it is easier to reason about the indices of points occurring in the scalar multiplication algorithm than it is to reason about their x-coordinates directly.

> So, the required check is equivalent to saying that the following "indexed version" of the above algorithm never asserts:
> 
>     acc := 2
>     for i from n-1 down to 0 {
>         p = k_{i+1} ? 1 : −1
>         assert acc ≠ ± p
>         assert (acc + p) ≠ acc    // X
>         acc := (acc + p) + acc
>         assert 0 < acc ≤ (q-1)/2
>     }
>     if k_0 = 0 {
>         assert acc ≠ 1
>         acc := acc - 1
>     }

The maximum value of $acc$ is:
```
    <--n 1s--->
  1011111111111
= 1100000000000 - 1
```
= $2^{n+1} + 2^n - 1$

> The assertion labelled X obviously cannot fail because $u \neq 0$. It is possible to see that acc is monotonically increasing except in the last conditional. It reaches its largest value when $k$ is maximal, i.e. $2^{n+1} + 2^n - 1$.

So to entirely avoid exceptional cases, we would need $2^{n+1} + 2^n - 1 < (q-1)/2$. But we can use $n$ larger by $c$ if the last $c$ iterations use [complete addition](./complete-add.md).

The first $i$ for which the algorithm using **only** incomplete addition fails is going to be $252$, since $2^{252+1} + 2^{252} - 1 > (q - 1)/2$. We need $n = 254$ to make the wraparound technique above work.

```python
sage: q = 0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001
sage: 2^253 + 2^252 - 1 < (q-1)//2
False
sage: 2^252 + 2^251 - 1 < (q-1)//2
True
```

So the last three iterations of the loop ($i = 2..0$) need to use [complete addition](./complete-add.md), as does the conditional subtraction at the end. Writing this out using ⸭ for incomplete addition (as we do in the spec), we have:

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

## Constraint program for optimized double-and-add
For each round $i$ of incomplete addition, we are computing $A_{i+1} = A_i + P_i + A_i$, where $A = (x_a, y_a)$ is the accumulated sum and $P = (x_p, y_p)$ is the point we are adding.

We compute $\lambda_{1, i}, \lambda_{2, i}$:
- $\lambda_{1, i} = \frac{y_{A,i} - y_{P, i}}{x_{A,i} - x_{P, i}},$
- $\lambda_{2, i} = \frac{y_{A, i} + y_{A, i+1}}{x_{A,i} - x_{A, i+1}}$

and similarly for $\lambda_{1, i+1}, \lambda_{2, i+1}$.

We witness $x_{A,i}, x_{P,i}, x_{A, i+1},$ and $\lambda_{1, i}, \lambda_{2, i}, \lambda_{1, i+1}, \lambda_{2, i+1},$ and specify the following constraints on them (copied from ["Faster variable-base scalar multiplication in zk-SNARK circuits"](https://github.com/zcash/zcash/issues/3924), with some variable name changes):

$$
\lambda_{2,i}^2 - (x_{A,i+1} + (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) + x_{A,i}) = 0,
$$

$$
\begin{aligned}
2 \cdot &\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - \big( \\
    &\begin{aligned}
        (\lambda_{1,i} + \lambda_{2,i}) &\cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i})) + \\
        (\lambda_{1,i+1} + \lambda_{2,i+1}) &\cdot (x_{A,i+1} - (\lambda_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1})) \\
    \end{aligned} \\
\big) &= 0.
\end{aligned}
$$
