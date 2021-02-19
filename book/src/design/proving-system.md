# Proving system

The Halo 2 proving system can be broken down into five stages:

1. Commit to polynomials encoding the main components of the circuit:
   - Cell assignments.
   - Permuted values and products for each lookup argument.
   - Equality constraint permutations.
2. Construct the vanishing argument to constrain all circuit relations to zero:
   - Standard and custom gates.
   - Lookup argument rules.
   - Equality constraint permutation rules.
3. Evaluate the above polynomials at all necessary points:
   - All relative rotations used by custom gates across all columns.
   - Vanishing argument pieces.
4. Construct the multipoint opening argument to check that all evaluations are consistent
   with their respective commitments.
5. Run the inner product argument to create a polynomial commitment opening proof for the
   multipoint opening argument polynomial.

These stages are presented in turn across this section of the book.

## Example

To aid our explanations, we will at times refer to the following example constraint
system:

- Four advice columns $a, b, c, d$.
- One fixed column $f$.
- Three custom gates:
  - $a \cdot b \cdot c_{-1} - d = 0$
  - $f_{-1} \cdot c = 0$
  - $f \cdot d \cdot a = 0$

## tl;dr

The table below provides a (probably too) succinct description of the Halo 2 protocol.
This description will likely be replaced by the Halo 2 paper and security proof, but for
now serves as a summary of the following sub-sections.

| Prover                                                                      |         | Verifier                           |
| --------------------------------------------------------------------------- | ------- | ---------------------------------- |
|                                                                             | $\larr$ | $t(X) = (X^n - 1)$                 |
|                                                                             | $\larr$ | $F = [F_0, F_1, \dots, F_{m - 1}]$ |
| $\mathbf{A} = [A_0, A_1, \dots, A_{m - 1}]$                                 | $\rarr$ |                                    |
|                                                                             | $\larr$ | $\theta$                           |
| $\mathbf{L} = [(A'_0, S'_0), \dots, (A'_{m - 1}, S'_{m - 1})]$              | $\rarr$ |                                    |
|                                                                             | $\larr$ | $\beta, \gamma$                    |
| $\mathbf{Z_P} = [Z_{P,0}, Z_{P,1}, \ldots]$                                 | $\rarr$ |                                    |
| $\mathbf{Z_L} = [Z_{L,0}, Z_{L,1}, \ldots]$                                 | $\rarr$ |                                    |
|                                                                             | $\larr$ | $y$                                |
| $h(X) = \frac{\text{gate}_0(X) + \dots + y^i \cdot \text{gate}_i(X)}{t(X)}$ |         |                                    |
| $h(X) = h_0(X) + \dots + X^{n(d-1)} h_{d-1}(X)$                             |         |                                    |
| $\mathbf{H} = [H_0, H_1, \dots, H_{d-1}]$                                   | $\rarr$ |                                    |
|                                                                             | $\larr$ | $x$                                |
| $evals = [A_0(x), \dots, H_{d - 1}(x)]$                                     | $\rarr$ |                                    |
|                                                                             |         | Checks $h(x)$                      |
|                                                                             | $\larr$ | $x_1, x_2$                         |
| Constructs $h'(X)$ multipoint opening poly                                  |         |                                    |
| $U = \text{Commit}(h'(X))$                                                  | $\rarr$ |                                    |
|                                                                             | $\larr$ | $x_3$                              |
| $\mathbf{q}_\text{evals} = [Q_0(x_3), Q_1(x_3), \dots]$                     | $\rarr$ |                                    |
| $u_\text{eval} = U(x_3)$                                                    | $\rarr$ |                                    |
|                                                                             | $\larr$ | $x_4$                              |

Then the prover and verifier:

- Construct $\text{finalPoly}(X)$ as a linear combination of $\mathbf{Q}$ and $U$ using
  powers of $x_4$;
- Construct $\text{finalPolyEval}$ as the equivalent linear combination of
  $\mathbf{q}_\text{evals}$ and $u_\text{eval}$; and
- Perform $\text{InnerProduct}(\text{finalPoly}(X), x_3, \text{finalPolyEval}).$

> TODO: Write up protocol components that provide zero-knowledge.
