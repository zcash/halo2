# Chip for Sinsemilla

The aim of the design is to optimize the number of bits that can be processed for each step of the algorithm (which requires a doubling and addition in $\mathbb{G}$) for a given table size. Using a single table of size $2^k$ group elements, we can process $k$ bits at a time. See [Generic Lookups with PLONK](https://hackmd.io/LTPc5f-3S0qNF6MtwD-Tdg?view) for one way to implement the necessary lookups in a PLONK-like circuit model.

Note that it is slightly more efficient to express a double-and-add $[2] A + R$ as $(A + R) + A$.

## Constraint program
(Refer to: https://github.com/zcash/zcash/issues/3924)

Let $\mathcal{P} = \left\{(j,\, x_{P[j]},\, y_{P[j]}) \text{ for } j \in \{0..2^k - 1\}\right\}$.

Input: $z_n \in \{0..2^{kn} - 1\}$ such that $z_0 = 0$ and $z_{l+1} = \sum\limits_{i=0}^{l} m_i \cdot 2^{k(l - i)}$.

Initialize $z_0 = 0, (x_{A,0},\, y_{A,0}) = Q.$

for $i$ from $0$ up to $n-1$:
$$
\begin{array}{l}
y_{P,i} = y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
x_{R,i} = \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
2 \cdot y_{A,i} = (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) \\
    (z_{i+1} - 2^k \cdot z_i,\, x_{P,i},\, y_{P,i}) \in \mathcal{P} \\
\lambda_{2,i}^2 = x_{A,i+1} + x_{R,i} + x_{A,i} \\
\lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) = y_{A,i} + y_{A,i+1} \\
\end{array}
$$

Output $(x_{A,n},\, y_{A,n})$

After substitution of $y_{P,i}$, $x_{R,i}$, $y_{A,i}$, and $y_{A,i+1}$, this becomes:

Initialize $z_0 = 0, (x_{A,0},\, y_{A,0}) = Q.$

$2 \cdot y_{A,0} = (\lambda_{1,0} + \lambda_{2,0}) \cdot (x_{A,0} - (\lambda_{1,0}^2 - x_{A,0} - x_{P,0}))$

for $i$ from $0$ up to $n-1$:
$$
\begin{array}{l}
    \texttt{// let } y_{P,i} = y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
    \texttt{// let } x_{R,i} = \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
    \texttt{// let } y_{A,i} = \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} \\
    (z_{i+1} - 2^k \cdot z_i,\, x_{P,i},\, \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})) \in \mathcal{P} \\
    \lambda_{2,i}^2 = x_{A,i+1} + (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) + x_{A,i} \\
    \texttt{if } i < n-1: 2 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) = \\
        \hspace{2em}(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))\, + \\
        \hspace{2em}(\lambda_{1,i+1} + \lambda_{2,i+1}) \cdot (x_{A,i+1} - (\lambda_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1}))
\end{array}
$$

$\lambda_{2,n-1} \cdot (x_{A,n-1} - x_{A,n}) = (\lambda_{1,n-1} + \lambda_{2,n-1}) \cdot (x_{A,n-1} - (\lambda_{1,n-1}^2 - x_{A,n-1} - x_{P,n-1})) + y_{A,n}$

## PLONK / Halo 2 constraints

This uses one [lookup argument](https://hackmd.io/iOw7-HpFQY6dPF1aFY8pAw).

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|}
\hline
\text{Step} &    x_A    &     z     &    \lambda_1    &   \lambda_2     &    x_P         & \text{table\_idx} & \text{table\_x} & \text{table\_y} & q_{Sinsemilla} \\\hline
    0       & x_Q       &   0       & \lambda_{1,0}   & \lambda_{2,0}   & x_{P[m_0]}     &     0             & x_{P[0]}        & y_{P[0]}        & 1              \\\hline
    1       & x_{A,1}   &   z_1     & \lambda_{1,1}   & \lambda_{2,1}   & x_{P[m_1]}     &     1             & x_{P[1]}        & y_{P[1]}        & 1              \\\hline
    2       & x_{A,2}   &   z_2     & \lambda_{1,2}   & \lambda_{2,2}   & x_{P[m_2]}     &     2             & x_{P[2]}        & y_{P[2]}        & 1              \\\hline
  \vdots    & \vdots    &   \vdots  & \vdots          & \vdots          & \vdots         &   \vdots          & \vdots          & \vdots          & 1              \\\hline
   n-1      & x_{A,n-1} &   z_{n-1} & \lambda_{1,n-1} & \lambda_{2,n-1} & x_{P[m_{n-1}]} &   \vdots          & \vdots          & \vdots          & 2              \\\hline
    n       & x_{A,n}   &   z_n     &                 &                 &                &   \vdots          & \vdots          & \vdots          &                \\\hline
  \vdots    &           &           &                 &                 &                &  2^k - 1          & x_{P[2^k - 1]}  & y_{P[2^k - 1]}  &                \\\hline

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
?   & q_{Sinsemilla} \Rightarrow (z_{i+1} - 2^k \cdot z_i,\, x_{P,i},\, \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))\hspace{2em}}{2} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})) \in \mathcal{P} \\\hline
3   & q_{Sinsemilla} \cdot (\lambda_{2,i}^2 - (x_{A,i+1} + (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}) + x_{A,i})) \\\hline
5   & q_{Sinsemilla} \cdot (2 - q_{ECC}) \cdot \left(2 \cdot \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - \big(
        (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i})) +
        (\lambda_{1,i+1} + \lambda_{2,i+1}) \cdot (x_{A,i+1} - (\lambda_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1}))\big)\right) \\\hline
\end{array}
$$

A further optimization is to toggle the lookup expression on $q_{Sinsemilla}.$ This removes the need to fill in unused cells with dummy values to pass the lookup argument. The optimized lookup argument would be:

$$
\begin{array}{}
&(\\&
&& q_S \cdot (z_{i + 1} - 2^k \cdot z_i) + (1 - q_S) \cdot 0, \\
&&& q_S \cdot x_{P, i} + (1 - q_S) \cdot x_{P, 0}, \\
&&& q_S \cdot y_p + (1 - q_S) \cdot y_{P, 0} \\
&),&
\end{array}
$$

where $y_P \equiv \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i})).$

Plus:
- $q_Q \cdot \left(E_0 - x_Q\right) = 0$
- $q_Q \cdot \left(E_1 - y_Q\right) = 0$