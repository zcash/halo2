# Witnessing points

We represent elliptic curve points in the circuit in their affine representation $(x, y)$.
The identity is represented as the pseudo-coordinate $(0, 0)$, which we
[assume](../ecc.md#chip-assumptions) is not a valid point on the curve.

## Non-identity points

To constrain a coordinate pair $(x, y)$ as representing a valid point on the curve, we
directly check the curve equation. For Pallas and Vesta, this is:

$$y^2 = x^3 + 5$$

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
4 & q_\text{point}^\text{non-id} \cdot (y^2 - x^3 - 5) = 0 \\\hline
\end{array}
$$

## Points including the identity

To allow $(x, y)$ to represent either a valid point on the curve, or the pseudo-coordinate
$(0, 0)$, we define a separate gate that enforces the curve equation check unless both $x$
and $y$ are zero.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
5 & (q_\text{point} \cdot x) \cdot (y^2 - x^3 - 5) = 0 \\\hline
5 & (q_\text{point} \cdot y) \cdot (y^2 - x^3 - 5) = 0 \\\hline
\end{array}
$$
