# Sinsemilla

## Overview
Sinsemilla is a collision-resistant hash function and commitment scheme designed to be efficient in algebraic circuit models that support [lookups](https://zcash.github.io/halo2/design/proving-system/lookup.html), such as PLONK or Halo 2.

The security properties of Sinsemilla are similar to Pedersen hashes; it is **not** designed to be used where a random oracle, PRF, or preimage-resistant hash is required. **The only claimed security property of the hash function is collision-resistance for fixed-length inputs.**

Sinsemilla is roughly 4 times less efficient than the algebraic hashes Rescue and Poseidon inside a circuit, but around 19 times more efficient than Rescue outside a circuit. Unlike either of these hashes, the collision resistance property of Sinsemilla can be proven based on cryptographic assumptions that have been well-established for at least 20 years. Sinsemilla can also be used as a computationally binding and perfectly hiding commitment scheme.

The general approach is to split the message into $k$-bit pieces, and for each piece, select from a table of $2^k$ bases in our cryptographic group. We combine the selected bases using a double-and-add algorithm. This ends up being provably as secure as a vector Pedersen hash, and makes advantageous use of the lookup facility supported by Halo 2.

## Description

This section is an outline of how Sinsemilla works: for the normative specification, refer to [§5.4.1.9 Sinsemilla Hash Function](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash) in the protocol spec. The incomplete point addition operator, ⸭, that we use below is also defined there.

Let $\mathbb{G}$ be a cryptographic group of prime order $q$. We write $\mathbb{G}$ additively, with identity $\mathcal{O}$, and using $[m] P$ for scalar multiplication of $P$ by $m$.

Let $k \geq 1$ be an integer chosen based on efficiency considerations (the table size will be $2^k$). Let $n$ be an integer, fixed for each instantiation, such that messages are $kn$ bits, where $2^n \leq \frac{q-1}{2}$. We use zero-padding to the next multiple of $k$ bits if necessary.

$\textsf{Setup}$: Choose $Q$ and $P[0..2^k - 1]$ as $2^k + 1$ independent, verifiably random generators of $\mathbb{G}$, using a suitable hash into $\mathbb{G}$, such that none of $Q$ or $P[0..2^k - 1]$ are $\mathcal{O}$.

> In Orchard, we define $Q$ to be dependent on a domain separator $D$. The protocol specification uses $\mathcal{Q}(D)$ in place of $Q$ and $\mathcal{S}(m)$ in place of $P[m]$.

$\textsf{Hash}(M)$:
- Split $M$ into $n$ groups of $k$ bits. Interpret each group as a $k$-bit little-endian integer $m_i$.
- let $\mathsf{Acc}_0 := Q$
- for $i$ from $0$ up to $n-1$:
  - let $\mathsf{Acc}_{i+1} := (\mathsf{Acc}_i \;⸭\; P[m_{i+1}]) \;⸭\; \mathsf{Acc}_i$
- return $\mathsf{Acc}_n$

Let $\textsf{ShortHash}(M)$ be the $x$-coordinate of $\textsf{Hash}(M)$. (This assumes that $\mathbb{G}$ is a prime-order elliptic curve in short Weierstrass form, as is the case for Pallas and Vesta.)

> It is slightly more efficient to express a double-and-add $[2] A + R$ as $(A + R) + A$. We also use incomplete additions: it is shown in the [Sinsemilla security argument](https://zips.z.cash/protocol/protocol.pdf#sinsemillasecurity) that in the case where $\mathbb{G}$ is a prime-order short Weierstrass elliptic curve, an exceptional case for addition would lead to finding a discrete logarithm, which can be assumed to occur with negligible probability even for adversarial input.

### Use as a commitment scheme
Choose another generator $H$ independently of $Q$ and $P[0..2^k - 1]$.

The randomness $r$ for a commitment is chosen uniformly on $[0, q)$.

Let $\textsf{Commit}_r(M) = \textsf{Hash}(M) \;⸭\; [r] H$.

Let $\textsf{ShortCommit}_r(M)$ be the $x\text{-coordinate}$ of $\textsf{Commit}_r(M)$. (This again assumes that $\mathbb{G}$ is a prime-order elliptic curve in short Weierstrass form.)

Note that unlike a simple Pedersen commitment, this commitment scheme ($\textsf{Commit}$ or $\textsf{ShortCommit}$) is not additively homomorphic.

## Efficient implementation
The aim of the design is to optimize the number of bits that can be processed for each step of the algorithm (which requires a doubling and addition in $\mathbb{G}$) for a given table size. Using a single table of size $2^k$ group elements, we can process $k$ bits at a time.

## Constraint program
Let $\mathcal{P} = \left\{(j,\, x_{P[j]},\, y_{P[j]}) \text{ for } j \in \{0..2^k - 1\}\right\}$.

Input: $m_{1..=n}$. (The message words are 1-indexed here, as in the [protocol spec](https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash), but we start the loop from $i = 0$ so that $(x_{A,i}, y_{A,i})$ corresponds to $\mathsf{Acc}_i$ in the protocol spec.)

Output: $(x_{A,n},\, y_{A,n})$.

- $(x_{A,0},\, y_{A,0}) = Q$
- for $i$ from $0$ up to $n-1$:
  - $y_{P,i} = y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})$
  - $x_{R,i} = \lambda_{1,i}^2 - x_{A,i} - x_{P,i}$
  - $2 \cdot y_{A,i} = (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i})$
  - $(m_{i+1},\, x_{P,i},\, y_{P,i}) \in \mathcal{P}$
  - $\lambda_{2,i}^2 = x_{A,i+1} + x_{R,i} + x_{A,i}$
  - $\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) = y_{A,i} + y_{A,i+1}$


## PLONK / Halo 2 constraints

### Message decomposition
We have an $n$-bit message $m = m_1 + 2^k m_2 + ... + 2^{k\cdot (n-1)} m_n$. (Note that the message words are 1-indexed as in the [protocol spec](https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash).)

Initialise the running sum $z_0 = \alpha$ and define $z_{i + 1} := \frac{z_{i} - m_{i+1}}{2^K}$. We will end up with $z_n = 0.$

Rearranging gives us an expression for each word of the original message $m_{i+1} = z_{i} - 2^k \cdot z_{i + 1}$, which we can look up in the table.

In other words, $z_{n-i} = \sum\limits_{h=0}^{i-1} 2^{kh} \cdot m_{h+1}$.

> For a little-endian decomposition as used here, the running sum is initialized to the scalar and ends at 0. For a big-endian decomposition as used in [variable-base scalar multiplication](https://hackmd.io/o9EzZBwxSWSi08kQ_fMIOw), the running sum would start at 0 and end with recovering the original scalar.
>
> The running sum only applies to message words within a single field element, i.e. if $n \geq \mathtt{PrimeField::NUM\_BITS}$ then we will have several disjoint running sums. A longer message can be constructed by splitting the message words across several field elements, and then running several instances of the constraints below. An additional $q_{S2}$ selector is set to $0$ for the last step of each element, except for the last element where it is set to $2$.
>
> In order to support chaining multiple field elements without a gap, we will use a slightly more complicated expression for $m_{i+1}$ that effectively forces $\mathbf{z}_n$ to zero for the last step of each element, as indicated by $q_{S2}$. This allows the cell that would have been $\mathbf{z}_n$ to be used to reinitialize the running sum for the next element.

### Generator lookup table
The Sinsemilla circuit makes use of $2^{10}$ pre-computed random generators. These are loaded into a lookup table:
$$
\begin{array}{|c|c|c|}
\hline
 table_{idx} & table_x         & table_y         \\\hline
 0           & x_{P[0]}        & y_{P[0]}        \\\hline
 1           & x_{P[1]}        & y_{P[1]}        \\\hline
 2           & x_{P[2]}        & y_{P[2]}        \\\hline
 \vdots      & \vdots          & \vdots          \\\hline
 2^{10} - 1  & x_{P[2^{10}-1]} & y_{P[2^{10}-1]} \\\hline
\end{array}
$$

### Layout
Note: $q_{S3}$ is synthesized from $q_{S1}$ and $q_{S2}$; it is shown here only for clarity.
$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\text{Step} &    x_A     &    x_P      &   bits   &    \lambda_1     &   \lambda_2      & q_{S1} & q_{S2} & q_{S3} &    q_{S4}  & \textsf{fixed\_y\_Q}\\\hline
    0       & x_Q        & x_{P[m_1]}  & z_0      & \lambda_{1,0}    & \lambda_{2,0}    & 1      & 1      & 0      &     1      &    y_Q              \\\hline
    1       & x_{A,1}    & x_{P[m_2]}  & z_1      & \lambda_{1,1}    & \lambda_{2,1}    & 1      & 1      & 0      &     0      &     0               \\\hline
    2       & x_{A,2}    & x_{P[m_3]}  & z_2      & \lambda_{1,2}    & \lambda_{2,2}    & 1      & 1      & 0      &     0      &     0               \\\hline
  \vdots    & \vdots     & \vdots      & \vdots   & \vdots           & \vdots           & 1      & 1      & 0      &     0      &     0               \\\hline
   n-1      & x_{A,n-1}  & x_{P[m_n]}  & z_{n-1}  & \lambda_{1,n-1}  & \lambda_{2,n-1}  & 1      & 0      & 0      &     0      &     0               \\\hline
    0'      & x'_{A,0}   & x_{P[m'_1]} & z'_0     & \lambda'_{1,0}   & \lambda'_{2,0}   & 1      & 1      & 0      &     0      &     0               \\\hline
    1'      & x'_{A,1}   & x_{P[m'_2]} & z'_1     & \lambda'_{1,1}   & \lambda'_{2,1}   & 1      & 1      & 0      &     0      &     0               \\\hline
    2'      & x'_{A,2}   & x_{P[m'_3]} & z'_2     & \lambda'_{1,2}   & \lambda'_{2,2}   & 1      & 1      & 0      &     0      &     0               \\\hline
  \vdots    & \vdots     & \vdots      & \vdots   & \vdots           & \vdots           & 1      & 1      & 0      &     0      &     0               \\\hline
   n-1'     & x'_{A,n-1} & x_{P[m'_n]} & z'_{n-1} & \lambda'_{1,n-1} & \lambda'_{2,n-1} & 1      & 2      & 2      &     0      &     0               \\\hline
    n'      &  x'_{A,n}  &             &          &       y_{A,n}    &                  & 0      & 0      & 0      &     0      &     0               \\\hline
\end{array}
$$

$x_Q$, $z_0$, $z'_0$, etc. would be copied in using equality constraints.

### Optimized Sinsemilla gate
$$
\begin{array}{lrcl}
\text{For } i \in [0, n), \text{ let} &x_{R,i} &=& \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
                                      &Y_{A,i} &=& (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) \\
                                      &y_{P,i} &=& Y_{A,i}/2 - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
                                      &m_{i+1} &=& z_{i} - 2^k \cdot (q_{S2,i} - q_{S3,i}) \cdot z_{i+1} \\
                                      &q_{S3}  &=& q_{S2} \cdot (q_{S2} - 1)
\end{array}
$$

The Halo 2 circuit API can automatically substitute $y_{P,i}$, $x_{R,i}$, $y_{A,i}$, and $y_{A,i+1}$, so we don't need to do that manually.

- $x_{A,0} = x_Q$
- $2 \cdot y_Q = Y_{A,0}$
- for $i$ from $0$ up to $n-1$:
  - $(m_{i+1},\, x_{P,i},\, y_{P,i}) \in \mathcal{P}$
  - $\lambda_{2,i}^2 = x_{A,i+1} + x_{R,i} + x_{A,i}$
  - $4 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) = 2 \cdot Y_{A,i} + (2 - q_{S3}) \cdot Y_{A,i+1} + 2 q_{S3} \cdot y_{A,n}$

Note that each term of the last constraint is multiplied by $4$ relative to the constraint program given earlier. This is a small optimization that avoids divisions by $2$.

By gating the lookup expression on $q_{S1}$, we avoid the need to fill in unused cells with dummy values to pass the lookup argument. The optimized lookup value (using a default index of $0$) is:

$$
\begin{array}{ll}
(&q_{S1} \cdot m_{i+1}, \\
 &q_{S1} \cdot x_{P,i} + (1 - q_{S1}) \cdot x_{P,0}, \\
 &q_{S1} \cdot y_{P,i} + (1 - q_{S1}) \cdot y_{P,0} \;\;\;)
\end{array}
$$

This increases the degree of the lookup argument to $6$.

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
4   & q_{S4} \cdot (2 \cdot y_Q - Y_{A,0}) = 0 \\\hline
6   & q_{S1,i} \Rightarrow (m_{i+1},\, x_{P,i},\, y_{P,i}) \in \mathcal{P} \\\hline
3   & q_{S1,i} \cdot \big(\lambda_{2,i}^2 - (x_{A,i+1} + x_{R,i} + x_{A,i})\big) \\\hline
5   & q_{S1,i} \cdot \left(4 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - (2 \cdot Y_{A,i} + (2 - q_{S3,i}) \cdot Y_{A,i+1} + 2 \cdot q_{S3,i} \cdot y_{A,n})\right) = 0 \\\hline
\end{array}
$$
