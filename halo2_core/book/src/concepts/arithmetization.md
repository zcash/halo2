# PLONKish Arithmetization

The arithmetization used by Halo 2 comes from [PLONK](https://eprint.iacr.org/2019/953), or
more precisely its extension UltraPLONK that supports custom gates and lookup arguments. We'll
call it [***PLONKish***](https://twitter.com/feministPLT/status/1413815927704014850).

***PLONKish circuits*** are defined in terms of a rectangular matrix of values. We refer to
***rows***, ***columns***, and ***cells*** of this matrix with the conventional meanings.

A PLONKish circuit depends on a ***configuration***:

* A finite field $\mathbb{F}$, where cell values (for a given statement and witness) will be
  elements of $\mathbb{F}$.
* The number of columns in the matrix, and a specification of each column as being
  ***fixed***, ***advice***, or ***instance***. Fixed columns are fixed by the circuit;
  advice columns correspond to witness values; and instance columns are normally used for
  public inputs (technically, they can be used for any elements shared between the prover
  and verifier).

* A subset of the columns that can participate in equality constraints.

* A ***polynomial degree bound***.

* A sequence of ***polynomial constraints***. These are multivariate polynomials over
  $\mathbb{F}$ that must evaluate to zero *for each row*. The variables in a polynomial
  constraint may refer to a cell in a given column of the current row, or a given column of
  another row relative to this one (with wrap-around, i.e. taken modulo $n$). The maximum
  degree of each polynomial is given by the polynomial degree bound.

* A sequence of ***lookup arguments*** defined over tuples of ***input expressions***
  (which are multivariate polynomials as above) and ***table columns***.

A PLONKish circuit also defines:

* The number of rows $n$ in the matrix. $n$ must correspond to the size of a multiplicative
  subgroup of $\mathbb{F}^\times$; typically a power of two.

* A sequence of ***equality constraints***, which specify that two given cells must have equal
  values.

* The values of the fixed columns at each row.

From a circuit description we can generate a ***proving key*** and a ***verification key***,
which are needed for the operations of proving and verification for that circuit.

> Note that we specify the ordering of columns, polynomial constraints, lookup arguments, and
> equality constraints, even though these do not affect the meaning of the circuit. This makes
> it easier to define the generation of proving and verification keys as a deterministic
> process.

Typically, a configuration will define polynomial constraints that are switched off and on by
***selectors*** defined in fixed columns. For example, a constraint $q_i \cdot p(...) = 0$ can
be switched off for a particular row $i$ by setting $q_i = 0$. In this case we sometimes refer
to a set of constraints controlled by a set of selector columns that are designed to be used
together, as a ***gate***. Typically there will be a ***standard gate*** that supports generic
operations like field multiplication and division, and possibly also ***custom gates*** that
support more specialized operations.
