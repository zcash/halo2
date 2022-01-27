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

The region layout for the lookup decomposition uses a single advice column $z$, and two selectors $q_{lookup}$ and $q_{running}.$
$$
\begin{array}{|c|c|c|}
\hline
    z    & q_\mathit{lookup} & q_\mathit{running} \\\hline
\hline
  z_0    &     1      &       1     \\\hline
  z_1    &     1      &       1     \\\hline
\vdots   &   \vdots   &     \vdots  \\\hline
z_{n-1}  &     1      &       1     \\\hline
z_n      &     0      &       0     \\\hline
\end{array}
$$
### Short range check
Using two $K$-bit lookups, we can range-constrain a field element $\alpha$ to be $n$ bits, where $n \leq K.$ To do this:

1. Constrain $0 \leq \alpha < 2^K$ to be within $K$ bits using a $K$-bit lookup.
2. Constrain $0 \leq \alpha \cdot 2^{K - n} < 2^K$ to be within $K$ bits using a $K$-bit lookup.

The short variant of the lookup decomposition introduces a $q_{bitshift}$ selector. The same advice column $z$ has here been renamed to $\textsf{word}$ for clarity:
$$
\begin{array}{|c|c|c|c|}
\hline
\textsf{word} & q_\mathit{lookup} & q_\mathit{running} & q_\mathit{bitshift} \\\hline
\hline
\alpha        &     1      &      0      &       0      \\\hline
\alpha'       &     1      &      0      &       1      \\\hline
2^{K-n}       &     0      &      0      &       0      \\\hline
\end{array}
$$

where $\alpha' = \alpha \cdot 2^{K - n}.$ Note that $2^{K-n}$ is assigned to a fixed column at keygen, and copied in at proving time. This is used in the gate enabled by the $q_\mathit{bitshift}$ selector to check that $\alpha$ was shifted correctly:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
       2      & q_\mathit{bitshift} \cdot (\alpha' - (\alpha \cdot 2^{K - n})) \\\hline
\end{array}
$$

### Combined lookup expression
Since the lookup decomposition and its short variant both make use of the same lookup table, we combine their lookup input expressions into a single one:

$$q_\mathit{lookup} \cdot \left(q_\mathit{running} \cdot (z_i - 2^K \cdot z_{i+1}) + (1 - q_\mathit{running}) \cdot \textsf{word} \right)$$

where $z_i$ and $\textsf{word}$ are the same cell (but distinguished here for clarity of usage).

## Short range decomposition
For a short range (for instance, $[0, \texttt{range})$ where $\texttt{range} \leq 8$), we can range-constrain each word using a degree-$\texttt{range}$ polynomial constraint instead of a lookup: $$\RangeCheck{word}{range} = \texttt{word} \cdot (1 - \texttt{word}) \cdots (\texttt{range} - 1 - \texttt{word}).$$
