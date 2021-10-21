# Halo 2 proofs

## Proofs as opaque byte streams

In proving system implementations like `bellman`, there is a concrete `Proof` struct that
encapsulates the proof data, is returned by a prover, and can be passed to a verifier.

`halo2` does not contain any proof-like structures, for several reasons:

- The Proof structures would contain vectors of (vectors of) curve points and scalars.
  This complicates serialization/deserialization of proofs because the lengths of these
  vectors depend on the configuration of the circuit. However, we didn't want to encode
  the lengths of vectors inside of proofs, because at runtime the circuit is fixed, and
  thus so are the proof sizes.
- It's easy to accidentally put stuff into a Proof structure that isn't also placed in the
  transcript, which is a hazard when developing and implementing a proving system.
- We needed to be able to create multiple PLONK proofs at the same time; these proofs
  share many different substructures when they are for the same circuit.

Instead, `halo2` treats proof objects as opaque byte streams. Creation and consumption of
these byte streams happens via the transcript:

- The `TranscriptWrite` trait represents something that we can write proof components to
  (at proving time).
- The `TranscriptRead` trait represents something that we can read proof components from
  (at verifying time).

Crucially, implementations of `TranscriptWrite` are responsible for simultaneously writing
to some `std::io::Write` buffer at the same time that they hash things into the transcript,
and similarly for `TranscriptRead`/`std::io::Read`.

As a bonus, treating proofs as opaque byte streams ensures that verification accounts for
the cost of deserialization, which isn't negligible due to point compression.

## Proof encoding

A Halo 2 proof, constructed over a curve $E(\mathbb{F}_p)$, is encoded as a stream of:

- Points $P \in E(\mathbb{F}_p)$) (for commitments to polynomials), and
- Scalars $s \in \mathbb{F}_q$) (for evaluations of polynomials, and blinding values).

For the Pallas and Vesta curves, both points and scalars have 32-byte encodings, meaning
that proofs are always a multiple of 32 bytes.

The `halo2` crate supports proving multiple instances of a circuit simultaneously, in
order to share common proof components and protocol logic.

In the encoding description below, we will use the following circuit-specific constants:

- $k$ - the size parameter of the circuit (which has $2^k$ rows).
- $A$ - the number of advice columns.
- $F$ - the number of fixed columns.
- $I$ - the number of instance columns.
- $L$ - the number of lookup arguments.
- $P$ - the number of permutation arguments.
- $\textsf{Col}_P$ - the number of columns involved in permutation argument $P$.
- $D$ - the maximum degree for the quotient polynomial.
- $Q_A$ - the number of advice column queries.
- $Q_F$ - the number of fixed column queries.
- $Q_I$ - the number of instance column queries.
- $M$ - the number of instances of the circuit that are being proven simultaneously.

As the proof encoding directly follows the transcript, we can break the encoding into
sections matching the Halo 2 protocol:

- PLONK commitments:
  - $A$ points (repeated $M$ times).
  - $2L$ points (repeated $M$ times).
  - $P$ points (repeated $M$ times).
  - $L$ points (repeated $M$ times).

- Vanishing argument:
  - $D - 1$ points.
  - $Q_I$ scalars (repeated $M$ times).
  - $Q_A$ scalars (repeated $M$ times).
  - $Q_F$ scalars.
  - $D - 1$ scalars.

- PLONK evaluations:
  - $(2 + \textsf{Col}_P) \times P$ scalars (repeated $M$ times).
  - $5L$ scalars (repeated $M$ times).

- Multiopening argument:
  - 1 point.
  - 1 scalar per set of points in the multiopening argument.

- Polynomial commitment scheme:
  - $1 + 2k$ points.
  - $2$ scalars.
