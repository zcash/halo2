# Cryptographic groups

In the section [Inverses and groups](fields.md#inverses-and-groups) we introduced the
concept of *groups*. A group has an identity and a group operation. In this section we
will write groups additively, i.e. the identity is $\mathcal{O}$ and the group operation
is $+$.

Some groups can be used as *cryptographic groups*. At the risk of oversimplifying, this
means that the problem of finding a discrete logarithm of a group element $P$ to a given
base $G$, i.e. finding $x$ such that $P = [x] G$, is hard in general.

## Pedersen commitment
The Pedersen commitment [[P99]] is a way to commit to a secret message in a verifiable
way. It uses two random public generators $G, H \in \mathbb{G},$ where $\mathbb{G}$ is a
cryptographic group of order $p$. A random secret $r$ is chosen in $\mathbb{Z}_q$, and the
message to commit to $m$ is from any subset of $\mathbb{Z}_q$. The commitment is 

$$c = \text{Commit}(m,r)=[m]G + [r]H.$$ 

To open the commitment, the committer reveals $m$ and $r,$ thus allowing anyone to verify
that $c$ is indeed a commitment to $m.$

[P99]: https://link.springer.com/content/pdf/10.1007%2F3-540-46766-1_9.pdf#page=3

Notice that the Pedersen commitment scheme is homomorphic:

$$
\begin{aligned}
\text{Commit}(m,r) + \text{Commit}(m',r') &= [m]G + [r]H + [m']G + [r']H \\
&= [m + m']G + [r + r']H \\
&= \text{Commit}(m + m',r + r').
\end{aligned}
$$

Assuming the discrete log assumption holds, Pedersen commitments are also perfectly hiding
and computationally binding:

* **hiding**: the adversary chooses messages $m_0, m_1.$ The committer commits to one of
  these messages $c = \text{Commit}(m_b;r), b \in \{0,1\}.$ Given $c,$ the probability of
  the adversary guessing the correct $b$ is no more than $\frac{1}{2}$.
* **binding**: the adversary cannot pick two different messages $m_0 \neq m_1,$ and
  randomness $r_0, r_1,$ such that $\text{Commit}(m_0,r_0) = \text{Commit}(m_1,r_1).$

### Vector Pedersen commitment
We can use a variant of the Pedersen commitment scheme to commit to multiple messages at
once, $\mathbf{m} = (m_1, \cdots, m_n)$. This time, we'll have to sample a corresponding
number of random public generators $\mathbf{G} = (G_0, \cdots, G_{n-1}),$ along with a
single random generator $H$ as before (for use in hiding). Then, our commitment scheme is:

$$
\begin{aligned}
\text{Commit}(\mathbf{m}; r) &= \text{Commit}((m_0, \cdots, m_{n-1}); r) \\
&= [r]H + [m_0]G_0 + \cdots + [m_{n-1}]G_{n-1} \\
&= [r]H + \sum_{i= 0}^{n-1} [m_i]G_i.
\end{aligned}
$$

> TODO: is this positionally binding?

## Diffie--Hellman

An example of a protocol that uses cryptographic groups is Diffie--Hellman key agreement
[[DH1976]]. The Diffie--Hellman protocol is a method for two users, Alice and Bob, to
generate a shared private key. It proceeds as follows:

1. Alice and Bob publicly agree on two prime numbers, $p$ and $G,$ where $p$ is large and
   $G$ is a primitive root $\pmod p.$ (Note that $g$ is a generator of the group
   $\mathbb{F}_p^\times.$)
2. Alice chooses a large random number $a$ as her private key. She computes her public key
   $A = [a]G \pmod p,$ and sends $A$ to Bob.
3. Similarly, Bob chooses a large random number $b$ as his private key. He computes his
   public key $B = [b]G \pmod p,$ and sends $B$ to Alice.
4. Now both Alice and Bob compute their shared key $K = [ab]G \pmod p,$ which Alice
   computes as
   $$K = [a]B \pmod p = [a]([b]G) \pmod p,$$
   and Bob computes as
   $$K = [b]A \pmod p = [b]([a]G) \pmod p.$$

[DH1976]: https://ee.stanford.edu/~hellman/publications/24.pdf

A potential eavesdropper would need to derive $K = [ab]g \pmod p$ knowing only
$g, p, A = [a]G,$ and $B = [b]G$: in other words, they would need to either get the
discrete logarithm $a$ from $A = [a]G$ or $b$ from $B = [b]G,$ which we assume to be
computationally infeasible in $\mathbb{F}_p^\times.$

More generally, protocols that use similar ideas to Diffie--Hellman are used throughout
cryptography. One way of instantiating a cryptographic group is as an
[elliptic curve](curves.md). Before we go into detail on elliptic curves, we'll describe
some algorithms that can be used for any group.

## Multiscalar multiplication

### TODO: Pippenger's algorithm
Reference: https://jbootle.github.io/Misc/pippenger.pdf
