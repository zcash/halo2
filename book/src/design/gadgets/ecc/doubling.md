# Point doubling

We will use formulae for curve arithmetic using affine coordinates on short Weierstrass curves,
derived from section 4.1 of [Hüseyin Hışıl's thesis](https://core.ac.uk/download/pdf/10898289.pdf).

- Input: $P = (x_p, y_p)$
- Output: $R = [2]P = (x_r, y_r)$

The formulae from Hışıl's thesis are:
- $x_3 = \left(\frac{3x_1^2}{2y_1}\right)^2 - 2x_1$
- $y_3 = \frac{3x_1^2}{2y_1} \cdot (x_1 - x_3) - y_1.$

Rename $(x_1, y_1)$ to $(x_p, y_p)$, and $(x_3, y_3)$ to $(x_r, y_r)$, giving

- $x_r = \left(\frac{3x_p^2}{2y_p}\right)^2 - 2x_p$
- $y_r = \frac{3x_p^2}{2y_p} \cdot (x_p - x_r) - y_p.$

which is equivalent to

- $x_r + 2x_p = \left(\frac{3x_p^2}{2y_p}\right)^2$
- $y_r + y_p = \frac{3x_p^2}{2y_p} \cdot (x_p - x_r).$

Assuming $y_p \neq 0$, we have

$
\begin{array}{lrrll}
&& x_r + 2x_p &=& \left(\frac{3x_p^2}{2y_p}\right)^2 \\[1.2ex]
&\Longleftrightarrow &(x_r + 2x_p) \cdot (2y_p)^2 &=& (3x_p^2)^2 \\[1ex]
&\Longleftrightarrow &(x_r + 2x_p) \cdot 4y_p^2 - 9x_p^4 &=& 0 \\[1.5ex]
\text{and} \\
&&y_r + y_p &=& \frac{3x_p^2}{2y_p} \cdot (x_p - x_r) \\[0.8ex]
&\Longleftrightarrow &(y_r + y_p) \cdot 2y_p &=& 3x_p^2 \cdot (x_p - x_r) \\[1ex]
&\Longleftrightarrow &(y_r + y_p) \cdot 2y_p - 3x_p^2 \cdot (x_p - x_r) &=& 0.
\end{array}
$

So we get the constraints:
- $(x_r + 2x_p) \cdot 4y_p^2 - 9x_p^4 = 0$
- $(y_r + y_p) \cdot 2y_p - 3x_p^2 \cdot (x_p - x_r) = 0.$

### Constraints <a name="doubling-constraints">
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
5 & q_\text{double} \cdot \left( (x_r + 2x_p) \cdot 4y_p^2 - 9x_p^4 = 0 \right) \\\hline
4 & q_\text{double} \cdot \left( (y_r + y_p) \cdot 2y_p - 3x_p^2 \cdot (x_p - x_r) = 0 \right) \\\hline
\end{array}
$$
