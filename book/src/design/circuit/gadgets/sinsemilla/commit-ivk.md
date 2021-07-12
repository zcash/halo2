# CommitIvk

## Message decomposition
$\mathsf{SinsemillaShortCommit}$ is used in the [$\mathsf{CommitIvk}$ function](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillacommit). The input to $\mathsf{SinsemillaShortCommit}$ is:

$$\mathsf{I2LEBSP}_{\ell_{\textsf{base}}^{\textsf{Orchard}}}(ak) || \mathsf{I2LEBSP}_{\ell_{\textsf{base}}^{\textsf{Orchard}}}(nk),$$

where $\mathsf{ak, nk}$ are Pallas base field elements, and $\ell_{\textsf{base}}^{\textsf{Orchard}} = 255.$

We break these inputs into the following `MessagePiece`s:

$$
\begin{aligned}
a \text{ (250 bits)} &= \text{bits } 0..=249 \text{ of } \mathsf{ak} \\
b \text{ (10 bits)}  &= b_0||b_1||b_2 \\
                     &= (\text{bits } 250..=253 \text{ of } \mathsf{ak}) || (\text{bit } 254 \text{ of } \mathsf{ak}) || (\text{bits } 0..=4 \text{ of } \mathsf{nk}) \\
c \text{ (240 bits)} &= \text{bits } 5..=244 \text{ of } \mathsf{nk} \\
d \text{ (10 bits)}  &= d_0||d_1 \\
                     &= (\text{bits } 245..=253 \text{ of } \mathsf{nk}) || (\text{bit } 254 \text{ of } \mathsf{nk})
\end{aligned}
$$

$a,b,c,d$ are constrained by the $\textsf{SinsemillaHash}$ to be $250$ bits, $10$ bits, $240$ bits, and $10$ bits respectively.

In a custom gate, we check this message decomposition by enforcing the following constraints:

1. $b = b_0 + 2^4 \cdot b_1 + 2^5 \cdot b_2$
<br>
$b_0, b_2$ are witnessed outside this gate, and constrained to be $4$ bits and $5$ bits respectively. $b_1$ is witnessed and boolean-constrained in this gate:
$$(b_1)(1 - b_1) = 0.$$
From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$b = b_0 + 2^4 \cdot b_1 + 2^5 \cdot b_2.$$

2. $d = d_0 + 2^9 \cdot d_1$
<br>
$d_0$ is witnessed outside this gate, and constrained to be $9$ bits. $d_1$ is witnessed and boolean-constrained in this gate:
$$(d_1)(1 - d_1) = 0.$$
From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$d = d_0 + 2^9 \cdot d_1.$$

We have now derived or witnessed every subpiece, and range-constrained every subpiece:
- $b_0$ ($4$ bits) is witnessed and constrained outside the gate;
- $b_1$ ($1$ bits) is witnessed and boolean-constrained in the gate;
- $b_2$ ($5$ bits) is witnessed and constrained outside the gate;
- $d_0$ ($9$ bits) is witnessed and constrained outside the gate;
- $d_1$ ($1$ bits) is witnessed and boolean-constrained in the gate,
and we use them to reconstruct the original field element inputs:

3. $\textsf{ak} = a + 2^{250} \cdot b_0 + 2^{254} \cdot b_1$
4. $\textsf{nk} = b_2 + 2^5 \cdot c + 2^{245} \cdot d_0 + 2^{254} \cdot d_1$

## Canonicity
The modulus of the Pallas base field is $p = 2^{254} + t_p,$ where $t_p = 45560315531419706090280762371685220353 < 2^{126}.$

### $\textsf{ak} = a (250 \text{ bits}) || b_0 (4 \text{ bits}) || b_1 (1 \text{ bit})$
We check that $\mathsf{I2LEBSP_{\ell_{base}^{Orchard}}(ak)}$ is a canonically-encoded $255$-bit value, i.e. $\textsf{ak} < p$. If the high bit is not set $b_1 = 0$, we are guaranteed that $\textsf{ak} < 2^{254}$. Thus, we are only interested in cases when $b_1 = 1 \implies \textsf{ak} \geq 2^{254}$. In these cases, we check that $\textsf{ak}_{0..=253} < t_p < 2^{126}$:

1. $b_1 = 1 \implies b_0 = 0.$
Since $b_1 = 1 \implies \textsf{ak}_{0..=253} < 2^{126},$ we know that $\textsf{ak}_{126..=253} = 0,$ and in particular $b_0 = \textsf{ak}_{250..=253} = 0.$ So, we constrain $$b_1 \cdot b_0 = 0.$$

2. $b_1 = 1 \implies 0 \leq a < 2^{126}.$
To check that $a < 2^{126}$, we use two constraints:

    a) $0 \leq a < 2^{130}$. This is expressed in the custom gate as $$b_1 \cdot z_{13,a} = 0,$$ where $z_{13,a}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(a).$

    b) $0 \leq a + 2^{130} - t_p < 2^{130}$. To check this, we decompose $a' = a + 2^{130} - t_p$ into thirteen 10-bit words (little-endian) using a running sum $z_{a'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$b_1 \cdot z_{13, a'} = 0.$$

### $\textsf{nk} = b_2 (5 \text{ bits}) || c (240 \text{ bits}) || d_0 (9 \text{ bits}) || d_1 (1 \text{ bit})$
We check that $\mathsf{I2LEBSP}_{\ell_{\textsf{base}}^{\textsf{Orchard}}}(nk)$ is a canonically-encoded $255$-bit value, i.e. $\textsf{nk} < p$. If the high bit is not set $d_1 = 0$, we are guaranteed that $nk < 2^{254}$. Thus, we are only interested in cases when $d_1 = 1 \implies nk \geq 2^{254}$. In these cases, we check that $\textsf{nk}_{0..=253} < t_p < 2^{126}$:

1. $d_1 = 1 \implies 0 \leq b_2 + 2^5 \cdot c < 2^{126}.$
To check that $0 \leq b_2 + 2^5 \cdot c < 2^{126}$, we use two constraints:

    a) $0 \leq b_2 + 2^5 \cdot c < 2^{140}$. $b_2$ is already constrained individually to be a $5$-bit value. $z_{13, c}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(c).$ By constraining $$d_1 \cdot z_{13,c} = 0,$$ we constrain $b_2 + 2^5 \cdot c < 2^{135} < 2^{140}.$

    b) $0 \leq b_2 + 2^5 \cdot c + 2^{140} - t_p < 2^{140}$. To check this, we decompose $b' = b_2 + 2^5 \cdot c + 2^{140} - t_p$ into fourteen 10-bit words (little-endian) using a running sum $z_{b'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$d_1 \cdot z_{14, b'} = 0.$$