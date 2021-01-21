# Commitments

As in Sapling, we require two kinds of commitment schemes in Orchard:
- $\mathit{HomomorphicCommit}$ is a linearly homomorphic commitment scheme with perfect hiding,
  and strong binding reducible to DL.
- $\mathit{Commit}$ and $\mathit{ShortCommit}$ are commitment schemes with perfect hiding, and
  strong binding reducible to DL.

By "strong binding" we mean that the scheme is collision resistant on the input and
randomness.

We instantiate $\mathit{HomomorphicCommit}$ with a Pedersen commitment, and use it for
value commitments:

$$\mathsf{cv} = \mathit{HomomorphicCommit}^{\mathsf{cv}}_{\mathsf{rcv}}(v)$$

We instantiate $\mathit{Commit}$ and $\mathit{ShortCommit}$ with Sinsemilla, and use them
for all other commitments:

$$\mathsf{ivk} = \mathit{ShortCommit}^{\mathsf{ivk}}_{\mathsf{rivk}}(\mathsf{ak}, \mathsf{nk})$$
$$\mathsf{cm} = \mathit{Commit}^{\mathsf{cm}}_{\mathsf{rcm}}(\text{rest of note})$$

This is the same split (and rationale) as in Sapling, but using the more PLONK-efficient
Sinsemilla instead of Bowe--Hopwood Pedersen hashes.

Note that we also deviate from Sapling by using $\mathit{ShortCommit}$ to deriving $\mathsf{ivk}$
instead of a full PRF. This removes an unnecessary (large) PRF primitive from the circuit,
at the cost of requiring $\mathsf{rivk}$ to be part of the full viewing key.
