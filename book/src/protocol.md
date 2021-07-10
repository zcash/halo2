# Protocol (WIP)

## Preliminaries

See [Notation](concepts/notation.md) for notational conventions. Briefly,
recall that $\group$ represents a cryptographic group with identity element
$\zero$; group elements are written in capital letters; scalars are written
in lowercase or Greek letters; and vectors of scalars or group elements are
written in boldface. The notation $\langle \mathbf{a}, \mathbf{b} \rangle$ is
used for an inner product of two same-length vectors of scalars $\mathbf{a},
\mathbf{b} \in \field^n,$ and also for a linear combination of group elements
such as $\langle \mathbf{a}, \mathbf{G} \rangle$ with $\mathbf{a} \in \field^n,
\mathbf{G} \in \group^n,$ computed by a multiscalar multiplication.
$\mathbf{0}^n$ is a vector of length $n$ that contains only zeroes in $\field.$

We take $\sec$ as our security parameter, and unless explicitly noted all
algorithms and adversaries are probabilistic (interactive) Turing machines that
run in polynomial time in this security parameter. We use $\negl$ to denote a
function that is negligible in $\sec.$

> **Discrete Log Relation Problem.** The advantage metric
$$
\adv^\dlrel_{\group,n}(\a, \sec) = \prob \left[ \dlgame(\a, \sec) \right]
$$
> is defined with respect to the following game.
$$
\begin{array}{l}
  \underline{\bold{Game}\; \dlgame(\a, \sec):} \\[0.5ex]
  \mathbf{G} \gets \group^n_\sec \\
  \mathbf{a} \gets \a(\mathbf{G}) \\
  \textnormal{Return } \left( \langle \mathbf{a}, \mathbf{G} \rangle = \zero \land \mathbf{a} \neq \mathbf{0}^n \right).
\end{array}
$$

Given an $n$-length vector $\mathbf{G} \in \group^n$ of group elements, the
_discrete log relation problem_ asks for $\mathbf{g} \in \field^n$ such that
$\mathbf{g} \neq \mathbf{0}^n$ and yet $\innerprod{\mathbf{g}}{\mathbf{G}} =
\zero,$ which we refer to as a _non-trivial_ discrete log relation. The hardness
of this problem is tightly implied by the hardness of the discrete log problem
in the group as shown in Lemma 3 of [[JT20]](https://eprint.iacr.org/2020/1213).
Formally, we use the game $\dlgame$ defined above to capture this problem.

### Interactive Proofs

_Interactive proofs_ are a triple of algorithms $\ip = (\setup, \prover,
\verifier).$ The algorithm $\setup(1^\sec)$ produces as its output some _public
parameters_ commonly refered to by $\pp.$ The prover $\prover$ and verifier
$\verifier$ are interactive machines (with access to $\pp$) and we denote by
$\langle \prover(x), \verifier(y) \rangle$ an algorithm that executes a
two-party protocol between them on inputs $x, y.$ The output of this protocol, a
_transcript_ of their interaction, contains all of the messages sent between
$\prover$ and $\verifier.$ At the end of the protocol, the verifier outputs a
decision bit.

### Zero knowledge Arguments of Knowledge

Proofs of knowledge are interactive proofs where the prover aims to convince the
verifier that they know a witness $w$ such that $(x, w) \in \relation$ for a
statement $x$ and polynomial-time decidable relation $\relation.$ We will work
with _arguments_ of knowledge which assume computationally-bounded provers.

We will analyze arguments of knowledge through the lens of four security
notions.

* **Completeness:** If the prover possesses a valid witness, can they _always_
  convince the verifier? It is useful to understand this property as it can have
  implications for the other security notions.
* **Soundness:** Can a cheating prover falsely convince the verifier of the
  correctness of a statement that is not actually correct? We refer to the
  probability that a given cheating prover can falsely convince the verifier as
  the _soundness error_ against that prover.
* **Knowledge soundness:** When the verifier is convinced the statement is
  correct, does the prover actually possess ("know") a valid witness? We refer
  to the probability that a given cheating prover falsely convinces the verifier
  of this knowledge as the _knowledge error_ against that prover.
* **Zero knowledge:** Does the verifier learn anything besides that which can be
  inferred from the correctness of the statement and the prover's knowledge of a
  valid witness?

First, we will visit the simple definition of completeness.

> **Perfect Completeness.** An interactive argument $(\setup, \prover, \verifier)$
> has _perfect completeness_ if for all polynomial-time decidable
> relations $\relation$ and for all non-uniform polynomial-time adversaries $\a,$
$$
\prob \left[
    (x, w) \notin \relation \lor
    \langle \prover(\pp, x, w), \verifier(\pp, x) \rangle \textnormal{ accepts }\,
    \middle|\begin{array}{l}
      \pp \gets \setup(1^\sec) \\
      (x, w) \gets \a(\pp)
    \end{array}
\right] = 1.
$$

#### Soundness

Complicating our analysis is that although our protocol is described as an
interactive argument, it is realized in practice as a _non-interactive argument_
through the use of the Fiat–Shamir transformation.

> **Public-coin.** We say that an interactive argument is _public-coin_ when all
> of the messages sent by the verifier are each sampled with fresh randomness.

> **Fiat–Shamir transformation.** In this transformation an interactive, public-coin
> argument can be made _non-interactive_ in the _random oracle model_ by
> replacing the verifier algorithm with a cryptographically strong hash function
> that produces sufficiently random-looking output.

This transformation means that in the concrete protocol a cheating prover can
easily "rewind" the verifier by forking the transcript and sending new messages
to the verifier. Studying the concrete security of our construction _after_
applying this transformation is important. Fortunately, we are able to follow a
framework of analysis by Ghoshal and Tessaro
([[GT20]](https://eprint.iacr.org/2020/1351)) that has been applied to
constructions similar to ours.

We will study our protocol through the notion of _state-restoration soundness_.
In this model the (cheating) prover is allowed to rewind the verifier to any
previous state it was in. The prover wins if they are able to produce an
accepting transcript.

> **State-Restoration Soundness.** Let $\ip$ be an interactive argument with
> $r = r(\sec)$ verifier challenges and let the $i$th challenge be sampled from
> $\ch_i.$ The advantage metric
$$
\adv^\srs_\ip(\prover, \sec) = \prob \left[ \srs^\ip_\prover(\sec) \right]
$$
> of a state restoration prover $\prover$ is defined with respect to the
> following game.
$$
\begin{array}{ll}
  \begin{array}{l}
    \underline{\bold{Game}\; \srs_\ip^\prover(\sec):} \\[0.5ex]
    \textnormal{win} \gets \tt{false} \\
    \tr \gets \epsilon \\
    \pp \gets \ip.\setup(1^\sec) \\
    (x, \textsf{st}_\prover) \gets \prover_\sec(\pp) \\
    \textnormal{Run } \prover^{\oracle_\srs}_\sec(\textsf{st}_\prover) \\
    \textnormal{Return win}. \\[5.5ex]
  \end{array} &
  \begin{array}{l}
    \underline{\bold{Oracle}\; \oracle_\srs(\tau = (a_1, c_1, ..., a_{i - 1}, c_{i - 1}), a_i):} \\[0.5ex]
    \textnormal{If } \tau \in \tr \textnormal{ then} \\
    \tab \textnormal{If } i \leq r \textnormal{ then} \\
    \tab \tab c_i \gets \ch_i;\, \tr \gets \tr \,||\, (\tau, a_i, c_i);\, \textnormal{Return } c_i \\
    \tab \textnormal{Else if } i = r + 1 \textnormal{ then} \\
    \tab \tab d \gets \ip.\verifier (\pp, x, (\tau, a_i));\, \tr \gets (\tau, a_i) \\
    \tab \tab \textnormal{If } d = 1 \textnormal{ then win} \gets \tt{true} \\
    \tab \tab \textnormal{Return } d \\
    \textnormal{Return } \bottom.
  \end{array}
\end{array}
$$

As shown in [[GT20]](https://eprint.iacr.org/2020/1351) (Theorem 1) state
restoration soundness is tightly related to soundness after applying the
Fiat–Shamir transformation.

#### Knowledge Soundness

We will show that our protocol satisfies a strengthened notion of knowledge
soundness known as _witness-extended emulation_. Informally, this notion states
that for any successful prover algorithm there exists an efficient _emulator_
that can extract a witness from it by rewinding it and supplying it with fresh
randomness.

However, we must slightly adjust our definition of witness-extended emulation to
account for the fact that our provers are state restoration provers and can
rewind the verifier. Further, to avoid the need for rewinding the state
restoration prover during witness extraction we study our protocol in the
algebraic group model.

> **Algebraic Group Model (AGM).** An adversary $\alg{\prover}$ is said to be
> _algebraic_ if whenever it outputs a group element $X$ it also outputs a
> _representation_ $\mathbf{x} \in \field^n$ such that
> $\langle \mathbf{x}, \mathbf{G} \rangle = X$ where $\mathbf{G} \in \group^n$
> is the vector of group elements that $\alg{\prover}$ has seen so far.
> Notationally, we write $\left[X\right]$ to describe a group element $X$ enhanced
> with this representation. We also write $[X]^{\mathbf{G}}_i$ to identify the
> component of the representation of $X$ that corresponds with $\mathbf{G}_i.$ In
> other words,
$$
X = \sum\limits_{i=0}^{n - 1} \left[ [X]^{\mathbf{G}}_i \right] \mathbf{G}_i.
$$

The algebraic group model allows us to perform so-called "online" extraction for
some protocols: the extractor can obtain the witness from the representations
themselves for a single (accepting) transcript.

> **State Restoration Witness-Extended Emulation.** Let $\ip$ be an interactive
> argument for relation $\relation$ with $r = r(\sec)$ challenges. For
> all non-uniform algebraic provers $\alg{\prover},$ extractors $\extractor,$
> and computationally unbounded distinguishers $\distinguisher,$ the advantage
> metric
$$
\adv^\srwee_{\ip, \relation}(\alg{\prover}, \distinguisher, \extractor, \sec) = \prob \left[ \weereal^{\prover,\distinguisher}_{\ip,\relation}(\sec) \right] - \prob \left[ \weeideal^{\extractor,\prover,\distinguisher}_{\ip,\relation}(\sec) \right]
$$
> is defined with respect to the following games.
$$
\begin{array}{ll}
  \begin{array}{l}
    \underline{\bold{Game}\; \weereal_{\smash{\ip,\relation}}^{\alg{\prover},\distinguisher}(\sec):} \\[0.5ex]
    \tr \gets \epsilon \\
    \pp \gets \ip.\setup(1^\sec) \\
    (x, \state{\prover}) \gets \alg{\prover}(\pp) \\
    \textnormal{Run } \alg{\prover}^{\oracle_\real}(\state{\prover}) \\
    b \gets \distinguisher(\tr) \\
    \textnormal{Return } b = 1. \\
  \\
    \underline{\bold{Game}\; \weeideal_{\smash{\ip,\relation}}^{\extractor,\alg{\prover},\distinguisher}(\sec):} \\[0.5ex]
    \tr \gets \epsilon \\
    \pp \gets \ip.\setup(1^\sec) \\
    (x, \state{\prover}) \gets \alg{\prover}(\pp) \\
    \state{\extractor} \gets (1^\sec, \pp, x) \\
    \textnormal{Run } \alg{\prover}^{\oracle_\ideal}(\state{\prover}) \\
    w \gets \extractor(\state{\extractor}, \bottom) \\
    b \gets \distinguisher(\tr) \\
    \textnormal{Return } (b = 1) \\
    \tab \land\, (\textnormal{Acc}(\tr) \implies (x, w) \in \relation). \\
  \end{array} &
  \begin{array}{l}
    \underline{\bold{Oracle}\; \oracle_\real(\tau = (a_1, c_1, ..., a_{i - 1}, c_{i - 1}), a_i):} \\[0.5ex]
    \textnormal{If } \tau \in \tr \textnormal{ then} \\
    \tab \textnormal{If } i \leq r \textnormal{ then} \\
    \tab \tab c_i \gets \ch_i;\, \tr \gets \tr \,||\, (\tau, a_i, c_i);\, \textnormal{Return } c_i \\
    \tab \textnormal{Else if } i = r + 1 \textnormal{ then} \\
    \tab \tab d \gets \ip.\verifier (\pp, x, (\tau, a_i));\, \tr \gets (\tau, a_i) \\
    \tab \tab \textnormal{If } d = 1 \textnormal{ then win} \gets \tt{true} \\
    \tab \tab \textnormal{Return } d \\
    \tab \textnormal{Return } \bottom.
  \\[3ex]
    \underline{\bold{Oracle}\; \oracle_\ideal(\tau, a):} \\[0.5ex]
    \textnormal{If } \tau \in \tr \textnormal{ then} \\
    \tab (r, \state{\extractor}) \gets \extractor(\state{\extractor}, \left[(\tau, a)\right]) \\
    \tab \tr \gets \tr || (\tau, a, r) \\
    \tab \textnormal{Return } r \\
   \textnormal{Return } \bottom. \\[5.8ex]
  \end{array}
\end{array}
$$

#### Zero Knowledge

We say that an argument of knowledge is _zero knowledge_ if the verifier
does not learn anything from their interaction besides that which can be learned
from the existence of a valid $w.$ More formally,

> **Perfect Special Honest-Verifier Zero Knowledge.** A public-coin interactive
> argument $(\setup, \prover, \verifier)$ has _perfect special honest-verifier
> zero knowledge_ (PSHVZK) if for all polynomial-time decidable relations
> $\relation$ and for all $(x, w) \in \relation$ and for all non-uniform
> polynomial-time adversaries $\a_1, \a_2,$ there exists a probabilistic
> polynomial-time simulator $\sim$ such that
$$
\begin{array}{rl}
 &\prob \left[ \a_1(\sigma, x, \tr) = 1 \;\;
    \middle|\begin{array}{l}
      \pp \gets \setup(1^\lambda); \\
      (x, w, \rho) \gets \a_2(\pp); \\
      tr \gets \langle \prover(\pp, x, w), \verifier(\pp, x, \rho) \rangle
    \end{array}
  \right] \\
\\
=&\prob \left[ \a_1(\sigma, x, \tr) = 1 \;\;
    \middle|\begin{array}{l}
      \pp \gets \setup(1^\lambda); \\
      (x, w, \rho) \gets \a_2(\pp); \\
      tr \gets \sim(\pp, x, \rho)
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

## Protocol (sketch)

The protocol we describe will seek to generalize protocols such as PLONK to more sophisticated protocols that involve custom logic gates of higher degree and complexity. We'll start by expressing the standard PLONK protocol here.

* There exists $\omega \in \field$ that is a $n = 2^k$ root of unity generating a domain $D$ of size $n.$ There also exists $\delta$ that generates a coset of this domain.
* The prover holds vectors $\mathbf{a}, \mathbf{b}, \mathbf{c} \in \field^n.$
* There exist fixed "selector" vectors $\mathbf{s_a}, \mathbf{s_b}, \mathbf{s_c}, \mathbf{s_m} \in \field^n$
  that are used to enable and modify the logic of gates described below.
* The also exist fixed vectors that are used to encode equality constraints, as described
  [here](design/proving-system/permutation.md).
* The prover's goal will be to show that for all $i \in [0, n)$
$$
\mathbf{s_a}_i \cdot \mathbf{a}_i + \mathbf{s_b}_i \cdot \mathbf{b}_i + \mathbf{s_m}_i \cdot \mathbf{a}_i \cdot \mathbf{b}_i - \mathbf{s_c}_i \cdot \mathbf{c}_i = 0.
$$
* TODO

## Protocol (formal)

Let $\omega \in \field$ be a $n = 2^k$ primitive root of unity forming the
domain $D = (\omega^0, \omega^1, ..., \omega^{n - 1})$ with $t(X) = X^n - 1$ the
vanishing polynomial over this domain. Let $k, n_g, n_a$ be positive integers.
We present an interactive argument $\halo = (\setup, \prover, \verifier)$ for
the relation
$$
\relation = \left\{
\begin{array}{l}
  \left(
    \begin{array}{l}
      \left(
        g(X, C_0, C_1, ..., C_{n_a - 1})
      \right); \\
      \left(
        a_0(X), a_1(X, C_0), ..., a_{n_a - 1}(X, C_0, C_1, ..., C_{n_a - 1})
      \right)
    \end{array}
  \right) : \\[3ex]
  g(\omega^i, C_0, C_1, ..., C_{n_a - 1}) = 0 \tab \forall i \in [0, 2^k)
\end{array}
\right\}
$$
where $a_0, a_1, ..., a_{n_a - 1}$ are (multivariate) polynomials with degree $n - 1$ in $X$ and $g$ has degree $n_g(n - 1)$ in $X.$

$\setup(\sec)$ returns $\pp = (\group, \field, \mathbf{G} \in \group^n, U, W \in \group).$

For all $i \in [0, n_a)$:
* Let $\mathbf{p_i}$ be the exhaustive set of integers $j$ (modulo $n$) such that $a_i(\omega^j X, \cdots)$ appears as a term in $g(X, \cdots).$
* Let $\mathbf{q}$ be a list of distinct sets of integers containing $\mathbf{p_i}$ and the set $\mathbf{q_0} = \{0\}.$
* Let $\sigma(i) = \mathbf{q}_j$ when $\mathbf{q}_j = \mathbf{p_i}.$

Let $n_q$ denote the size of $\mathbf{q},$ and let $n_e$ denote the size of every $\mathbf{p_i}$ without loss of generality.

In the following protocol, we take it for granted that each polynomial $a_i(X, \cdots)$ is defined such that $n_e + 1$ blinding factors are freshly sampled by the prover and are each present as an evaluation of $a_i(X, \cdots)$ over the domain $D.$

1. $\prover$ and $\verifier$ proceed in the following $n_a$ rounds of interaction, where in round $j$ (starting at $0$)
   * $\prover$ sets $a'_j(X) = a_j(X, c_0, c_1, ..., c_{j - 1}).$
   * $\prover$ sends a hiding commitment $A_j = \innerprod{\mathbf{a'}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{a'}$ are the coefficients of the univariate polynomial $a'_j(X)$ and $\cdot$ is some random, independently sampled blinding factor elided for exposition.
   * $\verifier$ responds with a challenge $c_j.$
2. $\prover$ and $\verifier$ set $g'(X) = g(X, c_0, c_1, ..., c_{n_a - 1}).$
3. $\prover$ sends a commitment $R = \innerprod{\mathbf{r}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{r} \in \field^n$ are the coefficients of a randomly sampled univariate polynomial $r(X)$ of degree $n - 1.$
4. $\prover$ computes univariate polynomial $h(X) = \frac{g'(X)}{t(X)}$ of degree $n_g(n - 1) - n.$
5. $\prover$ computes at most $n - 1$ degree polynomials $h_0(X), h_1(X), ..., h_{n_g - 2}(X)$ such that $h(X) = \sum\limits_{i=0}^{n_g - 2} X^{ni} h_i(X).$
6. $\prover$ sends commitments $H_i = \innerprod{\mathbf{h_i}}{\mathbf{G}} + [\cdot] W$ for all $i$ where $\mathbf{h_i}$ denotes the vector of coefficients for $h_i(X).$
7. $\verifier$ responds with challenge $x$ and computes $H' = \sum\limits_{i=0}^{n_g - 2} [x^{ni}] H_i.$
8. $\prover$ sets $h'(X) = \sum\limits_{i=0}^{n_g - 2} x^{ni} h_i(X).$
9. $\prover$ sends $r = r(x)$ and for all $i \in [0, n_a)$ sends $\mathbf{a_i}$ such that $(\mathbf{a_i})_j = a'_i(\omega^{(\mathbf{p_i})_j} x)$ for all $j \in [0, n_e].$
10. For all $i \in [0, n_a)$ $\prover$ and $\verifier$ set $s_i(X)$ to be the lowest degree univariate polynomial defined such that $s_i(\omega^{(\mathbf{p_i})_j} x) = (\mathbf{a_i})_j$ for all $j \in [0, n_e).$
11. $\verifier$ responds with challenges $x_1, x_2$ and initializes $Q_0, Q_1, ..., Q_{n_q - 1} = \zero.$
    * Starting at $i=0$ and ending at $n_a - 1,$ $\verifier$ sets $$Q_{\sigma(i)} := [x_1] Q_{\sigma(i)} + A_i.$$
    * $\verifier$ finally sets $$Q_0 := [x_1^2] Q_0 + [x_1] H' + R.$$
12. $\prover$ initializes $q_0(X), q_1(X), ..., q_{n_q - 1}(X) = 0.$
    * Starting at $i=0$ and ending at $n_a - 1,$ $\prover$ sets $$q_{\sigma(i)} := x_1 q_{\sigma(i)} + a'_i(X).$$
    * $\prover$ finally sets $$q_0(X) := x_1^2 q_0(X) + x_1 h'(X) + r(X).$$
13. $\prover$ and $\verifier$ initialize $r_0(X), r_1(X), ..., r_{n_q - 1}(X) = 0.$
    * Starting at $i=0$ and ending at $n_a - 1,$ $\prover$ and $\verifier$ set $$r_{\sigma(i)}(X) := x_1 r_{\sigma(i)}(X) + s_i(X).$$
    * Finally $\prover$ and $\verifier$ set $r_0 := x_1^2 r_0 + x_1 h + r,$ where $h$ is computed by $\verifier$ as $\frac{g'(x)}{t(x)}$ using the values $r, \mathbf{a}$ provided by $\prover.$
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
15. $\verifier$ responds with challenge $x_3.$
16. $\prover$ sends $\mathbf{u} \in \field^{n_q}$ such that $\mathbf{u}_i = q_i(x_3)$ for all $i \in [0, n_q).$
17. $\verifier$ responds with challenge $x_4.$
18. $\prover$ and $\verifier$ set $P = Q' + x_4 \sum\limits_{i=0}^{n_q - 1} [x_4^i] Q_i$ and
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
.$$
19. $\prover$ sets $p(X) = q'(X) + [x_4] \sum\limits_{i=0}^{n_q - 1} x_4^i q_i(X).$
20. $\prover$ samples a random polynomial $s(X)$ of degree $n - 1$ with a root at $x_3$ and sends a commitment $S = \innerprod{\mathbf{s}}{\mathbf{G}} + [\cdot] W$ where $\mathbf{s}$ defines the coefficients of $s(X).$
21. $\verifier$ responds with challenges $\xi, z.$
22. $\prover$ and $\verifier$ set $P' = P - [v] \mathbf{G}_0 + [\xi] S.$
23. $\prover$ sets $p'(X) = p(X) - v + \xi s(X).$
24. Initialize $\mathbf{p'}$ as the coefficients of $p'(X)$ and $\mathbf{G'} = \mathbf{G}$ and $\mathbf{b} = (x_3^0, x_3^1, ..., x_3^{n - 1}).$ $\prover$ and $\verifier$ will interact in the following $k$ rounds, where in the $j$th round starting in round $j=0$ and ending in round $j=k-1$:
    * $\prover$ sends $$\begin{array}{rl} & \kern0.08em L_j = \innerprod{\mathbf{p'}_\hi}{\mathbf{G'}_\lo} + [z \innerprod{\mathbf{p'}_\hi}{\mathbf{b}_\lo}] U + [\cdot] W \\[1ex] \text{and} & R_j = \innerprod{\mathbf{p'}_\lo}{\mathbf{G'}_\hi} + [z \innerprod{\mathbf{p'}_\lo}{\mathbf{b}_\hi}] U + [\cdot] W.\end{array}$$
    * $\verifier$ responds with challenge $u_j.$
    * $\prover$ and $\verifier$ set $\mathbf{G'} := \mathbf{G'}_\lo + u_j \mathbf{G'}_\hi$ and $\mathbf{b} = \mathbf{b}_\lo + u_j \mathbf{b}_\hi.$
    * $\prover$ sets $\mathbf{p'} := \mathbf{p'}_\lo + u_j^{-1} \mathbf{p'}_\hi.$
25. $\prover$ sends $c = \mathbf{p'}_0$ and synthetic blinding factor $f.$
26. $\verifier$ accepts only if $$\sum_{j=0}^{k - 1} [u_j^{-1}] L_j + P' + \sum_{j=0}^{k - 1} [u_j] R_j = [c] \mathbf{G'}_0 + [c \mathbf{b}_0 z] U + [f] W.$$

### Proof

We claim that $\protocol$ is _perfectly complete_ so long as a valid witness of each $a_i(X, \cdots)$ exists for all challenges $c_0, c_1, ..., c_{n_a - 1}.$ This is trivial and can be seen from inspection of the protocol.

We also claim that $\protocol$ has _perfect special honest-verifier zero knowledge_:

- Observe that throughout the protocol all commitments sent by the prover are blinded perfectly with the generator $W$ by a freshly sampled random blinding factor, and so all commitments are perfectly blinded and thus uniformly distributed.
- The value $c$ provided by the prover at the end is uniformly distributed amongst polynomials having a root at $x_3,$ and the commitment $S$ provided by the prover has a uniformly random blinding factor, so the value $f$ also provided at the end is uniformly distributed.
- Finally, the openings $r, \mathbf{a}, \mathbf{u}$ are uniformly distributed because the prover chooses $n_e + 1$ fresh blinding factors when defining each of $a_i(X, \cdots),$ which is equal to the number of evaluations of each polynomial that are present in the transcript, and because only a linear combination of the random polynomial $r(X)$ and $h'(X)$ are opened at $x$ the proof does not reveal any other information about $h(X).$

We will thus define a simulator $\sim$ for $\protocol$ as follows: first, sample random polynomials $a_i(X, \cdots)$ and proceed normally. Using knowledge of challenge $x$ sample random $h(X)$ of degree ${n_g(n - 1) - n}$ such that $h(X) \cdot t(X)$ agrees with $g'(X)$ at $x.$ Then, proceed normally for the rest of the protocol. The transcript produced by $\sim$ will be indistinguishable from that of an honest prover for all valid witnesses because the commitments and evaluations of polynomials are identically distributed.

#### Witness-extended Emulation

Let $\protocol = \protocol[\group]$ be the interactive argument described above for relation $\relation$ and some group $\group$ with scalar field $\field.$ Let $\mathcal{C}$ be a function that takes as input challenges $c_0, c_1, ...$ and univariate polynomials $a'_0(X), a'_1(X), ...$ and produces a satisfying witness for the relation $\relation$ with probability bounded above by $\epsilon \leq \negl$ whenever $g'(\omega^i) = 0$ for all $i \in [0, n).$ We can construct an extractor $\extractor$ such that for any non-uniform algebraic prover $\alg{\prover}$ making at most $q$ queries to its oracle, there exists a non-uniform adversary $\mathcal{F}$ with the property that for any computationally unbounded distinguisher $\mathcal{D}$

$$
\adv^\srwee_{\ip, \relation}(\alg{\prover}, \distinguisher, \extractor, \sec) \leq q\epsilon + \adv^\dlrel_{\group,n}(\mathcal{F}, \sec).
$$

_Proof._ We will prove this by invoking Theorem 3 of [[GT20]](https://eprint.iacr.org/2020/1351) by describing an extractor function $e,$ by defining a set of bad challenges $\badch_{\tr'_c}$ for each possible (valid) partial transcript $\tr'_c$ that contains all messages before the challenge $c$ is sampled, by upper bounding each $|\badch_{\tr'_c}| \leq \epsilon,$ and by showing that there exists adversary $\mathcal{F}$ such that

$$
p_\textnormal{fail}(\protocol, \alg{\prover}, e, \relation) \leq \adv^\dlrel_{\group,n}(\mathcal{F}, \sec)
$$

where $p_\textnormal{fail}$ represents the probability that $e$ does not return a valid witness.

Recall that a valid transcript is accepting on the condition that

$$
\sum_{i=0}^{k - 1} [u_j^{-1}] L_j + P' + \sum_{i=0}^{k - 1} [u_j] R_j = [c] \mathbf{G'}_0 + [c \mathbf{b}_0 z] U + [f] W
$$

and so by taking the representations of each of the prover's commitments with respect to each group element in $\pp$ and rearranging so that they equal zero, we will (efficiently) obtain discrete log relations. Our adversary $\mathcal{F}$ will simulate $\protocol$ to $\alg{\prover}$ and rearrange as stated, and return the first non-trivial discrete log relation (or a trivial relation in the absence of any). This establishes the upper bound on $p_\textnormal{fail}.$ In order to complete the proof, we must show the operation of the extractor function $e$ and prove that whenever $\mathcal{F}$ fails to obtain a non-trivial discrete log relation that the extractor function $e$ will obtain a valid witness from the transcript.

We will now proceed to describe the operation of the function $e$ by working our way backwards from the protocol execution. In the last step, the prover has supplied $c, f$ such that

$$
\sum_{i=0}^{k - 1} [u_j^{-1}] L_j + P' + \sum_{i=0}^{k - 1} [u_j] R_j = [c] \mathbf{G'}_0 + [c \mathbf{b}_0 z] U + [f] W.
$$

We will perform some substitutions to aid in exposition. First, let us define the polynomial

$$
\kappa(X) = \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} X^{2^j})
$$

so that we can write $\mathbf{b}_0 = \kappa(x_3).$ The coefficient vector $\mathbf{s}$ of $\kappa(X)$ is defined such that

$$\mathbf{s}_i = \prod\limits_{j=0}^{k-1} u_{k - 1 - j}^{f(i, j)}$$

where $f(i, j)$ returns $1$ when the $j$th bit of $i$ is set, and $0$ otherwise. We can also write $\mathbf{G'}_0 = \innerprod{\mathbf{s}}{\mathbf{G}}.$ By taking representations and rearranging, we get the $n$ equalities

$$
\sum_{j=0}^{k - 1} u_j^{-1} [L_j]^{\mathbf{G}}_i + [P']^{\mathbf{G}}_i + \sum_{j=0}^{k - 1} u_j [R_j]^{\mathbf{G}}_i = c \mathbf{s}_i \forall i \in [0, n)
$$

and the equality

$$
\sum_{j=0}^{k - 1} u_j^{-1} [L_j]^U + [P']^U + \sum_{j=0}^{k - 1} u_j [R_j]^U = cz \prod_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}).
$$

Recall that all of these equalities hold or else $\mathcal{F}$ will return a non-trivial discrete log relation. Substituting for $c,$ we obtain the following $n$ equalities

$$
\begin{array}{ll}
  &\sum\limits_{j=0}^{k - 1} u_j^{-1} [L_j]^U + [P']^U + \sum\limits_{j=0}^{k - 1} u_j [R_j]^U \\
= &s_i^{-1} z \prod\limits_{j=0}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\[2ex]
  &\tab \cdot \left( \sum\limits_{j=0}^{k - 1} u_j^{-1} [L_j]^{\mathbf{G}}_i + [P']^{\mathbf{G}}_i + \sum\limits_{j=0}^{k - 1} u_j [R_j]^{\mathbf{G}}_i \right) \\[3.5ex]
  &\forall i \in [0, n)
\end{array}
$$

If we let $r = k - 1$ we can rewrite the above as

$$
\begin{array}{ll}
  &\sum\limits_{j=0}^{r} u_j^{-1} [L_j]^U + [P']^U + \sum\limits_{j=0}^{r} u_j [R_j]^U \\
= &s_{2^{k - 1 - r} i}^{-1} z \prod\limits_{j=k - 1 - r}^{k - 1} (1 + u_{k - 1 - j} x_3^{2^j}) \\[3ex]
  &\tab \cdot \left(
    \sum\limits_{j=0}^{r} \left(
      \sum\limits_{h=0}^{2^{k - 1 - r} - 1} x_3^h \left(
        u_j^{-1} [L_j]^{\mathbf{G}}_{2^{k - 1 - r}i + h} + u_j [R_j]^{\mathbf{G}}_{2^{k - 1 - r}i + h}
      \right)
    \right)
    + \sum\limits_{h=0}^{2^{k - 1 - r} - 1} x_3^h \left(
      [P']^{\mathbf{G}}_{2^{k - 1 - r}i + h}
    \right)
  \right) \\[4ex]
  &\forall i \in [0, 2^{r + 1})
\end{array}
$$

By substituting and simplifying it can be seen that the former expression is a special case of the latter (where $r = k - 1$). We will now show that when these expressions hold for the case of $r \geq 0,$ they hold for the case of $r - 1$ also.

It is the case that for $k - 1 \geq r \geq 0$ we have that $\mathbf{s}_{2^{k - r}i + 2^{k - 1 - r}} = \mathbf{s}_{2^{k - r}i} \cdot u_r$ for all $i \in [0, 2^r).$ This allows us to relate the odd and even values of $i$ in the above equalities.

snip

Observe that in the base case of $r = -1$ where one equality holds ($i = 0$), we obtain

$$
\begin{array}{ll}
[P']^U &= s_{2^k i}^{-1} z \cdot \sum\limits_{h=0}^{2^k - 1} x_3^h \left(
    [P']^{\mathbf{G}}_{2^k i + h}
  \right) \\
&= z \cdot \sum\limits_{h=0}^{n-1} x_3^h [P']^{\mathbf{G}}_h
\end{array}
$$

Because $[P']$ was determined prior to the choice of $z,$ for nonzero $[P']^U$ this equality would hold with probability $\frac{1}{|\ch|}$ and so by bounding $|\badch_{\tr'_z}| \leq \frac{1}{|\ch|} \leq \epsilon$ we can conclude $$[P']^U = \sum\limits_{h=0}^{n-1} x_3^h [P']^{\mathbf{G}}_h = 0$$ and therefore that $[P']^{\mathbf{G}}$ describes a $n - 1$ degree polynomial that has a root at $x_3.$

Because $[S]$ was determined prior to the choice of $\xi,$ and that $P' = P - [v] \mathbf{G}_0 + [\xi] S,$ then if $[S]^\mathbf{G}$ does not describe the coefficients of a degree $n - 1$ polynomial with a root at $x_3$ then with high probability $[P']^{\mathbf{G}}$ would not either. By bounding $|\badch_{\tr'_\xi}| \leq \frac{n}{|\ch|} \leq \epsilon$ and correcting for $v$ we obtain that $[P]^{\mathbf{G}}$ describes the coefficients of a polynomial that evaluates to $v$ at $x_3.$

Recall that $P = Q' + x_4 \sum\limits_{i=0}^{n_q - 1} [x_4^i] Q_i$ and
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
and that $[Q'], [Q_0], ..., [Q_{n_q - 1}], \mathbf{u}$ are determined prior to the choice of $x_4.$ The probability that $[P]^{\mathbf{G}}$ evaluates to $v$ at $x_3$ and yet any of $[Q_i]^{\mathbf{G}}$ do not evaluate to $\mathbf{u}_i$ at $x_3$ or that $[Q']^{\mathbf{G}}$ does not evaluate to 
$$
\sum\limits_{i=0}^{n_q - 1}
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
$$
at $x_3$ is at most $\frac{n_q}{|\ch|}$ and so by bounding $|\badch_{\tr'_{x_4}}| \leq \frac{n_q}{|\ch|} \leq \epsilon$ we obtain from $[Q']^{\mathbf{G}}, [Q_0]^{\mathbf{G}}, ..., [Q_{n_q - 1}]^{\mathbf{G}}$ polynomials that evaluate to those respective values at $x_3.$ Henceforth we will refer to these polynomials as $q'(X), q_0(X), ..., q_{n_q - 1}(X),$ respectively.

Observe that
$$
q'(x_3) = \sum\limits_{i=0}^{n_q - 1}
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
$$
and that $q'(X)$ is a degree $n - 1$ polynomial determined prior to the choice of $x_3$ and $\mathbf{u}.$ Recall that $\mathbf{u}_i = q_i(x_3).$ By bounding $|\badch_{\tr'_{x_3}}| \leq \frac{n}{|\ch|} \leq \epsilon$ similarly to before we can conclude
$$
q'(X) = \sum\limits_{i=0}^{n_q - 1}
x_2^i
\left(
\frac
{ q_i(X) - r_i(X) }
{\prod\limits_{j=0}^{n_e - 1}
  \left(
    X - \omega^{\left(
      \mathbf{q_i}
    \right)_j} x
  \right)
}
\right).
$$

The probability that $q'(X)$ is a degree $n - 1$ polynomial and yet any of the
numerators ${q_i(X) - r_i(X)}$ are not perfectly divisible by the denominators $\prod_{j=0}^{n_e - 1}
  \left(
    X - \omega^{\left(
      \mathbf{q_i}
    \right)_j} x
  \right)$
is at most $\frac{n_q}{|\ch|}$ and so by bounding $|\badch_{\tr'_{x_2}}| \leq \frac{n_q}{|\ch|} \leq \epsilon$ we can conclude via the factor theorem and the prover's supplied values $\mathbf{a}, r$ the values of each $q_i(\omega^{(\mathbf{q_i})_j} x).$

Next we will bound $|\badch_{\tr'_{x_1}}| \leq \frac{n_e}{|\ch|} \leq \epsilon$ so that we can extract the correct evaluations of all $[A_i]^\mathbf{G}, [H']^\mathbf{G}, [R]^\mathbf{G}$ at the various points $\omega^{(\mathbf{q_i})_j} x.$

Since $h'(X)$ is determined after the choice of $x,$ and yet $h'(x) = g'(x)/t(x),$ then the probability that $h'(X)$ is a $n - 1$ degree polynomial that agrees at $x$ and yet $g'(X)$ is not perfectly divisible by $t(X)$ is at most $\frac{n}{|\ch|}$ and so by bounding $|\badch_{\tr'_{x}}| \leq \frac{n}{|\ch|} \leq \epsilon$ we can conclude that for all ${i \in D, g'(\omega_i) = 0.}$

We now invoke $\mathcal{C}$ with the challenges $c_0, ..., c_{n_a - 1}$ and univariate polynomials $a'_0(X), ..., a'_{n_a - 1}$ extracted from each $[A_i]^\mathbf{G},$ to extract a witness satisfying the relation with probability at most $\epsilon,$ and so by bounding each $|\badch_{\tr'_{c_i}}| \leq \epsilon$ we must extract a satisfying witness.
