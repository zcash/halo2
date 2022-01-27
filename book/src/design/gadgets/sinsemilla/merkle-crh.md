# MerkleCRH

## Message decomposition
$\mathsf{SinsemillaHash}$ is used in the [$\mathsf{MerkleCRH^{Orchard}}$ hash function](https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh). The input to $\mathsf{SinsemillaHash}$ is:

$${l\star} \,||\, {\textsf{left}\star} \,||\, {\textsf{right}\star},$$

where:
- ${l\star} = \textsf{I2LEBSP}_{10}(l) = \textsf{I2LEBSP}_{10}(\textsf{MerkleDepth}^\textsf{Orchard} - 1 - \textsf{layer})$,
- ${\textsf{left}\star} = \textsf{I2LEBSP}_{\ell_{\textsf{Merkle}}^{\textsf{Orchard}}}(\textsf{left})$,
- ${\textsf{right}\star} = \textsf{I2LEBSP}_{\ell_{\textsf{Merkle}}^{\textsf{Orchard}}}(\textsf{right})$,

with $\ell_{\textsf{Merkle}}^{\textsf{Orchard}} = 255.$ $\textsf{left}$ and $\textsf{right}$ are allowed to be non-canonical $255$-bit encodings.

We break these inputs into the following `MessagePiece`s:

$$
\begin{aligned}
a \text{ (250 bits)} &= a_0 \,||\, a_1 \\
                     &= {l\star} \,||\, (\text{bits } 0..=239 \text{ of } \textsf{ left }) \\
b \text{ (20 bits)}  &= b_0 \,||\, b_1 \,||\, b_2 \\
                     &= (\text{bits } 240..=249 \text{ of } \textsf{left}) \,||\, (\text{bits } 250..=254 \text{ of } \textsf{left}) \,||\, (\text{bits } 0..=4 \text{ of } \textsf{right}) \\
c \text{ (250 bits)} &= \text{bits } 5..=254 \text{ of } \textsf{right}
\end{aligned}
$$

$a,b,c$ are constrained by the $\textsf{SinsemillaHash}$ to be $250$ bits, $20$ bits, and $250$ bits respectively.

In a custom gate, we check this message decomposition by enforcing the following constraints:

1. $a_0 = l$
<br>
$z_{1,a}$, the index-1 running sum output of $\textsf{SinsemillaHash}(a)$, is copied into the gate. $z_{1,a}$ has been constrained by the $\textsf{SinsemillaHash}$ to be $240$ bits. We recover the subpieces $a_0, a_1$ using $a, z_{1,a}$:
$$
\begin{aligned}
z_{1,a} &= \frac{a - a_0}{2^{10}}\\
        &= a_1 \\
        \implies a_0 &= a - z_{1,a} \cdot 2^{10}.
\end{aligned}
$$
$l + 1$ is loaded into a fixed column at each layer of the hash. It is used both as a gate selector, and to fix the value of $l$. We check that $$a_0 = (l + 1) - 1.$$
> Note: The reason for using $l + 1$ instead of $l$ is that $l = 0$ when $\textsf{layer} = 31$ (hashing two leaves). We cannot have a zero-valued selector, since a constraint gated by a zero-valued selector is never checked.

2. $b_1 + 2^5 \cdot b_2 = z_{1,b}$
<br>
$z_{1,b}$, the index-1 running sum output of $\textsf{SinsemillaHash}(b)$, is copied into the gate. $z_{1,b}$ has been constrained by the $\textsf{SinsemillaHash}$ to be $10$ bits. We witness the subpieces $b_1, b_2$ outside this gate, and constrain them each to be $5$ bits. Inside the gate, we check that $$b_1 + 2^5 \cdot b_2 = z_{1,b}.$$
We also recover the subpiece $b_0$ using $(b, z_{1,b})$:
$$
\begin{aligned}
z_{1,b} &= \frac{b - b_{0..=10}}{2^{10}}\\
        \implies b_0 &= b - (z_{1,b} \cdot 2^{10}).
\end{aligned}
$$

We have now derived or witnessed every subpiece, and range-constrained every subpiece:
- $a_0$ ($10$ bits), derived as $a_0 = a - 2^{10} \cdot z_{1,a}$;
- $a_1$ ($240$ bits), equal to $z_{1,a}$;
- $b_0$ ($10$ bits), derived as $b_0 = b - 2^{10} \cdot z_{1,b}$;
- $b_1$ ($5$ bits) is witnessed and constrained outside the gate;
- $b_2$ ($5$ bits) is witnessed and constrained outside the gate;
- $b_1 + 2^5 \cdot b_2$ is constrained to equal $z_{1, b}$,
and we use them to reconstruct the original field element inputs:

3. $\mathsf{left} = a_1 + 2^{240} \cdot b_0 + 2^{254} \cdot b_1$

4. $\mathsf{right} = b_2 + 2^5 \cdot c$

## Circuit components
The Orchard circuit spans $10$ advice columns while the $\textsf{Sinsemilla}$ chip only uses $5$ advice columns. We distribute the path hashing evenly across two $\textsf{Sinsemilla}$ chips to make better use of the available circuit area. Since the output from the previous layer hash is copied into the next layer hash, we maintain continuity even when moving from one chip to the other.
