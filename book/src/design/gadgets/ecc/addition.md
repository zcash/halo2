We will use formulae for curve arithmetic using affine coordinates on short Weierstrass curves,
derived from section 4.1 of [Hüseyin Hışıl's thesis](https://core.ac.uk/download/pdf/10898289.pdf).

## Incomplete addition

- Inputs: $P = (x_p, y_p), Q = (x_q, y_q)$
- Output: $R = P \;⸭\; Q = (x_r, y_r)$

The formulae from Hışıl's thesis are:

- $x_3 = \left(\frac{y_1 - y_2}{x_1 - x_2}\right)^2 - x_1 - x_2$
- $y_3 = \frac{y_1 - y_2}{x_1 - x_2} \cdot (x_1 - x_3) - y_1$

Rename:
- $(x_1, y_1)$ to $(x_q, y_q)$
- $(x_2, y_2)$ to $(x_p, y_p)$
- $(x_3, y_3)$ to $(x_r, y_r)$.

Let $\lambda = \frac{y_q - y_p}{x_q - x_p} = \frac{y_p - y_q}{x_p - x_q}$, which we implement as

$\lambda \cdot (x_p - x_q) = y_p - y_q$

Also,
- $x_r = \lambda^2 - x_q - x_p$
- $y_r = \lambda \cdot (x_q - x_r) - y_q$

which is equivalent to

- $x_r + x_q + x_p = \lambda^2$

Assuming $x_p \neq x_q$,

$
\begin{array}{lrrll}
&&(x_r + x_q + x_p) \cdot (x_p - x_q)^2 &=& \lambda^2 \cdot (x_p - x_q)^2 \\
&\implies &(x_r + x_q + x_p) \cdot (x_p - x_q)^2 &=& \big(\lambda \cdot (x_p - x_q)\big)^2 \\[1.2ex]
\text{and} \\
&         &y_r &=& \lambda \cdot (x_q - x_r) - y_q \\
&\implies &y_r + y_q &=& \lambda \cdot (x_q - x_r) \\
&\implies &(y_r + y_q) \cdot (x_p - x_q) &=& \lambda \cdot (x_p - x_q) \cdot (x_q - x_r)
\end{array}
$

Substituting for $\lambda \cdot (x_p - x_q)$, we get the constraints:
- $(x_r + x_q + x_p) \cdot (x_p - x_q)^2 - (y_p - y_q)^2 = 0$
  - Note that this constraint is unsatisfiable for $P \;⸭\; (-P)$ (when $P \neq \mathcal{O}$),
    and so cannot be used with arbitrary inputs.
- $(y_r + y_q) \cdot (x_p - x_q) - (y_p - y_q) \cdot (x_q - x_r) = 0$


## Complete addition

$\hspace{1em} \begin{array}{rcll}
\mathcal{O} &+& \mathcal{O} &= \mathcal{O} \\[0.4ex]
\mathcal{O} &+& (x_q, y_q)  &= (x_q, y_q) \\[0.4ex]
 (x_p, y_p) &+& \mathcal{O} &= (x_p, y_p) \\[0.6ex]
   (x, y)   &+& (x, y)      &= [2] (x, y) \\[0.6ex]
   (x, y)   &+& (x, -y)     &= \mathcal{O} \\[0.4ex]
 (x_p, y_p) &+& (x_q, y_q)  &= (x_p, y_p) \;⸭\; (x_q, y_q), \text{ if } x_p \neq x_q.
\end{array}$

Suppose that we represent $\mathcal{O}$ as $(0, 0)$. ($0$ is not an $x$-coordinate of a valid point because we would need $y^2 = x^3 + 5$, and $5$ is not square in $\mathbb{F}_q$. Also $0$ is not a $y$-coordinate of a valid point because $-5$ is not a cube in $\mathbb{F}_q$.)

$$
\begin{aligned}
P + Q &= R\\
(x_p, y_p) + (x_q, y_q) &= (x_r, y_r) \\
                \lambda &= \frac{y_q - y_p}{x_q - x_p} \\
                    x_r &= \lambda^2 - x_p - x_q \\
                    y_r &= \lambda(x_p - x_r) - y_p
\end{aligned}
$$

For the doubling case, Hışıl's thesis tells us that $\lambda$ has to
instead be computed as $\frac{3x^2}{2y}$.

Define $\mathsf{inv0}(x) = \begin{cases} 0, &\text{if } x = 0 \\ 1/x, &\text{otherwise.} \end{cases}$

Witness $\alpha, \beta, \gamma, \delta, \lambda$ where:

$\hspace{1em}
\begin{array}{rl}
\alpha  \,\,=&\!\! \mathsf{inv0}(x_q - x_p) \\[0.4ex]
\beta   \,\,=&\!\! \mathsf{inv0}(x_p) \\[0.4ex]
\gamma  \,\,=&\!\! \mathsf{inv0}(x_q) \\[0.4ex]
\delta  \,\,=&\!\! \begin{cases}
                     \mathsf{inv0}(y_q + y_p), &\text{if } x_q = x_p \\
                     0, &\text{otherwise}
                   \end{cases} \\[2.5ex]
\lambda \,\,=&\!\! \begin{cases}
                     \frac{y_q - y_p}{x_q - x_p}, &\text{if } x_q \neq x_p \\[1.2ex]
                     \frac{3{x_p}^2}{2y_p} &\text{if } x_q = x_p \wedge y_p \neq 0 \\[0.8ex]
                     0, &\text{otherwise.}
                   \end{cases}
\end{array}
$

### Constraints

$$
\begin{array}{|c|rcl|l|}
\hline
\text{Degree} & \text{Constraint}\hspace{7em} &&& \text{Meaning} \\\hline
4 & q_\mathit{add} \cdot (x_q - x_p) \cdot ((x_q - x_p) \cdot \lambda - (y_q - y_p)) &=& 0 & x_q \neq x_p \implies \lambda = \frac{y_q - y_p}{x_q - x_p} \\\hline \\[-2.3ex]
5 & q_\mathit{add} \cdot (1 - (x_q - x_p) \cdot \alpha) \cdot \left(2y_p \cdot \lambda - 3{x_p}^2\right) &=& 0 & \begin{cases} x_q = x_p \wedge y_p \neq 0 \implies \lambda = \frac{3{x_p}^2}{2y_p} \\ x_q = x_p \wedge y_p = 0 \implies x_p = 0 \end{cases} \\\hline
6 & q_\mathit{add} \cdot x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda^2 - x_p - x_q - x_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge x_q \neq x_p \implies x_r = \lambda^2 - x_p - x_q \\
6 & q_\mathit{add} \cdot x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge x_q \neq x_p \implies y_r = \lambda \cdot (x_p - x_r) - y_p \\
6 & q_\mathit{add} \cdot x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda^2 - x_p - x_q - x_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge y_q \neq -y_p \implies x_r = \lambda^2 - x_p - x_q \\
6 & q_\mathit{add} \cdot x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge y_q \neq -y_p \implies y_r = \lambda \cdot (x_p - x_r) - y_p \\\hline
4 & q_\mathit{add} \cdot (1 - x_p \cdot \beta) \cdot (x_r - x_q) &=& 0 & x_p = 0 \implies x_r = x_q \\
4 & q_\mathit{add} \cdot (1 - x_p \cdot \beta) \cdot (y_r - y_q) &=& 0 & x_p = 0 \implies y_r = y_q \\\hline
4 & q_\mathit{add} \cdot (1 - x_q \cdot \gamma) \cdot (x_r - x_p) &=& 0 & x_q = 0 \implies x_r = x_p \\
4 & q_\mathit{add} \cdot (1 - x_q \cdot \gamma) \cdot (y_r - y_p) &=& 0 & x_q = 0 \implies y_r = y_p \\\hline
4 & q_\mathit{add} \cdot (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot x_r &=& 0 & x_q = x_p \wedge y_q = -y_p \implies x_r = 0 \\
4 & q_\mathit{add} \cdot (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot y_r &=& 0 & x_q = x_p \wedge y_q = -y_p \implies y_r = 0 \\\hline
\end{array}
$$

Max degree: 6

### Analysis of constraints
$$
\begin{array}{rl}
1. & (x_q - x_p) \cdot ((x_q - x_p) \cdot \lambda - (y_q - y_p)) = 0 \\
   & \\
   & \begin{aligned}
         \text{At least one of } &x_q - x_p = 0 \\
                      \text{or } &(x_q - x_p) \cdot \lambda - (y_q - y_p) = 0 \\
       \end{aligned} \\
   & \text{must be satisfied for the constraint to be satisfied.} \\
   & \\
   & \text{If } x_q - x_p \neq 0, \text{ then } (x_q - x_p) \cdot \lambda - (y_q - y_p) = 0, \text{ and} \\
   & \text{by rearranging both sides we get } \lambda = (y_q - y_p) / (x_q - x_p). \\
   & \\
   & \text{Therefore:} \\
   & \hspace{2em} x_q \neq x_p \implies \lambda = (y_q - y_p) / (x_q - x_p).\\
   & \\
2. & (1 - (x_q - x_p) \cdot \alpha) \cdot (2y_p \cdot \lambda - 3x_p^2) = 0 \\
   & \\
   & \begin{aligned}
         \text{At least one of } &(1 - (x_q - x_p) \cdot \alpha) = 0 \\
                      \text{or } &(2y_p \cdot \lambda - 3x_p^2) = 0
       \end{aligned} \\
   & \text{must be satisfied for the constraint to be satisfied.} \\
   & \\
   & \text{If } x_q = x_p, \text{ then } 1 - (x_q - x_p) \cdot \alpha = 0 \text{ has no solution for } \alpha, \\
   & \text{so it must be that } 2y_p \cdot \lambda - 3x_p^2 = 0. \\
   & \\
   & \text{If } x_q = x_p \text{ and } y_p = 0 \text{ then } x_p = 0, \text{ and the constraint is satisfied.}\\
   & \\
   & \text{If } x_q = x_p \text{ and } y_p \neq 0 \text{ then by rearranging both sides} \\
   & \text{we get } \lambda = 3x_p^2 / 2y_p. \\
   & \\
   & \text{Therefore:} \\
   & \hspace{2em} (x_q = x_p) \wedge y_p \neq 0 \implies \lambda = 3x_p^2 / 2y_p. \\
   & \\
3.\text{ a)} & x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda^2 - x_p - x_q - x_r) = 0 \\
  \text{ b)} & x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) = 0 \\
  \text{ c)} & x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda^2 - x_p - x_q - x_r) = 0 \\
  \text{ d)} & x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) = 0 \\
   & \\
   & \begin{aligned}
       \text{At least one of } &x_p = 0 \\
                    \text{or } &x_p = 0 \\
                    \text{or } &(x_q - x_p) = 0 \\
                    \text{or } &(\lambda^2 - x_p - x_q - x_r) = 0 \\
     \end{aligned} \\
   & \text{must be satisfied for constraint (a) to be satisfied.} \\
   & \\
   & \text{If } x_p \neq 0 \wedge x_q \neq 0 \wedge x_q \neq x_p, \\[1.5ex]
   & \text{• Constraint (a) imposes that } x_r = \lambda^2 - x_p - x_q. \\
   & \text{• Constraint (b) imposes that } y_r = \lambda \cdot (x_p - x_r) - y_p. \\
   & \\
   & \text{If } x_p \neq 0 \wedge x_q \neq 0 \wedge y_q \neq -y_p, \\[1.5ex]
   & \text{• Constraint (c) imposes that } x_r = \lambda^2 - x_p - x_q. \\
   & \text{• Constraint (d) imposes that } y_r = \lambda \cdot (x_p - x_r) - y_p. \\
   & \\
   & \text{Therefore:} \\
   & \begin{aligned}
                 &(x_p \neq 0) \wedge (x_q \neq 0) \wedge ((x_q \neq x_p) \vee (y_q \neq -y_p)) \\
        \implies &(x_r = \lambda^2 - x_p - x_q) \wedge (y_r = \lambda \cdot (x_p - x_r) - y_p).
     \end{aligned} \\
   & \\
4.\text{ a)} & (1 - x_p \cdot \beta) \cdot (x_r - x_q) = 0 \\
  \text{ b)} & (1 - x_p \cdot \beta) \cdot (y_r - y_q) = 0 \\
  & \\
  & \begin{aligned}
      \text{At least one of } 1 - x_p \cdot \beta &= 0 \\
                             \text{or } x_r - x_q &= 0
    \end{aligned} \\
  & \text{must be satisfied for constraint (a) to be satisfied.} \\
  & \\
  & \text{If } x_p = 0 \text{ then } 1 - x_p \cdot \beta = 0 \text{ has no solutions for } \beta, \\
  & \text{and so it must be that } x_r - x_q = 0. \\
  & \\
  & \text{Similarly, constraint (b) imposes that if } x_p = 0 \\
  & \text{then } y_r - y_q = 0. \\
  & \\
  & \text{Therefore:} \\
  & \hspace{2em} x_p = 0 \implies (x_r, y_r) = (x_q, y_q). \\
  & \\
5.\text{ a)} & (1 - x_q \cdot \beta) \cdot (x_r - x_p) = 0 \\
  \text{ b)} & (1 - x_q \cdot \beta) \cdot (y_r - y_p) = 0 \\
  & \\
  & \begin{aligned}
      \text{At least one of } 1 - x_q \cdot \beta &= 0 \\
                             \text{or } x_r - x_p &= 0
    \end{aligned} \\
  & \text{must be satisfied for constraint (a) to be satisfied.} \\
  & \\
  & \text{If } x_q = 0 \text{ then } 1 - x_q \cdot \beta = 0 \text{ has no solutions for } \beta, \\
  & \text{and so it must be that } x_r - x_p = 0. \\
  & \\
  & \text{Similarly, constraint (b) imposes that if } x_q = 0 \\
  & \text{then } y_r - y_p = 0. \\
  & \\
  & \text{Therefore:} \\
  & \hspace{2em} x_q = 0 \implies (x_r, y_r) = (x_p, y_p). \\
  & \\
6.\text{ a)} & (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot x_r = 0 \\
  \text{ b)} & (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot y_r = 0 \\
  & \\
  & \begin{aligned}
      \text{At least one of } &1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta = 0 \\
                   \text{or } &x_r = 0
    \end{aligned} \\
  & \text{must be satisfied for constraint (a) to be satisfied,} \\
  & \text{and similarly replacing } x_r \text{ by } y_r. \\
  & \\
  & \text{If } x_r \neq 0 \text{ or } y_r = 0, \text{ then it must be that } 1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta = 0. \\
  & \\
  & \text{However, if } x_q = x_p \wedge y_q = -y_p, \text{ then there are no solutions for } \alpha \text { and } \delta. \\
  & \\
  & \text{Therefore: } \\
  & \hspace{2em} x_q = x_p \wedge y_q = -y_p \implies (x_r, y_r) = (0, 0).
\end{array}
$$

#### Propositions:

$
\begin{array}{cl}
(1)& x_q \neq x_p \implies \lambda = (y_q - y_p) / (x_q - x_p) \\[0.8ex]
(2)& (x_q = x_p) \wedge y_p \neq 0 \implies \lambda = 3x_p^2 / 2y_p \\[0.8ex]
(3)& (x_p \neq 0) \wedge (x_q \neq 0) \wedge ((x_q \neq x_p) \vee (y_q \neq -y_p)) \\[0.4ex]
    &\implies (x_r = \lambda^2 - x_p - x_q) \wedge (y_r = \lambda \cdot (x_p - x_r) - y_p) \\[0.8ex]
(4)& x_p = 0 \implies (x_r, y_r) = (x_q, y_q) \\[0.8ex]
(5)& x_q = 0 \implies (x_r, y_r) = (x_p, y_p) \\[0.8ex]
(6)& x_q = x_p \wedge y_q = -y_p \implies (x_r, y_r) = (0, 0)
\end{array}
$

#### Cases:

$(x_p, y_p) + (x_q, y_q) = (x_r, y_r)$

Note that we rely on the fact that $0$ is not a valid $x$-coordinate or $y$-coordinate of a
point on the Pallas curve other than $\mathcal{O}$.

* $(0, 0) + (0, 0)$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q = x_p \\
        (2)&\text{holds because } y_p = 0 \\
        (3)&\text{holds because } x_p = 0 \\
        (4)&\text{holds because } (x_r, y_r) = (x_q, y_q) = (0, 0) \\
        (5)&\text{holds because } (x_r, y_r) = (x_p, y_p) = (0, 0) \\
        (6)&\text{holds because } (x_r, y_r) = (0, 0). \\
        \end{array}
        $

    - Soundness: $(x_r, y_r) = (0, 0)$ is the only solution to $(6).$

* $(x, y) + (0, 0)$ for $(x, y) \neq (0, 0)$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q \neq x_p, \text{ therefore } \lambda = (y_q - y_p) / (x_q - x_p) \text{ is a solution} \\
        (2)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ is a solution} \\
        (3)&\text{holds because } x_q = 0 \\
        (4)&\text{holds because } x_p \neq 0, \text{ therefore } \beta = x_p^{-1} \text{ is a solution} \\
        (5)&\text{holds because } (x_r, y_r) = (x_p, y_p) \\
        (6)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ and } \delta = 0 \text{ is a solution.}
        \end{array}
        $

    - Soundness: $(x_r, y_r) = (x_p, y_p)$ is the only solution to $(5).$

* $(0, 0) + (x, y)$ for $(x, y) \neq (0, 0)$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q \neq x_p, \text{ therefore } \lambda = (y_q - y_p) / (x_q - x_p) \text{ is a solution} \\
        (2)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ is a solution} \\
        (3)&\text{holds because } x_p = 0 \\
        (4)&\text{holds because } x_p = 0 \text{ only when } (x_r, y_r) = (x_q, y_q) \\
        (5)&\text{holds because } x_q \neq 0, \text{ therefore } \gamma = x_q^{-1} \text{ is a solution}\\
        (6)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ and } \delta = 0 \text{ is a solution.}
        \end{array}
        $

    - Soundness: $(x_r, y_r) = (x_q, y_q)$ is the only solution to $(4).$

* $(x, y) + (x, y)$ for $(x, y) \neq (0, 0)$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q = x_p \\
        (2)&\text{holds because } x_q = x_p \wedge y_p \neq 0, \text{ therefore } \lambda = 3x_p^2 / 2y_p \text{ is a solution}\\
        (3)&\text{holds because } x_r = \lambda^2 - x_p - x_q \wedge y_r = \lambda \cdot (x_p - x_r) - y_p \text{ in this case} \\
        (4)&\text{holds because } x_p \neq 0, \text{ therefore } \beta = x_p^{-1} \text{ is a solution} \\
        (5)&\text{holds because } x_p \neq 0, \text{ therefore } \gamma = x_q^{-1} \text{ is a solution} \\
        (6)&\text{holds because } x_q = x_p \text{ and } y_q \neq -y_p, \text{ therefore } \alpha = 0 \text{ and } \delta = (y_q + y_p)^{-1} \text{ is a solution.} \\
        \end{array}
        $

    - Soundness: $\lambda$ is computed correctly, and $(x_r, y_r) = (\lambda^2 - x_p - x_q, \lambda \cdot (x_p - x_r) - y_p)$ is the only solution.

* $(x, y) + (x, -y)$ for $(x, y) \neq (0, 0)$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q = x_p \\
        (2)&\text{holds because } x_q = x_p \wedge y_p \neq 0, \text{ therefore } \lambda = 3x_p^2 / 2y_p \text{ is a solution} \\
           &\text{(although } \lambda \text{ is not used in this case)} \\
        (3)&\text{holds because } x_q = x_p \text{ and } y_q = -y_p \\
        (4)&\text{holds because } x_p \neq 0, \text{ therefore } \beta = x_p^{-1} \text{ is a solution} \\
        (5)&\text{holds because } x_q \neq 0, \text{ therefore } \gamma = x_q^{-1} \text{ is a solution} \\
        (6)&\text{holds because } (x_r, y_r) = (0, 0) \\
        \end{array}
        $

    - Soundness: $(x_r, y_r) = (0, 0)$ is the only solution to $(6).$

* $(x_p, y_p) + (x_q, y_q)$ for $(x_p, y_p) \neq (0,0)$ and $(x_q, y_q) \neq (0, 0)$ and $x_p \neq x_q$
    - Completeness:

        $
        \begin{array}{cl}
        (1)&\text{holds because } x_q \neq x_p, \text{ therefore } \lambda = (y_q - y_p) / (x_q - x_p) \text{ is a solution} \\
        (2)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ is a solution} \\
        (3)&\text{holds because } x_r = \lambda^2 - x_p - x_q \wedge y_r = \lambda \cdot (x_p - x_r) - y_p \text{ in this case} \\
        (4)&\text{holds because } x_p \neq 0, \text{ therefore } \beta = x_p^{-1} \text{ is a solution} \\
        (5)&\text{holds because } x_q \neq 0, \text{ therefore } \gamma = x_q^{-1} \text{ is a solution} \\
        (6)&\text{holds because } x_q \neq x_p, \text{ therefore } \alpha = (x_q - x_p)^{-1} \text{ and } \delta = 0 \text{ is a solution.}
        \end{array}
        $

    - Soundness: $\lambda$ is computed correctly, and $(x_r, y_r) = (\lambda^2 - x_p - x_q, \lambda \cdot (x_p - x_r) - y_p)$ is the only solution.
