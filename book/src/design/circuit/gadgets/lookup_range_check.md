# Lookup decomposition
This gadget makes use of a $K$-bit lookup table to decompose a field element $\alpha$ into $K$-bit words. Firstly, express $$\alpha = k_0 + 2^{K} \cdot k_1 + 2^{2K} \cdot k_2 + \cdots,$$ where each $k_i$ a $K$-bit value.

We initialize the running sum $z_0 = \mathbf{s},$ and compute subsequent terms $z_i = \frac{z_{i - 1} - k_{i-1}}{2^{K}}.$ This gives us:

$$
\begin{aligned}
z_0 &= \alpha \\
    &= k_0 + 2^{K} \cdot k_1 + 2^{2K} \cdot k_2 +  2^{3K} \cdot k_3 + \cdots, \\
z_1 &= (z_0 - k_0) / 2^K \\
    &= k_1 + 2^{K} \cdot k_2 +  2^{2K} \cdot k_3 + \cdots, \\
z_2 &= (z_1 - k_1) / 2^K \\
    &= k_2 +  2^{K} \cdot k_3 + \cdots, \\
    &\vdots
\end{aligned}
$$

Each $K$-bit word $k_i = z_i - 2^K \cdot z_{i+1}$ is range-constrained by a lookup in the $K$-bit table. This gadget takes in $\alpha, n$ and returns $(z_0, z_1, \cdots, z_n).$

## Strict mode
Strict mode constrains the running sum output $z_{n}$ to be zero, thus range-constraining the field element to be within $n \cdot K$ bits.

## Short range check
Using two $K$-bit lookups, we can range-constrain a field element $\alpha$ to be $n$ bits, where $n \leq K.$ To do this:

1. Constrain $0 \leq \alpha < 2^K$ to be within $K$ bits using a $K$-bit lookup.
2. Constrain $0 \leq \alpha \cdot 2^{K - n} < 2^K$ to be within $K$ bits using a $K$-bit lookup.