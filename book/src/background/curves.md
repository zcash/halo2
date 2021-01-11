# Elliptic curves

Elliptic curves constructed over finite fields are another important cryptographic tool.
There are several ways to define the curve equation, but for our purposes, let
$\mathbb{F}_p$ be a large (255-bit) field, and then let the set of solutions $(x, y)$ to
$y^2 = x^3 + b$ for some constant $b$ define the $\mathbb{F}_p$-rational points on an
elliptic curve $E(\mathbb{F}_p)$. These $(x, y)$ coordinates are called "affine
coordinates". Each of the $\mathbb{F}_p$-rational points, together with a "point at
infinity" $\mathcal{O}$ that serves as the group identity, can be interpreted as an
element of a group. By convention, elliptic curve groups are written additively.

![](https://i.imgur.com/JvLS6yE.png)
*"Three points on a line sum to zero, which is the point at infinity."*

The group addition law is simple: to add two points together, find the line that
intersects both points and obtain the third point, and then negate its $y$-coordinate. The
case that a point is being added to itself, called point doubling, requires special
handling: we find the line tangent to the point, and then find the single other point that
intersects this line and then negate. Otherwise, in the event that a point is being
"added" to its negation, the result is the point at infinity.

The ability to add and double points naturally gives us a way to scale them by integers.
The number of points on the curve is known as the "group order". If this number is prime
$q$, we call the numbers that we scale curve points by "scalars" and consider that they
are all elements of a scalar field $\mathbb{F}_q$.

Elliptic curves, when properly designed, have an important security property. Given two
random elements $G, H \in E(\mathbb{F}_p)$ finding $a$ such that $[a] G = H$, otherwise
known as the discrete log of $H$ with respect to $G$, is considered computationally
infeasible with classical computers. This is called the elliptic curve discrete log
assumption.

## Curve arithmetic

### Point doubling

The simplest situation is doubling a point $(x_0, y_0)$. Continuing with our example
$y^2 = x^3 + b$, this is done first by computing the derivative
$$
\lambda = \frac{dy}{dx} = \frac{3x^2}{2y}.
$$

To obtain expressions for $(x_1, y_1) = (x_0, y_0) + (x_0, y_0),$ we consider 

$$
\begin{aligned}
\frac{-y_1 - y_0}{x_1 - x_0} = \lambda &\implies -y_1 = \lambda(x_1 - x_0) + y_0 \\
&\implies \boxed{y_1 = \lambda(x_0 - x_1) - y_0}.
\end{aligned}
$$

To get the expression for $x_1,$ we substitute $y = \lambda(x_0 - x) - y_0$ into the
elliptic curve equation:

$$
\begin{aligned}
y^2 = x^3 + b &\implies (\lambda(x_0 - x) - y_0)^2 = x^3 + b \\
&\implies x^3 - \lambda^2 x^2 + \cdots = 0 \leftarrow\text{(rearranging terms)} \\
&= (x - x_0)(x - x_0)(x - x_1) \leftarrow\text{(known roots $x_0, x_0, x_1$)} \\
&= x^3 - (x_0 + x_0 + x_1)x^2 + \cdots.
\end{aligned}
$$

Comparing coefficients for the $x^2$ term gives us
$\lambda^2 = x_0 + x_0 + x_1 \implies \boxed{x_1 = \lambda^2 - 2x_0}.$


### Projective coordinates
This unfortunately requires an expensive inversion of $2y$. We can avoid this by arranging
our equations to "defer" the computation of the inverse, since we often do not need the
actual affine $(x', y')$ coordinate of the resulting point immediately after an individual
curve operation. Let's introduce a third coordinate $Z$ and scale our curve equation by
$Z^3$ like so:

$$
Z^3 y^2 = Z^3 x^3 + Z^3 b
$$

Our original curve is just this curve at the restriction $Z = 1$. If we allow the affine
point $(x, y)$ to be represented by $X = xZ$, $Y = yZ$ and $Z \neq 0$ then we have the
[homogenous projective curve](https://en.wikipedia.org/wiki/Homogeneous_coordinates)

$$
Y^2 Z = X^3 + Z^3 b.
$$

Obtaining $(x, y)$ from $(X, Y, Z)$ is as simple as computing $(X/Z, Y/Z)$ when
$Z \neq 0$. (When $Z = 0,$ we are dealing with the point at infinity $O := (0:1:0)$.) In
this form, we now have a convenient way to defer the inversion required by doubling a
point. The general strategy is to express $x', y'$ as rational functions using $x = X/Z$
and $y = Y/Z$, rearrange to make their denominators the same, and then take the resulting
point $(X, Y, Z)$ to have $Z$ be the shared denominator and $X = x'Z, Y = y'Z$.

> Projective coordinates are often, but not always, more efficient than affine
> coordinates. There may be exceptions to this when either we have a different way to
> apply Montgomery's trick, or when we're in the circuit setting where multiplications and
> inversions are about equally as expensive (at least in terms of circuit size).

The following shows an example of doubling a point $(X, Y, Z) = (xZ, yZ, Z)$ without an
inversion. Substituting with $X, Y, Z$ gives us
$$
\lambda = \frac{3x^2}{2y} = \frac{3(X/Z)^2}{2(Y/Z)} = \frac{3 X^2}{2YZ}
$$

and gives us
$$
\begin{aligned}
x' &= \lambda^2 - 2x \\
&= \lambda^2 - \frac{2X}{Z} \\
&= \frac{9 X^4}{4Y^2Z^2} - \frac{2X}{Z} \\
&= \frac{9 X^4 - 8XY^2Z}{4Y^2Z^2} \\
&= \frac{18 X^4 Y Z - 16XY^3Z^2}{8Y^3Z^3} \\
\\
y' &= \lambda (x - x') - y \\
&= \lambda (\frac{X}{Z} - \frac{9 X^4 - 8XY^2Z}{4Y^2Z^2}) - \frac{Y}{Z} \\
&= \frac{3 X^2}{2YZ} (\frac{X}{Z} - \frac{9 X^4 - 8XY^2Z}{4Y^2Z^2}) - \frac{Y}{Z} \\
&= \frac{3 X^3}{2YZ^2} - \frac{27 X^6 - 24X^3Y^2Z}{8Y^3Z^3} - \frac{Y}{Z} \\
&= \frac{12 X^3Y^2Z - 8Y^4Z^2 - 27 X^6 + 24X^3Y^2Z}{8Y^3Z^3}
\end{aligned}
$$

Notice how the denominators of $x'$ and $y'$ are the same. Thus, instead of computing
$(x', y')$ we can compute $(X, Y, Z)$ with $Z = 8Y^3Z^3$ and $X, Y$ set to the
corresponding numerators such that $X/Z = x'$ and $Y/Z = y'$. This completely avoids the
need to perform an inversion when doubling, and something analogous to this can be done
when adding two distinct points.

### TODO: Point addition
$$
\begin{aligned}
P + Q &= R\\
(x_p, y_p) + (x_q, y_q) &= (x_r, y_r) \\
\lambda &= \frac{y_q - y_p}{x_q - x_p} \\
x_r &= \lambda^2 - x_p - x_q \\
y_r &= \lambda(x_p - x_r) - y_p
\end{aligned}
$$

----------

Important notes:

* There exist efficient formulae[^complete-formulae] for point addition that do not have
  edge cases (so-called "complete" formulae) and that unify the addition and doubling
  cases together. The result of adding a point to its negation using those formulae
  produces $Z = 0$, which represents the point at infinity.
* In addition, there are other models like the Jacobian representation where
  $(x, y) = (xZ^2, yZ^3, Z)$ where the curve is rescaled by $Z^6$ instead of $Z^3$, and
  this representation has even more efficient arithmetic but no unified/complete formulae.
* We can easily compare two curve points $(X_1, Y_1, Z_1)$ and $(X_2, Y_2, Z_2)$ for
  equality in the homogenous projective coordinate space by "homogenizing" their
  Z-coordinates; the checks become $X_1 Z_2 = X_2 Z_1$ and $Y_1 Z_2 = Y_2 Z_1$.

## Curve endomorphisms

Imagine that $\mathbb{F}_p$ has a primitive cube root of unity, or in other words that
$3 | p - 1$ and so an element $\zeta_p$ generates a $3$-order multiplicative subgroup.
Notice that a point $(x, y)$ on our example elliptic curve $y^2 = x^3 + b$ has two cousin
points: $(\zeta_p x, \zeta_p^2 x)$, because the computation $x^3$ effectively kills the
$\zeta$ component of the $x$-coordinate. Applying the map $(x, y) \mapsto (\zeta_p x, y)$
is an application of an endomorphism over the curve. The exact mechanics involved are
complicated, but when the curve has a prime $q$ number of points (and thus a prime
"order") the effect of the endomorphism is to multiply the point by a scalar in
$\mathbb{F}_q$ which is also a primitive cube root $\zeta_q$ in the scalar field.

## Curve point compression
TODO

## Cycles of curves
Let $E_p$ be an elliptic curve over a finite field $\mathbb{F}_p,$ where $p$ is a prime.
We denote this by $E_p/\mathbb{F}_p.$ and we denote the group of points of $E_p$ over
$\mathbb{F}_p,$ with order $q = \#E(\mathbb{F}_p).$ For this curve, we call $\mathbb{F}_p$
the "base field" and  $\mathbb{F}_q$ the "scalar field".

We instantiate our proof system over the elliptic curve $E_p/\mathbb{F}_p$. This allows us
to prove statements about $\mathbb{F}_q$-arithmetic circuit satisfiability.

> **(aside) If our curve $E_p$ is over $\mathbb{F}_p,$ why is the arithmetic circuit instead in $\mathbb{F}_q$?**
> The proof system is basically working on encodings of the scalars in the circuit (or
> more precisely, commitments to polynomials whose coefficients are scalars). The scalars
> are in $\mathbb{F}_q$ when their encodings/commitments are elliptic curve points in
> $E_p/\mathbb{F}_p$.

However, most of the verifier's arithmetic computations are over the base field
$\mathbb{F}_p,$ and are thus efficiently expressed as an $\mathbb{F}_p$-arithmetic
circuit.

> **(aside) Why are the verifier's computations (mainly) over $\mathbb{F}_p$?**
> The Halo 2 verifier actually has to perform group operations using information output by
> the circuit. Group operations like point doubling and addition use arithmetic in
> $\mathbb{F}_p$, because the coordinates of points are in $\mathbb{F}_p.$ 

This motivates us to construct another curve with scalar field $\mathbb{F}_p$, which has
an $\mathbb{F}_p$-arithmetic circuit that can efficiently verify proofs from the first
curve. As a bonus, if this second curve had base field $E_q/\mathbb{F}_q,$ it would
generate proofs that could be efficiently verified in the first curve's
$\mathbb{F}_q$-arithmetic circuit. In other words, we instantiate a second proof system
over $E_q/\mathbb{F}_q,$ forming a 2-cycle with the first:

![](https://i.imgur.com/bNMyMRu.png)

### TODO: Pallas-Vesta curves
Reference: https://github.com/zcash/pasta

## Hashing to curves

Sometimes it is useful to be able to produce a random point on an elliptic curve
$E_p/\mathbb{F}_p$ corresponding to some input, in such a way that no-one will know its
discrete logarithm (to any other base).

This is described in detail in the [Internet draft on Hashing to Elliptic Curves][cfrg-hash-to-curve].
Several algorithms can be used depending on efficiency and security requirements. The
framework used in the Internet Draft makes use of several functions:

* ``hash_to_field``: takes a byte sequence input and maps it to a element in the base
  field $\mathbb{F}_p$
* ``map_to_curve``: takes an $\mathbb{F}_p$ element and maps it to $E_p$.

[cfrg-hash-to-curve]: https://datatracker.ietf.org/doc/draft-irtf-cfrg-hash-to-curve/?include_text=1

### TODO: Simplified SWU
Reference: https://eprint.iacr.org/2019/403.pdf

## References
[^complete-formulae]: Renes, J., Costello, C., & Batina, L. (2016, May). "Complete addition formulas for prime order elliptic curves." In Annual International Conference on the Theory and Applications of Cryptographic Techniques (pp. 403-428). Springer, Berlin, Heidelberg. https://eprint.iacr.org/2015/1060.pdf
