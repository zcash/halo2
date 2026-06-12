# Rebuttal: halo2_proofs_plonk_protocol (Round 3)

**Report**: agent_out/reports/130626/halo2_proofs_plonk_protocol.md
**Date**: 2026-06-13

> **Note on this submission**: Verbatim copy of 2026-06-12 report. Verdicts unchanged.

## Executive Summary

This is the third submission of an identical two-finding report. No new evidence, no new code
references, and no response to the specific technical rebuttals issued in round 2 (120626_2) has
been provided. Both findings have been re-examined against the codebase. Neither verdict changes.

F1 remains Low/Informational. The Debug-based VK hash is a legitimate hardening gap, but the
Critical/network-halt scenario requires a toolchain version change that cannot occur given the
pinned compiler, and the auditor has not produced a counterexample showing divergent output from
any two real Rust releases. F2 remains confirmed at Low. The EOF omission is real; the
double-spend impact escalation is unsupported speculation about callers outside this codebase.

---

## Finding 1: fmt::Debug for VK Hash

**Auditor Severity**: Critical (Deterministic Network Fork / DoS)
**Prior Verdict (120626_2)**: Overstated — Low/Informational
**Round 3 Verdict**: Unchanged — Low/Informational

### Analysis

The code at `halo2_proofs/src/plonk.rs` does exactly what the report describes:

```rust
let s = format!("{:?}", vk.pinned());
hasher.update(&(s.len() as u64).to_le_bytes());
hasher.update(s.as_bytes());
```

This was confirmed in round 1 and is not in dispute.

What the auditor still has not addressed:

**1. Toolchain is pinned.** The repository carries a `rust-toolchain` file locking to 1.60.0. The
entire catastrophic scenario — prover and verifier diverging because one was recompiled on a newer
toolchain — cannot happen without a deliberate human decision to upgrade and simultaneously break
the pin. A vulnerability that requires an operator to first circumvent the project's own build
controls is not a remote-exploit finding. It is a process recommendation.

**2. All Debug impls in the chain are manually written, not derived.** The auditor claims that
derived `fmt::Debug` formatting "routinely changes" across compiler versions. That is true for
derived impls. The types exercised here — field elements, curve points, `PinnedVerificationKey`,
`PinnedConstraintSystem`, `PinnedGates` — use hand-written `Debug` implementations over stable
math representations (byte arrays, integers, fixed-size arrays). No compiler version has ever
changed the `Debug` output of `[u8; 32]` or a manually formatted struct. The auditor has not
pointed to a single derived impl anywhere in this call chain, because there is not one.

**3. No divergence example has been produced.** Three rounds in, no concrete demonstration exists
that any two Rust releases — 1.60.0 versus anything — produce different `format!("{:?}", vk.pinned())`
output for the same circuit. The auditor's argument rests entirely on the theoretical instability
of `fmt::Debug` as a language feature, applied without evidence to this specific instantiation.

**4. The hardening recommendation is correct.** Replacing the Debug-based hash with a canonical
binary serialization would eliminate any future risk and is good engineering hygiene. That is
worth recording. It does not make this Critical.

**Severity**: Low/Informational. Hardening gap, not an exploitable vulnerability in any
demonstrated attack path.

---

## Finding 2: Unconsumed Transcript Bytes

**Auditor Severity**: High (Replay Attacks / Double-Spend Vector)
**Prior Verdict (120626_2)**: Confirmed at Low
**Round 3 Verdict**: Unchanged — Confirmed at Low

### Analysis

The EOF omission is real and has been confirmed since round 1. After `verify_proof` returns, no
code asserts that the `Blake2bRead` reader is exhausted. A proof of N bytes and a proof of N+k
bytes (where the trailing k bytes are arbitrary) will both pass verification, producing distinct
byte arrays that hash to distinct values.

What remains unsupported is the impact escalation to High/double-spend:

**1. Proof bytes are not used as nullifiers in this codebase.** The auditor's double-spend scenario
requires a caller that uses `hash(proof_bytes)` as a replay-protection nullifier. No such caller
exists in `halo2_proofs`. This is a library; its callers define their own nullifier schemes. The
auditor has pointed to no halo2-based application in scope that uses proof-byte hashes as
identifiers. Invoking snarkjs CVEs by analogy — "this class of bug has warranted High-severity
CVEs elsewhere" — does not establish that the same impact applies here when the requisite calling
pattern is absent from the codebase under review.

**2. Malleability requires an active capability, not just an absent check.** An attacker can
produce a mutated proof only if they already possess a valid proof to append bytes to. Obtaining a
valid proof is not a cheap primitive: it either requires knowing the witness (legitimate prover) or
breaking the proof system (assumed hard). The attack surface is therefore limited to scenarios
where a legitimate prover deliberately or accidentally produces oversized proof bytes — not
arbitrary third-party forgery.

**3. The EOF check remediation is straightforward and should be implemented.** The auditor's
proposed `assert_eof` method is correct. The fix is low-effort and eliminates the malleability
class entirely.

**Severity**: Low. Malleability confirmed; double-spend impact requires a calling pattern not
present in this codebase or demonstrated in scope.

---

## Summary Table

| # | Finding | Auditor Severity | Prior Verdict (120626_2) | Round 3 Verdict | Final Severity |
|---|---------|-----------------|--------------------------|-----------------|----------------|
| F1 | fmt::Debug VK Hash | Critical | Overstated → Low/Informational | Unchanged | Low/Informational |
| F2 | Unconsumed Transcript Bytes | High | Confirmed at Low | Unchanged | Low |

---

## Notes to Auditor

Submitting an identical report for a third consecutive round without addressing the specific
technical rebuttals issued in round 2 does not advance the discussion. The rebuttals in 120626_2
identified two concrete, verifiable claims: (a) the toolchain is pinned, and (b) the Debug impls
in this call chain are manually written. Neither claim has been engaged with. If either claim is
factually wrong, provide the file path and line number. If a divergence example exists across
toolchain versions, produce it. Absent that, these verdicts are final.
