# Commitment tree

One of the things we learned from Sapling is that having a single global commitment tree
makes life hard for light client wallets. When a new note is received, the wallet derives
its incremental witness from the state of the global tree at the point when the note's
commitment is appended; this incremental state then needs to be updated with every
subsequent commitment in the block in-order. It isn't efficient for a server to
pre-compute and send over the necessary incremental updates for every new note in a block,
and if a wallet requested a specific update from the server it would leak the specific
note that was received.

Orchard addresses this by splitting the commitment tree into several sub-trees:

- Bundle tree, that accumulates the commitments within a single bundle (and thus a single
  transaction).
- Block tree, that accumulates the bundle tree roots within a single block.
- Global tree, that accumulates the block tree roots.

Each of these trees has a fixed depth (necessary for being able to create proofs).

Chains that integrate Orchard can decouple the limits on commitments-per-subtree from
higher-layer constraints like block size, by enabling their blocks and transactions to be
structured internally as a series of Orchard blocks or txs (e.g. a Zcash block would
contain a `Vec<BlockTreeRoot>`, that each get appended in-order).

Zcash level: we also bind these roots into the FlyClient history leaves, so that light
clients can assert they are valid independently of the full block.

TODO: Sean is pretty sure we can just improve the Incremental Merkle Tree implementation
to work around this, without domain-separating the tree. If we can do that instead, it may
be simpler.
