# Protocol Description

## Preliminaries

We take $\sec$ as our security parameter, and unless explicitly noted all
algorithms and adversaries are probabilistic (interactive) Turing machines that
run in polynomial time in this security parameter. We use $\negl$ to denote a
function that is negligible in $\sec$.

### Cryptographic Groups

We let $\group$ denote a cyclic group of prime order $p$. The identity of a
group is written as $\zero$. We refer to the scalars of elements in $\group$ as
elements in a scalar field $\field$ of size $p$. Group elements are written in
capital letters while scalars are written in lowercase or Greek letters. Vectors
of scalars or group elements are written in boldface, i.e. $\mathbf{a} \in
\field^n$ and $\mathbf{G} \in \group^n$. Group operations are written additively
and the multiplication of a group element $G$ by a scalar $a$ is written $[a]
G$.

We will often use the notation $\langle \mathbf{a}, \mathbf{b} \rangle$ to
describe the inner product of two like-length vectors of scalars $\mathbf{a},
\mathbf{b} \in \field^n$. We also use this notation to represent the linear
combination of group elements such as $\langle \mathbf{a}, \mathbf{G} \rangle$
with $\mathbf{a} \in \field^n, \mathbf{G} \in \group^n$, computed in practice by
a multiscalar multiplication.

We use $\mathbf{0}^n$ to describe a vector of length $n$ that contains only
zeroes in $\field$.

> **Discrete Log Relation Problem.** The advantage metric
$$
\adv^\dlrel_{\group,n}(\a, \sec) = \textnormal{Pr} \left[ \mathsf{G}^\dlrel_{\group,n}(\a, \sec) \right]
$$
> is defined with respect the following game.
$$
\begin{array}{ll}
&\underline{\bold{Game} \, \mathsf{G}^\dlrel_{\group,n}(\a, \sec):} \\
&\mathbf{G} \gets \group^n_\sec \\
&\mathbf{a} \gets \a(\mathbf{G}) \\
&\textnormal{Return} \, \left( \langle \mathbf{a}, \mathbf{G} \rangle = \zero \land \mathbf{a} \neq \mathbf{0}^n \right)
\end{array}
$$

Given an $n$-length vector $\mathbf{G} \in \group^n$ of group elements, the
_discrete log relation problem_ asks for $\mathbf{g} \in \field^n$ such that
$\mathbf{g} \neq \mathbf{0}^n$ and yet $\innerprod{\mathbf{g}}{\mathbf{G}} =
\zero$, which we refer to as a _non-trivial_ discrete log relation. The hardness
of this problem is tightly implied by the hardness of the discrete log problem
in the group as shown in Lemma 3 of [[JT20]](https://eprint.iacr.org/2020/1213).
Formally, we use the game $\dlgame$ defined above to capture this problem.

### Interactive Proofs

_Interactive proofs_ are a triple of algorithms $\ip = (\setup, \prover,
\verifier)$. The algorithm $\setup(1^\sec)$ produces as its output some _public
parameters_ commonly referred to by $\pp$. The prover $\prover$ and verifier
$\verifier$ are interactive machines (with access to $\pp$) and we denote by
$\langle \prover(x), \verifier(y) \rangle$ an algorithm that executes a
two-party protocol between them on inputs $x, y$. The output of this protocol, a
_transcript_ of their interaction, contains all of the messages sent between
$\prover$ and $\verifier$. At the end of the protocol, the verifier outputs a
decision bit.

### Zero knowledge Arguments of Knowledge

Proofs of knowledge are interactive proofs where the prover aims to convince the
verifier that they know a witness $w$ such that $(x, w) \in \relation$ for a
statement $x$ and polynomial-time decidable relation $\relation$. We will work
with _arguments_ of knowledge which assume computationally-bounded provers.

We will analyze arguments of knowledge through the lens of four security
notions.

* **Completeness:** If the prover possesses a valid witness, can they _always_
  convince the verifier? It is useful to understand this property as it can have
  implications for the other security notions.
* **Soundness:** Can a cheating prover falsely convince the verifier of the
  correctness of a statement that is not actually correct? We refer to the
  probability that a cheating prover can falsely convince the verifier as the
  _soundness error_.
* **Knowledge soundness:** When the verifier is convinced the statement is
  correct, does the prover actually possess ("know") a valid witness? We refer
  to the probability that a cheating prover falsely convinces the verifier of
  this knowledge as the _knowledge error_.
* **Zero knowledge:** Does the verifier learn anything besides that which can be
  inferred from the correctness of the statement and the prover's knowledge of a
  valid witness?

First, we will visit the simple definition of completeness.

> **Perfect Completeness.** An interactive argument $(\setup, \prover, \verifier)$
> has _perfect completeness_ if for all polynomial-time decidable
> relations $\relation$ and for all non-uniform polynomial-time adversaries $\a$
$$
Pr \left[
    (x, w) \notin \relation \lor
    \langle \prover(\pp, x, w), \verifier(\pp, x) \rangle \, \textnormal{accepts}
    \, \middle| \,
    \begin{array}{ll}
    &\pp \gets \setup(1^\sec) \\
    &(x, w) \gets \a(\pp)
    \end{array}
\right] = 1
$$

#### Soundness

Complicating our analysis is that although our protocol is described as an
interactive argument, it is realized in practice as a _non-interactive argument_
through the use of the Fiat-Shamir transformation.

> **Public coin.** We say that an interactive argument is _public coin_ when all
> of the messages sent by the verifier are each sampled with fresh randomness.

> **Fiat-Shamir transformation.** In this transformation an interactive, public
> coin argument can be made _non-interactive_ in the _random oracle model_ by
> replacing the verifier algorithm with a cryptographically strong hash function
> that produces sufficiently random looking output.

This transformation means that in the concrete protocol a cheating prover can
easily "rewind" the verifier by forking the transcript and sending new messages
to the verifier. Studying the concrete security of our construction _after_
applying this transformation is important. Fortunately, we are able to follow a
framework of analysis by Ghoshal and Tessaro
([[GT20](https://eprint.iacr.org/2020/1351)]) that has been applied to
constructions similar to ours.

We will study our protocol through the notion of _state-restoration soundness_.
In this model the (cheating) prover is allowed to rewind the verifier to any
previous state it was in. The prover wins if they are able to produce an
accepting transcript.

> **State-Restoration Soundness.** Let $\ip$ be an interactive argument with $r
> = r(\sec)$ verifier challenges and let the $i$th challenge be sampled from
> $\ch_i$. The advantage metric
$$
\adv^\srs_\ip(\prover, \sec) = \textnormal{Pr} \left[ \srs^\ip_\prover(\sec) \right]
$$
> of a state restoration prover $\prover$ is defined with respect to the
> following game.
$$
\begin{array}{ll}
\begin{array}{ll}
&\underline{\bold{Game} \, \srs_\ip^\prover(\sec):} \\
&\textnormal{win} \gets \tt{false}; \\
&\tr \gets \epsilon \\
&\pp \gets \ip.\setup(1^\sec) \\
&(x, \textsf{st}_\prover) \gets \prover_\sec(\pp) \\
&\textnormal{Run} \, \prover^{\oracle_\srs}_\sec(\textsf{st}_\prover) \\
&\textnormal{Return win}
\end{array} &
\begin{array}{ll}
&\underline{\bold{Oracle} \, \oracle_\srs(\tau = (a_1, c_1, ..., a_{i - 1}, c_{i - 1}), a_i):} \\
& \textnormal{If} \, \tau \in \tr \, \textnormal{then} \\
& \, \, \textnormal{If} \, i \leq r \, \textnormal{then} \\
& \, \, \, \, c_i \gets \ch_i; \tr \gets \tr || (\tau, a_i, c_i); \textnormal{Return} \, c_i \\
& \, \, \textnormal{Else if} \, i = r + 1 \, \textnormal{then} \\
& \, \, \, \, d \gets \ip.\verifier (\pp, x, (\tau, a_i)); \tr \gets (\tau, a_i) \\
& \, \, \, \, \textnormal{If} \, d = 1 \, \textnormal{then win} \gets \tt{true} \\
& \, \, \, \, \textnormal{Return} \, d \\
&\textnormal{Return} \bottom
\end{array}
\end{array}
$$

As shown in [[GT20](https://eprint.iacr.org/2020/1351)] (Theorem 1) state
restoration soundness is tightly related to soundness after applying the
Fiat-Shamir transformation.

#### Knowledge Soundness

We will show that our protocol satisfies a strengthened notion of knowledge
soundness known as _witness extended emulation_. Informally, this notion states
that for any successful prover algorithm there exists an efficient _emulator_
that can extract a witness from it by rewinding it and supplying it with fresh
randomness.

However, we must slightly adjust our definition of witness extended emulation to
account for the fact that our provers are state restoration provers and can
rewind the verifier. Further, to avoid the need for rewinding the state
restoration prover during witness extraction we study our protocol in the
algebraic group model.

> **Algebraic Group Model (AGM).** An adversary $\alg{\prover}$ is said to be
> _algebraic_ if whenever it outputs a group element $X$ it also outputs a
> _representation_ $\mathbf{x} \in \field^n$ such that $\langle \mathbf{x}, \mathbf{G} \rangle = X$ where $\mathbf{G} \in \group^n$ is the vector of group
> elements that $\alg{\prover}$ has seen so far.
> Notationally, we write $\rep{X}$ to describe a group element $X$ enhanced
> with this representation. We also write $\repv{X}{G}{i}$ to identify the
> component of the representation of $X$ that corresponds with $\mathbf{G}_i$. In
> other words,
$$
X = \sum\limits_{i=0}^{n - 1} \left[ \repv{X}{G}{i} \right] \mathbf{G}_i
$$

The algebraic group model allows us to perform so-called "online" extraction for
some protocols: the extractor can obtain the witness from the representations
themselves for a single (accepting) transcript.

> **State Restoration Witness Extended Emulation** Let $\ip$ be an interactive
> argument for relation $\relation$ with $r = r(\sec)$ challenges. We define for
> all non-uniform algebraic provers $\alg{\prover}$, extractors $\extractor$,
> and computationally unbounded distinguishers $\distinguisher$ the advantage
> metric
$$
\adv^\srwee_{\ip, \relation}(\alg{\prover}, \distinguisher, \extractor, \sec) = \textnormal{Pr} \left[ \weereal^{\prover,\distinguisher}_{\ip,\relation}(\sec) \right] - \textnormal{Pr} \left[ \weeideal^{\extractor,\prover,\distinguisher}_{\ip,\relation}(\sec) \right]
$$
> is defined with the respect to the following games.
$$
\begin{array}{ll}
\begin{array}{ll}
&\underline{\bold{Game} \, \weereal_{\ip,\relation}^{\alg{\prover},\distinguisher}(\sec):} \\
&\tr \gets \epsilon \\
&\pp \gets \ip.\setup(1^\sec) \\
&(x, \state{\prover}) \gets \alg{\prover}(\pp) \\
&\textnormal{Run} \, \alg{\prover}^{\oracle_\real}(\state{\prover}) \\
&b \gets \distinguisher(\tr) \\
&\textnormal{Return} \, b = 1 \\
&\underline{\bold{Game} \, \weeideal_{\ip,\relation}^{\extractor,\alg{\prover},\distinguisher}(\sec):} \\
&\tr \gets \epsilon \\
&\pp \gets \ip.\setup(1^\sec) \\
&(x, \state{\prover}) \gets \alg{\prover}(\pp) \\
&\state{\extractor} \gets (1^\sec, \pp, x) \\
&\textnormal{Run} \, \alg{\prover}^{\oracle_\ideal}(\state{\prover}) \\
&w \gets \extractor(\state{\extractor}, \bottom) \\
&b \gets \distinguisher(\tr) \\
&\textnormal{Return} \, (b = 1) \\
&\, \, \land (\textnormal{Acc}(\tr) \implies (x, w) \in \relation) \\
\end{array} &
\begin{array}{ll}
&\underline{\bold{Oracle} \, \oracle_\real(\tau = (a_1, c_1, ..., a_{i - 1}, c_{i - 1}), a_i):} \\
& \textnormal{If} \, \tau \in \tr \, \textnormal{then} \\
& \, \, \textnormal{If} \, i \leq r \, \textnormal{then} \\
& \, \, \, \, c_i \gets \ch_i; \tr \gets \tr || (\tau, a_i, c_i); \textnormal{Return} \, c_i \\
& \, \, \textnormal{Else if} \, i = r + 1 \, \textnormal{then} \\
& \, \, \, \, d \gets \ip.\verifier (\pp, x, (\tau, a_i)); \tr \gets (\tau, a_i) \\
& \, \, \, \, \textnormal{If} \, d = 1 \, \textnormal{then win} \gets \tt{true} \\
& \, \, \, \, \textnormal{Return} \, d \\
& \, \, \textnormal{Return} \, \bottom \\
\\
\\
&\underline{\bold{Oracle} \, \oracle_\ideal(\tau, a):} \\
& \textnormal{If} \, \tau \in \tr \, \textnormal{then} \\
& \, \, (r, \state{\extractor}) \gets \extractor(\state{\extractor}, \left[(\tau, a)\right]) \\
& \, \, \tr \gets \tr || (\tau, a, r) \\
& \, \, \textnormal{Return} \, r \\
&\textnormal{Return} \, \bottom
\end{array}
\end{array}
$$

#### Zero Knowledge

We say that an argument of knowledge is _zero knowledge_ if the verifier also
does not learn anything from their interaction besides that which can be learned
from the existence of a valid $w$. More formally,

> **Perfect Special Honest-Verifier Zero Knowledge.** A public coin interactive
> argument $(\setup, \prover, \verifier)$ has _perfect special honest-verifier
> zero knowledge_ (PSHVZK) if for all polynomial-time decidable relations
> $\relation$ and for all $(x, w) \in \relation$ and for all non-uniform
> polynomial-time adversaries $\a_1, \a_2$ there exists a probabilistic
> polynomial-time simulator $\sim$ such that
$$
\begin{array}{rl}
&Pr \left[ \a_1(\sigma, x, \tr) = 1 \, \middle| \,
\begin{array}{ll}
&\pp \gets \setup(1^\lambda); \\
&(x, w, \rho) \gets \a_2(\pp); \\
&tr \gets \langle \prover(\pp, x, w), \verifier(\pp, x, \rho) \rangle
\end{array}
 \right] \\
 \\
=&Pr \left[ \a_1(\sigma, x, \tr) = 1 \, \middle| \,
\begin{array}{ll}
&\pp \gets \setup(1^\lambda); \\
&(x, w, \rho) \gets \a_2(\pp); \\
&tr \gets \sim(\pp, x, \rho)
\end{array}
\right]
\end{array}
$$
> where $\rho$ is the internal randomness of the verifier.

In this (common) definition of zero-knowledge the verifier is expected to act
"honestly" and send challenges that correspond only with their internal
randomness; they cannot adaptively respond to the prover based on the prover's
messages. We use a strengthened form of this definition that forces the simulator
to output a transcript with the same (adversarially provided) challenges that
the verifier algorithm sends to the prover.

## Protocol

Let $\omega \in \field$ be a $n = 2^k$ primitive root of unity forming the
domain $D = (\omega^0, \omega^1, ..., \omega^{n - 1})$ with $t(X) = X^n - 1$ the
vanishing polynomial over this domain. Let $n_g, n_a, n_e$ be positive integers with $n_a, n_e \lt n$ and $n_g \geq 4$.
We present an interactive argument $\halo = (\setup, \prover, \verifier)$ for
the relation
$$
\relation = \left\{
\begin{array}{ll}
&\left(
\begin{array}{ll}
\left(
g(X, C_0, ..., C_{n_a - 1}, a_0(X), ..., a_{n_a - 1}\left(X, C_0, ..., C_{n_a - 1}, a_0(X), ..., a_{n_a - 2}(X) \right))
\right); \\
\left(
a_0(X), a_1(X, C_0, a_0(X)), ..., a_{n_a - 1}\left(X, C_0, ..., C_{n_a - 1}, a_0(X), ..., a_{n_a - 2}(X) \right)
\right)
\end{array}
\right) : \\
\\
& g(\omega^i, \cdots) = 0 \, \, \, \, \forall i \in [0, 2^k)
\end{array}
\right\}
$$
where $a_0, a_1, ..., a_{n_a - 1}$ are (multivariate) polynomials with degree $n - 1$ in $X$ and $g$ has degree $n_g(n - 1)$ at most in any indeterminates $X, C_0, C_1, ...$.

$\setup(\sec)$ returns $\pp = (\group, \field, \mathbf{G} \in \group^n, U, W \in \group)$.

For all $i \in [0, n_a)$:
* Let $\mathbf{p_i}$ be the exhaustive set of integers $j$ (modulo $n$) such that $a_i(\omega^j X, \cdots)$ appears as a term in $g(X, \cdots)$.
* Let $\mathbf{q}$ be a list of distinct sets of integers containing $\mathbf{p_i}$ and the set $\mathbf{q_0} = \{0\}$.
* Let $\sigma(i) = \mathbf{q}_j$ when $\mathbf{q}_j = \mathbf{p_i}$.

Let $n_q \leq n_a$ denote the size of $\mathbf{q}$, and let $n_e$ denote the size of every $\mathbf{p_i}$ without loss of generality.

In the following protocol, we take it for granted that each polynomial $a_i(X, \cdots)$ is defined such that $n_e + 1$ blinding factors are freshly sampled by the prover and are each present as an evaluation of $a_i(X, \cdots)$ over the domain $D$. In all of the following, the verifier's challenges cannot be zero or an element in $D$, and some additional limitations are placed on specific challenges as well.

1. $\prover$ and $\verifier$ proceed in the following $n_a$ rounds of interaction, where in round $j$ (starting at $0$)
  * $\prover$ sets $a'_j(X) = a_j(X, c_0, c_1, ..., c_{j - 1}, a_0(X, \cdots), ..., a_{j - 1}(X, \cdots, c_{j - 1}))$
  * $\prover$ sends a hiding commitment $A_j = \innerprod{\mathbf{a'}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{a'}$ are the coefficients of the univariate polynomial $a'_j(X)$ and $\cdot$ is some random, independently sampled blinding factor elided for exposition. (This elision notation is used throughout this protocol description to simplify exposition.)
  * $\verifier$ responds with a challenge $c_j$.
2. $\prover$ sets $g'(X) = g(X, c_0, c_1, ..., c_{n_a - 1}, \cdots)$.
3. $\prover$ sends a commitment $R = \innerprod{\mathbf{r}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{r} \in \field^n$ are the coefficients of a randomly sampled univariate polynomial $r(X)$ of degree $n - 1$.
4. $\prover$ computes univariate polynomial $h(X) = \frac{g'(X)}{t(X)}$ of degree $n_g(n - 1) - n$.
5. $\prover$ computes at most $n - 1$ degree polynomials $h_0(X), h_1(X), ..., h_{n_g - 2}(X)$ such that $h(X) = \sum\limits_{i=0}^{n_g - 2} X^{ni} h_i(X)$.
6. $\prover$ sends commitments $H_i = \innerprod{\mathbf{h_i}}{\mathbf{G}} + [\cdot] W$ for all $i$ where $\mathbf{h_i}$ denotes the vector of coefficients for $h_i(X)$.
7. $\verifier$ responds with challenge $x$ and computes $H' = \sum\limits_{i=0}^{n_g - 2} [x^{ni}] H_i$.
8. $\prover$ sets $h'(X) = \sum\limits_{i=0}^{n_g - 2} x^{ni} h_i(X)$.
9. $\prover$ sends $r = r(x)$ and for all $i \in [0, n_a)$ sends $\mathbf{a_i}$ such that $(\mathbf{a_i})_j = a'_i(\omega^{(\mathbf{p_i})_j} x)$ for all $j \in [0, n_e - 1]$.
10. For all $i \in [0, n_a)$ $\prover$ and $\verifier$ set $s_i(X)$ to be the lowest degree univariate polynomial defined such that $s_i(\omega^{(\mathbf{p_i})_j} x) = (\mathbf{a_i})_j$ for all $j \in [0, n_e - 1)$.
11. $\verifier$ responds with challenges $x_1, x_2$ and initializes $Q_0, Q_1, ..., Q_{n_q - 1} = \zero$.
  * Starting at $i=0$ and ending at $n_a - 1$ $\verifier$ sets $Q_{\sigma(i)} := [x_1] Q_{\sigma(i)} + A_i$.
  * $\verifier$ finally sets $Q_0 := [x_1^2] Q_0 + [x_1] H' + R$.
12. $\prover$ initializes $q_0(X), q_1(X), ..., q_{n_q - 1}(X) = 0$.
  * Starting at $i=0$ and ending at $n_a - 1$ $\prover$ sets $q_{\sigma(i)} := x_1 q_{\sigma(i)} + a'(X)$.
  * $\prover$ finally sets $q_0(X) := x_1^2 q_0(X) + x_1 h'(X) + r(X)$.
13. $\prover$ and $\verifier$ initialize $r_0(X), r_1(X), ..., r_{n_q - 1}(X) = 0$.
  * Starting at $i = 0$ and ending at $n_a - 1$ $\prover$ and $\verifier$ set $r_{\sigma(i)}(X) := x_1 r_{\sigma(i)}(X) + s_i(X)$.
  * Finally $\prover$ and $\verifier$ set $r_0 := x_1^2 r_0 + x_1 h + r$ and where $h$ is computed by $\verifier$ as $\frac{g'(x)}{t(x)}$ using the values $r, \mathbf{a}$ provided by $\prover$.
14. $\prover$ sends $Q' = \innerprod{\mathbf{q'}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{q'}$ defines the coefficients of the polynomial
$$q'(X) = \sum\limits_{i=0}^{n_q - 1}

x_2^i
  \left(
  \frac
  {q_i(X) - r_i(X)}
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      X - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
$$
15. $\verifier$ responds with challenge $x_3$.
16. $\prover$ sends $\mathbf{u} \in \field^{n_q}$ such that $\mathbf{u}_i = q_i(x_3)$ for all $i \in [0, n_q)$.
17. $\verifier$ responds with challenge $x_4$.
18. $\verifier$ sets $P = Q' + x_4 \sum\limits_{i=0}^{n_q - 1} [x_4^i] Q_i$ and $v = $
$$
\sum\limits_{i=0}^{n_q - 1}
\left(
x_2^i
  \left(
  \frac
  { \mathbf{u}_i - r_i(x_3) }
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      x_3 - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
\right)
+
x_4 \sum\limits_{i=0}^{n_q - 1} x_4 \mathbf{u}_i
$$
19. $\prover$ sets $p(X) = q'(X) + [x_4] \sum\limits_{i=0}^{n_q - 1} x_4^i q_i(X)$.
20. $\prover$ samples a random polynomial $s(X)$ of degree $n - 1$ with a root at $x_3$ and sends a commitment $S = \innerprod{\mathbf{s}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{s}$ defines the coefficients of $s(X)$.
21. $\verifier$ responds with challenges $\xi, z$.
22. $\verifier$ sets $P' = P - [v] \mathbf{G}_0 + [\xi] S$.
23. $\prover$ sets $p'(X) = p(X) - p(x_3) + \xi s(X)$ (where $p(x_3)$ should correspond with the verifier's computed value $v$).
24. Initialize $\mathbf{p'}$ as the coefficients of $p'(X)$ and $\mathbf{G'} = \mathbf{G}$ and $\mathbf{b} = (x_3^0, x_3^1, ..., x_3^{n - 1})$. $\prover$ and $\verifier$ will interact in the following $k$ rounds, where in the $j$th round starting in round $j=0$ and ending in round $j=k-1$:
  * $\prover$ sends $L_j = \innerprod{\mathbf{p'}_\hi}{\mathbf{G'}_\lo} + [z \innerprod{\mathbf{p'}_\hi}{\mathbf{b}_\lo}] U + [\cdot] W$ and $R_j = \innerprod{\mathbf{p'}_\lo}{\mathbf{G'}_\hi} + [z \innerprod{\mathbf{p'}_\lo}{\mathbf{b}_\hi}] U + [\cdot] W$.
  * $\verifier$ responds with challenge $u_j$ chosen such that $1 + u_{k-1-j} x_3^{2^j}$ is nonzero.
  * $\prover$ and $\verifier$ set $\mathbf{G'} := \mathbf{G'}_\lo + u_j \mathbf{G'}_\hi$ and $\mathbf{b} := \mathbf{b}_\lo + u_j \mathbf{b}_\hi$.
  * $\prover$ sets $\mathbf{p'} := \mathbf{p'}_\lo + u_j^{-1} \mathbf{p'}_\hi$.
25. $\prover$ sends $c = \mathbf{p'}_0$ and synthetic blinding factor $f$ computed from the elided blinding factors.
26. $\verifier$ accepts only if $\sum_{j=0}^{k - 1} [u_j^{-1}] L_j + P' + \sum_{j=0}^{k - 1} [u_j] R_j = [c] \mathbf{G'}_0 + [c \mathbf{b}_0 z] U + [f] W$.

### Zero-knowledge and Completeness

We claim that this protocol is _perfectly complete_. This can be verified by
inspection of the protocol; given a valid witness $a_i(X, \cdots) \forall i$ the
prover succeeds in convincing the verifier with probability $1$.

We claim that this protocol is _perfect special honest-verifier zero knowledge_.
We do this by showing that a simulator $\sim$ exists which can produce an
accepting transcript that is equally distributed with a valid prover's
interaction with a verifier with the same public coins. The simulator will act
as an honest prover would, with the following exceptions:

1. In step $1$ of the protocol $\sim$ chooses random degree $n - 1$ polynomials (in $X$) $a_i(X, \cdots) \forall i$.
2. In step $5$ of the protocol $\sim$ chooses a random $n - 1$ degree polynomials $h_0(X), h_1(X), ..., h_{n_g - 2}(X)$.
3. In step $14$ of the protocol $\sim$ chooses a random $n - 1$ degree polynomial $q'(X)$.
4. In step $20$ of the protocol $\sim$ uses its foreknowledge of the verifier's choice of $\xi$ to produce a degree $n - 1$ polynomial $s(X)$ conditioned only such that $p(X) - v + \xi s(X)$ has a root at $x_3$.

First, let us consider why this simulator always succeeds in producing an
_accepting_ transcript. $\sim$ lacks a valid witness and simply commits to
random polynomials whenever knowledge of a valid witness would be required by
the honest prover. The verifier places no conditions on the scalar values in the
transcript. $\sim$ must only guarantee that the check in step $26$ of the
protocol succeeds. It does so by using its knowledge of the challenge $\xi$ to
produce a polynomial which interferes with $p'(X)$ to ensure it has a root at
$x_3$. The transcript will thus always be accepting due to perfect completeness.

In order to see why $\sim$ produces transcripts distributed identically to the
honest prover, we will look at each piece of the transcript and compare the
distributions. First, note that $\sim$ (just as the honest prover) uses a
freshly random blinding factor for every group element in the transcript, and so
we need only consider the _scalars_ in the transcript. $\sim$ acts just as the
prover does except in the mentioned cases so we will analyze each case:

1. $\sim$ and an honest prover reveal $n_e$ openings of each polynomial $a_i(X, \cdots)$, and at most one additional opening of each $a_i(X, \cdots)$ in step $16$. However, the honest prover blinds their polynomials $a_i(X, \cdots)$ (in $X$) with $n_e + 1$ random evaluations over the domain $D$. Thus, the openings of $a_i(X, \cdots)$ at the challenge $x$ (which is prohibited from being $0$ or in the domain $D$ by the protocol) are distributed identically between $\sim$ and an honest prover.
2. Neither $\sim$ nor the honest prover reveal $h(x)$ as it is computed by the verifier. However, the honest prover may reveal $h'(x_3)$ --- which has a non-trivial relationship with $h(X)$ --- were it not for the fact that the honest prover also commits to a random degree $n - 1$ polynomial $r(X)$ in step $3$, producing a commitment $R$ and ensuring that in step $12$ when the prover sets $q_0(X) := x_1^2 q_0(X) + x_1 h'(X) + r(X)$ the distribution of $q_0(x_3)$ is uniformly random. Thus, $h'(x_3)$ is never revealed by the honest prover nor by $\sim$.
3. The expected value of $q'(x_3)$ is computed by the verifier (in step $18$) and so the simulator's actual choice of $q'(X)$ is irrelevant.
4. $p(X) - v + \xi s(X)$ is conditioned on having a root at $x_3$, but otherwise no conditions are placed on $s(X)$ and so the distribution of the degree $n - 1$ polynomial $p(X) - v + \xi s(X)$ is uniformly random whether or not $s(X)$ has a root at $x_3$. Thus, the distribution of $c$ produced in step $25$ is identical between $\sim$ and an honest prover. The synthetic blinding factor $f$ also revealed in step $25$ is a trivial function of the prover's other blinding factors and so is distributed identically between $\sim$ and an honest prover.

Notes:

1. In an earlier version of our protocol, the prover would open each individual commitment $H_0, H_1, ...$ at $x$ as part of the multipoint opening argument, and the verifier would confirm that a linear combination of these openings (with powers of $x^n$) agreed to the expected value of $h(x)$. This was done because it's more efficient in recursive proofs. However, it was unclear to us what the expected distribution of the openings of these commitments $H_0, H_1, ...$ was and so proving that the argument was zero-knowledge is difficult. Instead, we changed the argument so that the _verifier_ computes a linear combination of the commitments and that linear combination is opened at $x$. This avoided leaking $h_i(x)$.
2. As mentioned, in step $3$ the prover commits to a random polynomial as a way of ensuring that $h'(x_3)$ is not revealed in the multiopen argument. This is done because it's unclear what the distribution of $h'(x_3)$ would be.
3. Technically it's also possible for us to prove zero-knowledge with a simulator that uses its foreknowledge of the challenge $x$ to commit to an $h(X)$ which agrees at $x$ to the value it will be expected to. This would obviate the need for the random polynomial $s(X)$ in the protocol. This may make the analysis of zero-knowledge for the remainder of the protocol a little bit tricky though, so we didn't go this route.
4. Group element blinding factors are _technically_ not necessary after step $23$ in which the polynomial is completely randomized. However, it's simpler in practice for us to ensure that every group element in the protocol is randomly blinded to make edge cases involving the point at infinity harder.
5. It is crucial that the verifier cannot challenge the prover to open polynomials at points in $D$ as otherwise the transcript of an honest prover will be forced to contain what could be portions of the prover's witness. We therefore restrict the space of challenges to include all elements of the field except $D$ and, for simplicity, we also prohibit the challenge of $0$.

## Witness-extended Emulation

Let $\protocol = \protocol[\group]$ be the interactive argument described above for relation $\relation$ and some group $\group$ with scalar field $\field$. We can always construct an extractor $\extractor$ such that for any non-uniform algebraic prover $\alg{\prover}$ making at most $q$ queries to its oracle, there exists a non-uniform adversary $\dlreladv$ with the property that for any computationally unbounded distinguisher $\distinguisher$

$$
\adv^\srwee_{\protocol, \relation}(\alg{\prover}, \distinguisher, \extractor, \sec) \leq q\epsilon + \adv^\dlrel_{\group,n+2}(\dlreladv, \sec)
$$

where $\epsilon \leq \frac{n_g \cdot (n - 1)}{|\ch|}$.

_Proof._ We will prove this by invoking Theorem 1 of [[GT20]](https://eprint.iacr.org/2020/1351). First, we note that the challenge space for all rounds is the same, i.e. $\forall i \ \ch = \ch_i$. Theorem 1 requires us to define:

- $\badch(\tr') \in \ch$ for all partial transcripts $\tr' = (\pp, x, [a_0], c_0, \ldots, [a_i])$ such that $|\badch(\tr')| / |\ch| \leq \epsilon$.
- an extractor function $e$ that takes as input an accepting extended transcript $\tr$ and either returns a valid witness or fails.
- a function $\pfail(\protocol, \alg{\prover}, e, \relation)$ returning a probability.

We say that an accepting extended transcript $\tr$ contains "bad challenges" if and only if there exists a partial extended transcript $\tr'$, a challenge $c_i \in \badch(\tr')$, and some sequence of prover messages and challenges $([a_{i+1}], c_{i+1}, \ldots [a_j])$ such that $\tr = \tr' \,||\, (c_i, [a_{i+1}], c_{i+1}, \ldots [a_j])$.

Theorem 1 requires that $e$, when given an accepting extended transcript $\tr$ that does not contain "bad challenges", returns a valid witness for that transcript except with probability bounded above by $\pfail(\protocol, \alg{\prover}, e, \relation)$.

Our strategy is as follows: we will define $e$, establish an upper bound on $\pfail$ with respect to an adversary $\dlreladv$ that plays the $\dlrel_{\group,n+2}$ game, substitute these into Theorem 1, and then walk through the protocol to determine the upper bound of the size of $\badch(\tr')$. The adversary $\dlreladv$ plays the $\dlrel_{\group,n+2}$ game as follows: given the inputs $U, W \in \mathbb{G}, \mathbf{G} \in \mathbb{G}^n$, the adversary $\dlreladv$ simulates the game $\srwee_{\protocol, \relation}$ to $\alg{\prover}$ using the inputs from the $\dlrel_{\group,n+2}$ game as public parameters. If $\alg{\prover}$ manages to produce an accepting extended transcript $\tr$, $\dlreladv$ invokes a function $h$ on $\tr$ and returns its output. We shall define $h$ in such a way that for an accepting extended transcript $\tr$ that does not contain "bad challenges", $e(\tr)$ _always_ returns a valid witness whenever $h(\tr)$ does _not_ return a non-trivial discrete log relation. This means that the probability $\pfail(\protocol, \alg{\prover}, e, \relation)$ is no greater than $\adv^\dlrel_{\group,n+2}(\dlreladv, \sec)$, establishing our claim.

#### Helpful substitutions

We will perform some substitutions to aid in exposition. First, let us define the polynomial

$$
\kappa(X) = \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} X^{2^j})
$$

so that we can write $\mathbf{b}_0 = \kappa(x_3)$. The coefficient vector $\mathbf{s}$ of $\kappa(X)$ is defined such that

$$\mathbf{s}_i = \prod\limits_{j=0}^{k-1} u_{k - 1 - j}^{f(i, j)}$$

where $f(i, j)$ returns $1$ when the $j$th bit of $i$ is set, and $0$ otherwise. We can also write $\mathbf{G'}_0 = \innerprod{\mathbf{s}}{\mathbf{G}}$.

### Description of function $h$

Recall that an accepting transcript $\tr$ is such that

$$
\sum_{i=0}^{k - 1} [u_j^{-1}] \rep{L_j} + \rep{P'} + \sum_{i=0}^{k - 1} [u_j] \rep{R_j} = [c] \mathbf{G'}_0 + [c z \mathbf{b}_0] U + [f] W
$$

By inspection of the representations of group elements with respect to $\mathbf{G}, U, W$ (recall that $\alg{\prover}$ is algebraic and so $\dlreladv$ has them), we obtain the $n$ equalities

$$
\sum_{i=0}^{k - 1} u_j^{-1} \repv{L_j}{G}{i} + \repv{P'}{G}{i} + \sum_{i=0}^{k - 1} u_j \repv{R_j}{G}{i} = c \mathbf{s}_i \forall i \in [0, n)
$$

and the equalities

$$
\sum_{i=0}^{k - 1} u_j^{-1} \repr{L_j}{U} + \repr{P'}{U} + \sum_{i=0}^{k - 1} u_j \repr{R_j}{U} = c z \kappa(x_3)
$$

$$
\sum_{i=0}^{k - 1} u_j^{-1} \repr{L_j}{W} + \repr{P'}{W} + \sum_{i=0}^{k - 1} u_j \repr{R_j}{W} = f
$$

We define the linear-time function $h$ that returns the representation of

$$
\begin{array}{rll}
\sum\limits_{i=0}^{n - 1} &\left[ \sum_{i=0}^{k - 1} u_j^{-1} \repv{L_j}{G}{i} + \repv{P'}{G}{i} + \sum_{i=0}^{k - 1} u_j \repv{R_j}{G}{i} - c \mathbf{s}_i \right] & \mathbf{G}_i \\[1ex]
+ &\left[ \sum_{i=0}^{k - 1} u_j^{-1} \repr{L_j}{U} + \repr{P'}{U} + \sum_{i=0}^{k - 1} u_j \repr{R_j}{U} - c z \kappa(x_3) \right] & U \\[1ex]
+ &\left[ \sum_{i=0}^{k - 1} u_j^{-1} \repr{L_j}{W} + \repr{P'}{W} + \sum_{i=0}^{k - 1} u_j \repr{R_j}{W} - f \right] & W
\end{array}
$$

which is always a discrete log relation. If any of the equalities above are not satisfied, then this discrete log relation is non-trivial. This is the function invoked by $\dlreladv$.

#### The extractor function $e$

The extractor function $e$ simply returns $a_i(X)$ from the representation $\rep{A_i}$ for $i \in [0, n_a)$. Due to the restrictions we will place on the space of bad challenges in each round, we are guaranteed to obtain polynomials such that $g(X, C_0, C_1, \cdots, a_0(X), a_1(X), \cdots)$ vanishes over $D$ whenever the discrete log relation returned by the adversary's function $h$ is trivial. This trivially gives us that the extractor function $e$ succeeds with probability bounded above by $\pfail$ as required.

#### Defining $\badch(\tr')$

Recall from before that the following $n$ equalities hold:

$$
\sum_{i=0}^{k - 1} u_j^{-1} \repv{L_j}{G}{i} + \repv{P'}{G}{i} + \sum_{i=0}^{k - 1} u_j \repv{R_j}{G}{i} = c \mathbf{s}_i \forall i \in [0, n)
$$

as well as the equality

$$
\sum_{i=0}^{k - 1} u_j^{-1} \repr{L_j}{U} + \repr{P'}{U} + \sum_{i=0}^{k - 1} u_j \repr{R_j}{U} = c z \kappa(x_3)
$$

For convenience let us introduce the following notation

$$
\begin{array}{ll}
\mv{G}{i}{m} &= \sum_{i=0}^{m - 1} u_j^{-1} \repv{L_j}{G}{i} + \repv{P'}{G}{i} + \sum_{i=0}^{m - 1} u_j \repv{R_j}{G}{i} \\[1ex]
\m{U}{m} &= \sum_{i=0}^{m - 1} u_j^{-1} \repr{L_j}{U} + \repr{P'}{U} + \sum_{i=0}^{m - 1} u_j \repr{R_j}{U}
\end{array}
$$

so that we can rewrite the above (after expanding for $\kappa(x_3)$) as

$$
\mv{G}{i}{k} = c \mathbf{s}_i \forall i \in [0, n)
$$

$$
\m{U}{k} = c z \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j})
$$

We can combine these equations by multiplying both sides of each instance of the first equation by $\mathbf{s}_i^{-1}$ (because $\mathbf{s}_i$ is never zero) and substituting for $c$ in the second equation, yielding the following $n$ equalities:

$$
\m{U}{k} = \mv{G}{i}{k} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \forall i \in [0, n)
$$

> **Lemma 1.** If $\m{U}{k} = \mv{G}{i}{k} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \forall i \in [0, n)$ then it follows that $\repr{P'}{U} = z \sum\limits_{i=0}^{2^k - 1} x_3^i \repv{P'}{G}{i}$ for all transcripts that do not contain bad challenges.
>
> _Proof._ It will be useful to introduce yet another abstraction defined starting with
> $$
\z{k}{m}{i} = \mv{G}{i}{m}
$$
> and then recursively defined for all integers $r$ such that $0 \lt r \leq k$
> $$
\z{k - r}{m}{i} = \z{k - r + 1}{m}{i} + x_3^{2^{k - r}} \z{k - r + 1}{m}{i + 2^{k - r}}
$$
> This allows us to rewrite our above equalities as
> $$
\m{U}{k} = \z{k}{k}{i} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \forall i \in [0, n)
$$
>
> We will now show that for all integers $r$ such that $0 \lt r \leq k$ that whenever the following holds for $r$
> $$
\m{U}{r} = \z{r}{r}{i} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \forall i \in [0, 2^r)
$$
> that the same _also_ holds for
> $$
\m{U}{r - 1} = \z{r - 1}{r - 1}{i} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 2 - j} x_3^{2^j}) \forall i \in [0, 2^{r-1})
$$
>
> For all integers $r$ such that $0 \lt r \leq k$ we have that $\mathbf{s}_{i + 2^{r - 1}} = u_{r - 1} \mathbf{s}_i \forall i \in [0, 2^{r - 1})$ by the definition of $\mathbf{s}$. This gives us $\mathbf{s}_{i+2^{r - 1}}^{-1} = \mathbf{s}_i^{-1} u_{r - 1}^{-1} \forall i \in [0, 2^{r - 1})$ as no value in $\mathbf{s}$ nor any challenge $u_r$ are zeroes. We can use this to relate one half of the equalities with the other half as so:
> $$
\begin{array}{rl}
\m{U}{r} &= \z{r}{r}{i} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\
&= \z{r}{r}{i + 2^{r - 1}} \cdot \mathbf{s}_i^{-1} u_{r - 1}^{-1} z \prod_{j=0}^{r - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> Notice that $\z{r}{r}{i}$ can be rewritten as $u_{r - 1}^{-1} \repv{L_{r - 1}}{G}{i} + \z{r}{r - 1}{i} + u_{r - 1} \repv{R_{r - 1}}{G}{i}$ for all $i \in [0, 2^{r})$. Thus we can rewrite the above as
>
> $$
\begin{array}{rl}
\m{U}{r} &= \left( u_{r - 1}^{-1} \repv{L_{r - 1}}{G}{i} + \z{r}{r - 1}{i} + u_{r - 1} \repv{R_{r - 1}}{G}{i} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\
&= \left( u_{r - 1}^{-1} \repv{L_{r - 1}}{G}{i + 2^{r - 1}} + \z{r}{r - 1}{i + 2^{r - 1}} + u_{r - 1} \repv{R_{r - 1}}{G}{i + 2^{r - 1}} \right) \\
&\cdot \; \mathbf{s}_i^{-1} u_{r - 1}^{-1} z \prod_{j=0}^{r - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> Now let us rewrite these equalities substituting $u_{r - 1}$ with formal indeterminate $X$.
>
> $$
\begin{array}{rl}
& X^{-1} \repr{L_{r - 1}}{U} + \m{U}{r - 1} + X \repr{R_{r - 1}}{U} \\
&= \left( X^{-1} \repv{L_{r - 1}}{G}{i} + \z{r}{r - 1}{i} + X \repv{R_{r - 1}}{G}{i} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (1 + x_3^{2^{r - 1}} X) \\
&= \left( X^{-1} \repv{L_{r - 1}}{G}{i + 2^{r - 1}} + \z{r}{r - 1}{i + 2^{r - 1}} + X \repv{R_{r - 1}}{G}{i + 2^{r - 1}} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (X^{-1} + x_3^{2^{r - 1}}) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> Now let us rescale everything by $X^2$ to remove negative exponents.
>
> $$
\begin{array}{rl}
& X \repr{L_{r - 1}}{U} + X^2 \m{U}{r - 1} + X^3 \repr{R_{r - 1}}{U} \\
&= \left( X^{-1} \repv{L_{r - 1}}{G}{i} + \z{r}{r - 1}{i} + X \repv{R_{r - 1}}{G}{i} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (X^2 + x_3^{2^{r - 1}} X^3) \\
&= \left( X^{-1} \repv{L_{r - 1}}{G}{i + 2^{r - 1}} + \z{r}{r - 1}{i + 2^{r - 1}} + X \repv{R_{r - 1}}{G}{i + 2^{r - 1}} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (X + x_3^{2^{r - 1}} X^2) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> This gives us $2^{r - 1}$ triples of maximal degree-$4$ polynomials in $X$ that agree at $u_{r - 1}$ despite having coefficients determined prior to the choice of $u_{r - 1}$. The probability that two of these polynomials would agree at $u_{r - 1}$ and yet be distinct would be $\frac{4}{|\ch|}$ by the Schwartz-Zippel lemma and so by the union bound the probability that the three of these polynomials agree and yet any of them is distinct from another is $\frac{8}{|\ch|}$. By the union bound again the probability that any of the $2^{r - 1}$ triples have multiple distinct polynomials is $\frac{2^{r - 1}\cdot8}{|\ch|}$. By restricting the challenge space for $u_{r - 1}$ accordingly we obtain $|\badch(\trprefix{\tr'}{u_r})|/|\ch| \leq \frac{2^{r - 1}\cdot8}{|\ch|}$ for integers $0 \lt r \leq k$ and thus $|\badch(\trprefix{\tr'}{u_k})|/|\ch| \leq \frac{4n}{|\ch|} \leq \epsilon$.
> 
> We can now conclude an equality of polynomials, and thus of coefficients. Consider the coefficients of the constant terms first, which gives us the $2^{r - 1}$ equalities
> $$
0 = 0 = \mathbf{s}_i^{-1} z \left( \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \right) \cdot \repv{L_{r - 1}}{G}{i + 2^{r - 1}} \forall i \in [0, 2^{r - 1})
$$
>
> No value of $\mathbf{s}$ is zero, $z$ is never chosen to be $0$ and each $u_j$ is chosen so that $1 + u_{k - 1 - j} x_3^{2^j}$ is nonzero, so we can then conclude 
> $$
0 = \repv{L_{r - 1}}{G}{i + 2^{r - 1}} \forall i \in [0, 2^{r - 1})
$$
>
> An identical process can be followed with respect to the coefficients of the $X^4$ term in the equalities to establish $0 = \repv{R_{r - 1}}{G}{i} \forall i \in [0, 2^{r - 1})$ contingent on $x_3$ being nonzero, which it always is. Substituting these in our equalities yields us something simpler
> 
> $$
\begin{array}{rl}
& X \repr{L_{r - 1}}{U} + X^2 \m{U}{r - 1} + X^3 \repr{R_{r - 1}}{U} \\
&= \left( X^{-1} \repv{L_{r - 1}}{G}{i} + \z{r}{r - 1}{i} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (X^2 + x_3^{2^{r - 1}} X^3) \\
&= \left( \z{r}{r - 1}{i + 2^{r - 1}} + X \repv{R_{r - 1}}{G}{i + 2^{r - 1}} \right) \\
&\cdot \; \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) (X + x_3^{2^{r - 1}} X^2) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> Now we will consider the coefficients in $X$, which yield the equalities
>
> $$
\begin{array}{rl}
\repr{L_{r - 1}}{U} &= \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \cdot \repv{L_{r - 1}}{G}{i} \\
&= \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \cdot \z{r}{r - 1}{i + 2^{r - 1}} \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> which for similar reasoning as before yields the equalities
> $$
\repv{L_{r - 1}}{G}{i} = \z{r}{r - 1}{i + 2^{r - 1}} \forall i \in [0, 2^{r - 1})
$$
>
> Finally we will consider the coefficients in $X^2$ which yield the equalities
>
> $$
\begin{array}{rl}
\m{U}{r - 1} &= \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \cdot \left( \z{r}{r - 1}{i} + \repv{L_{r - 1}}{G}{i} x_3^{2^{r - 1}} \right) \\
&\forall i \in [0, 2^{r - 1})
\end{array}
$$
>
> which by substitution gives us $\forall i \in [0, 2^{r - 1})$
> $$
\m{U}{r - 1} = \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \cdot \left( \z{r}{r - 1}{i} + \z{r}{r - 1}{i + 2^{r - 1}} x_3^{2^{r - 1}} \right)
$$
>
> Notice that by the definition of $\z{r - 1}{m}{i}$ we can rewrite this as
>
> $$
\m{U}{r - 1} = \z{r - 1}{r - 1}{i} \cdot \mathbf{s}_i^{-1} z \prod_{j=0}^{r - 2} (1 + u_{k - 1 - j} x_3^{2^j}) \forall i \in [0, 2^{r - 1})
$$
>
> which is precisely in the form we set out to demonstrate.
>
> We now proceed by induction from the case $r = k$ (which we know holds) to reach $r = 0$, which gives us
$$
\m{U}{0} = \z{0}{0}{0} \cdot \mathbf{s}_0^{-1} z
$$
>
> and because $\m{U}{0} = \repr{P'}{U}$ and $\z{0}{0}{0} = \sum_{i=0}^{2^k - 1} x_3^i \repv{P'}{G}{i}$, we obtain $\repr{P'}{U} = z \sum_{i=0}^{2^k - 1} x_3^i \repv{P'}{G}{i}$, which completes the proof.

Having established that $\repr{P'}{U} = z \sum_{i=0}^{2^k - 1} x_3^i \repv{P'}{G}{i}$, and given that $x_3$ and $\repv{P'}{G}{i}$ are fixed in advance of the choice of $z$, we have that at most one value of $z \in \ch$ (which is nonzero) exists such that $\repr{P'}{U} = z \sum_{i=0}^{2^k - 1} x_3^i \repv{P'}{G}{i}$ and yet $\repr{P'}{U} \neq 0$. By restricting $|\badch(\trprefix{\tr'}{z})|/|\ch| \leq \frac{1}{|\ch|} \leq \epsilon$ accordingly we obtain $\repr{P'}{U} = 0$ and therefore that the polynomial defined by $\repr{P'}{\mathbf{G}}$ has a root at $x_3$.

By construction $P' = P - [v] \mathbf{G}_0 + [\xi] S$, giving us that the polynomial defined by $\repr{P + [\xi] S}{\mathbf{G}}$ evaluates to $v$ at the point $x_3$. We have that $v, P, S$ are fixed prior to the choice of $\xi$, and so either the polynomial defined by $\repr{S}{\mathbf{G}}$ has a root at $x_3$ (which implies the polynomial defined by $\repr{P}{\mathbf{G}}$ evaluates to $v$ at the point $x_3$) or else $\xi$ is the single solution in $\ch$ for which $\repr{P + [\xi] S}{\mathbf{G}}$ evaluates to $v$ at the point $x_3$ while $\repr{P}{\mathbf{G}}$ itself does not. We avoid the latter case by restricting $|\badch(\trprefix{\tr'}{\xi})|/|\ch| \leq \frac{1}{|\ch|} \leq \epsilon$ accordingly and can thus conclude that the polynomial defined by $\repr{P}{\mathbf{G}}$ evaluates to $v$ at $x_3$.

The remaining work deals strictly with the representations of group elements sent previously by the prover and their relationship with $P$ as well as the challenges chosen in each round of the protocol. We will simplify things first by using $p(X)$ to represent the polynomial defined by $\repr{P}{\mathbf{G}}$, as it is the case that this $p(X)$ corresponds exactly with the like-named polynomial in the protocol itself. We will make similar substitutions for the other group elements (and their corresponding polynomials) to aid in exposition, as the remainder of this proof is mainly tedious application of the Schwartz-Zippel lemma to upper bound the bad challenge space size for each of the remaining challenges in the protocol.

Recall that $P = Q' + x_4 \sum\limits_{i=0}^{n_q - 1} [x_4^i] Q_i$, and so by substitution we have $p(X) = q'(X) + x_4 \sum\limits_{i=0}^{n_q - 1} x_4^i q_i(X)$. Recall also that

$$
v = \sum\limits_{i=0}^{n_q - 1}
\left(
x_2^i
  \left(
  \frac
  { \mathbf{u}_i - r_i(x_3) }
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      x_3 - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
\right)
+
x_4 \sum\limits_{i=0}^{n_q - 1} x_4 \mathbf{u}_i
$$

We have already established that $p(x_3) = v$. Notice that the coefficients in the above expressions for $v$ and $P$ are fixed prior to the choice of $x_4 \in \ch$. By the Schwartz-Zippel lemma we have that only at most $n_q + 1$ possible choices of $x_4$ exist such that these expressions are satisfied and yet $q_i(x_3) \neq \mathbf{u}_i$ for any $i$ or

$$
q'(x_3) \neq \sum\limits_{i=0}^{n_q - 1}
\left(
x_2^i
  \left(
  \frac
  { \mathbf{u}_i - r_i(x_3) }
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      x_3 - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
\right)
$$

By restricting $|\badch(\trprefix{\tr'}{x_4})|/|\ch| \leq \frac{n_q + 1}{|\ch|} \leq \epsilon$ we can conclude that all of the aforementioned inequalities are untrue. Now we can substitute $\mathbf{u}_i$ with $q_i(x_3)$ for all $i$ to obtain

$$
q'(x_3) = \sum\limits_{i=0}^{n_q - 1}
\left(
x_2^i
  \left(
  \frac
  { q_i(x_3) - r_i(x_3) }
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      x_3 - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
\right)
$$

Suppose that $q'(X)$ (which is the polynomial defined by $\repr{Q'}{\mathbf{G}}$, and is of degree at most $n - 1$) does _not_ take the form

$$\sum\limits_{i=0}^{n_q - 1}

x_2^i
  \left(
  \frac
  {q_i(X) - r_i(X)}
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      X - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
$$

and yet $q'(X)$ agrees with this expression at $x_3$ as we've established above. By the Schwartz-Zippel lemma this can only happen for at most $n - 1$ choices of $x_3 \in \ch$ and so by restricting $|\badch(\trprefix{\tr'}{x_3})|/|\ch| \leq \frac{n - 1}{|\ch|} \leq \epsilon$ we obtain that

$$q'(X) = \sum\limits_{i=0}^{n_q - 1}

x_2^i
  \left(
  \frac
  {q_i(X) - r_i(X)}
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      X - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
  \right)
$$

Next we will extract the coefficients of this polynomial in $x_2$ (which are themselves polynomials in formal indeterminate $X$) by again applying the Schwartz-Zippel lemma with respect to $x_2$; again, this leads to the restriction $|\badch(\trprefix{\tr'}{x_2})|/|\ch| \leq \frac{n_q}{|\ch|} \leq \epsilon$ and we obtain the following polynomials of degree at most $n - 1$ for all $i \in [0, n_q - 1)$

$$
\frac
  {q_i(X) - r_i(X)}
  {\prod\limits_{j=0}^{n_e - 1}
    \left(
      X - \omega^{\left(
        \mathbf{q_i}
      \right)_j} x
    \right)
  }
$$

Having established that these are each non-rational polynomials of degree at most $n - 1$ we can then say (by the factor theorem) that for each $i \in [0, n_q - 1]$ and $j \in [0, n_e - 1]$ we have that $q_i(X) - r_i(X)$ has a root at $\omega^{\left(\mathbf{q_i}\right)_j} x$. Note that we can interpret each $q_i(X)$ as the restriction of a _bivariate_ polynomial at the point $x_1$ whose degree with respect to $x_1$ is at most $n_a + 1$ and whose coefficients consist of various polynomials $a'_i(X)$ (from the representation $\repr{A'_i}{\mathbf{G}}$) as well as $h'(X)$ (from the representation $\repr{H'_i}{\mathbf{G}}$) and $r(X)$ (from the representation $\repr{R}{\mathbf{G}}$). By similarly applying the Schwartz-Zippel lemma and restricting the challenge space with $|\badch(\trprefix{\tr'}{x_1})|/|\ch| \leq \frac{n_a + 1}{|\ch|} \leq \epsilon$ we obtain (by construction of each $q'_i(X)$ and $r_i(X)$ in steps 12 and 13 of the protocol) that the prover's claimed value of $r$ in step 9 is equal to $r(x)$; that the value $h$ computed by the verifier in step 13 is equal to $h'(x)$; and that for all $i \in [0, n_q - 1]$ the prover's claimed values $(\mathbf{a_i})_j = a'_i(\omega^{(\mathbf{p_i})_j} x)$ for all $j \in [0, n_e - 1]$.

By construction of $h'(X)$ (from the representation $\repr{H'}{\mathbf{G}}$) in step 7 we know that $h'(x) = h(x)$ where by $h(X)$ we refer to the polynomial of degree at most $(n_g - 1) \cdot (n - 1)$ whose coefficients correspond to the concatenated representations of each $\repr{H_i}{\mathbf{G}}$. As before, suppose that $h(X)$ does _not_ take the form $g'(X) / t(X)$. Then because $h(X)$ is determined prior to the choice of $x$ then by the Schwartz-Zippel lemma we know that it would only agree with $g'(X) / t(X)$ at $(n_g - 1) \cdot (n - 1)$ points at most if the polynomials were not equal. By restricting again $|\badch(\trprefix{\tr'}{x})|/|\ch| \leq \frac{(n_g - 1) \cdot (n - 1)}{|\ch|} \leq \epsilon$ we obtain $h(X) = g'(X) / t(X)$ and because $h(X)$ is a non-rational polynomial by the factor theorem we obtain that $g'(X)$ vanishes over the domain $D$.

We now have that $g'(X)$ vanishes over $D$ but wish to show that $g(X, C_0, C_1, \cdots)$ vanishes over $D$ at all points to complete the proof. This just involves a sequence of applying the same technique to each of the challenges; since the polynomial $g(\cdots)$ has degree at most $n_g \cdot (n - 1)$ in any indeterminate by definition, and because each polynomial $a_i(X, C_0, C_1, ..., C_{i - 1}, \cdots)$ is determined prior to the choice of concrete challenge $c_i$ by similarly bounding $|\badch(\trprefix{\tr'}{c_i})|/|\ch| \leq \frac{n_g \cdot (n - 1)}{|\ch|} \leq \epsilon$ we ensure that $g(X, C_0, C_1, \cdots)$ vanishes over $D$, completing the proof.
