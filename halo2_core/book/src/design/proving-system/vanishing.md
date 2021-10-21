# Vanishing argument

Having committed to the circuit assignments, the prover now needs to demonstrate that the
various circuit relations are satisfied:

- The custom gates, represented by polynomials $\text{gate}_i(X)$.
- The rules of the lookup arguments.
- The rules of the equality constraint permutations.

Each of these relations is represented as a polynomial of degree $d$ (the maximum degree
of any of the relations) with respect to the circuit columns. Given that the degree of the
assignment polynomials for each column is $n - 1$, the relation polynomials have degree
$d(n - 1)$ with respect to $X$.

> In our [example](../proving-system.md#example), these would be the gate polynomials, of
> degree $3n - 3$:
>
> - $\text{gate}_0(X) = a_0(X) \cdot a_1(X) \cdot a_2(X \omega^{-1}) - a_3(X)$
> - $\text{gate}_1(X) = f_0(X \omega^{-1}) \cdot a_2(X)$
> - $\text{gate}_2(X) = f_0(X) \cdot a_3(X) \cdot a_0(X)$

A relation is satisfied if its polynomial is equal to zero. One way to demonstrate this is
to divide each polynomial relation by the vanishing polynomial $t(X) = (X^n - 1)$, which
is the lowest-degree monomial that has roots at every $\omega^i$. If relation's polynomial
is perfectly divisible by $t(X)$, it is equal to zero over the domain (as desired).

This simple construction would require a polynomial commitment per relation. Instead, we
commit to all of the circuit relations simultaneously: the verifier samples $y$, and then
the prover constructs the quotient polynomial

$$h(X) = \frac{\text{gate}_0(X) + y \cdot \text{gate}_1(X) + \dots + y^i \cdot \text{gate}_i(X) + \dots}{t(X)},$$

where the numerator is a random (the prover commits to the cell assignments before the
verifier samples $y$) linear combination of the circuit relations.

- If the numerator polynomial (in formal indeterminate $X$) is perfectly divisible by
  $t(X)$, then with high probability all relations are satisfied.
- Conversely, if at least one relation is not satisfied, then with high probability
  $h(x) \cdot t(x)$ will not equal the evaluation of the numerator at $x$. In this case,
  the numerator polynomial would not be perfectly divisible by $t(X)$.

## Committing to $h(X)$

$h(X)$ has degree $(d - 1)n - d$ (because the divisor $t(X)$ has degree $n$). However, the
polynomial commitment scheme we use for Halo 2 only supports committing to polynomials of
degree $n - 1$ (which is the maximum degree that the rest of the protocol needs to commit
to). Instead of increasing the cost of the polynomial commitment scheme, the prover split
$h(X)$ into pieces of degree $n - 1$

$$h_0(X) + X^n h_1(X) + \dots + X^{n(d-1)} h_{d-1}(X),$$

and produces blinding commitments to each piece

$$\mathbf{H} = [\text{Commit}(h_0(X)), \text{Commit}(h_1(X)), \dots, \text{Commit}(h_{d-1}(X))].$$

## Evaluating the polynomials

At this point, all properties of the circuit have been committed to. The verifier now
wants to see if the prover committed to the correct $h(X)$ polynomial. The verifier
samples $x$, and the prover produces the purported evaluations of the various polynomials
at $x$, for all the relative offsets used in the circuit, as well as $h(X)$.

> In our [example](../proving-system.md#example), this would be:
>
> - $a_0(x)$
> - $a_1(x)$
> - $a_2(x)$, $a_2(x \omega^{-1})$
> - $a_3(x)$
> - $f_0(x)$, $f_0(x \omega^{-1})$
> - $h_0(x)$, ..., $h_{d-1}(x)$

The verifier checks that these evaluations satisfy the form of $h(X)$:

$$\frac{\text{gate}_0(x) + \dots + y^i \cdot \text{gate}_i(x) + \dots}{t(x)} = h_0(x) + \dots + x^{n(d-1)} h_{d-1}(x)$$

Now content that the evaluations collectively satisfy the gate constraints, the verifier
needs to check that the evaluations themselves are consistent with the original
[circuit commitments](circuit-commitments.md), as well as $\mathbf{H}$. To implement this
efficiently, we use a [multipoint opening argument](multipoint-opening.md).
