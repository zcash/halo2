# Elliptic Curves

## `EccChip`

`halo2_gadgets` provides a chip that implements `EccInstructions` using 10 advice columns.
The chip is currently restricted to the Pallas curve, but will be extended to support the
[Vesta curve](https://github.com/zcash/halo2/issues/578) in the near future.

### Chip assumptions

A non-exhaustive list of assumptions made by `EccChip`:
- $0$ is not an $x$-coordinate of a valid point on the curve.
  - Holds for Pallas because $5$ is not square in $\mathbb{F}_q$.
- $0$ is not a $y$-coordinate of a valid point on the curve.
  - Holds for Pallas because $-5$ is not a cube in $\mathbb{F}_q$.

### Layout

The following table shows how columns are used by the gates for various chip sub-areas:

- $W$ - witnessing points.
- $AI$ - incomplete point addition.
- $AC$ - complete point addition.
- $MF$ - Fixed-base scalar multiplication.
- $MVI$ - variable-base scalar multiplication, incomplete rounds.
- $MVC$ - variable-base scalar multiplication, complete rounds.
- $MVO$ - variable-base scalar multiplication, overflow check.

$$
\begin{array}{|c||c|c|c|c|c|c|c|c|c|c|}
\hline
\text{Sub-area} & a_0 & a_1 & a_2 & a_3 & a_4 & a_5 & a_6 & a_7 & a_8 & a_9 \\\hline
\hline
 W  &  x  &  y  \\\hline
\hline
 AI & x_p & y_p & x_q &  y_q  \\\hline
    &     &     & x_r &  y_r  \\\hline
\hline
 AC & x_p & y_p & x_q &  y_q  &    \lambda     &     \alpha     & \beta  &  \gamma  &     \delta     &                \\\hline
    &     &     & x_r &  y_r  \\\hline
\hline
 MF & x_p & y_p & x_q &  y_q  & \text{window}  & u \\\hline
    &     &     & x_r &  y_r  \\\hline
\hline
MVI & x_p & y_p & \lambda_2^{lo} & x_A^{hi} & \lambda_1^{hi} & \lambda_2^{hi} & z^{lo} & x_A^{lo} & \lambda_1^{lo} & z^{hi}       \\\hline
\hline
MVC & x_p & y_p &      x_q       &   y_q    &    \lambda     &     \alpha     & \beta  &  \gamma  &     \delta     & z^{complete} \\\hline
    &     &     &      x_r       &   y_r    \\\hline
\end{array}
$$
