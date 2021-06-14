# Sinsemilla

## Overview
Sinsemilla is a collision-resistant hash function and commitment scheme designed to be efficient in algebraic circuit models that support [lookups](https://zcash.github.io/halo2/design/proving-system/lookup.html), such as PLONK or Halo 2.

The security properties of Sinsemilla are similar to Pedersen hashes; it is **not** designed to be used where a random oracle, PRF, or preimage-resistant hash is required. **The only claimed security property of the hash function is collision-resistance for fixed-length inputs.**

Sinsemilla is roughly 4 times less efficient than the algebraic hashes Rescue and Poseidon inside a circuit, but around 19 times more efficient than Rescue outside a circuit. Unlike either of these hashes, the collision resistance property of Sinsemilla can be proven based on cryptographic assumptions that have been well-established for at least 20 years. Sinsemilla can also be used as a computationally binding and perfectly hiding commitment scheme.

The general approach is to split the message into $k$-bit pieces, and for each piece, select from a table of $2^k$ bases in our cryptographic group. We combine the selected bases using a double-and-add algorithm. This ends up being provably as secure as a vector Pedersen hash, and makes advantageous use of the lookup facility supported by Halo 2.

## Specification

This section is an outline of how Sinsemilla works: for the normative specification, refer to [§5.4.1.9 Sinsemilla Hash Function](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash) in the protocol spec.

Let $\mathbb{G}$ be a cryptographic group of prime order $q$. We write $\mathbb{G}$ additively, with identity $\mathcal{O}$, and using $[m] P$ for scalar multiplication of $P$ by $m$.

Let $k \geq 1$ be an integer chosen based on efficiency considerations (the table size will be $2^k$). Let $n$ be a **fixed** integer such that messages are $kn$ bits, where $2^n \leq \frac{q-1}{2}$. We use zero-padding to the next multiple of $k$ bits if necessary.

$\textsf{Setup}$: Choose $Q$ and $P[0..2^k - 1]$ as $2^k + 1$ independent, verifiably random generators of $\mathbb{G}$, using a suitable hash into $\mathbb{G}$, such that none of $Q$ or $P[0..2^k - 1]$ are $\mathcal{O}$.

$\textsf{Hash}(M)$:
- Split $M$ into $n$ groups of $k$ bits. Interpret each group as a $k$-bit little-endian integer $m_i$.
- $A_1 := Q$
- for $i$ from $1$ up to $n$:
  - $A_{i+1} := [2] A_i ⸭ P[m_i] = (A_i ⸭ P[m_i]) ⸭ A_i$
- return $A_{n+1}$

Let $\textsf{ShortHash}(M)$ be the $x$-coordinate of $\textsf{Hash}(M)$. (This assumes that $\mathbb{G}$ is a prime-order elliptic curve in short Weierstrass form, as is the case for Pallas and Vesta.)

### Use as a commitment scheme
Choose another generator $H$ independently of $Q$ and $P[0..2^k - 1]$.

The randomness $r$ for a commitment is chosen uniformly on $[0, q)$.

Let $\textsf{Commit}_r(M) = \textsf{Hash}(M) ⸭ [r] H$.

Let $\textsf{ShortCommit}_r(M)$ be the $x\text{-coordinate}$ of $\textsf{Commit}_r(M)$. (This again assumes that $\mathbb{G}$ is a prime-order elliptic curve in short Weierstrass form.)

Note that unlike a simple Pedersen commitment, this commitment scheme ($\textsf{Commit}$ or $\textsf{ShortCommit}$) is not additively homomorphic.

## Efficient implementation
The aim of the design is to optimize the number of bits that can be processed for each step of the algorithm (which requires a doubling and addition in $\mathbb{G}$) for a given table size. Using a single table of size $2^k$ group elements, we can process $k$ bits at a time.

Note that it is slightly more efficient to express a double-and-add $[2] A + R$ as $(A + R) + A$. It is shown in the [Sinsemilla security argument](https://zips.z.cash/protocol/protocol.pdf#sinsemillasecurity) that in the case where $\mathbb{G}$ is a prime-order short Weierstrass elliptic curve, provided a negligible probability of failure is acceptable, it suffices to use incomplete additions.

## Constraint program
Let $\mathcal{P} = \left\{(j,\, x_{P[j]},\, y_{P[j]}) \text{ for } j \in \{0..2^k - 1\}\right\}$.

Input: $m_i, i \in [1..n]$. (Note that the message words are 1-indexed as in the [protocol spec](https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash)).

Output: $(x_{A,n+1},\, y_{A,n+1})$.

> $(x_{A,1},\, y_{A,1}) = Q$
>
> for $i$ from $1$ up to $n$:
> $$
\begin{aligned}
    y_{P,i} &= y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})\\
    x_{R,i} &= \lambda_{1,i}^2 - x_{A,i} - x_{P,i}\\
    2 \cdot y_{A,i} &= (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i})\\
    (m_i,\, x_{P,i},\, y_{P,i}) &\in \mathcal{P}\\
    \lambda_{2,i}^2 &= x_{A,i+1} + x_{R,i} + x_{A,i}\\
    \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) &= y_{A,i} + y_{A,i+1}\\
\end{aligned}
$$

After substitution of $y_{P,i}$, $x_{R,i}$, $y_{A,i}$, and $y_{A,i+1}$, this becomes:

> $(x_{A,1},\, y_{A,1}) = Q$
>
> $2 \cdot y_{A,1} = (\lambda_{1,1} + \lambda_{2,1}) \cdot (x_{A,1} - (\lambda_{1,1}^2 - x_{A,1} - x_{P,1}))$
>
> for $i$ from $1$ up to $n$:
> $$
\begin{aligned}
    &\textsf{// let } y_{P,i} = y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
    &\textsf{// let } x_{R,i} = \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
    &\textsf{// let } y_{A,i} = \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} \\
    &(m_i,\, x_{P,i},\, \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})) \in \mathcal{P} \\
    &\lambda_{2,i}^2 = x_{A,i+1} + (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) + x_{A,i} \\
    &\textsf{if } i < n: \\
        &\hspace{2em} 2 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) =\\
        &\hspace{2em}(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))\, +\\
        &\hspace{2em}(\lambda_{1,i+1} + \lambda_{2,i+1}) \cdot (x_{A,i+1} - (\lambda_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1}))\\
\end{aligned}
$$
>
> $\lambda_{2,n} \cdot (x_{A,n} - x_{A,n+1}) = (\lambda_{1,n} + \lambda_{2,n}) \cdot (x_{A,n} - (\lambda_{1,n}^2 - x_{A,n} - x_{P,n})) + y_{A,n+1}$

## PLONK / Halo 2 constraints

### Message decomposition
We have an $n$-bit message $m = m_1 + 2^k m_2 + ... + 2^{k\cdot (n-1)} m_n$. (Note that the message words are 1-indexed as in the protocol spec: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash)

Initialise the running sum $z_0 = \alpha$ and define $z_{i + 1} := \frac{z_{i} - m_{i+1}}{2^K}$. We will end up with $z_n = 0.$

Rearranging gives us an expression for each word of the original message $m_{i+1} = z_{i} - 2^k \cdot z_{i + 1}$, which we can look up in the table.

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\text{Step} &    x_A    &     bits  &    \lambda_1    &   \lambda_2     &    x_P       & q_{Sinsemilla1}& q_{Sinsemilla2} & table_{idx}&    table_x     &    table_y      \\\hline
    1       & x_Q       &   z_0     & \lambda_{1,1}   & \lambda_{2,1}   & x_{P[m_1]}   & 1              & 1               &     0      & x_{P[0]}       & y_{P[0]}        \\\hline
    2       & x_{A,2}   &   z_1     & \lambda_{1,2}   & \lambda_{2,2}   & x_{P[m_2]}   & 1              & 1               &     1      & x_{P[1]}       & y_{P[1]}        \\\hline
    3       & x_{A,3}   &   z_2     & \lambda_{1,3}   & \lambda_{2,3}   & x_{P[m_3]}   & 1              & 1               &     2      & x_{P[2]}       & y_{P[2]}        \\\hline
  \vdots    & \vdots    &   \vdots  & \vdots          & \vdots          & \vdots       & 1              & 1               &   \vdots   & \vdots         & \vdots          \\\hline
    n       & x_{A,n}   &   z_{n-1} & \lambda_{1,n}   & \lambda_{2,n}   & x_{P[m_{n}]} & 1              & 0               &   \vdots   & \vdots         & \vdots          \\\hline
            & x_{A,n+1} &   z_n     &                 &                 &              &                &                 &   \vdots   & \vdots         & \vdots          \\\hline
  \vdots    &           &           &                 &                 &              &                &                 &  2^k - 1   & x_{P[2^k - 1]} & y_{P[2^k - 1]}  \\\hline
\end{array}
$$

### Specification of Sinsemilla gate:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
3   & q_{Sinsemilla,i} \cdot \left(\lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) - y_{A,i} + y_{P,i}\right) = 0 \\\hline
4   & q_{Sinsemilla,i} \cdot \left((\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i})) - 2 y_{A,i}\right) = 0 \\\hline
3   & q_{Sinsemilla,i} \cdot \left(\lambda_{2,i}^2 - x_{A,i+1} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) - x_{A,i}\right) = 0 \\\hline
3   & q_{Sinsemilla,i} \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - y_{A,i} - y_{A,i+1}\right) = 0 \\\hline
\end{array}
$$

Optimized:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
5*  & q_{Sinsemilla1} \Rightarrow (z_{i} - 2^k \cdot z_{i+1},\, x_{P,i},\, y_{P,i} \in \mathcal{P} \\\hline
3   & q_{Sinsemilla1,i} \cdot (\lambda_{2,i}^2 - (x_{A,i+1} + (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) + x_{A,i})) \\\hline
5   & q_{Sinsemilla2,i} \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - y_{A,i} - y_{A,i+1}\right) = 0 \\\hline
\end{array}
$$
where
$$
\begin{aligned}
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i})}{2},\\
y_{A,i+1} &= \frac{(\lambda_{1,i+1} + \lambda_{2,i+1}) \cdot (x_{A,i+1} - (\lambda_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1})}{2},\\
y_{P,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})).
\end{aligned}
$$

* The degree of a lookup gate is $1 + \textsf{input\_degree} + \textsf{table\_degree}$, where $\textsf{input\_degree}$ is the maximum degree of the polynomial expressions being looked up, and $\textsf{table\_degree}$ is the maximum degree of the table expressions in the lookup.

A further optimization is to toggle the lookup expression on $q_{Sinsemilla1}.$ This removes the need to fill in unused cells with dummy values to pass the lookup argument. The optimized lookup argument would be:

$$
\begin{array}{}
&(\\&
&& q_S \cdot (z_{i} - 2^k \cdot z_{i+1}) + (1 - q_S) \cdot 0, \\
&&& q_S \cdot x_{P, i} + (1 - q_S) \cdot x_{P, 0}, \\
&&& q_S \cdot y_{P, i} + (1 - q_S) \cdot y_{P, 0} \\
&),&
\end{array}
$$

This increases the degree of the lookup gate to $6$.
