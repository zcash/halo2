# Multipoint opening argument

Consider the commitments $A, B, C, D$ to polynomials $a(X), b(X), c(X), d(X)$.
Let's say that $a$ and $b$ were queried at the point $x$, while $c$ and $d$
were queried at both points $x$ and $\omega x$. (Here, $\omega$ is the primitive
root of unity in the multiplicative subgroup over which we constructed the
polynomials).

To open these commitments, we could create a polynomial $Q$ for each point that we queried
at (corresponding to each relative rotation used in the circuit). But this would not be
efficient in the circuit; for example, $c(X)$ would appear in multiple polynomials.

Instead, we can group the commitments by the sets of points at which they were queried:
$$
\begin{array}{cccc}
&\{x\}&     &\{x, \omega x\}& \\
 &A&            &C& \\
 &B&            &D&
\end{array}
$$

For each of these groups, we combine them into a polynomial set, and create a single $Q$
for that set, which we open at each rotation.

## Optimization steps

The multipoint opening optimization takes as input:

- A random $x$ sampled by the verifier, at which we evaluate $a(X), b(X), c(X), d(X)$.
- Evaluations of each polynomial at each point of interest, provided by the prover:
  $a(x), b(x), c(x), d(x), c(\omega x), d(\omega x)$

These are the outputs of the [vanishing argument](vanishing.md#evaluating-the-polynomials).

The multipoint opening optimization proceeds as such:

1. Sample random $x_1$, to keep $a, b, c, d$ linearly independent.
2. Accumulate polynomials and their corresponding evaluations according
   to the point set at which they were queried:
    `q_polys`:
    $$
    \begin{array}{rccl}
    q_1(X) &=& a(X) &+& x_1 b(X) \\
    q_2(X) &=& c(X) &+& x_1 d(X)
    \end{array}
    $$
    `q_eval_sets`:
    ```math
            [
                [a(x) + x_1 b(x)],
                [
                    c(x) + x_1 d(x),
                    c(\omega x) + x_1 d(\omega x)
                ]
            ]
    ```
    NB: `q_eval_sets` is a vector of sets of evaluations, where the outer vector
    goes over the point sets, and the inner vector goes over the points in each set.
3. Interpolate each set of values in `q_eval_sets`:
    `r_polys`:
    $$
    \begin{array}{cccc}
    r_1(X) s.t.&&& \\
        &r_1(x) &=& a(x) + x_1 b(x) \\
    r_2(X) s.t.&&& \\
        &r_2(x) &=& c(x) + x_1 d(x) \\
        &r_2(\omega x) &=& c(\omega x) + x_1 d(\omega x) \\
    \end{array}
    $$
4. Construct `f_polys` which check the correctness of `q_polys`:
    `f_polys`
    $$
    \begin{array}{rcl}
    f_1(X) &=& \frac{ q_1(X) - r_1(X)}{X - x} \\
    f_2(X) &=& \frac{ q_2(X) - r_2(X)}{(X - x)(X - \omega x)} \\
    \end{array}
    $$

    If $q_1(x) = r_1(x)$, then $f_1(X)$ should be a polynomial.
    If $q_2(x) = r_2(x)$ and $q_2(\omega x) = r_2(\omega x)$
    then $f_2(X)$ should be a polynomial.
5. Sample random $x_2$ to keep the `f_polys` linearly independent.
6. Construct $f(X) = f_1(X) + x_2 f_2(X)$.
7.  Sample random $x_3$, at which we evaluate $f(X)$:
    $$
    \begin{array}{rcccl}
    f(x_3) &=& f_1(x_3) &+& x_2 f_2(x_3) \\
           &=& \frac{q_1(x_3) - r_1(x_3)}{x_3 - x} &+& x_2\frac{q_2(x_3) - r_2(x_3)}{(x_3 - x)(x_3 - \omega x)}
    \end{array}
    $$
8.  Sample random $x_4$ to keep $f(X)$ and `q_polys` linearly independent.
9.  Construct `final_poly`, $$final\_poly(X) = f(X) + x_4 q_1(X) + x_4^2 q_2(X),$$
    which is the polynomial we commit to in the inner product argument.
