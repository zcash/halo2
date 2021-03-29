# Incomplete addition
Inputs: $P = (x_P, y_P), Q = (x_Q, y_Q)$
Output: $A = P + Q = (x_A, y_A)$

Formulae:
- $\lambda \cdot (x_p - x_{q}) = y_p - y_{q}$
- $x_{a} = \lambda^2 - x_{q} - x_p$
- $y_{a} = \lambda(x_{q} - x_{a}) - y_{q}$

Substituting for $\lambda$, we get the constraints:
- $(x_{a} + x_{q} + x_p) \cdot (x_p - x_q)^2 - (y_p - y_{q})^2 = 0$
- $(y_{a} + y_{q})(x_p - x_{q}) - (y_p - y_{q})(x_{q} - x_{a}) = 0$