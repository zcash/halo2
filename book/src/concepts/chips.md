# Chips

The previous section gives a fairly low-level description of a circuit. When implementing circuits we will
typically use a higher-level API which aims for the desirable characteristics of auditability,
efficiency, modularity, and expressiveness.

Some of the terminology and concepts used in this API are taken from an analogy with
integrated circuit design and layout. [As for integrated circuits](https://opencores.org/),
the above desirable characteristics are easier to obtain by composing ***chips*** that provide
efficient pre-built implementations of particular functionality.

For example, we might have chips that implement particular cryptographic primitives such as a
hash function or cipher, or algorithms like scalar multiplication or pairings.

In PLONKish circuits, it is possible to build up arbitrary logic just from standard gates that do
field multiplication and addition. However, very significant efficiency gains can be obtained by
using custom gates.

Using our API, we define chips that "know" how to use particular sets of custom gates. This
creates an abstraction layer that isolates the implementation of a high-level circuit from the
complexity of using custom gates directly.

> Even if we sometimes need to "wear two hats", by implementing both a high-level circuit and
> the chips that it uses, the intention is that this separation will result in code that is
> easier to understand, audit, and maintain/reuse. This is partly because some potential
> implementation errors are ruled out by construction.

Gates in PLONKish circuits refer to cells by ***relative references***, i.e. to the cell in a given
column, and the row at a given offset relative to the one in which the gate's selector is set. We
call this an ***offset reference*** when the offset is nonzero (i.e. offset references are a subset
of relative references).

Relative references contrast with ***absolute references*** used in equality constraints,
which can point to any cell.

The motivation for offset references is to reduce the number of columns needed in the
configuration, which reduces proof size. If we did not have offset references then we would
need a column to hold each value referred to by a custom gate, and we would need to use
equality constraints to copy values from other cells of the circuit into that column. With
offset references, we not only need fewer columns; we also do not need equality constraints to
be supported for all of those columns, which improves efficiency.

In R1CS (another arithmetization which may be more familiar to some readers, but don't worry
if it isn't), a circuit consists of a "sea of gates" with no semantically significant ordering.
Because of offset references, the order of rows in a PLONKish circuit, on the other hand, *is*
significant. We're going to make some simplifying assumptions and define some abstractions to
tame the resulting complexity: the aim will be that, [at the gadget level](gadgets.md) where
we do most of our circuit construction, we will not have to deal with relative references or
with gate layout explicitly.

We will partition a circuit into ***regions***, where each region contains a disjoint subset
of cells, and relative references only ever point *within* a region. Part of the responsibility
of a chip implementation is to ensure that gates that make offset references are laid out in
the correct positions in a region.

Given the set of regions and their ***shapes***, we will use a separate ***floor planner***
to decide where (i.e. at what starting row) each region is placed. There is a default floor
planner that implements a very general algorithm, but you can write your own floor planner if
you need to.

Floor planning will in general leave gaps in the matrix, because the gates in a given row did
not use all available columns. These are filled in —as far as possible— by gates that do
not require offset references, which allows them to be placed on any row.

Chips can also define lookup tables. If more than one table is defined for the same lookup
argument, we can use a ***tag column*** to specify which table is used on each row. It is also
possible to perform a lookup in the union of several tables (limited by the polynomial degree
bound).

## Composing chips
In order to combine functionality from several chips, we compose them in a tree. The top-level
chip defines a set of fixed, advice, and instance columns, and then specifies how they
should be distributed between lower-level chips.

In the simplest case, each lower-level chips will use columns disjoint from the other chips.
However, it is allowed to share a column between chips. It is important to optimize the number
of advice columns in particular, because that affects proof size.

The result (possibly after optimization) is a PLONKish configuration. Our circuit implementation
will be parameterized on a chip, and can use any features of the supported lower-level chips via
the top-level chip.

Our hope is that less expert users will normally be able to find an existing chip that
supports the operations they need, or only have to make minor modifications to an existing
chip. Expert users will have full control to do the kind of
[circuit optimizations](https://zips.z.cash/protocol/canopy.pdf#circuitdesign)
[that ECC is famous  for](https://electriccoin.co/blog/cultivating-sapling-faster-zksnarks/) 🙂.
