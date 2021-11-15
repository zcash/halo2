# Gadgets

When implementing a circuit, we could use the features of the chips we've selected directly.
Typically, though, we will use them via ***gadgets***. This indirection is useful because,
for reasons of efficiency and limitations imposed by PLONKish circuits, the chip interfaces will
often be dependent on low-level implementation details. The gadget interface can provide a more
convenient and stable API that abstracts away from extraneous detail.

For example, consider a hash function such as SHA-256. The interface of a chip supporting
SHA-256 might be dependent on internals of the hash function design such as the separation
between message schedule and compression function. The corresponding gadget interface can
provide a more convenient and familiar `update`/`finalize` API, and can also handle parts
of the hash function that do not need chip support, such as padding. This is similar to how
[accelerated](https://software.intel.com/content/www/us/en/develop/articles/intel-sha-extensions.html)
[instructions](https://developer.arm.com/documentation/ddi0514/g/introduction/about-the-cortex-a57-processor-cryptography-engine)
for cryptographic primitives on CPUs are typically accessed via software libraries, rather
than directly.

Gadgets can also provide modular and reusable abstractions for circuit programming
at a higher level, similar to their use in libraries such as
[libsnark](https://github.com/christianlundkvist/libsnark-tutorial) and
[bellman](https://electriccoin.co/blog/bellman-zksnarks-in-rust/). As well as abstracting
*functions*, they can also abstract *types*, such as elliptic curve points or integers of
specific sizes.

