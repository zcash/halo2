# Elliptic Curve Cryptography

## Incomplete addition
- Inputs: $P = (x_p, y_p), Q = (x_q, y_q)$
- Output: $R = P + Q = (x_r, y_r)$

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

Witness $\lambda, \alpha, \beta, \gamma, \delta, A, B, C, D$.

$
\begin{array}{rcl|rcl}
\text{Constraint} &&& \text{Meaning} \\\hline
            A \cdot (1-A) &=& 0 & A \in \mathbb{B} \\
            B \cdot (1-B) &=& 0 & B \in \mathbb{B} \\
            C \cdot (1-C) &=& 0 & C \in \mathbb{B} \\
            D \cdot (1-D) &=& 0 & D \in \mathbb{B} \\
 (x_q - x_p) \cdot \alpha &=& 1-A & x_q = x_p &\implies& A \\
          x_p \cdot \beta &=& 1-B & x_p = 0 &\implies& B \\
              B \cdot x_p &=& 0 & B &\implies& x_p = 0 \\
         x_q \cdot \gamma &=& 1-C & x_q = 0 &\implies& C \\
              C \cdot x_q &=& 0 & C &\implies& x_q = 0 \\
 (y_q + y_p) \cdot \delta &=& 1-D & y_q = -y_p &\implies& D \\
(x_q - x_p) \cdot ((x_q - x_p) \cdot \lambda - (y_q - y_p)) &=& 0 & x_q \neq x_p &\implies& \lambda = \frac{y_q - y_p}{x_q - x_p} \\
A \cdot \left(2y_p \cdot \lambda - 3{x_p}^2\right) &=& 0 & A \wedge y_p \neq 0 &\implies& \lambda = \frac{3{x_p}^2}{2y_p} \\
\\
(1-B) \cdot (1-C) \cdot (\lambda^2 - x_p - x_q - x_r) && & (¬B \wedge ¬C &\implies& x_r = \lambda^2 - x_p - x_q) \\
+ B \cdot (x_r - x_q) &=& 0 & \wedge (B &\implies& x_r = x_q) \\
\\
(1-B) \cdot (1-C) \cdot (\lambda \cdot (x_p - x_r) - y_p - y_r) && & (¬B \wedge ¬C &\implies& y_r = \lambda \cdot (x_p - x_r) - y_p) \\
+ B \cdot (y_r - y_q) &=& 0 & \wedge (B &\implies& y_r = y_q) \\
\\
      C \cdot (x_r - x_p) &=& 0 & C &\implies& x_r = x_p \\
      C \cdot (y_r - y_p) &=& 0 & C &\implies& y_r = y_p \\
              D \cdot x_r &=& 0 & D &\implies& x_r = 0 \\
              D \cdot y_r &=& 0 & D &\implies& y_r = 0 \\
\end{array}
$

Max degree: $4$

Note: It is the cross-interaction of the two $B$ constraints that fully constrain
the implications. For example, the contrapositive of the first constraint's implication
$x_p = 0 \implies B$ is $¬B \implies x_p \neq 0$, which is the other half of the
second constraint's implication. The same applies to $C$.
