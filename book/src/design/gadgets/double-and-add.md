# Double-and-add

The double-and-add algorithm is used both in [Sinsemilla](./sinsemilla.md) and
the [incomplete addition part of variable-base scalar multiplication](./ecc/var-base-scalar-mul.md#incomplete-addition). This helper extracts the common Hlogic in the steady-state part of double-and-add, while leaving the initialization and finalization to the caller.

The double-and-add algorithm combines the points $P_i, i \in [0, n)$ using an
accumulator $Acc$ initialized to some $InitAcc$:

$$
\begin{array}{l}
Acc := InitAcc \\
\text{for $i$ from $n-1$ down to $0$:} \\
\hspace{2em} Acc := (Acc \;⸭\; P_i) \;⸭\; Acc \\
\text{return $Acc$}
\end{array}
$$

Recalling the [incomplete addition formulae](ecc/addition.md#incomplete-addition):

$$
\begin{aligned}
x_3 &= \left(\frac{y_1 - y_2}{x_1 - x_2}\right)^2 - x_1 - x_2 \\
y_3 &= \frac{y_1 - y_2}{x_1 - x_2} \cdot (x_1 - x_3) - y_1 \\
\end{aligned}
$$

Let $\lambda = \frac{y_1 - y_2}{x_1 - x_2}$.

Renaming the variables $(x_1, y_1) \rightarrow (x_{A_i}, y_{A_i})$,
$(x_2, y_2) \rightarrow (x_{P_i}, y_{P_i})$, and $(x_3, y_3) \rightarrow (x_{R_i}, y_{R_i})$:

$$
\begin{aligned}
\lambda_{1,i} &= \frac{y_{A,i} - y_{P,i}}{x_{A,i} - x_{P,i}} \\
&\implies y_{A,i} - y_{P,i} = \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
&\implies y_{P,i} = y_{A,i} - \lambda_{1,i} \cdot (x_{A,i} - x_{P,i}) \\
x_{R,i} &= \lambda_{1,i}^2 - x_{A,i} - x_{P,i} \\
y_{R,i} &= \lambda_{1,i} \cdot (x_{A,i} - x_{R,i}) - y_{A,i}. \\
\end{aligned}
$$

This gives us $x_R, y_R$ which we can use in the second incomplete addition $R_i \;⸭\; A_i$.
Now, we rename the variables $(x_1, y_1) \rightarrow (x_{R,i}, y_{R,i})$,
$(x_2, y_2) \rightarrow (x_{A,i}, y_{A,i})$, and $(x_3, y_3) \rightarrow (x_{A, i+1}, y_{A, i+1})$:

$$
\begin{aligned}
\lambda_{2,i} &= \frac{y_{A,i} - y_{R,i}}{x_{A,i} - x_{R,i}} \\
&\implies y_{A,i} - y_{R,i} = \lambda_{2,i} \cdot (x_{A,i} - x_{R,i}) \\
&\implies y_{A,i} - \left( \lambda_{1,i} \cdot (x_{A,i} - x_{R,i}) - y_{A,i} \right) = \lambda_{2,i} \cdot (x_{A,i} - x_{R,i}) \\
&\implies 2 \cdot y_{A,i} = (\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - x_{R,i}) \\
x_{A,i+1} &= \lambda_{2,i}^2 - x_{A,i} - x_{R,i} \hspace{2em}\texttt{secant check} \\
y_{A,i+1} &= \lambda_{2,i} \cdot (x_{A,i} - x_{A,i+1}) - y_{A,i} \hspace{2em}\texttt{gradient check} \\
\end{aligned}
$$

## Layout and constraints:
We can compute double-and-add using a width-four circuit layout, where each row
computes $A_{i+1} := (A_i \;⸭\; P_i) \;⸭\; A_i \equiv R_i \;⸭\; A_i$.

$$
\begin{array}{|c|c|c|c|}
    x_P    &    x_A     &    \lambda_1    &    \lambda_2     \\\hline
  x_{P,0}  & x_{A,init} &  \lambda_{1,0}  &  \lambda_{2,0}   \\\hline
    ...    &     ...    &        ...      &      ...         \\\hline
  x_{P,i}  &   x_{A,i}  &  \lambda_{1,i}  &  \lambda_{2,i}   \\\hline
 x_{P,i+1} &  x_{A,i+1} &  \lambda_{1,i+1}&  \lambda_{2,i+1} \\\hline
    ...    &     ...    &        ...      &      ...         \\\hline
 x_{P,n-1} &  x_{A,n-1} & \lambda_{1,n-1} & \lambda_{2,n-1}  \\\hline
\end{array}
$$

Our selectors are not specified by this helper; instead, the caller provides:
- $\texttt{q\_secant}$, which activates $\texttt{secant check}$ on every line of the double-and-add; and
- $\texttt{q\_gradient}$, which activates $\texttt{gradient check}$ on all but the last line of the double-and-add.

The constraints on each row are then

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
  2 + deg(q)  & \texttt{q\_secant} \cdot \left(\lambda_{2,i}^2 - x_{A,i-1} - x_{R,i} - x_{A,i}\right) = 0 \\\hline
  2 + deg(q)  & \texttt{q\_gradient} \cdot \left(\lambda_{2,i} \cdot (x_{A,i} - x_{A,i-1}) - y_{A,i} - y_{A,i-1}\right) = 0 \\\hline
\end{array}
$$

where

$$
\begin{aligned}
x_{R,i} &= \lambda_{1,i}^2 - x_{A,i} - x_{P,i}, \\
y_{A,i} &= \frac{(\lambda_{1,i} + \lambda_{2,i}) \cdot (x_{A,i} - (\lambda_{1,i}^2 - x_{A,i} - x_{P,i}))}{2}, \\
y_{A,i-1} &= \frac{(\lambda_{1,i-1} + \lambda_{2,i-1}) \cdot (x_{A,i-1} - (\lambda_{1,i-1}^2 - x_{A,i-1} - x_{P,i-1}))}{2}. \\
\end{aligned}
$$

### Initialization
The caller is responsible for constraining the initial accumulator to the
desired $(x_{A,init}, y_{A,init})$. This is not done by the double-and-add
helper.

As an example, the caller could witness $(x_{A,init}, y_{A,init})$, and
copy in $x_{A,init}$ to the starting position. To constrain $y_{A,init}$,
they could do:

$$
\begin{array}{|c|c|c|c|c|}
    x_P    &    x_A     &    \lambda_1    &    \lambda_2     & q\_init  \\\hline
           &            &    y_{A,init}   &                  &     1    \\\hline
  x_{P,0}  & x_{A,init} &  \lambda_{1,0}  &  \lambda_{2,0}   &          \\\hline
\end{array}
$$

with a copy-constraint on $x_{A,init}$, and the constraint on $y_{A,init}$:
$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
       3      & q\_init \cdot \left(y_{A,0} - y_{A,init}\right) = 0 \\\hline
\end{array}
$$
where
$$
y_{A,0} = \frac{(\lambda_{1,0} + \lambda_{2,0}) \cdot (x_{A,0} - (\lambda_{1,0}^2 - x_{A,0} - x_{P,0}))}{2}.
$$

### Final output
To export the final $x_A, y_A$, the caller may choose to witness $y_{A,witnessed}$
and constrain it to be consistent with $x_{A,n-1}, x_{P,n-1}, \lambda_{1,n-1}$,
and $\lambda_{2,n-1}$. This is not done by the double-and-add helper.

As an example, the caller could do:

$$
\begin{array}{|c|c|c|c|c|}
    x_P    &    x_A     &    \lambda_1    &    \lambda_2     & q\_final \\\hline
 x_{P,n-1} &  x_{A,n-1} & \lambda_{1,n-1} & \lambda_{2,n-1}  &     1    \\\hline
           &            & y_{A,witnessed} &                  &          \\\hline
\end{array}
$$

with the constraint:

$$
\begin{array}{|c|l|}
\hline
\text{Degree} & \text{Constraint} \\\hline
       3      & q\_final \cdot \left(y_{A,n-1} - y_{A,witnessed}\right) = 0 \\\hline
\end{array}
$$
where
$$
y_{A,n-1} = \frac{(\lambda_{1,n-1} + \lambda_{2,n-1}) \cdot (x_{A,n-1} - (\lambda_{1,n-1}^2 - x_{A,n-1} - x_{P,n-1}))}{2}.
$$
