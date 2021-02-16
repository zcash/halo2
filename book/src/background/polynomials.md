# Polynomials

Let $A(X)$ be a polynomial over $\mathbb{F}_p$ with formal indeterminate $X$. As an example,

$$
A(X) = a_0 + a_1 X + a_2 X^2 + a_3 X^3
$$

defines a degree-$3$ polynomial. $a_0$ is referred to as the constant term. Polynomials of
degree $n-1$ have $n$ coefficients. We will often want to compute the result of replacing
the formal indeterminate $X$ with some concrete value $x$, which we denote by $A(x)$.

> In mathematics this is commonly referred to as "evaluating $A(X)$ at a point $x$".
> The word "point" here stems from the geometrical usage of polynomials in the form
> $y = A(x)$, where $(x, y)$ is the coordinate of a point in two-dimensional space.
> However, the polynomials we deal with are almost always constrained to equal zero, and
> $x$ will be an [element of some field](fields.md). This should not be confused
> with points on an [elliptic curve](curves.md), which we also make use of, but never in
> the context of polynomial evaluation.

Important notes:

* Multiplication of polynomials produces a product polynomial that is the sum of the
  degrees of its factors. Polynomial division subtracts from the degree.
  $$\deg(A(X)B(X)) = \deg(A(X)) + \deg(B(X)),$$
  $$\deg(A(X)/B(X)) = \deg(A(X)) -\deg(B(X)).$$
* Given a polynomial $A(X)$ of degree $n-1$, if we obtain $n$ evaluations of the
  polynomial at distinct points then these evaluations perfectly define the polynomial. In
  other words, given these evaluations we can obtain a unique polynomial $A(X)$ of degree
  $n-1$ via polynomial interpolation.
* $[a_0, a_1, \cdots, a_{n-1}]$ is the **coefficient representation** of the polynomial
  $A(X)$. Equivalently, we could use its **evaluation representation**
  $$[(x_0, A(x_0)), (x_1, A(x_1)), \cdots, (x_{n-1}, A(x_{n-1}))]$$
  at $n$ distinct points. Either representation uniquely specifies the same polynomial.

> #### (aside) Horner's rule
> Horner's rule allows for efficient evaluation of a polynomial of degree $n-1$, using
> only $n-1$ multiplications and $n-1$ additions. It is the following identity:
> $$\begin{aligned}a_0 &+ a_1X + a_2X^2 + \cdots + a_{n-1}X^{n-1} \\ &= a_0 + X\bigg( a_1 + X \Big( a_2 + \cdots + X(a_{n-2} + X a_{n-1}) \Big)\!\bigg),\end{aligned}$$

## Fast Fourier Transform (FFT)
The FFT is an efficient way of converting between the coefficient and evaluation
representations of a polynomial. It evaluates the polynomial at the $n$th roots of unity
$\{\omega^0, \omega^1, \cdots, \omega^{n-1}\},$ where $\omega$ is a primitive $n$th root
of unity. By exploiting symmetries in the roots of unity, each round of the FFT reduces
the evaluation into a problem only half the size. Most commonly we use polynomials of
length some power of two, $n = 2^k$, and apply the halving reduction recursively.

### Motivation: Fast polynomial multiplication
In the coefficient representation, it takes $O(n^2)$ operations to multiply two
polynomials $A(X)\cdot B(X) = C(X)$:

$$
\begin{aligned}
A(X) &= a_0 + a_1X + a_2X^2 + \cdots + a_{n-1}X^{n-1}, \\
B(X) &= b_0 + b_1X + b_2X^2 + \cdots + b_{n-1}X^{n-1}, \\
C(X) &= a_0\cdot (b_0 + b_1X + b_2X^2 + \cdots + b_{n-1}X^{n-1}) \\
&+ a_1X\cdot (b_0 + b_1X + b_2X^2 + \cdots + b_{n-1}X^{n-1})\\
&+ \cdots \\
&+ a_{n-1}X^{n-1} \cdot (b_0 + b_1X + b_2X^2 + \cdots + b_{n-1}X^{n-1}),
\end{aligned}
$$

where each of the $n$ terms in the first polynomial has to be multiplied by the $n$ terms
of the second polynomial.

In the evaluation representation, however, polynomial multiplication only requires $O(n)$
operations:

$$
\begin{aligned}
A&: \{(x_0, A(x_0)), (x_1, A(x_1)), \cdots, (x_{n-1}, A(x_{n-1}))\}, \\
B&: \{(x_0, B(x_0)), (x_1, B(x_1)), \cdots, (x_{n-1}, B(x_{n-1}))\}, \\
C&: \{(x_0, A(x_0)B(x_0)), (x_1, A(x_1)B(x_1)), \cdots, (x_{n-1}, A(x_{n-1})B(x_{n-1}))\},
\end{aligned}
$$

where each evaluation is multiplied pointwise.

This suggests the following strategy for fast polynomial multiplication:

1. Evaluate polynomials at all $n$ points;
2. Perform fast pointwise multiplication in the evaluation representation ($O(n)$);
3. Convert back to the coefficient representation.

The challenge now is how to **evaluate** and **interpolate** the polynomials efficiently.
Naively, evaluating a polynomial at $n$ points would require $O(n^2)$ operations (we use
the $O(n)$ Horner's rule at each point):

$$
\begin{bmatrix}
A(1) \\
A(\omega) \\
A(\omega^2) \\
\vdots \\
A(\omega^{n-1})
\end{bmatrix} =
\begin{bmatrix}
1&1&1&\dots&1 \\
1&\omega&\omega^2&\dots&\omega^{n-1} \\
1&\omega^2&\omega^{2\cdot2}&\dots&\omega^{2\cdot(n-1)} \\
\vdots&\vdots&\vdots& &\vdots \\
1&\omega^{n-1}&\omega^{2(n-1)}&\cdots&\omega^{(n-1)^2}\\
\end{bmatrix} \cdot
\begin{bmatrix}
a_0 \\
a_1 \\
a_2 \\
\vdots \\
a_{n-1}
\end{bmatrix}.
$$

For convenience, we will denote the matrices above as:
$$\hat{\mathbf{A}} = \mathbf{V}_\omega \cdot \mathbf{A}. $$

($\hat{\mathbf{A}}$ is known as the *Discrete Fourier Transform* of $\mathbf{A}$;
$\mathbf{V}_\omega$ is also called the *Vandermonde matrix*.)

### The (radix-2) Cooley-Tukey algorithm
Our strategy is to divide a DFT of size $n$ into two interleaved DFTs of size $n/2$. Given
the polynomial $A(X) = a_0 + a_1X + a_2X^2 + \cdots + a_{n-1}X^{n-1},$ we split it up into
even and odd terms:

$$
\begin{aligned}
A_{\text{even}} &= a_0 + a_2X + \cdots + a_{n-2}X^{\frac{n}{2} - 1}, \\
A_{\text{odd}} &= a_1 + a_3X + \cdots + a_{n-1}X^{\frac{n}{2} - 1}. \\
\end{aligned}
$$

To recover the original polynomial, we do
$A(X) = A_{\text{even}} (X^2) + X A_{\text{odd}}(X^2).$

Trying this out on points $\omega_n^i$ and $\omega_n^{\frac{n}{2} + i}$,
$i \in [0..\frac{n}{2}-1],$ we start to notice some symmetries:

$$
\begin{aligned}
A(\omega_n^i) &= A_{\text{even}} ((\omega_n^i)^2) + \omega_n^i A_{\text{odd}}((\omega_n^i)^2), \\
A(\omega_n^{\frac{n}{2} + i}) &= A_{\text{even}} ((\omega_n^{\frac{n}{2} + i})^2) + \omega_n^{\frac{n}{2} + i} A_{\text{odd}}((\omega_n^{\frac{n}{2} + i})^2) \\
&= A_{\text{even}} ((-\omega_n^i)^2) - \omega_n^i A_{\text{odd}}((-\omega_n^i)^2) \leftarrow\text{(negation lemma)} \\
&= A_{\text{even}} ((\omega_n^i)^2) - \omega_n^i A_{\text{odd}}((\omega_n^i)^2).
\end{aligned}
$$

Notice that we are only evaluating $A_{\text{even}}(X)$ and $A_{\text{odd}}(X)$ over half
the domain $\{(\omega_n^0)^2, (\omega_n)^2, \cdots, (\omega_n^{\frac{n}{2} -1})^2\} = \{\omega_{n/2}^i\}, i = [0..\frac{n}{2}-1]$ (halving lemma).
This gives us all the terms we need to reconstruct $A(X)$ over the full domain
$\{\omega^0, \omega, \cdots, \omega^{n -1}\}$: which means we have transformed a
length-$n$ DFT into two length-$\frac{n}{2}$ DFTs. 

We choose $n = 2^k$ to be a power of two (by zero-padding if needed), and apply this
divide-and-conquer strategy recursively. By the Master Theorem[^master-thm], this gives us
an evaluation algorithm with $O(n\log_2n)$ operations, also known as the Fast Fourier
Transform (FFT).

### Inverse FFT
So we've evaluated our polynomials and multiplied them pointwise. What remains is to
convert the product from the evaluation representation back to coefficient representation.
To do this, we simply call the FFT on the evaluation representation. However, this time we
also:
- replace $\omega^i$ by $\omega^{-i}$ in the Vandermonde matrix, and
- multiply our final result by a factor of $1/n$.

In other words:
$$\mathbf{A} = \frac{1}{n} \mathbf{V}_{\omega^{-1}} \cdot \hat{\mathbf{A}}. $$

(To understand why the inverse FFT has a similar form to the FFT, refer to Slide 13-1 of
[^ifft]. The below image was also taken from [^ifft].)

![](https://i.imgur.com/lSw30zo.png)


## The Schwartz-Zippel lemma
The Schwartz-Zippel lemma informally states that "different polynomials are different at
most points." Formally, it can be written as follows:

> Let $p(x_1, x_2, \cdots, x_n)$ be a nonzero polynomial of $n$ variables with degree $d$.
> Let $S$ be a finite set of numbers with at least $d$ elements in it. If we choose random
> $\alpha_1, \alpha_1, \cdots, \alpha_n$ from $S$,
> $$\text{Pr}[p(\alpha_1, \alpha_2, \cdots, \alpha_n) = 0] \leq \frac{d}{|S|}.$$

In the familiar univariate case $p(X)$, this reduces to saying that a nonzero polynomial
of degree $d$ has at most $d$ roots.

The Schwartz-Zippel lemma is used in polynomial equality testing.  Given two multi-variate
polynomials $p_1(x_1,\cdots,x_n)$ and $p_2(x_1,\cdots,x_n)$ of degrees $d_1, d_2$
respectively, we can test if
$p_1(\alpha_1, \cdots, \alpha_n) - p_2(\alpha_1, \cdots, \alpha_n) = 0$ for random
$\alpha_1, \cdots, \alpha_n \leftarrow S,$ where the size of $S$ is at least
$|S| \geq (d_1 + d_2).$  If the two polynomials are identical, this will always be true,
whereas if the two polynomials are different then the equality holds with probability at
most $\frac{\max(d_1,d_2)}{|S|}$.

## Vanishing polynomial
Consider the order-$n$ multiplicative subgroup $\mathcal{H}$ with primitive root of unity
$\omega$. For all $\omega^i \in \mathcal{H}, i \in [n-1],$ we have
$(\omega^i)^n = (\omega^n)^i = (\omega^0)^i = 1.$ In other words, every element of
$\mathcal{H}$ fulfils the equation 

$$
\begin{aligned}
Z_H(X) &= X^n - 1 \\
&= (X-\omega^0)(X-\omega^1)(X-\omega^2)\cdots(X-\omega^{n-1}),
\end{aligned}
$$

meaning every element is a root of $Z_H(X).$ We call $Z_H(X)$ the **vanishing polynomial**
over $\mathcal{H}$ because it evaluates to zero on all elements of $\mathcal{H}.$

This comes in particularly handy when checking polynomial constraints. For instance, to
check that $A(X) + B(X) = C(X)$ over $\mathcal{H},$ we simply have to check that
$A(X) + B(X) - C(X)$ is some multiple of $Z_H(X)$. In other words, if dividing our
constraint by the vanishing polynomial still yields some polynomial
$\frac{A(X) + B(X) - C(X)}{Z_H(X)} = H(X),$ we are satisfied that $A(X) + B(X) - C(X) = 0$
over $\mathcal{H}.$

## Lagrange basis functions

> TODO: explain what a basis is in general (briefly).

Polynomials are commonly written in the monomial basis (e.g. $X, X^2, ... X^n$). However,
when working over a multiplicative subgroup of order $n$, we find a more natural expression
in the Lagrange basis.

Consider the order-$n$ multiplicative subgroup $\mathcal{H}$ with primitive root of unity
$\omega$. The Lagrange basis corresponding to this subgroup is a set of functions
$\{\mathcal{L}_i\}_{i = 0}^{n-1}$, where 

$$
\mathcal{L_i}(\omega^j) = \begin{cases}
1 & \text{if } i = j, \\
0 & \text{otherwise.}
\end{cases}
$$

We can write this more compactly as $\mathcal{L_i}(\omega^j) = \delta_{ij},$ where
$\delta$ is the Kronecker delta function. 

Now, we can write our polynomial as a linear combination of Lagrange basis functions,

$$A(X) = \sum_{i = 0}^{n-1} a_i\mathcal{L_i}(X), X \in \mathcal{H},$$

which is equivalent to saying that $p(X)$ evaluates to $a_0$ at $\omega^0$,
to $a_1$ at $\omega^1$, to $a_2$ at $\omega^2, \cdots,$ and so on.

When working over a multiplicative subgroup, the Lagrange basis function has a convenient
sparse representation of the form

$$
\mathcal{L}_i(X) = \frac{c_i\cdot(X^{n} - 1)}{X - \omega^i},
$$

where $c_i$ is the barycentric weight. (To understand how this form was derived, refer to
[^barycentric].) For $i = 0,$ we have
$c = 1/n \implies \mathcal{L}_0(X) = \frac{1}{n} \frac{(X^{n} - 1)}{X - 1}$.

Suppose we are given a set of evaluation points $\{x_0, x_1, \cdots, x_{n-1}\}$.
Since we cannot assume that the $x_i$'s form a multiplicative subgroup, we consider also
the Lagrange polynomials $\mathcal{L}_i$'s in the general case. Then we can construct:

$$
\mathcal{L}_i(X) = \prod_{j\neq i}\frac{X - x_j}{x_i - x_j}, i \in [0..n-1].
$$

Here, every $X = x_j \neq x_i$ will produce a zero numerator term $(x_j - x_j),$ causing
the whole product to evaluate to zero. On the other hand, $X= x_i$ will evaluate to
$\frac{x_i - x_j}{x_i - x_j}$ at every term, resulting in an overall product of one. This
gives the desired Kronecker delta behaviour $\mathcal{L_i}(x_j) = \delta_{ij}$ on the
set $\{x_0, x_1, \cdots, x_{n-1}\}$.

### Lagrange interpolation
Given a polynomial in its evaluation representation

$$A: \{(x_0, A(x_0)), (x_1, A(x_1)), \cdots, (x_{n-1}, A(x_{n-1}))\},$$

we can reconstruct its coefficient form in the Lagrange basis:

$$A(X) = \sum_{i = 0}^{n-1} A(x_i)\mathcal{L_i}(X), $$

where $X \in \{x_0, x_1,\cdots, x_{1-n}\}.$

## References
[^master-thm]: [Dasgupta, S., Papadimitriou, C. H., & Vazirani, U. V. (2008). "Algorithms" (ch. 2). New York: McGraw-Hill Higher Education.](https://people.eecs.berkeley.edu/~vazirani/algorithms/chap2.pdf)

[^ifft]: [Golin, M. (2016). "The Fast Fourier Transform and Polynomial Multiplication" [lecture notes], COMP 3711H Design and Analysis of Algorithms, Hong Kong University of Science and Technology.](http://www.cs.ust.hk/mjg_lib/Classes/COMP3711H_Fall16/lectures/FFT_Slides.pdf)

[^barycentric]: [Berrut, J. and Trefethen, L. (2004). "Barycentric Lagrange Interpolation."](https://people.maths.ox.ac.uk/trefethen/barycentric.pdf)
