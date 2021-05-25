# Elliptic Curve Cryptography

## Point doubling
- Input: $P = (x_p, y_p)$
- Output: $[2]P = (x_r, y_r)$

Formulae:
$\lambda = \frac{3 x_p^2}{2 \cdot y_p}$
$x_r = \lambda^2 - 2 \cdot x_p$
$y_r = \lambda(x_p - x_r) - y_p$

Substituting for $\lambda$, we get the constraints:
- $4 \cdot y_p^2(x_r + 2 \cdot x_p) - 9 x_p^4 = 0$
- $2 \cdot y_p(y_r + y_p) - 3 x_p^2(x_p - x_r) = 0$

## Incomplete addition
- Inputs: $P = (x_p, y_p), Q = (x_q, y_q)$
- Output: $R = P \;⸭\; Q = (x_r, y_r)$

Formulae:
- $\lambda \cdot (x_p - x_q) = y_p - y_q$
- $x_r = \lambda^2 - x_q - x_p$
- $y_r = \lambda(x_q - x_r) - y_q$

Substituting for $\lambda$, we get the constraints:
- $(x_r + x_q + x_p) \cdot (x_p - x_q)^2 - (y_p - y_q)^2 = 0$
  - Note that this constraint is unsatisfiable for $P \;⸭\; (-P)$ (when $P \neq \mathcal{O}$),
    and so cannot be used with arbitrary inputs.
- $(y_r + y_q)(x_p - x_q) - (y_p - y_q)(x_q - x_r) = 0$


## Complete addition

To implement complete addition inside the circuit, we need to check the following cases:

$\begin{array}{rcll}
\mathcal{O} &+& \mathcal{O} &= \mathcal{O} ✓\\
\mathcal{O} &+& (x_q, y_q)  &= (x_q, y_q) ✓\\
 (x_p, y_p) &+& \mathcal{O} &= (x_p, y_p) ✓\\
   (x, y)   &+& (x, y)      &= [2] (x, y) ✓\\
   (x, y)   &+& (x, -y)     &= \mathcal{O} ✓\\
 (x_p, y_p) &+& (x_q, y_q)  &= (x_p, y_p) \;⸭\; (x_q, y_q), \text{if } x_p \neq x_q ✓
\end{array}$

We represent $\mathcal{O}$ as $(0, 0)$.

> $0$ is not an $x$-coordinate of a valid point because we would need $y^2 = x^3 + 5$, and $5$ is not square in $\mathbb{F}_q$.

$$
\begin{aligned}
P + Q &= R\\
(x_p, y_p) + (x_q, y_q) &= (x_r, y_r) \\
                \lambda &= \frac{y_p - y_q}{x_p - x_q} \\
                    x_r &= \lambda^2 - x_q - x_p \\
                    y_r &= \lambda(x_q - x_r) - y_q
\end{aligned}
$$

For the doubling case, $\lambda$ has to instead be computed as $\frac{3x^2}{2y}$.

Define $\mathsf{inv0}(x) = \begin{cases} 0, &\text{if } x = 0 \\ 1/x, &\text{otherwise.} \end{cases}$

Witness $\alpha, \beta, \gamma, \delta, \lambda$ where:

* $\alpha = \mathsf{inv0}(x_q - x_p)$
* $\beta = \mathsf{inv0}(x_p)$
* $\gamma = \mathsf{inv0}(x_q)$
* $\delta = \begin{cases}
              \mathsf{inv0}(y_q + y_p), &\text{if } x_q = x_p \\
              0, &\text{otherwise}
            \end{cases}$
* $\lambda = \begin{cases}
               \frac{y_q - y_p}{x_q - x_p}, &\text{if } x_q \neq x_p \\[0.5ex]
               \frac{3{x_p}^2}{2y_p} &\text{if } x_q = x_p \wedge y_p \neq 0 \\[0.5ex]
               0, &\text{otherwise.}
             \end{cases}$

### Constraints

$$
\begin{array}{|c|rcl|l|}
\hline
\text{Degree} & \text{Constraint}\hspace{7em} &&& \text{Meaning} \\\hline
4 & q_\mathit{Complete} \cdot (x_q - x_p) \cdot ((x_q - x_p) \cdot \lambda - (y_q - y_p)) &=& 0 & x_q \neq x_p \implies \lambda = \frac{y_q - y_p}{x_q - x_p} \\\hline
5 & q_\mathit{Complete} \cdot (1 - (x_q - x_p) \cdot \alpha) \cdot \left(2y_p \cdot \lambda - 3{x_p}^2\right) &=& 0 & \begin{cases} x_q = x_p \wedge y_p \neq 0 \implies \lambda = \frac{3{x_p}^2}{2y_p} \\ x_q = x_p \wedge y_p = 0 \implies x_p = 0 \end{cases} \\\hline
6 & q_\mathit{Complete} \cdot x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda^2 - x_p - x_q - x_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge x_q \neq x_p \implies x_r = \lambda^2 - x_p - x_q \\
6 & q_\mathit{Complete} \cdot x_p \cdot x_q \cdot (x_q - x_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge x_q \neq x_p \implies y_r = \lambda \cdot (x_p - x_r) - y_p \\
6 & q_\mathit{Complete} \cdot x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda^2 - x_p - x_q - x_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge y_q \neq -y_p \implies x_r = \lambda^2 - x_p - x_q \\
6 & q_\mathit{Complete} \cdot x_p \cdot x_q \cdot (y_q + y_p) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) &=& 0 & x_p \neq 0 \wedge x_q \neq 0 \wedge y_q \neq -y_p \implies y_r = \lambda \cdot (x_p - x_r) - y_p \\\hline
4 & q_\mathit{Complete} \cdot (1 - x_p \cdot \beta) \cdot (x_r - x_q) &=& 0 & x_p = 0 \implies x_r = x_q \\
4 & q_\mathit{Complete} \cdot (1 - x_p \cdot \beta) \cdot (y_r - y_q) &=& 0 & x_p = 0 \implies y_r = y_q \\\hline
4 & q_\mathit{Complete} \cdot (1 - x_q \cdot \gamma) \cdot (x_r - x_p) &=& 0 & x_q = 0 \implies x_r = x_p \\
4 & q_\mathit{Complete} \cdot (1 - x_q \cdot \gamma) \cdot (y_r - y_p) &=& 0 & x_q = 0 \implies y_r = y_p \\\hline
4 & q_\mathit{Complete} \cdot (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot x_r &=& 0 & x_q = x_p \wedge y_q = -y_p \implies x_r = 0 \\
4 & q_\mathit{Complete} \cdot (1 - (x_q - x_p) \cdot \alpha - (y_q + y_p) \cdot \delta) \cdot y_r &=& 0 & x_q = x_p \wedge y_q = -y_p \implies y_r = 0 \\\hline
\end{array}
$$

Max degree: 6
