# Fields

A fundamental component of many cryptographic protocols is the algebraic structure known
as a [field]. Fields are sets of objects (usually numbers) with two associated binary
operators $+$ and $\times$ such that various [field axioms][field-axioms] hold. The real
numbers $\mathbb{R}$ are an example of a field with uncountably many elements.

[field]: https://en.wikipedia.org/wiki/Field_(mathematics)
[field-axioms]: https://en.wikipedia.org/wiki/Field_(mathematics)#Classic_definition

Halo makes use of _finite fields_ which have a finite number of elements. Finite fields
are fully classified as follows:

- if $\mathbb{F}$ is a finite field, it contains $|\mathbb{F}| = p^k$ elements for some
  integer $k \geq 1$ and some prime $p$;
- any two finite fields with the same number of elements are isomorphic. In particular,
  all of the arithmetic in a prime field $\mathbb{F}_p$ is isomorphic to addition and
  multiplication of integers modulo $p$, i.e. in $\mathbb{Z}_p$. This is why we often
  refer to $p$ as the _modulus_.

We'll write a field as $\mathbb{F}_q$ where $q = p^k$. The prime $p$ is called its
_characteristic_. In the cases where $k \gt 1$ the field $\mathbb{F}_q$ is a $k$-degree
extension of the field $\mathbb{F}_p$. (By analogy, the complex numbers
$\mathbb{C} = \mathbb{R}(i)$ are an extension of the real numbers.) However, in Halo we do
not use extension fields. Whenever we write $\mathbb{F}_p$ we are referring to what
we call a _prime field_ which has a prime $p$ number of elements, i.e. $k = 1$.

Important notes:

* There are two special elements in any field: $0$, the additive identity, and
  $1$, the multiplicative identity.
* The least significant bit of a field element, when represented as an integer in binary
  format, can be interpreted as its "sign" to help distinguish it from its additive
  inverse (negation). This is because for some nonzero element $a$ which has a least
  significant bit $0$ we have that $-a = p - a$ has a least significant bit $1$, and vice
  versa. We could also use whether or not an element is larger than $(p - 1) / 2$ to give
  it a "sign."

Finite fields will be useful later for constructing [polynomials](polynomials.md) and
[elliptic curves](curves.md). Elliptic curves are examples of groups, which we discuss
next.

## Groups

Groups are simpler and more limited than fields; they have only one binary operator $\cdot$
and fewer axioms. They also have an identity, which we'll denote as $1$.

[group]: https://en.wikipedia.org/wiki/Group_(mathematics)
[group-axioms]: https://en.wikipedia.org/wiki/Group_(mathematics)#Definition

Any non-zero element $a$ in a group has an _inverse_ $b = a^{-1}$,
which is the _unique_ element $b$ such that $a \cdot b = 1$.
     
For example, the set of nonzero elements of $\mathbb{F}_p$ forms a group, where the
group operation is given by multiplication on the field.

[group]: https://en.wikipedia.org/wiki/Group_(mathematics)

> #### (aside) Additive vs multiplicative notation 
> If $\cdot$ is written as $\times$ or omitted (i.e. $a \cdot b$ written as $ab$), the
> identity as $1$, and inversion as $a^{-1}$, as we did above, then we say that the group
> is "written multiplicatively". If $\cdot$ is written as $+$, the identity as $0$ or
> $\mathcal{O}$, and inversion as $-a$, then we say it is "written additively".
>
> It's conventional to use additive notation for elliptic curve groups, and multiplicative
> notation when the elements come from a finite field.
>
> When additive notation is used, we also write
>
> $$[k] A = \underbrace{A + A + \cdots + A}_{k \text{ times}}$$
>
> for nonnegative $k$ and call this "scalar multiplication"; we also often use uppercase
> letters for variables denoting group elements. When multiplicative notation is used, we
> also write
>
> $$a^k = \underbrace{a \times a \times \cdots \times a}_{k \text{ times}}$$
>
> and call this "exponentiation". In either case we call the scalar $k$ such that
> $[k] g = a$ or $g^k = a$ the "discrete logarithm" of $a$ to base $g$. We can extend
> scalars to negative integers by inversion, i.e. $[-k] A + [k] A = \mathcal{O}$ or
> $a^{-k} \times a^k = 1$.

The _order_ of an element $a$ of a finite group is defined as the smallest positive integer
$k$ such that $a^k = 1$ (in multiplicative notation) or $[k] a = \mathcal{O}$ (in additive
notation). The order _of the group_ is the number of elements.

Groups always have a [generating set], which is a set of elements such that we can produce
any element of the group as (in multiplicative terminology) a product of powers of those
elements. So if the generating set is $g_{1..k}$, we can produce any element of the group
as $\prod\limits_{i=1}^{k} g_i^{a_i}$. There can be many different generating sets for a
given group.

[generating set]: https://en.wikipedia.org/wiki/Generating_set_of_a_group

A group is called [cyclic] if it has a (not necessarily unique) generating set with only
a single element — call it $g$. In that case we can say that $g$ generates the group, and
that the order of $g$ is the order of the group.

Any finite cyclic group $\mathbb{G}$ of order $n$ is [isomorphic] to the integers
modulo $n$ (denoted $\mathbb{Z}/n\mathbb{Z}$), such that:

- the operation $\cdot$ in $\mathbb{G}$ corresponds to addition modulo $n$;
- the identity in $\mathbb{G}$ corresponds to $0$;
- some generator $g \in \mathbb{G}$ corresponds to $1$.

Given a generator $g$, the isomorphism is always easy to compute in the
$\mathbb{Z}/n\mathbb{Z} \rightarrow \mathbb{G}$ direction; it is just $a \mapsto g^a$
(or in additive notation, $a \mapsto [a] g$).
It may be difficult in general to compute in the $\mathbb{G} \rightarrow \mathbb{Z}/n\mathbb{Z}$
direction; we'll discuss this further when we come to [elliptic curves](curves.md).

If the order $n$ of a finite group is prime, then the group is cyclic, and every
non-identity element is a generator.

[isomorphic]: https://en.wikipedia.org/wiki/Isomorphism
[cyclic]: https://en.wikipedia.org/wiki/Cyclic_group

### The multiplicative group of a finite field

We use the notation $\mathbb{F}_p^\times$ for the multiplicative group (i.e. the group
operation is multiplication in $\mathbb{F}_p$) over the set $\mathbb{F}_p - \{0\}$.

A quick way of obtaining the inverse in $\mathbb{F}_p^\times$ is $a^{-1} = a^{p - 2}$.
The reason for this stems from [Fermat's little theorem][fermat-little], which states
that $a^p = a \pmod p$ for any integer $a$. If $a$ is nonzero, we can divide by $a$ twice
to get $a^{p-2} = a^{-1}.$

[fermat-little]: https://en.wikipedia.org/wiki/Fermat%27s_little_theorem

Let's assume that $\alpha$ is a generator of $\mathbb{F}_p^\times$, so it has order $p-1$
(equal to the number of elements in $\mathbb{F}_p^\times$). Therefore, for any element in
$a \in \mathbb{F}_p^\times$ there is a unique integer $i \in \{0..p-2\}$ such that $a = \alpha^i$.

Notice that $a \times b$ where $a, b \in \mathbb{F}_p^\times$ can really be interpreted as
$\alpha^i \times \alpha^j$ where $a = \alpha^i$ and $b = \alpha^j$. Indeed, it holds that
$\alpha^i \times \alpha^j = \alpha^{i + j}$ for all $0 \leq i, j \lt p - 1$. As a result
the multiplication of nonzero field elements can be interpreted as addition modulo $p - 1$
with respect to some fixed generator $\alpha$. The addition just happens "in the exponent."

This is another way to look at where $a^{p - 2}$ comes from for computing inverses in the
field:

$$p - 2 \equiv -1 \pmod{p - 1},$$

so $a^{p - 2} = a^{-1}$.

### Montgomery's Trick

Montgomery's trick, named after Peter Montgomery (RIP) is a way to compute many group
inversions at the same time. It is commonly used to compute inversions in
$\mathbb{F}_p^\times$, which are quite computationally expensive compared to multiplication.

Imagine we need to compute the inverses of three nonzero elements $a, b, c \in \mathbb{F}_p^\times$.
Instead, we'll compute the products $x = ab$ and $y = xc = abc$, and compute the inversion

$$z = y^{p - 2} = \frac{1}{abc}.$$

We can now multiply $z$ by $x$ to obtain $\frac{1}{c}$ and multiply $z$ by $c$ to obtain
$\frac{1}{ab}$, which we can then multiply by $a, b$ to obtain their respective inverses.

This technique generalizes to arbitrary numbers of group elements with just a single
inversion necessary.

## Multiplicative subgroups

A _subgroup_ of a group $G$ with operation $\cdot$, is a subset of elements of $G$ that
also form a group under $\cdot$.

In the previous section we said that $\alpha$ is a generator of the $(p - 1)$-order
multiplicative group $\mathbb{F}_p^\times$. This group has _composite_ order, and so by
the Chinese remainder theorem[^chinese-remainder] it has strict subgroups. As an example
let's imagine that $p = 11$, and so $p - 1$ factors into $5 \cdot 2$. Thus, there is a
generator $\beta$ of the $5$-order subgroup and a generator $\gamma$ of the $2$-order
subgroup. All elements in $\mathbb{F}_p^\times$, therefore, can be written uniquely as
$\beta^i \cdot \gamma^j$ for some $i$ (modulo $5$) and some $j$ (modulo $2$).

If we have $a = \beta^i \cdot \gamma^j$ notice what happens when we compute

$$
a^5 = (\beta^i \cdot \gamma^j)^5
    = \beta^{i \cdot 5} \cdot \gamma^{j \cdot 5}
    = \beta^0 \cdot \gamma^{j \cdot 5}
    = \gamma^{j \cdot 5};
$$

we have effectively "killed" the $5$-order subgroup component, producing a value in the
$2$-order subgroup.

[Lagrange's theorem (group theory)][lagrange-group] states that the order of any subgroup
$H$ of a finite group $G$ divides the order of $G$. Therefore, the order of any subgroup
of $\mathbb{F}_p^\times$ must divide $p-1.$

[lagrange-group]: https://en.wikipedia.org/wiki/Lagrange%27s_theorem_(group_theory)

[PLONK-based] proving systems like Halo 2 are more convenient to use with fields that have
a large number of multiplicative subgroups with a "smooth" distribution (which makes the
performance cliffs smaller and more granular as circuit sizes increase). The Pallas and
Vesta curves specifically have primes of the form

$$T \cdot 2^S = p - 1$$

with $S = 32$ and $T$ odd (i.e. $p - 1$ has 32 lower zero-bits). This means they have
multiplicative subgroups of order $2^k$ for all $k \leq 32$. These 2-adic subgroups are
nice for [efficient FFTs], as well as enabling a wide variety of circuit sizes.

[PLONK-based]: plonkish.md
[efficient FFTs]: polynomials.md#fast-fourier-transform-fft

## Square roots

In a field $\mathbb{F}_p$ exactly half of all nonzero elements are squares; the remainder
are non-squares or "quadratic non-residues". In order to see why, consider an $\alpha$
that generates the $2$-order multiplicative subgroup of $\mathbb{F}_p^\times$ (this exists
because $p - 1$ is divisible by $2$ since $p$ is a prime greater than $2$) and $\beta$ that
generates the $t$-order multiplicative subgroup of $\mathbb{F}_p^\times$ where $p - 1 = 2t$.
Then every element $a \in \mathbb{F}_p^\times$ can be written uniquely as
$\alpha^i \cdot \beta^j$ with $i \in \mathbb{Z}_2$ and $j \in \mathbb{Z}_t$. Half of all
elements will have $i = 0$ and the other half will have $i = 1$.

Let's consider the simple case where $p \equiv 3 \pmod{4}$ and so $t$ is odd (if $t$ is
even, then $p - 1$ would be divisible by $4$, which contradicts $p$ being $3 \pmod{4}$).
If $a \in \mathbb{F}_p^\times$ is a square, then there must exist
$b = \alpha^i \cdot \beta^j$ such that $b^2 = a$. But this means that

$$a = (\alpha^i \cdot \beta^j)^2 = \alpha^{2i} \cdot \beta^{2j} = \beta^{2j}.$$

In other words, all squares in this particular field do not generate the $2$-order
multiplicative subgroup, and so since half of the elements generate the $2$-order subgroup
then at most half of the elements are square. In fact exactly half of the elements are
square (since squaring each nonsquare element gives a unique square). This means we can
assume all squares can be written as $\beta^m$ for some $m$, and therefore finding the
square root is a matter of exponentiating by $2^{-1} \pmod{t}$.

In the event that $p \equiv 1 \pmod{4}$ then things get more complicated because
$2^{-1} \pmod{t}$ does not exist. Let's write $p - 1$ as $2^k \cdot t$ with $t$ odd. The
case $k = 0$ is impossible, and the case $k = 1$ is what we already described, so consider
$k \geq 2$. $\alpha$ generates a $2^k$-order multiplicative subgroup and $\beta$ generates
the odd $t$-order multiplicative subgroup. Then every element $a \in \mathbb{F}_p^\times$
can be written as $\alpha^i \cdot \beta^j$ for $i \in \mathbb{Z}_{2^k}$ and
$j \in \mathbb{Z}_t$. If the element is a square, then there exists some $b = \sqrt{a}$
which can be written $b = \alpha^{i'} \cdot \beta^{j'}$ for $i' \in \mathbb{Z}_{2^k}$ and
$j' \in \mathbb{Z}_t$. This means that $a = b^2 = \alpha^{2i'} \cdot \beta^{2j'}$,
therefore we have $i \equiv 2i' \pmod{2^k}$, and $j \equiv 2j' \pmod{t}$. $i$ would have
to be even in this case because otherwise it would be impossible to have
$i \equiv 2i' \pmod{2^k}$ for any $i'$. In the case that $a$ is not a square, then $i$ is
odd, and so half of all elements are squares.

In order to compute the square root, we can first raise the element
$a = \alpha^i \cdot  \beta^j$ to the power $t$ to "kill" the $t$-order component, giving

$$a^t = \alpha^{it \pmod 2^k} \cdot \beta^{jt \pmod t} = \alpha^{it \pmod 2^k}$$

and then raise this result to the power $t^{-1} \pmod{2^k}$ to undo the effect of the
original exponentiation on the $2^k$-order component:

$$(\alpha^{it \bmod 2^k})^{t^{-1} \pmod{2^k}} = \alpha^i$$

(since $t$ is relatively prime to $2^k$). This leaves bare the $\alpha^i$ value which we
can trivially handle. We can similarly kill the $2^k$-order component to obtain
$\beta^{j \cdot 2^{-1} \pmod{t}}$, and put the values together to obtain the square root.

It turns out that in the cases $k = 2, 3$ there are simpler algorithms that merge several
of these exponentiations together for efficiency. For other values of $k$, the only known
way is to manually extract $i$ by squaring until you obtain the identity for every single
bit of $i$. This is the essence of the [Tonelli-Shanks square root algorithm][ts-sqrt] and
describes the general strategy. (There is another square root algorithm that uses
quadratic extension fields, but it doesn't pay off in efficiency until the prime becomes
quite large.)

[ts-sqrt]: https://en.wikipedia.org/wiki/Tonelli%E2%80%93Shanks_algorithm

## Roots of unity

In the previous sections we wrote $p - 1 = 2^k \cdot t$ with $t$ odd, and stated that an
element $\alpha \in \mathbb{F}_p^\times$ generated the $2^k$-order subgroup. For
convenience, let's denote $n := 2^k.$ The elements $\{1, \alpha, \ldots, \alpha^{n-1}\}$
are known as the $n$th [roots of unity](https://en.wikipedia.org/wiki/Root_of_unity).

The **primitive root of unity**, $\omega,$ is an $n$th root of unity such that
$\omega^i \neq 1$ except when $i \equiv 0 \pmod{n}$.

Important notes:

- If $\alpha$ is an $n$th root of unity, $\alpha$ satisfies $\alpha^n - 1 = 0.$ If
  $\alpha \neq 1,$ then
  $$1 + \alpha + \alpha^2 + \cdots + \alpha^{n-1} = 0.$$
- Equivalently, the roots of unity are solutions to the equation
  $$X^n - 1 = (X - 1)(X - \alpha)(X - \alpha^2) \cdots (X - \alpha^{n-1}).$$
- **$\boxed{\omega^{\frac{n}{2}+i} =  -\omega^i}$ ("Negation lemma")**. Proof:
  $$
  \begin{aligned}
  \omega^n = 1 &\implies \omega^n - 1 = 0 \\
  &\implies (\omega^{n/2} + 1)(\omega^{n/2} - 1) = 0.
  \end{aligned}
  $$
  Since the order of $\omega$ is $n$, $\omega^{n/2} \neq 1.$ Therefore, $\omega^{n/2} = -1.$

- **$\boxed{(\omega^{\frac{n}{2}+i})^2 =  (\omega^i)^2}$ ("Halving lemma")**. Proof:
  $$
  (\omega^{\frac{n}{2}+i})^2 = \omega^{n + 2i} = \omega^{n} \cdot \omega^{2i} = \omega^{2i} = (\omega^i)^2.
  $$
  In other words, if we square each element in the $n$th roots of unity, we would get back
  only half the elements, $\{(\omega_n^i)^2\} = \{\omega_{n/2}\}$ (i.e. the $\frac{n}{2}$th roots
  of unity). There is a two-to-one mapping between the elements and their squares.

## References
[^chinese-remainder]: [Friedman, R. (n.d.) "Cyclic Groups and Elementary Number Theory II" (p. 5).](http://www.math.columbia.edu/~rf/numbertheory2.pdf)
