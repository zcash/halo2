# NoteCommit

## Message decomposition
$\mathsf{SinsemillaCommit}$ is used in the [$\mathsf{NoteCommit}$ function](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillacommit). The input to $\mathsf{SinsemillaCommit}$ is:

$$\mathsf{g\star_d} || \mathsf{pk\star_d} || \mathsf{I2LEBSP}_{64}(v) || \mathsf{I2LEBSP}_{\ell_{\textsf{base}}^{\textsf{Orchard}}}(\rho) || \mathsf{I2LEBSP}_{\ell_{\textsf{base}}^{\textsf{Orchard}}}(\psi),$$

where $\mathsf{g\star_d, pk\star_d}$ are representations of Pallas curve points, with $255$ bits used for the $x$-coordinate and $1$ bit used for the $y$-coordinate; $\rho, \psi$ are Pallas base field elements, $v$ is a $64$-bit value, and $\ell_{\textsf{base}}^{\textsf{Orchard}} = 255.$

We break these inputs into the following `MessagePiece`s:

$$
\begin{aligned}
a \text{ (250 bits)} &= \text{bits } 0..=249 \text{ of } \mathsf{x(g_d)} \\
b \text{ (10 bits)}  &= b_0 || b_1 || b_2 || b_3 \\
                     &= (\text{bits } 250..=253 \textsf{ of } \mathsf{x(g_d)}) || (bit 254 \textsf{ of } \mathsf{x(g_d)}) || (ỹ \text{ bit of } \mathsf{g_d}) || (\text{bits } 0..=3 \textsf{ of } \mathsf{pk\star_d}) \\
c \text{ (250 bits)} &= \text{bits } 4..=253 \textsf{ of } \mathsf{pk\star_d} \\
d \text{ (60 bits)}  &= d_0 || d_1 || d_2 || d_3 \\
                     &= (\text{bit } 254 \text{ of } \mathsf{x(pk_d)}) || (ỹ \text{ bit of } \mathsf{pk_d}) || (0..=7 \text{ of v}) || (8..=57 \text{ of v}) \\
e \text{ (10 bits)}  &= e_0 || e_1 \\
                     &= (\text{bits } 58..=63 \text{ of v}) || (\text{bits } 0..=3 \text{ of} \rho) \\
f \text{ (250 bits)} &= \text{bits } 4..=253 \text{ of } \rho \\
g \text{ (250 bits)} &= g_0 || g_1 || g_2 \\
                     &= (\text{bit } 254 \text{ of } \rho) || (\text{bits } 0..=8 \text{ of } \psi) || (\text{bits } 9..=248 \text{ of } \psi) \\
h \text{ (10 bits)}  &= h_0 || h_1 || h_2 \\
                     &= (\text{bits } 249..=253 \text{ of } \psi) || (\text{bit } 254 \text{ of } \psi) || 4 \text{ zero bits } \\
\end{aligned}
$$

$a,b,c,d$ are constrained by the $\textsf{SinsemillaHash}$ to be:
- $a = 250$ bits,
- $b = 10$ bits,
- $c = 250$ bits,
- $d = 60$ bits,
- $e = 10$ bits,
- $f = 250$ bits,
- $g = 250$ bits,
- $h = 10$ bits.

In a custom gate, we check this message decomposition by enforcing the following constraints:

1. $b = b_0 + 2^4 \cdot b_1 + 2^5 \cdot b_2 + 2^6 \cdot b_3$
<br>
$b_0, b_3$ are witnessed outside this gate, and constrained to be $4$ bits each. $b_1, b_2$ are witnessed and boolean-constrained in this gate:
$$
\begin{aligned}
(b_1)(1 - b_1) &= 0 \\
(b_2)(1 - b_2) &= 0 \\
\end{aligned}
$$
From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$b = b_0 + 2^4 \cdot b_1 + 2^5 \cdot b_2 + 2^6 \cdot b_3$$

2. $d = d_0 + 2 \cdot d_1 + 2^2 \cdot d_2 + 2^{10} \cdot d_3$
<br>
$d_0, d_1$ are witnessed and boolean-constrained in this gate:
$$
\begin{aligned}
(d_0)(1 - d_0) &= 0 \\
(d_1)(1 - d_1) &= 0 \\
\end{aligned}
$$
$d_2$ is witnessed outside this gate, and constrained to be $8$ bits. $d_3$ is copied into this gate as $d_3 = z_{1,d}$, where $z_{1,d}$ is the index-1 running sum output of $\textsf{SinsemillaHash}(d),$ constrained by the hash to be $50$ bits. From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$d = d_0 + 2 \cdot d_1 + 2^2 \cdot d_2 + 2^{10} \cdot d_3$$

3. $e = e_0 + 2^6 \cdot e_1$
<br>
$e_0, e_1$ are witnessed outside this gate, and constrained to be $6$ bits and $4$ bits respectively.
From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$e_0 + 2^6 \cdot e_1$$

4. $g = g_0 + 2 \cdot g_1 + 2^{10} \cdot g_2$
<br>
$g_0$ is witnessed and boolean-constrained in this gate: $$(g_0)(g_0 - 1) = 0.$$ $g_1$ is witnessed outside this gate, and constrained to be $9$ bits. $g_2$ is copied into this gate as $g_2 = z_{1,g}$, where $z_{1,g}$ is the index-1 running sum output of $\textsf{SinsemillaHash}(g),$ constrained by the hash to be $240$ bits. From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$g = g_0 + 2 \cdot g_1 + 2^{10} \cdot g_2.$$

5. $h = h_0 + 2^5 \cdot h_1$
<br>
$h_0$ is witnessed outside this gate, and constrained to be $5$ bits. $h_1$ is witnessed and boolean-constrained in this gate: $$(h_1)(h_1 - 1) = 0.$$ From these witnessed subpieces, we check that we recover the original `MessagePiece` input to the hash:
$$h = h_0 + 2^5 \cdot h_1$$

We have now derived or witnessed every subpiece, and range-constrained every subpiece:
- $b_0$ ($4$ bits) is witnessed and constrained outside the gate;
- $b_1$ ($1$ bit) is witnessed and boolean-constrained in the gate;
- $b_2$ ($1$ bit) is witnessed and boolean-constrained in the gate;
- $b_3$ ($4$ bits) is witnessed and constrained outside the gate;
- $d_0$ ($1$ bit) is witnessed and boolean-constrained in the gate;
- $d_1$ ($1$ bit) is witnessed and boolean-constrained in the gate;
- $d_2$ ($8$ bits) is witnessed and constrained outside the gate;
- $d_3$ ($50$ bits), equal to $z_{1,d}$;
- $e_0$ ($6$ bits) is witnessed and constrained outside the gate;
- $e_1$ ($4$ bit) is witnessed and constrained outside the gate;
- $g_0$ ($1$ bit) is witnessed and boolean-constrained in the gate;
- $g_1$ ($9$ bits) is witnessed and constrained outside the gate;
- $g_2$ ($240$ bits), equal to $z_{1,g}$;
- $h_0$ ($5$ bits) is witnessed and constrained outside the gate;
- $h_1$ ($1$ bit) is witnessed and boolean-constrained in the gate;

and we use them to reconstruct the original field element inputs:

6. $\mathsf{x(g_d)} = a + 2^250 \cdot b_0 + 2^254 \cdot b_1$
7. $\mathsf{pk_d} = b_3 + 2^4 \cdot c + 2^254 \cdot d_0$
8. $\mathsf{v} = d_2 + 2^8 \cdot d_3 + 2^58 \cdot e_0$
9. $\rho = e_1 + 2^4 \cdot f + 2^254 \cdot g_0$
10. $\psi = g_1 + 2^9 \cdot g_2 + 2^249 \cdot h_0 + 2^254 \cdot h_1$

## Canonicity
The modulus of the Pallas base field is $p = 2^{254} + t_p,$ where $t_p = 45560315531419706090280762371685220353 < 2^{126}.$

### $\mathsf{x(g_d)} = a \text{ (250 bits) } || b_0 \text{ (4 bits) } || b_1 \text{ (1 bit) }$
We check that $\mathsf{x(g_d)}$ is a canonically-encoded $255$-bit value, i.e. $\mathsf{x(g_d)} < p$. If the high bit is not set $b_1 = 0$, we are guaranteed that $\mathsf{x(g_d)} < 2^{254}$. Thus, we are only interested in cases when $b_1 = 1 \implies \mathsf{x(g_d)} \geq 2^{254}$. In these cases, we check that $\mathsf{x(g_d)}_{0..=253} < t_p < 2^{126}$:

1. $b_1 = 1 \implies b_0 = 0.$
Since $b_1 = 1 \implies \mathsf{x(g_d)}_{0..=253} < 2^{126},$ we know that $\mathsf{x(g_d)}_{126..=253} = 0,$ and in particular $b_0 = \mathsf{x(g_d)}_{250..=253} = 0.$

2. $b_1 = 1 \implies 0 \leq a < 2^{126}.$
To check that $a < 2^{126}$, we use two constraints:

    a) $0 \leq a < 2^{130}$. This is expressed in the custom gate as $$b_1 \cdot z_{13,a} = 0,$$ where $z_{13,a}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(a).$

    b) $0 \leq a + 2^{130} - t_p < 2^{130}$. To check this, we decompose $a' = a + 2^{130} - t_p$ into thirteen 10-bit words (little-endian) using a running sum $z_{a'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$b_1 \cdot z_{13, a'} = 0.$$


### $\mathsf{x(pk_d)} = b_3 \text{ (4 bits) } || c \text{ (250 bits) } || d_0 \text{ (1 bit) }$
We check that $\mathsf{x(pk_d)}$ is a canonically-encoded $255$-bit value, i.e. $\mathsf{x(pk_d)} < p$. If the high bit is not set $d_0 = 0$, we are guaranteed that $\mathsf{x(pk_d)} < 2^{254}$. Thus, we are only interested in cases when $d_0 = 1 \implies \mathsf{x(pk_d)} \geq 2^{254}$. In these cases, we check that $\mathsf{x(pk_d)}_{0..=253} < t_p < 2^{126}$:

1. $d_0 = 0 \implies 0 \leq b_3 + 2^{4} \cdot c < 2^{126}.$
To check that $0 \leq b_3 + 2^{4} \cdot c < 2^{126},$ we use two constraints:

    a) $0 \leq b_3 + 2^{4} \cdot c < 2^{140}.$ $b_3$ is already constrained individually to be a $4$-bit value. $z_{13, c}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(c).$ By constraining $$d_0 \cdot z_{13,c} = 0,$$ we constrain $b_3 + 2^4 \cdot c < 2^{134} < 2^{140}.$

    b) $0 \leq b_3 + 2^{4} \cdot c + 2^{140} - t_p < 2^{140}$. To check this, we decompose $b' = b_3 + 2^{4} \cdot c + 2^{140} - t_p$ into fourteen 10-bit words (little-endian) using a running sum $z_{b'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$d_0 \cdot z_{14, b'} = 0.$$

### $\rho = e_1 \text{ (4 bits) } || f \text{ (250 bits) } || g_0 \text{ (1 bit) }$
We check that $\rho$ is a canonically-encoded $255$-bit value, i.e. $\rho < p$. If the high bit is not set $g_0 = 0$, we are guaranteed that $\rho < 2^{254}$. Thus, we are only interested in cases when $g_0 = 1 \implies \rho \geq 2^{254}$. In these cases, we check that $\rho_{0..=253} < t_p < 2^{126}$:

1. $g_0 = 0 \implies 0 \leq e_1 + 2^{4} \cdot f < 2^{126}.$
To check that $0 \leq e_1 + 2^{4} \cdot f < 2^{126},$ we use two constraints:

    a) $0 \leq e_1 + 2^{4} \cdot f < 2^{140}.$ $e_1$ is already constrained individually to be a $4$-bit value. $z_{13, f}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(c).$ By constraining $$g_0 \cdot z_{13, f} = 0,$$ we constrain $e_1 + 2^4 \cdot f < 2^{134} < 2^{140}.$

    b) $0 \leq e_1 + 2^{4} \cdot f + 2^{140} - t_p < 2^{140}$. To check this, we decompose $e' = e_1 + 2^{4} \cdot f + 2^{140} - t_p$ into fourteen 10-bit words (little-endian) using a running sum $z_{e'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$g_0 \cdot z_{14, e'} = 0.$$

### $\psi = g_1 \text{ (9 bits) } || g_2 \text{ (240 bits) } || h_0 \text{ (5 bits) } || h_1 \text{ (1 bit) }$
We check that $\psi$ is a canonically-encoded $255$-bit value, i.e. $\psi < p$. If the high bit is not set $h_1 = 0$, we are guaranteed that $\psi < 2^{254}$. Thus, we are only interested in cases when $h_1 = 1 \implies \psi \geq 2^{254}$. In these cases, we check that $\psi_{0..=253} < t_p < 2^{126}$:

1. $h_1 = 0 \implies h_0 = 0.$
Since $h_1 = 1 \implies \psi_{0..=253} < 2^{126},$ we know that $\psi_{126..=253} = 0,$ and in particular $h_0 = \psi_{249..=253} = 0.$ So, we constrain $$h_1 \cdot h_0 = 0.$$

2. $h_1 = 0 \implies 0 \leq g_1 + 2^{9} \cdot g_2 < 2^{126}.$
To check that $0 \leq g_1 + 2^{9} \cdot g_2 < 2^{126},$ we use two constraints:

    a) $0 \leq g_1 + 2^{9} \cdot g_2 < 2^{140}.$ $e_1$ is already constrained individually to be a $4$-bit value. $z_{13, f}$ is the index-13 running sum output by $\textsf{SinsemillaHash}(c).$ By constraining $$h_1 \cdot z_{13, f} = 0,$$ we constrain $e_1 + 2^4 \cdot f < 2^{134} < 2^{140}.$

    b) $0 \leq g_1 + 2^{9} \cdot g_2 + 2^{140} - t_p < 2^{140}$. To check this, we decompose $e' = g_1 + 2^{9} \cdot g_2 + 2^{140} - t_p$ into fourteen 10-bit words (little-endian) using a running sum $z_{e'}$, looking up each word in a $10$-bit lookup table. We then enforce in the custom gate that $$h_1 \cdot z_{14, e'} = 0.$$