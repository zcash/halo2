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
