# [WIP] UltraPLONK arithmetisation
We work over a multiplicative subgroup
$\mathcal{H} =\{1, \omega, \omega^2, \cdots, \omega^{n-1}\},$ where $\omega$ is primitive
root of unity, in the Lagrange basis corresponding to these points.

## Polynomial rules
A polynomial rule defines a constraint that must hold between its specified columns at
every row (i.e. at every point in the multiplicative subgroup).

e.g.

```text
a * sa + b * sb + a * b * sm + c * sc + PI = 0 
```

## Columns
- **fixed (i.e. "selector") columns**: fixed for all instances of a particular circuit.
  These columns toggle parts of a polynomial rule "on" or "off" to form a "custom gate".
- **advice columns**: variable values assigned in each instance of the circuit.
  Corresponds to the prover's secret witness.
- **public input**: like advice columns, but publicly known values.

Each column is a vector of $n$ values, e.g. $\mathbf{a} = [a_0, a_1, \cdots, a_{n-1}]$. We
can think of the vector as the evaluation form of the column polynomial
$a(X), X \in \mathcal{H}.$ To recover the coefficient form, we can use
[Lagrange interpolation](polynomials.md#lagrange-interpolation), such that
$a(\omega^i) = a_i.$

## Copy constraints
- Define permutation between a set of columns, e.g. $\sigma(a, b, c)$
- Copy specific cells between these columns, e.g. $b_1 = c_0$
- Construct permuted columns which should evaluate to same value as original columns

## Permutation grand product
$$Z(\omega^i) := \prod_{0 \leq j \leq i} \frac{C_k(\omega^j) + \beta\delta^k \omega^j + \gamma}{C_k(\omega^j) + \beta S_k(\omega^j) + \gamma},$$
where $i = 0, \cdots, n-1$ indexes over the size of the multiplicative subgroup, and
$k = 0, \cdots, m-1$ indexes over the advice columns involved in the permutation. This is
a running product, where each term includes the cumulative product of the terms before it.

> TODO: what is $\delta$? keep columns linearly independent

Check the constraints:

1. First term is equal to one
   $$\mathcal{L}_0(X) \cdot (1 - Z(X)) = 0$$

2. Running product is well-constructed. For each row, we check that this holds:
   $$Z(\omega^i) \cdot{(C(\omega^i) + \beta S_k(\omega^i) + \gamma)} - Z(\omega^{i-1}) \cdot{(C(\omega^i) + \delta^k \beta \omega^i + \gamma)} = 0$$
   Rearranging gives 
   $$Z(\omega^i) = Z(\omega^{i-1}) \frac{C(\omega^i) + \beta\delta^k \omega^i + \gamma}{C(\omega^i) + \beta S_k(\omega^i) + \gamma},$$
   which is how we defined the grand product polynomial in the first place.

### Lookup
Reference: [Generic Lookups with PLONK (DRAFT)](/LTPc5f-3S0qNF6MtwD-Tdg?view)

### Vanishing argument
We want to check that the expressions defined by the gate constraints, permutation
constraints and loookup constraints evaluate to zero at all points in the multiplicative
subgroup. To do this, the prover collapses all the expressions into one polynomial 
$$H(X) = \sum_{i=0}^e y^i E_i(X),$$
where $e$ is the number of expressions and $y$ is a random challenge used to keep the
constraints linearly independent. The prover then divides this by the vanishing polynomial
(see section: [Vanishing polynomial](polynomials.md#vanishing-polynomial)) and commits to
the resulting quotient

$$\text{Commit}(Q(X)), \text{where } Q(X) = \frac{H(X)}{Z_H(X)}.$$

The verifier responds with a random evaluation point $x,$ to which the prover replies with
the claimed evaluations $q = Q(x), \{e_i\}_{i=0}^e = \{E_i(x)\}_{i=0}^e.$ Now, all that
remains for the verifier to check is that the evaluations satisfy

$$q \stackrel{?}{=} \frac{\sum_{i=0}^e y^i e_i}{Z_H(x)}.$$

Notice that we have yet to check that the committed polynomials indeed evaluate to the
claimed values at
$x, q \stackrel{?}{=} Q(x), \{e_i\}_{i=0}^e \stackrel{?}{=} \{E_i(x)\}_{i=0}^e.$
This check is handled by the polynomial commitment scheme (described in the next section).
