# Tips and tricks

This section contains various ideas and snippets that you might find useful while writing
halo2 circuits.

## Small range constraints

A common constraint used in R1CS circuits is the boolean constraint: $b * (1 - b) = 0$.
This constraint can only be satisfied by $b = 0$ or $b = 1$.

In halo2 circuits, you can similarly constrain a cell to have one of a small set of
values. For example, to constrain $a$ to the range $[0..5]$, you would create a gate of
the form:

$$a \cdot (1 - a) \cdot (2 - a) \cdot (3 - a) \cdot (4 - a) = 0$$

while to constraint $c$ to be either 7 or 13, you would use:

$$(7 - c) \cdot (13 - c) = 0$$

> The underlying principle here is that we create a polynomial constraint with roots at
> each value in the set of possible values we want to allow. In R1CS circuits, the maximum
> supported polynomial degree is 2 (due to all constraints being of the form $a * b = c$).
> In halo2 circuits, you can use arbitrary-degree polynomials - with the proviso that
> higher-degree constraints are more expensive to use.

Note that the roots don't have to be constants; for example $(a - x) \cdot (a - y) \cdot (a - z) = 0$ will constrain $a$ to be equal to one of $\{ x, y, z \}$ where the latter can be arbitrary polynomials, as long as the whole expression stays within the maximum degree bound.

## Small set interpolation
We can use Lagrange interpolation to create a polynomial constraint that maps
$f(X) = Y$ for small sets of $X \in \{x_i\}, Y \in \{y_i\}$. 

For instance, say we want to map a 2-bit value to a "spread" version interleaved
with zeros. We first precompute the evaluations at each point:

$$
\begin{array}{rcl}
00 \rightarrow 0000 &\implies& 0 \rightarrow 0 \\
01 \rightarrow 0001 &\implies& 1 \rightarrow 1 \\
10 \rightarrow 0100 &\implies& 2 \rightarrow 4 \\
11 \rightarrow 0101 &\implies& 3 \rightarrow 5
\end{array}
$$

Then, we construct the Lagrange basis polynomial for each point using the
identity:
$$\mathcal{l}_j(X) = \prod_{0 \leq m < k,\; m \neq j} \frac{x - x_m}{x_j - x_m},$$
where $k$ is the number of data points. ($k = 4$ in our example above.)

Recall that the Lagrange basis polynomial $\mathcal{l}_j(X)$ evaluates to $1$ at
$X = x_j$ and $0$ at all other $x_i, j \neq i.$

Continuing our example, we get four Lagrange basis polynomials:

$$
\begin{array}{ccc}
l_0(X) &=& \frac{(X - 3)(X - 2)(X - 1)}{(-3)(-2)(-1)} \\[1ex]
l_1(X) &=& \frac{(X - 3)(X - 2)(X)}{(-2)(-1)(1)} \\[1ex]
l_2(X) &=& \frac{(X - 3)(X - 1)(X)}{(-1)(1)(2)} \\[1ex]
l_3(X) &=& \frac{(X - 2)(X - 1)(X)}{(1)(2)(3)}
\end{array}
$$

Our polynomial constraint is then

$$
\begin{array}{cccccccccccl}
&f(0) \cdot l_0(X) &+& f(1) \cdot l_1(X) &+& f(2) \cdot l_2(X) &+& f(3) \cdot l_3(X) &-& f(X) &=& 0 \\
\implies& 0 \cdot l_0(X) &+& 1 \cdot l_1(X) &+& 4 \cdot l_2(X) &+& 5 \cdot l_3(X) &-& f(X) &=& 0. \\
\end{array}
$$
