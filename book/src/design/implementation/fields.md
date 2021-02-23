# Fields

## Sarkar square-root algorithm (table-based variant)
We use a technique from [Sarkar2020](https://eprint.iacr.org/2020/1407.pdf) to compute
square roots in `halo2`. The intuition behind the algorithm is that we can split the task
into computing square roots in each multiplicative subgroup.

Suppose we want to find the square root of $u$ modulo an odd prime $p$, where $u$ is a
non-zero square in $\mathbb{Z}_p^\times$. We write $p - 1 \equiv 2^{n}m$ with $n \geq 1$
and $m$ odd; $g = z^m$ where $z$ is a non-square in $\mathbb{Z}_p^\times$.

Let $x_3 = uv^2, x_2 = x_3^{2^8}, x_1 = x_2^{2^8}, x_0 = x_1^{2^8}.$

#### Precompute the following tables:
$$
gtab = \begin{bmatrix}
g^0 & g^1 & ... & g^{255} \\
(g^{2^8})^0 & (g^{2^8})^1 & ... & (g^{2^8})^{255} \\
(g^{2^{16}})^0 & (g^{2^{16}})^1 & ... & (g^{2^{16}})^{255} \\
(g^{2^{24}})^0 & (g^{2^{24}})^1 & ... & (g^{2^{24}})^{255}
\end{bmatrix}
$$

$$
invtab = \begin{bmatrix}
(g^{2^{-24}})^0 & (g^{2^{-24}})^1 & ... & (g^{2^{-24}})^{255}
\end{bmatrix}
$$

### i = 0, 1
Using $invtab$, we lookup $t_0$ s.t. $x_0 = (g^{2^{-24}})^{t_0} \implies x_0 \cdot g^{t_0 \cdot 2^{24}} = 1.$

Update global variable: $t = t_0.$

Define $\alpha_1 = x_1 \cdot (g^{2^{16}})^{t}.$

### i = 2
Lookup $t_1$ s.t. 
$$
\begin{array}{l}
\alpha_1 = (g^{2^{-24}})^{t_1} &\implies x_1 \cdot (g^{2^{16}})^{t_0} = (g^{2^{-24}})^{t_1} \\
&\implies
x_1 \cdot g^{(t_0 + 2^8 \cdot t_1) \cdot 2^{16}} = 1.
\end{array}
$$

Update global variable:
$t = t_0 + 2^8 \cdot t_1$

Define $\alpha_2 = x_2 \cdot (g^{2^8})^{t}.$
         
### i = 3
Lookup $t_2$ s.t. 

$$
\begin{array}{l}
\alpha_2 = (g^{2^{-24}})^{t_2} &\implies x_2 \cdot (g^{2^8})^{t_0 + 2^8\cdot {t_1}} = (g^{2^{-24}})^{t_2} \\
&\implies x_2 \cdot (g^{2^8})^{t_0 + 2^8 \cdot t_1 + 2^{16} \cdot t_2} = 1.
\end{array}
$$

Update global variable:
$t = t_0 + 2^8 \cdot t_1 + 2^{16} \cdot t_2$

Define $\alpha_3 = x_3 \cdot g^{t}.$

### Final result
Lookup $t_3$ s.t.

$$
\begin{array}{l}
\alpha_3 = (g^{2^{-24}})^{t_3} &\implies x_3 \cdot g^{t_0 + 2^8\cdot {t_1} + 2^{16} \cdot t_2} = (g^{2^{-24}})^{t_3} \\
&\implies x_3 \cdot g^{t_0 + 2^8 \cdot t_1 + 2^{16} \cdot t_2 + 2^{24} \cdot t_3} = 1.
\end{array}
$$

Update global variable:
$t = t_0 + 2^8 \cdot t_1 + 2^{16} \cdot t_2 + 2^{24} \cdot t_3$

We can now write
$$
\begin{array}{l}
x_3 \cdot g^{t} = 1 &\implies x_3 \cdot g^{t + 1} = g \\
&\implies uv^2 \cdot g^{t + 1} = g \\
&\implies uv^2 = g^{-t} \\
&\implies uv \cdot g^{t / 2} = v^{-1} g^{-t / 2}.
\end{array}
$$

Squaring the RHS, we observe that $(v^{-1} g^{-t / 2})^2 = v^{-2}g^{-t} = u.$ Therefore, the square root of $u$ is $v^{-1} g^{-t / 2}.$
