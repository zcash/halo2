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

## Constraint program for optimized double-and-add (incomplete addition)
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
\hspace{1.5em} \lambda_{1,i}^2 = x_{R,i} + x_{A,i} + x_{P,i} \\
\hspace{1.5em} (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) = 2 y_{\mathsf{A},i} \\
\hspace{1.5em} \lambda_{2,i}^2 = x_{A,i-1} + x_{R,i} + x_{A,i} \\
\hspace{1.5em} \lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) = y_{A,i} + y_{A,i-1}, \\
\end{array}$

where $x_{R,i} = (\lambda_{1,i}^2 - x_{A,i} - x_T).$ The helper $\BoolCheck{x} = x \cdot (1 - x)$.
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


### Circuit design
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
            &             &                           &     x_{A,129}      &    y_{A,129}     &                  &            &             &            &   \mathbf{z}_4        & x_{A,4}     & \lambda_{1,4}       & \lambda_{2,4}     &     0      &     0       &     1      \\\hline
            &             &                           &                    &                  &                  &            &             &            &                       & x_{A,3}     &     y_{A,3}         &                   &            &             &            \\\hline

\end{array}
$$

For each $hi$ and $lo$ half, we have three sets of gates. Note that $i$ is going from $255..=3$; $i$ is NOT indexing the rows.

#### $q_1 = 1$
This gate is only used on the first row (before the for loop). We check that $\lambda_1, \lambda_2$ are initialized to values consistent with the initial $y_A.$
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_1 \cdot \left(y_{A,n}^\text{witnessed} - y_{A,n}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
y_{A,n} &= \frac{(\lambda_{1,n} + \lambda_{2,n}) \cdot (x_{A,n} - (\lambda_{1,n}^2 - x_{A,n} - x_T))}{2},\\
y_{A,n}^\text{witnessed} &\text{ is witnessed.}
\end{aligned}
$$

#### $q_2 = 1$
This gate is used on all rows corresponding to the for loop except the last.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & q_2 \cdot \left(x_{T,cur} - x_{T,next}\right) = 0 \\\hline
2 & q_2 \cdot \left(y_{T,cur} - y_{T,next}\right) = 0 \\\hline
3 & q_2 \cdot \BoolCheck{\mathbf{k}_i} = 0, \text{ where } \mathbf{k}_i = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\\hline
4 & q_2 \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{T,i}) - y_{A,i} + (2\mathbf{k}_i - 1) \cdot y_{T,i}\right) = 0 \\\hline
3 & q_2 \cdot \left(\lambda_{2,i}^2 - x_{A,i-1} - \lambda_{1,i}^2 + x_{T,i}\right) = 0 \\\hline
3 & q_2 \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) - y_{A,i} - y_{A,i-1}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2}, \\
y_{A,i-1} &= \frac{(\lambda_{1,i-1} + \lambda_{2,i-1}) \cdot (x_{A,i-1} - (\lambda_{1,i-1}^2 - x_{A,i-1} - x_T))}{2}, \\
\end{aligned}
$$

#### $q_3 = 1$
This gate is used on the final iteration of the for loop, handling the special case where we check that the output $y_A$ has been witnessed correctly.
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3 & q_3 \cdot \BoolCheck{\mathbf{k}_i} = 0, \text{ where } \mathbf{k}_i = \mathbf{z}_{i} - 2\mathbf{z}_{i+1} \\\hline
4 & q_3 \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{T,i}) - y_{A,i} + (2\mathbf{k}_i - 1) \cdot y_{T,i}\right) = 0 \\\hline
3 & q_3 \cdot \left(\lambda_{2,i}^2 - x_{A,i-1} - \lambda_{1,i}^2 + x_{T,i}\right) = 0 \\\hline
3 & q_3 \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) - y_{A,i} - y_{A,i-1}^\text{witnessed}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_T))}{2},\\
y_{A,i-1}^\text{witnessed} &\text{ is witnessed.}
\end{aligned}
$$

## Overflow check

$\mathbf{z}_i$ cannot overflow for any $i \geq 1$, because it is a weighted sum of bits only up to $2^{n-1} = 2^{253}$, which is smaller than $p$ (and also $q$).

However, $\mathbf{z}_0 = \alpha + t_q$ *can* overflow $[0, p)$.

Since overflow can only occur in the final step that constrains $\mathbf{z}_0 = 2 \cdot \mathbf{z}_1 + \mathbf{k}_0$, we have $\mathbf{z}_0 = k \pmod{p}$. It is then sufficient to also check that $\mathbf{z}_0 = \alpha + t_q \pmod{p}$ (so that $k = \alpha + t_q \pmod{p}$) and that $k \in [t_q, p + t_q)$. These conditions together imply that $k = \alpha + t_q$ as an integer, and so $2^{254} + k = \alpha \pmod{q}$ as required.

> Note: the bits $\mathbf{k}_{254..0}$ do not represent a value reduced modulo $q$, but rather a representation of the unreduced $\alpha + t_q$.

### Optimized check for $k \in [t_q, p + t_q)$

Since $t_p + t_q < 2^{130}$, we have $$[t_q, p + t_q) = [t_q, t_q + 2^{130}) \;\cup\; [2^{130}, 2^{254}) \;\cup\; \big([2^{254}, 2^{254} + 2^{130}) \;\cap\; [p + t_q - 2^{130}, p + t_q)\big).$$

We may assume that $k = \alpha + t_q \pmod{p}$.

Therefore,
$\begin{array}{rcl}
k \in [t_q, p + t_q) &\Leftrightarrow& \big(k \in [t_q, t_q + 2^{130}) \;\vee\; k \in [2^{130}, 2^{254})\big) \;\vee\; \\
                     &               & \big(k \in [2^{254}, 2^{254} + 2^{130}) \;\wedge\; k \in [p + t_q - 2^{130}, p + t_q)\big) \\
                     \\
                     &\Leftrightarrow& \big(\mathbf{k}_{254} = 0 \implies (k \in [t_q, t_q + 2^{130}) \;\vee\; k \in [2^{130}, 2^{254}))\big) \;\wedge \\
                     &               & \big(\mathbf{k}_{254} = 1 \implies (k \in [2^{254}, 2^{254} + 2^{130}) \;\wedge\; k \in [p + t_q - 2^{130}, p + t_q)\big) \\
                     \\
                     &\Leftrightarrow& \big(\mathbf{k}_{254} = 0 \implies (\alpha \in [0, 2^{130}) \;\vee\; k \in [2^{130}, 2^{254})\big) \;\wedge \\
                     &               & \big(\mathbf{k}_{254} = 1 \implies (k \in [2^{254}, 2^{254} + 2^{130}) \;\wedge\; (\alpha + 2^{130}) \bmod p \in [0, 2^{130}))\big) \;\;Ⓐ
\end{array}$

> Given $k \in [2^{254}, 2^{254} + 2^{130})$, we prove equivalence of $k \in [p + t_q - 2^{130}, p + t_q)$ and $(\alpha + 2^{130}) \bmod p \in [0, 2^{130})$ as follows:
> * shift the range by $2^{130} - p - t_q$ to give $k + 2^{130} - p - t_q \in [0, 2^{130})$;
> * observe that $k + 2^{130} - p - t_q$ is guaranteed to be in $[2^{130} - t_p - t_q, 2^{131} - t_p - t_q)$ and therefore cannot overflow or underflow modulo $p$;
> * using the fact that $k = \alpha + t_q \pmod{p}$, observe that $(k + 2^{130} - p - t_q) \bmod p = (\alpha + t_q + 2^{130} - p - t_q) \bmod p = (\alpha + 2^{130}) \bmod p$.
>
> (We can see in a different way that this is correct by observing that it checks whether $\alpha \bmod p \in [p - 2^{130}, p)$, so the upper bound is aligned as we would expect.)

Now, we can continue optimizing from $Ⓐ$:

$\begin{array}{rcl}
k \in [t_q, p + t_q) &\Leftrightarrow& \big(\mathbf{k}_{254} = 0 \implies (\alpha \in [0, 2^{130}) \;\vee\; k \in [2^{130}, 2^{254})\big) \;\wedge \\
                     &               & \big(\mathbf{k}_{254} = 1 \implies (k \in [2^{254}, 2^{254} + 2^{130}) \;\wedge\; (\alpha + 2^{130}) \bmod p \in [0, 2^{130}))\big) \\
                     \\
                     &\Leftrightarrow& \big(\mathbf{k}_{254} = 0 \implies (\alpha \in [0, 2^{130}) \;\vee\; \mathbf{k}_{253..130} \text{ are not all } 0)\big) \;\wedge \\
                     &               & \big(\mathbf{k}_{254} = 1 \implies (\mathbf{k}_{253..130} \text{ are all } 0 \;\wedge\; (\alpha + 2^{130}) \bmod p \in [0, 2^{130}))\big)
\end{array}$

Constraining $\mathbf{k}_{253..130}$ to be all-$0$ or not-all-$0$ can be implemented almost "for free", as follows.

Recall that $\mathbf{z}_i = \sum_{h=i}^{n} (\mathbf{k}_{h} \cdot 2^{h-i})$, so we have:

$\begin{array}{rcl}
                                 \mathbf{z}_{130} &=& \sum_{h=130}^{254} (\mathbf{k}_h \cdot 2^{h-130}) \\
                                 \mathbf{z}_{130} &=& \mathbf{k}_{254} \cdot 2^{254-130} + \sum_{h=130}^{253} (\mathbf{k}_h \cdot 2^{h-130}) \\
\mathbf{z}_{130} - \mathbf{k}_{254} \cdot 2^{124} &=& \sum_{h=130}^{253} (\mathbf{k}_h \cdot 2^{h-130})
\end{array}$

So $\mathbf{k}_{253..130}$ are all $0$ exactly when $\mathbf{z}_{130} = \mathbf{k}_{254} \cdot 2^{124}$.

Finally, we can merge the $130$-bit decompositions for the $\mathbf{k}_{254} = 0$ and $\mathbf{k}_{254} = 1$ cases by checking that $(\alpha + \mathbf{k}_{254} \cdot 2^{130}) \bmod p \in [0, 2^{130})$.

### Overflow check constraints

Let $s = \alpha + \mathbf{k}_{254} \cdot 2^{130}$. The constraints for the overflow check are:

$$
\begin{aligned}
\mathbf{z}_0 &= \alpha + t_q \pmod{p} \\
\mathbf{k}_{254} = 1 \implies \big(\mathbf{z}_{130} &= 2^{124} \;\wedge\; s \bmod p \in [0, 2^{130})\big) \\
\mathbf{k}_{254} = 0 \implies \big(\mathbf{z}_{130} &\neq 0 \;\vee\; s \bmod p \in [0, 2^{130})\big)
\end{aligned}
$$

Define $\mathsf{inv0}(x) = \begin{cases} 0, &\text{if } x = 0 \\ 1/x, &\text{otherwise.} \end{cases}$

Witness $\eta = \mathsf{inv0}(\mathbf{z}_{130})$, and decompose $s \bmod p$ as $\mathbf{s}_{129..0}$.

Then the needed gates are:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2 & \text{q\_mul}^\text{overflow} \cdot \left(s - (\alpha + \mathbf{k}_{254} \cdot 2^{130})\right) = 0 \\\hline
2 & \text{q\_mul}^\text{overflow} \cdot \left(\mathbf{z}_0 - \alpha - t_q\right) = 0 \\\hline
3 & \text{q\_mul}^\text{overflow} \cdot \left(\mathbf{k}_{254} \cdot (\mathbf{z}_{130} - 2^{124})\right) = 0 \\\hline
3 & \text{q\_mul}^\text{overflow} \cdot \left(\mathbf{k}_{254} \cdot (s - \sum\limits_{i=0}^{129} 2^i \cdot \mathbf{s}_i)/2^{130}\right) = 0 \\\hline
5 & \text{q\_mul}^\text{overflow} \cdot \left((1 - \mathbf{k}_{254}) \cdot (1 - \mathbf{z}_{130} \cdot \eta) \cdot (s - \sum\limits_{i=0}^{129} 2^i \cdot \mathbf{s}_i)/2^{130}\right) = 0 \\\hline
\end{array}
$$
where $(s - \sum\limits_{i=0}^{129} 2^i \cdot \mathbf{s}_i)/2^{130}$ can be computed by another running sum. Note that the factor of $1/2^{130}$ has no effect on the constraint, since the RHS is zero.

#### Running sum range check
We make use of a $10$-bit [lookup range check](../decomposition.md#lookup-decomposition) in the circuit to subtract the low $130$ bits of $\mathbf{s}$. The range check subtracts the first $13 \cdot 10$ bits of $\mathbf{s},$ and right-shifts the result to give $(s - \sum\limits_{i=0}^{129} 2^i \cdot \mathbf{s}_i)/2^{130}.$
