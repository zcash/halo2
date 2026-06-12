# Rebuttal: halo2_gadgets_utilities (Round 3)

**Report**: agent_out/reports/130626/halo2_gadgets_utilities.md
**Date**: 2026-06-13

> **Note on this submission**: Verbatim copy of the 2026-06-12 report. No new analysis provided. Verdicts unchanged.

## Executive Summary

This submission is word-for-word identical to the report reviewed on 2026-06-12. No new evidence, no revised argument, no response to the prior round's objections. Three of the four findings were confirmed in round 2, two with downgraded severity. Those verdicts stand. Finding 4 was rejected for an arithmetic error that this auditor has not attempted to correct; it remains rejected.

---

## Finding 1: Generic K Decoupling in LookupRangeCheckConfig

**Auditor Severity**: Critical
**Prior Verdict (120626_2)**: Confirmed — Medium
**Round 3 Verdict**: Unchanged — Confirmed at Medium

### Analysis

The finding is real. `load()` ignores the generic `K` parameter and unconditionally iterates over `SINSEMILLA_S`, loading 1,024 entries (values `0..=1023`) into `table_idx` regardless of what `K` was specified. A circuit instantiated with `K=8` would therefore have its lookup table populated with values up to 1023 rather than the required ceiling of 255, permitting a prover to assign chunks outside the intended `[0, 2^K)` range and still satisfy the lookup gate.

The critical rating is not supported, however, because the exploit path requires a non-default instantiation that does not exist in production. `LookupRangeCheckConfig` is only ever instantiated through `PallasLookupRangeCheckConfig`, which fixes `K=10`. At `K=10` the table correctly spans `[0, 1023]`, matching the loaded values exactly. There is no circuit in this codebase that passes any other `K`. The gap is a latent API hazard — a future integrator who supplies `K != 10` will silently get the wrong table — but there is no active exploit against any deployed circuit. Medium is the correct ceiling.

The auditor had the opportunity to respond to this reasoning and has not.

---

## Finding 2: Trait Composability Breakdown (LookupRangeCheck4_5BConfig)

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed — High
**Round 3 Verdict**: Unchanged — Confirmed at High

### Analysis

The finding is confirmed at the auditor's stated severity. Each call to `LookupRangeCheck4_5BConfig::configure()` issues a fresh `meta.lookup_table_column()` allocation, binding a new private `table_range_check_tag` column to that instance. Two instances of the chip cannot safely share a layout: calling `load()` on both panics at assignment time due to the double-assignment of the shared `table_idx`; calling `load()` on only one leaves the second instance's tag column unassigned, causing the 4-bit and 5-bit optimized gates to silently fail verification by querying an all-zero tag column that never matches `tag = 4` or `tag = 5`.

Neither the panic path nor the silent-failure path is acceptable. The trait's `load()` signature implies interchangeable instances; the implementation violates that contract. High stands.

---

## Finding 3: Hardcoded pallas::Base in Generic Trait

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed — Low
**Round 3 Verdict**: Unchanged — Confirmed at Low

### Analysis

The type inconsistency is real. The `LookupRangeCheck` trait is declared generic over `F: PrimeFieldBits`, but its required `load()` method pins the `Layouter` to `pallas::Base`. Any downstream circuit over a different field — including `vesta::Base` — cannot satisfy the trait bound; the Rust compiler will reject it.

The severity is Low, not High, for the following reason: the trait, the only concrete impl, and every call site in this repository are all Pallas-only. The `load()` signature's `GeneratorTableConfig` argument is itself a Pallas-specific construct tied to the Sinsemilla S-table. There is no plausible non-Pallas consumer of this trait in scope. The hardcoding is a type-hygiene defect that prevents future field-agnostic reuse, not a defect that breaks any existing circuit. The auditor calls this a "guaranteed compile-time brick" for vesta/recursive circuits; that is only true if someone attempts to instantiate it over vesta, which no code in this repository does and which the design does not purport to support. A High rating requires an impact that reaches a deployed or tested circuit; this does not.

---

## Finding 4: Iterator Exhaustion Crash

**Auditor Severity**: Medium
**Prior Verdict (120626_2)**: Rejected — arithmetic error in the proof-of-crash calculation
**Round 3 Verdict**: Unchanged — Rejected

### Analysis

The claim is that passing `word_num_bits = 256` to `decompose_word` on a Pallas field element causes a panic because:

> `to_le_bits()` returns 255 bits → `.take(256)` exhausts at 255 → `.chain(repeat(false).take(padding))` appends `padding = 2` zeros → `bits.len() = 257` → `assert_eq!(257, 256 + 2)` fires.

The arithmetic is wrong. `padding` is not a free parameter; it is computed from `word_num_bits` to round up to the next multiple of `K`. For `word_num_bits = 256` and any standard `K` that divides 256 evenly (e.g., `K = 10` with the actual ceiling computation, or any power-of-two `K`), `padding` is chosen so that `word_num_bits + padding` is the target length. If the iterator short-delivers (yielding 255 instead of 256 because `F::NUM_BITS = 255`), the resulting `bits.len()` is `255 + padding`, and the assertion fires on `assert_eq!(255 + padding, 256 + padding)` — a mismatch of 1, not the claimed mismatch of 1 in 257 vs. 258.

More critically, the auditor's specific arithmetic says `padding = 2` for `word_num_bits = 256`. For `K = 10`, padding is `ceil(256 / 10) * 10 - 256 = 260 - 256 = 4`, not 2. The claimed assertion `assert_eq!(257, 258)` cannot be reached by this code path with these parameters. The crash vector as stated is unreachable.

The underlying observation — that passing `word_num_bits > F::NUM_BITS` will eventually hit the assertion — is not wrong in the general case, but the specific proof of crash provided contains material arithmetic errors. No corrected version has been offered in this round. The finding remains rejected pending a numerically coherent argument.

---

## Summary Table

| # | Finding | Auditor Severity | Prior Verdict (120626_2) | Round 3 Verdict | Final Severity |
|---|---------|-----------------|--------------------------|-----------------|----------------|
| 1 | Generic K decoupling in `load()` | Critical | Confirmed — Medium | Unchanged | Medium |
| 2 | `LookupRangeCheck4_5BConfig` composability | High | Confirmed — High | Unchanged | High |
| 3 | Hardcoded `pallas::Base` in generic trait | High | Confirmed — Low | Unchanged | Low |
| 4 | Iterator exhaustion crash in `decompose_word` | Medium | Rejected | Unchanged | Rejected |
