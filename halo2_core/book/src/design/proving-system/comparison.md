# Comparison to other work

## BCMS20 Appendix A.2

Appendix A.2 of [BCMS20] describes a polynomial commitment scheme that is similar to the
one described in [BGH19] (BCMS20 being a generalization of the original Halo paper). Halo
2 builds on both of these works, and thus itself uses a polynomial commitment scheme that
is very similar to the one in BCMS20.

[BGH19]: https://eprint.iacr.org/2019/1021
[BCMS20]: https://eprint.iacr.org/2020/499

The following table provides a mapping between the variable names in BCMS20, and the
equivalent objects in Halo 2 (which builds on the nomenclature from the Halo paper):

|     BCMS20     |       Halo 2        |
| :------------: | :-----------------: |
|      $S$       |         $H$         |
|      $H$       |         $U$         |
|      $C$       |    `msm` or $P$     |
|    $\alpha$    |       $\iota$       |
|    $\xi_0$     |         $z$         |
|    $\xi_i$     |    `challenge_i`    |
|      $H'$      |       $[z] U$       |
|   $\bar{p}$    |      `s_poly`       |
| $\bar{\omega}$ |   `s_poly_blind`    |
|   $\bar{C}$    | `s_poly_commitment` |
|     $h(X)$     |       $g(X)$        |
|   $\omega'$    |   `blind` / $\xi$   |
|  $\mathbf{c}$  |    $\mathbf{a}$     |
|      $c$       | $a = \mathbf{a}_0$  |
|      $v'$      |        $ab$         |

Halo 2's polynomial commitment scheme differs from Appendix A.2 of BCMS20 in two ways:

1. Step 8 of the $\text{Open}$ algorithm computes a "non-hiding" commitment $C'$ prior to
   the inner product argument, which opens to the same value as $C$ but is a commitment to
   a randomly-drawn polynomial. The remainder of the protocol involves no blinding. By
   contrast, in Halo 2 we blind every single commitment that we make (even for instance
   and fixed polynomials, though using a blinding factor of 1 for the fixed polynomials);
   this makes the protocol simpler to reason about. As a consequence of this, the verifier
   needs to handle the cumulative blinding factor at the end of the protocol, and so there
   is no need to derive an equivalent to $C'$ at the start of the protocol.

   - $C'$ is also an input to the random oracle for $\xi_0$; in Halo 2 we utilize a
     transcript that has already committed to the equivalent components of $C'$ prior to
     sampling $z$.

2. The $\text{PC}_\text{DL}.\text{SuccinctCheck}$ subroutine (Figure 2 of BCMS20) computes
   the initial group element $C_0$ by adding $[v] H' = [v \epsilon] H$, which requires two
   scalar multiplications. Instead, we subtract $[v] G_0$ from the original commitment $P$,
   so that we're effectively opening the polynomial at the point to the value zero. The
   computation $[v] G_0$ is more efficient in the context of recursion because $G_0$ is a
   fixed base (so we can use lookup tables).
