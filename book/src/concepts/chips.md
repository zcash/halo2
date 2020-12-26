# Chips

In order to combine functionality from several cores, we use a ***chip***. To implement a
chip, we define a set of fixed, advice, and auxiliary columns, and then specify how they
should be distributed between cores.

In the simplest case, each core will use columns disjoint from the other cores. However, it
is allowed to share a column between cores. It is important to optimize the number of advice
columns in particular, because that affects proof size.

The result (possibly after optimization) is a UPA configuration. Our circuit implementation
will be parameterized on a chip, and can use any features of the supported cores via the chip.

Our hope is that less expert users will normally be able to find an existing chip that
supports the operations they need, or only have to make minor modifications to an existing
chip. Expert users will have full control to do the kind of
[circuit optimizations](https://zips.z.cash/protocol/canopy.pdf#circuitdesign)
[that ECC is famous  for](https://electriccoin.co/blog/cultivating-sapling-faster-zksnarks/) 🙂.
