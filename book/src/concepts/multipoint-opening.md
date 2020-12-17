# Multipoint opening argument

Consider the commitments $A, B, C, D$ to polynomials $a(X), b(X), c(X), d(X)$.
Let's say that $a$ and $b$ were queried at the point $x$, while $c$ and $d$
were queried at both points $x$ and $\omega x$. (Here, $\omega$ is the primitive
root of unity in the multiplicative subgroup over which we constructed the
polynomials).

We can group the commitments in terms of the sets of points at which they were
queried:
$$
\begin{array}{cccc}
&\{x\}&     &\{x, \omega x\}& \\
 &A&            &C& \\
 &B&            &D&
\end{array}
$$

The multipoint opening optimisation proceeds as such:

1. Sample random $x_3$, at which we evaluate $a(X), b(X), c(X), d(X)$.
2. The prover provides evaluations of each polynomial at each point of interest:
   $a(x_3), b(x_3), c(x_3), d(x_3), c(\omega x_3), d(\omega x_3)$
3. Sample random $x_4$, to keep $a, b, c, d$ linearly independent.
4. Accumulate polynomials and their corresponding evaluations according
   to the point set at which they were queried:
    `q_polys`:
    $$
    \begin{array}{rccl}
    q_1(X) &=& a(X) &+& x_4 b(X) \\
    q_2(X) &=& c(X) &+& x_4 d(X)
    \end{array}
    $$
    `q_eval_sets`:
    ```math
            [
                [a(x_3) + x_4 b(x_3)],
                [
                    c(x_3) + x_4 d(x_3),
                    c(\omega x_3) + x_4 d(\omega x_3)
                ]
            ]
    ```
    NB: `q_eval_sets` is a vector of sets of evaluations, where the outer vector
    goes over the point sets, and the inner vector goes over the points in each set.
5. Interpolate each set of values in `q_eval_sets`:
    `r_polys`:
    $$
    \begin{array}{cccc}
    r_1(X) s.t.&&& \\
        &r_1(x_3) &=& a(x_3) + x_4 b(x_3) \\
    r_2(X) s.t.&&& \\
        &r_2(x_3) &=& c(x_3) + x_4 d(x_3) \\
        &r_2(\omega x_3) &=& c(\omega x_3) + x_4 d(\omega x_3) \\
    \end{array}
    $$
6. Construct `f_polys` which check the correctness of `q_polys`:
    `f_polys`
    $$
    \begin{array}{rcl}
    f_1(X) &=& \frac{ q_1(X) - r_1(X)}{X - x_3} \\
    f_2(X) &=& \frac{ q_2(X) - r_2(X)}{(X - x_3)(X - \omega x_3)} \\
    \end{array}
    $$

    If $q_1(x_3) = r_1(x_3)$, then $f_1(X)$ should be a polynomial.
    If $q_2(x_3) = r_2(x_3)$ and $q_2(\omega x_3) = r_2(\omega x_3)$
    then $f_2(X)$ should be a polynomial.
7. Sample random $x_5$ to keep the `f_polys` linearly independent.
8. Construct $f(X) = f_1(X) + x_5 f_2(X)$.
9. Sample random $x_6$, at which we evaluate $f(X)$:
    $$
    \begin{array}{rcccl}
    f(x_6) &=& f_1(x_6) &+& x_5 f_2(x_6) \\
           &=& \frac{q_1(x_6) - r_1(x_6)}{x_6 - x_3} &+& x_5\frac{q_2(x_6) - r_2(x_6)}{(x_6 - x_3)(x_6 - \omega x_3)}
    \end{array}
    $$
10. Sample random $x_7$ to keep $f(X)$ and `q_polys` linearly independent.
11. Construct `final_poly`, $$final\_poly(X) = f(X) + x_7 q_1(X) + x_7^2 q_2(X),$$
    which is the polynomial we commit to in the inner product argument.
