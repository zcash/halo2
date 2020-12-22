# 16-bit table chip for SHA-256

The main chip implementation for SHA-256 in halo2 is based around a 16-bit lookup table.
This requires a minimum of $2^{16}$ circuit rows, and is therefore suitable for use in
larger circuits.

## Specification

SHA-256 is specified in [NIST FIPS PUB 180-4](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf).

Unlike the specification, we use $\boxplus$ for addition modulo $2^{32}$, and $+$ for
field addition.

$\oplus$ is used for XOR.

Let's target a max constraint degree of $9$. That will allow us to handle constraining
carries and "small pieces" to a range of up to $\{0..7\}$ in one row.

## Compression round

$$
\begin{array}{rcl}
Ch(E, F, G)  &=& (E \wedge F) \oplus (¬E \wedge G) \\
Maj(A, B, C) &=& (A \wedge B) \oplus (A \wedge C) \oplus (B \wedge C) \\
             &=& count(A, B, C) \geq 2 \\
\Sigma_0(A)  &=& (A ⋙ 2) \oplus (A ⋙ 13) \oplus (A ⋙ 22) \\
\Sigma_1(E)  &=& (E ⋙ 6) \oplus (E ⋙ 11) \oplus (E ⋙ 25) \\
H' &=& H + Ch(E, F, G) + \Sigma_1(E) + K_t + W_t \\
E_{new} &=& reduce_6(H' + D) \\
A_{new} &=& reduce_7(H' + Maj(A, B, C) + \Sigma_0(A))
\end{array}
$$

where $reduce_i$ must handle a carry in $\{0, \ldots, i-1\}$.

There are $64$ compression rounds. $A, B, C, D, E, F, G, H$ are $32$ bits each. Note that
we will rely on having each of these words in both "dense" and "spread" forms; this is
explained below. 

![The SHA-256 compression function](./compression.png)

$(a_L, a_H) \boxplus (b_L, b_H) = (c_L, c_H)$, where
$\hspace{3em}\_ \cdot 2^{32} + (c_H : \mathbb{Z}_{2^{16}}) \cdot 2^{16} + (c_L : \mathbb{Z}_{2^{16}}) = (a_H + b_H) \cdot 2^{16} + a_L + b_L$

Note that this correctly handles the carry from $a_L + b_L$.
 
More generally any bit-decomposition of the output can be used, not just a decomposition
into $16$-bit chunks.

Define $\mathtt{spread}$ as a table mapping a $16$-bit input to an output interleaved with
zero bits. We do not require a separate table for range checks because $\mathtt{spread}$
can be used.

In fact, this way we additionally get the "spread" form of the output for free; in
particular this is true for the output of the bottom-right $\boxplus$ which becomes
$A_{new}$, and the output of the leftmost $\boxplus$ which becomes $E_{new}$. We will use
this below to optimize $Maj$ and $Ch$.

### Maj function

$Maj$ can be done in $4$ lookups: $2\; \mathtt{spread} * 2$ chunks

- As mentioned above, after the first round we already have $A$ in spread form $A'$.
  Similarly, $B$ and $C$ are equal to the $A$ and $B$ respectively of the previous round,
  and therefore in the steady state we already have them in spread form $B'$ and $C'$. In
  fact we can also assume we have them in spread form in the first round, either from the
  fixed IV or from the use of $\mathtt{spread}$ to reduce the output of the feedforward in
  the previous block.
- Add the spread forms in the field: $M' = A' + B' + C'$;
  - We can add them as $32$-bit words or in pieces; it's equivalent
- Witness the compressed even bits $M^{even}_i$ and the compressed odd bits $M^{odd}_i$ for $i = \{0..1\}$;
- Constrain $M' = \mathtt{spread}(M^{even}_0) + 2 \cdot \mathtt{spread}(M^{odd}_0) + 2^{32} \cdot \mathtt{spread}(M^{even}_1) + 2^{33} \cdot \mathtt{spread}(M^{odd}_1)$, where $M^{odd}_i$ is the $Maj$ function output.

> Note: by "even" bits we mean the bits of weight an even-power of $2$, i.e. of weight
> $2^0, 2^2, \ldots$. Similarly by "odd" bits we mean the bits of weight an odd-power of
> $2$.

### Ch function
> TODO: can probably be optimised to $4$ or $5$ lookups using an additional table.
> 
$Ch$ can be done in $8$ lookups: $4\; \mathtt{spread} * 2$ chunks

- As mentioned above, after the first round we already have $E$ in spread form $E'$.
  Similarly, $F$ and $G$ are equal to the $E$ and $F$ respectively of the previous round,
  and therefore in the steady state we already have them in spread form $F'$ and $G'$. In
  fact we can also assume we have them in spread form in the first round, either from the
  fixed IV or from the use of $\mathtt{spread}$ to reduce the output of the feedforward in
  the previous block.
- Calculate $P' = E' + F'$ and $Q' = (evens - E') + G'$, where $evens = \mathtt{spread}(2^{32} - 1)$.
  - We can add them as $32$-bit words or in pieces; it's equivalent.
  - $evens - E'$ works to compute the spread of $¬E$ even though negation and
    $\mathtt{spread}$ do not commute in general. It works because each spread bit in $E'$
    is subtracted from $1$, so there are no borrows.
- Witness $P^{even}_i, P^{odd}_i, Q^{even}_i, Q^{odd}_i$ such that
  $P' = \mathtt{spread}(P^{even}_0) + 2 \cdot \mathtt{spread}(P^{odd}_0) + 2^{32} \cdot \mathtt{spread}(P^{even}_1) + 2^{33} \cdot \mathtt{spread}(P^{odd}_1)$, and similarly for $Q'$.
- $\{P^{odd}_i + Q^{odd}_i\}_{i=0..1}$ is the $Ch$ function output.

### Σ_0 function

$\Sigma_0(A)$ can be done in $6$ lookups.

To achieve this we first split $A$ into pieces $(a, b, c, d)$, of lengths $(2, 11, 9, 10)$
bits respectively counting from the little end. At the same time we obtain the spread
forms of these pieces. This can all be done in two PLONK rows, because the $10$ and
$11$-bit pieces can be handled using $\mathtt{spread}$ lookups, and the $9$-bit piece can
be split into $3 * 3$-bit subpieces. The latter and the remaining $2$-bit piece can be
range-checked by polynomial constraints in parallel with the two lookups, two small pieces
in each row. The spread forms of these small pieces are found by interpolation.

Note that the splitting into pieces can be combined with the reduction of $A_{new}$, i.e.
no extra lookups are needed for the latter. In the last round we reduce $A_{new}$ after
adding the feedforward (requiring a carry of $\{0, \ldots, 7\}$ which is fine).

$(A ⋙ 2) \oplus (A ⋙ 13) \oplus (A ⋙ 22)$ is equivalent to
$(A ⋙ 2) \oplus (A ⋙ 13) \oplus (A ⋘ 10)$:

![](./upp_sigma_0.png)

Then, using $4$ more $\mathtt{spread}$ lookups we obtain the result as the even bits of a
linear combination of the pieces:

$$
\begin{array}{rcccccccl}
     &    (a    &||&    d    &||&    c   &||&   b) & \oplus \\
     &    (b    &||&    a    &||&    d   &||&   c) & \oplus \\
     &    (c    &||&    b    &||&    a   &||&   d) & \\
&&&&\Downarrow \\
R' = & 4^{30} a &+& 4^{20} d &+& 4^{11} c &+&   b\;&+ \\
     & 4^{21} b &+& 4^{19} a &+& 4^{ 9} d &+&   c\;&+ \\
     & 4^{23} c &+& 4^{12} b &+& 4^{10} a &+&   d\;&
\end{array}
$$

That is, we witness the compressed even bits $R^{even}_i$ and the compressed odd bits
$R^{odd}_i$, and constrain
$$R' = \mathtt{spread}(R^{even}_0) + 2 \cdot \mathtt{spread}(R^{odd}_0) + 2^{32} \cdot \mathtt{spread}(R^{even}_1) + 2^{33} \cdot \mathtt{spread}(R^{odd}_1)$$
where $\{R^{even}_i\}_{i=0..1}$ is the $\Sigma_0$ function output.

### Σ_1 function

$\Sigma_1(E)$ can be done in $6$ lookups.

To achieve this we first split $E$ into pieces $(a, b, c, d)$, of lengths $(6, 5, 14, 7)$
bits respectively counting from the little end. At the same time we obtain the spread
forms of these pieces. This can all be done in two PLONK rows, because the $7$ and
$14$-bit pieces can be handled using $\mathtt{spread}$ lookups, the $5$-bit piece can be
split into $3$ and $2$-bit subpieces, and the $6$-bit piece can be split into $2 * 3$-bit
subpieces. The four small pieces can be range-checked by polynomial constraints in
parallel with the two lookups, two small pieces in each row. The spread forms of these
small pieces are found by interpolation.

Note that the splitting into pieces can be combined with the reduction of $E_{new}$, i.e.
no extra lookups are needed for the latter. In the last round we reduce $E_{new}$ after
adding the feedforward (requiring a carry of $\{0, \ldots, 6\}$ which is fine).

$(E ⋙ 6) \oplus (E ⋙ 11) \oplus (E ⋙ 25)$ is equivalent to
$(E ⋙ 6) \oplus (E ⋙ 11) \oplus (E ⋘ 7)$.

![](./upp_sigma_1.png)

Then, using $4$ more $\mathtt{spread}$ lookups we obtain the result as the even bits of a
linear combination of the pieces, in the same way we did for $\Sigma_0$:

$$
\begin{array}{rcccccccl}
     &    (a    &||&    d    &||&    c   &||&   b) & \oplus \\
     &    (b    &||&    a    &||&    d   &||&   c) & \oplus \\
     &    (c    &||&    b    &||&    a   &||&   d) & \\
&&&&\Downarrow \\
R' = & 4^{26} a &+& 4^{19} d &+& 4^{ 5} c &+&   b\;&+ \\
     & 4^{27} b &+& 4^{21} a &+& 4^{14} d &+&   c\;&+ \\
     & 4^{18} c &+& 4^{13} b &+& 4^{ 7} a &+&   d\;&
\end{array}
$$

That is, we witness the compressed even bits $R^{even}_i$ and the compressed odd bits
$R^{odd}_i$, and constrain
$$R' = \mathtt{spread}(R^{even}_0) + 2 \cdot \mathtt{spread}(R^{odd}_0) + 2^{32} \cdot \mathtt{spread}(R^{even}_1) + 2^{33} \cdot \mathtt{spread}(R^{odd}_1)$$
where $\{R^{even}_i\}_{i=0..1}$ is the $\Sigma_1$ function output.

## Block decomposition

For each block $M \in \{0,1\}^{512}$ of the padded message, $64$ words of $32$ bits each
are constructed as follows:
- The first $16$ are obtained by splitting $M$ into $32$-bit blocks $$M = W_1 || W_2 || \cdots || W_{15} || W_{16};$$
- The remaining $48$ words are constructed using the formula:
$$W_i = \sigma_1(W_{i-2}) \boxplus W_{i-7} \boxplus \sigma_0(W_{i-15}) \boxplus W_{i-16},$$ for $i = 17, \ldots, 64$.

> Note: $1$-based numbering is used for the $W$ word indices.

$$
\begin{array}{ccc}
\sigma_0(X) &=& (X ⋙ 7) \oplus (X ⋙ 18) \oplus (X ≫ 3) \\
\sigma_1(X) &=& (X ⋙ 17) \oplus (X ⋙ 19) \oplus (X ≫ 10) \\
\end{array}
$$

> Note: $≫$ is a right-**shift**, not a rotation.

### σ_0 function

$(X ⋙ 7) \oplus (X ⋙ 18) \oplus (X ≫ 3)$ is equivalent to
$(X ⋙ 7) \oplus (X ⋘ 14) \oplus (X ≫ 3)$.

![](./low_sigma_0.png)

As above but with pieces $(a, b, c, d)$ of lengths $(3, 4, 11, 14)$ counting from the
little end. Split $b$ into two $2$-bit subpieces.

$$
\begin{array}{rcccccccl}
     & (0^{[3]} &||&    d    &||&    c   &||&   b) & \oplus \\
     & (\;\;\;b &||&    a    &||&    d   &||&   c) & \oplus \\
     & (\;\;\;c &||&    b    &||&    a   &||&   d) & \\
&&&&\Downarrow \\
R' = &          & & 4^{15} d &+& 4^{ 4} c &+&   b\;&+ \\
     & 4^{28} b &+& 4^{25} a &+& 4^{11} d &+&   c\;&+ \\
     & 4^{21} c &+& 4^{17} b &+& 4^{14} a &+&   d\;&
\end{array}
$$

### σ_1 function

$(X ⋙ 17) \oplus (X ⋙ 19) \oplus (X ≫ 10)$ is equivalent to
$(X ⋘ 15) \oplus (X ⋘ 13) \oplus (X ≫ 10)$.

![](./low_sigma_1.png)

TODO: this diagram doesn't match the expression on the right. This is just for consistency
with the other diagrams.

As above but with pieces $(a, b, c, d)$ of lengths $(10, 7, 2, 13)$ counting from the
little end. Split $b$ into $(3, 2, 2)$-bit subpieces.

$$
\begin{array}{rcccccccl}
     & (0^{[10]}&||&    d    &||&    c   &||&   b) & \oplus \\
     & (\;\;\;b &||&    a    &||&    d   &||&   c) & \oplus \\
     & (\;\;\;c &||&    b    &||&    a   &||&   d) & \\
&&&&\Downarrow \\
R' = &          & & 4^{ 9} d &+& 4^{ 7} c &+&   b\;&+ \\
     & 4^{25} b &+& 4^{15} a &+& 4^{ 2} d &+&   c\;&+ \\
     & 4^{30} c &+& 4^{23} b &+& 4^{13} a &+&   d\;&
\end{array}
$$

### Message scheduling

We apply $\sigma_0$ to $W_{2..49}$, and $\sigma_1$ to $W_{15..62}$. In order to avoid
redundant applications of $\mathtt{spread}$, we can merge the splitting into pieces for
$\sigma_0$ and $\sigma_1$ in the case of $W_{15..49}$. Merging the piece lengths
$(3, 4, 11, 14)$ and $(10, 7, 2, 13)$ gives pieces of lengths $(3, 4, 3, 7, 1, 1, 13)$.

![](./bit_reassignment.png)

If we can do the merged split in $3$ rows (as opposed to a total of $4$ rows when
splitting for $\sigma_0$ and $\sigma_1$ separately), we save $35$ rows.

> These might even be doable in $2$ rows; not sure.
> [name=Daira]

We can merge the reduction mod $2^{32}$ of $W_{17..62}$ into their splitting when they are
used to compute subsequent words, similarly to what we did for $A$ and $E$ in the round
function.

We will still need to reduce $W_{63..64}$ since they are not split. (Technically we could
leave them unreduced since they will be reduced later when they are used to compute
$A_{new}$ and $E_{new}$ -- but that would require handling a carry of up to $10$ rather
than $6$, so it's not worth the complexity.)

The resulting message schedule cost is:
- $2$ rows to constrain $W_1$ to $32$ bits
  - This is technically optional, but let's do it for robustness, since the rest of the
    input is constrained for free.
- $13*2$ rows to split $W_{2..14}$ into $(3, 4, 11, 14)$-bit pieces
- $35*3$ rows to split $W_{15..49}$ into $(3, 4, 3, 7, 1, 1, 13)$-bit pieces (merged with
  a reduction for $W_{17..49}$)
- $13*2$ rows to split $W_{50..62}$ into $(10, 7, 2, 13)$-bit pieces (merged with a
  reduction)
- $4*48$ rows to extract the results of $\sigma_0$ for $W_{2..49}$
- $4*48$ rows to extract the results of $\sigma_1$ for $W_{15..62}$
- $2*2$ rows to reduce $W_{63..64}$
- $= 547$ rows.

## Overall cost

For each round:
- $8$ rows for $Ch$
- $4$ rows for $Maj$
- $6$ rows for $\Sigma_0$
- $6$ rows for $\Sigma_1$
- $reduce_6$ and $reduce_7$ are always free
- $= 24$ per round

This gives $24*64 = 1792$ rows for all of "step 3", to which we need to add:

- $547$ rows for message scheduling
- $2*8$ rows for $8$ reductions mod $2^{32}$ in "step 4"

giving a total of $2099$ rows.

## Tables

We only require one table $\mathtt{spread}$, with $2^{16}$ rows and $3$ columns. We need a
tag column to allow selecting $(7, 10, 11, 13, 14)$-bit subsets of the table for
$\Sigma_{0..1}$ and $\sigma_{0..1}$.

### `spread` table

| row          | tag | table (16b)      | spread (32b)                     |
|--------------|-----|------------------|----------------------------------|
| $0$          |  0  | 0000000000000000 | 00000000000000000000000000000000 |
| $1$          |  0  | 0000000000000001 | 00000000000000000000000000000001 |
| $2$          |  0  | 0000000000000010 | 00000000000000000000000000000100 |
| $3$          |  0  | 0000000000000011 | 00000000000000000000000000000101 |
| ...          |  0  |       ...        |                ...               |
| $2^{7} - 1$  |  0  | 0000000001111111 | 00000000000000000001010101010101 |
| $2^{7}$      |  1  | 0000000010000000 | 00000000000000000100000000000000 |
| ...          |  1  |       ...        |                ...               |
| $2^{10} - 1$ |  1  | 0000001111111111 | 00000000000001010101010101010101 |
| ...          |  2  |       ...        |                ...               |
| $2^{11} - 1$ |  2  | 0000011111111111 | 00000000010101010101010101010101 |
| ...          |  3  |       ...        |                ...               |
| $2^{13} - 1$ |  3  | 0001111111111111 | 00000001010101010101010101010101 |
| ...          |  4  |       ...        |                ...               |
| $2^{14} - 1$ |  4  | 0011111111111111 | 00000101010101010101010101010101 |
| ...          |  5  |       ...        |                ...               |
| $2^{16} - 1$ |  5  | 1111111111111111 | 01010101010101010101010101010101 |

For example, to do an $11$-bit $\mathtt{spread}$ lookup, we polynomial-constrain the tag
to be in $\{0, 1, 2\}$. For the most common case of a $16$-bit lookup, we don't need to
constrain the tag. Note that we can fill any unused rows beyond $2^{16}$ with a duplicate
entry, e.g. all-zeroes.
