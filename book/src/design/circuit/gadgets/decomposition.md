# Decomposition
Given a field element $\alpha$, these gadgets decompose it into $W$ $K$-bit windows $$\alpha = k_0 + 2^{K} \cdot k_1 + 2^{2K} \cdot k_2 + \cdots + 2^{(W-1)K} \cdot k_{W-1}$$ where each $k_i$ a $K$-bit value.

This is done using a running sum $z_i, i \in [0..W).$ We initialize the running sum $z_0 = \alpha,$ and compute subsequent terms $z_{i+1} = \frac{z_i - k_i}{2^{K}}.$ This gives us:

$$
\begin{aligned}
z_0 &= \alpha \\
    &= k_0 + 2^{K} \cdot k_1 + 2^{2K} \cdot k_2 +  2^{3K} \cdot k_3 + \cdots, \\
z_1 &= (z_0 - k_0) / 2^K \\
    &= k_1 + 2^{K} \cdot k_2 +  2^{2K} \cdot k_3 + \cdots, \\
z_2 &= (z_1 - k_1) / 2^K \\
    &= k_2 +  2^{K} \cdot k_3 + \cdots, \\
    &\vdots \\
\downarrow &\text{ (in strict mode)} \\
z_W &= (z_{W-1} - k_{W-1}) / 2^K \\
    &= 0 \text{ (because } z_{W-1} = k_{W-1} \text{)}
\end{aligned}
$$

### Strict mode
Strict mode constrains the running sum output $z_{W}$ to be zero, thus range-constraining the field element to be within $W \cdot K$ bits.

In strict mode, we are also assured that $z_{W-1} = k_{W-1}$ gives us the last window in the decomposition.

## Lookup decomposition
This gadget makes use of a $K$-bit lookup table to decompose a field element $\alpha$ into $K$-bit words. Each $K$-bit word $k_i = z_i - 2^K \cdot z_{i+1}$ is range-constrained by a lookup in the $K$-bit table.

### Short range check
Using two $K$-bit lookups, we can range-constrain a field element $\alpha$ to be $n$ bits, where $n \leq K.$ To do this:

1. Constrain $0 \leq \alpha < 2^K$ to be within $K$ bits using a $K$-bit lookup.
2. Constrain $0 \leq \alpha \cdot 2^{K - n} < 2^K$ to be within $K$ bits using a $K$-bit lookup.

## Short range decomposition
For a short range (for instance, $2^K \leq 8$), we can range-constrain each word using a polynomial constraint instead of a lookup:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
2^K + 1 & q_\text{decompose-base-field} \cdot \texttt{range\_check}(z_i - z_{i+1} \cdot 2^K, 2^K) = 0 \\\hline
2 & q_\text{strict} \cdot z_W = 0 \\\hline
\end{array}
$$
where $\texttt{range\_check}(k_i, \texttt{range}) = k_i \cdot (1 - k_i) \cdots (\texttt{range} - 1 - k_i).$
