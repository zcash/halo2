# Proof systems

The aim of any ***proof system*** is to be able to prove ***instances*** of ***statements***.

A statement is a high-level description of what is to be proven, parameterized by
***public inputs*** of given types. An instance is a statement with particular values for
these public inputs.

Normally the statement will also have ***private inputs***. The private inputs and any
intermediate values that make an instance of a statement hold, are collectively called a
***witness*** for that instance.

> The intermediate values depend on how we express the statement. We assume that we can
> compute them efficiently from the private and public inputs (if that were not the case
> then we would consider them part of the private inputs).

A ***Non-interactive Argument of Knowledge*** allows a ***prover*** to create a ***proof***
for a given instance and witness. The proof is data that can be used to convince a
***verifier*** that the creator of the proof knew a witness for which the statement holds on
this instance. The security property that such proofs cannot falsely convince a verifier is
called ***knowledge soundness***.

> This property is subtle given that proofs can be ***malleable***. That is, depending on the
> proof system it may be possible to take an existing proof (or set of proofs) and, without
> knowing the witness(es), modify it/them to produce a distinct proof of the same or a related
> statement. Higher-level protocols that use malleable proof systems need to take this into
> account.

If a proof yields no information about the witness (other than that a witness exists and was
known to the prover), then we say that the proof system is ***zero knowledge***.

A proof system will define an ***arithmetization***, which is a way of describing statements
--- typically in terms of polynomial constraints on variables over a field. An arithmetized
statement is called a ***circuit***.

If the proof is short ---i.e. it has length polylogarithmic in the circuit size--- then
we say that the proof system is ***succinct***, and call it a ***SNARK***
(***Succinct Non-Interactive Argument of Knowledge***).

> By this definition, a SNARK need not have verification time polylogarithmic in the circuit
> size. Some papers use the term ***efficient*** to describe a SNARK with that property, but
> we'll avoid that term since it's ambiguous for SNARKs that support amortized or recursive
> verification, which we'll get to later.

A ***zk-SNARK*** is a zero-knowledge SNARK.
