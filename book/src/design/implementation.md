# Implementation

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
