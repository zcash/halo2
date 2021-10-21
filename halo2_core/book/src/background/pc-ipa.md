# Polynomial commitment using inner product argument
We want to commit to some polynomial $p(X) \in \mathbb{F}_p[X]$, and be able to provably
evaluate the committed polynomial at arbitrary points. The naive solution would be for the
prover to simply send the polynomial's coefficients to the verifier: however, this
requires $O(n)$ communication. Our polynomial commitment scheme gets the job done using
$O(\log n)$ communication.

### `Setup`
Given a parameter $d = 2^k,$ we generate the common reference string
$\sigma = (\mathbb{G}, \mathbf{G}, H, \mathbb{F}_p)$ defining certain constants for this
scheme:
* $\mathbb{G}$ is a group of prime order $p;$
* $\mathbf{G} \in \mathbb{G}^d$ is a vector of $d$ random group elements;
* $H \in \mathbb{G}$ is a random group element; and
* $\mathbb{F}_p$ is the finite field of order $p.$

### `Commit`
The Pedersen vector commitment $\text{Commit}$ is defined as

$$\text{Commit}(\sigma, p(X); r) = \langle\mathbf{a}, \mathbf{G}\rangle + [r]H,$$

for some polynomial $p(X) \in \mathbb{F}_p[X]$ and some blinding factor
$r \in \mathbb{F}_p.$ Here, each element of the vector $\mathbf{a}_i \in \mathbb{F}_p$ is
the coefficient for the $i$th degree term of $p(X),$ and $p(X)$ is of maximal degree
$d - 1.$

### `Open` (prover) and `OpenVerify` (verifier)
The modified inner product argument is an argument of knowledge for the relation

$$\boxed{\{((P, x, v); (\mathbf{a}, r)): P = \langle\mathbf{a}, \mathbf{G}\rangle + [r]H, v = \langle\mathbf{a}, \mathbf{b}\rangle\}},$$

where $\mathbf{b} = (1, x, x^2, \cdots, x^{d-1})$ is composed of increasing powers of the
evaluation point $x.$ This allows a prover to demonstrate to a verifier that the
polynomial contained “inside” the commitment $P$ evaluates to $v$ at $x,$ and moreover,
that the committed polynomial has maximum degree $d − 1.$

The inner product argument proceeds in $k = \log_2 d$ rounds. For our purposes, it is
sufficient to know about its final outputs, while merely providing intuition about the
intermediate rounds. (Refer to Section 3 in the [Halo] paper for a full explanation.)

[Halo]: https://eprint.iacr.org/2019/1021.pdf

Before beginning the argument, the verifier selects a random group element $U$ and sends it
to the prover. We initialize the argument at round $k,$ with the vectors
$\mathbf{a}^{(k)} := \mathbf{a},$ $\mathbf{G}^{(k)} := \mathbf{G}$ and
$\mathbf{b}^{(k)} := \mathbf{b}.$ In each round $j = k, k-1, \cdots, 1$:

* the prover computes two values $L_j$ and $R_j$ by taking some inner product of
  $\mathbf{a}^{(j)}$ with $\mathbf{G}^{(j)}$ and $\mathbf{b}^{(j)}$. Note that are in some
  sense "cross-terms": the lower half of $\mathbf{a}$ is used with the higher half of
  $\mathbf{G}$ and $\mathbf{b}$, and vice versa:

$$
\begin{aligned}
L_j &= \langle\mathbf{a_{lo}^{(j)}}, \mathbf{G_{hi}^{(j)}}\rangle + [l_j]H + [\langle\mathbf{a_{lo}^{(j)}}, \mathbf{b_{hi}^{(j)}}\rangle] U\\
R_j &= \langle\mathbf{a_{hi}^{(j)}}, \mathbf{G_{lo}^{(j)}}\rangle + [l_j]H + [\langle\mathbf{a_{hi}^{(j)}}, \mathbf{b_{lo}^{(j)}}\rangle] U\\
\end{aligned}
$$

* the verifier issues a random challenge $u_j$;
* the prover uses $u_j$ to compress the lower and higher halves of $\mathbf{a}^{(j)}$,
  thus producing a new vector of half the original length 
  $$\mathbf{a}^{(j-1)} = \mathbf{a_{hi}^{(j)}}\cdot u_j^{-1} + \mathbf{a_{lo}^{(j)}}\cdot u_j.$$
  The vectors $\mathbf{G}^{(j)}$ and $\mathbf{b}^{(j)}$ are similarly compressed to give
  $\mathbf{G}^{(j-1)}$ and $\mathbf{b}^{(j-1)}$.
* $\mathbf{a}^{(j-1)}$, $\mathbf{G}^{(j-1)}$ and $\mathbf{b}^{(j-1)}$ are input to the
  next round $j - 1.$

Note that at the end of the last round $j = 1,$ we are left with $a := \mathbf{a}^{(0)}$,
$G := \mathbf{G}^{(0)}$, $b := \mathbf{b}^{(0)},$ each of length 1. The intuition is that
these final scalars, together with the challenges $\{u_j\}$ and "cross-terms"
$\{L_j, R_j\}$ from each round, encode the compression in each round. Since the prover did
not know the challenges $U, \{u_j\}$ in advance, they would have been unable to manipulate
the round compressions. Thus, checking a constraint on these final terms should enforce
that the compression had been performed correctly, and that the original $\mathbf{a}$
satisfied the relation before undergoing compression.

Note that $G, b$ are simply rearrangements of the publicly known $\mathbf{G}, \mathbf{b},$
with the round challenges $\{u_j\}$ mixed in: this means the verifier can compute $G, b$
independently and verify that the prover had provided those same values.
