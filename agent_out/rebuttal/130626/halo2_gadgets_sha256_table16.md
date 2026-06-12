# Rebuttal: halo2_gadgets_sha256_table16 (Round 3)

**Report**: agent_out/reports/130626/halo2_gadgets_sha256_table16.md
**Date**: 2026-06-13

> **Note on this submission**: This report is a verbatim copy of the equivalent report submitted on 2026-06-12. Every finding — its description, its exploit, and its severity — is word-for-word identical to the Round 2 submission. All five findings were fully confirmed in the previous round. That verdict is unchanged. The note regarding our prior Round 1 position on w_lo/w_hi (Finding 3) stands: we accepted the correction at Round 2 and that acceptance remains in force.

---

## Executive Summary

All five findings are confirmed. Two are Critical, two are High, one is Medium. The SHA-256 table16 chip is gated behind an unstable/experimental feature flag, which limits immediate exposure but does not reduce the severity of the underlying soundness gaps. No new analysis is required or warranted for a verbatim re-submission. The auditor has offered no additional argument, no new evidence, and no correction to anything reviewed in Round 2. The Round 2 verdict controls.

---

## Finding 1: Unconstrained Carries in Compression Gates

**Auditor Severity**: Critical
**Prior Verdict (120626_2)**: Confirmed
**Round 3 Verdict**: Unchanged — Confirmed
**Revised Severity**: Critical (module gated by unstable feature flag)

### Analysis

The code is unchanged from the version examined at Round 2. The three compression addition gates — `s_h_prime`, `s_a_new`, and `s_e_new` — in `compression_gates.rs` each carry the same structural omission. For `s_h_prime` (line 364):

```rust
let check = sum - (h_prime_carry * F::from(1 << 32)) - h_prime;
```

`h_prime_carry` is read from column `a_9` (extras[5]) at `Rotation::next()` relative to the gate row. It appears in no other constraint in the gate. There is no `Gate::range_check`, boolean constraint, or lookup applied to it. The same is true for `a_new_carry` (line 388) and `e_new_carry` (line 410).

The message schedule `s_word` gate (`schedule_gates.rs`) applies an explicit carry range check:

```rust
let carry_check = Gate::range_check(carry, 0, 3);
```

No such check exists in any of the three compression gates. The gate polynomial is satisfied for any carry value, so a prover sets `h_prime_carry = (sum - target) * (2^32)^{-1} mod p` and assigns `h_prime = target` to force an arbitrary 32-bit output. This fully decouples the stated H' from the actual arithmetic.

The re-submitted report adds nothing to what was examined in Round 2. The finding is confirmed.

### Conclusion

The carry range check present in the message schedule is absent from all three compression addition gates. Confirmed Critical. Verdict unchanged.

---

## Finding 2: Message Schedule Generation Row Disconnected from Decomposition Row

**Auditor Severity**: Critical
**Prior Verdict (120626_2)**: Confirmed
**Round 3 Verdict**: Unchanged — Confirmed
**Revised Severity**: Critical (module gated by unstable feature flag)

### Analysis

The code is unchanged. For each new schedule word W_i (i = 16..63), the implementation writes to two separate rows of column `a_5` (message_schedule) with no copy constraint between them.

**Generation row.** In `subregion2.rs` lines 254–258, the computed word is assigned to `a_5, get_word_row(new_word_idx - 16) + 1`. The `s_word` selector is enabled here and validates the word arithmetically.

**Decomposition row.** Immediately after (line 266), `assign_word_and_halves` is called. Inside `schedule_util.rs` lines 162–178, this assigns the full 32-bit word to `self.message_schedule, get_word_row(word_idx)` and the halves `w_lo`, `w_hi` to `a_3`, `a_4` at the same row. The `s_decompose_0` gate here checks only that `w_lo + w_hi * 2^16 = word`; it says nothing about the word's value.

These two rows are always distinct. For W_16 the generation row is row 1 and the decomposition row is row 103; the gap is at least 40 rows for every word in the range. There is no `constrain_equal`, `copy_advice`, or any other linking constraint anywhere in the path.

All downstream computation — `w_halves[i]` passed to compression, and the recursive schedule generation reading `w_halves[new_word_idx - 7]` — consumes the decomposition row. The `s_word`-validated generation row cell is never read again.

A prover assigns the correct computation at the generation row to satisfy `s_word`, and independently assigns arbitrary values at the decomposition row. All 48 words W_16..W_63 can be forged, yielding 48 × 32 = 1536 bits of unconstrained schedule entropy entering compression.

The re-submitted report adds nothing to what was examined in Round 2. The finding is confirmed.

### Conclusion

No constraint links the `s_word`-validated generation cell to the decomposition cell consumed by downstream gates. Confirmed Critical. Verdict unchanged.

---

## Finding 3: w_lo/w_hi Shift Cancels Exactly; Carry Range Check Provides No Protection

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed — prior Round 1 position retracted
**Round 3 Verdict**: Unchanged — Confirmed. Our Round 1 retraction stands.
**Revised Severity**: High (module gated by unstable feature flag)

### Analysis

The code is unchanged. The `s_word` gate evaluates halves linearly over the prime field:

```
word_check = lo + hi * 2^16 - carry * 2^32 - word = 0
```

The `s_decompose_0` gate enforces only the same linear combination on the individual word's halves. No gate individually range-constrains `w_lo` or `w_hi` to their nominal 16-bit windows.

If a prover shifts `w_lo` by `+X * 2^16` and compensates `w_hi` by `-X`:

- `s_decompose_0`: `(w_lo + X * 2^16) + (w_hi - X) * 2^16 = w_lo + w_hi * 2^16` — unchanged, gate passes.
- In `s_word` one level up, the contributions of `w_minus_16_lo` and `w_minus_16_hi` to the total sum also cancel: `lo_sum` increases by `X * 2^16` and `hi_sum * 2^16` decreases by `X * 2^16`. The total is invariant. The carry is unaffected; the [0,3] range check on the carry provides zero protection.

The shift is invisible to every gate in the circuit. This was the conclusion reached at Round 2. The re-submitted report adds nothing to what was already examined. Our Round 1 defense — that the carry range check would catch the discrepancy in most cases — was wrong, and we retracted it at Round 2. That retraction stands.

### Conclusion

The auditor's cancellation argument is algebraically correct. Confirmed High. The Round 1 retraction stands and is not revisited. Verdict unchanged.

---

## Finding 4: Spread Halves via SpreadVar::without_lookup Are Unconstrained Field Elements

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed
**Round 3 Verdict**: Unchanged — Confirmed
**Revised Severity**: High (module gated by unstable feature flag)

### Analysis

The code is unchanged. `assign_word_halves` in `compression_util.rs` uses `SpreadVar::without_lookup` for both `spread_word_lo` and `spread_word_hi`. That function places both the dense and spread column values as free advice witnesses with no lookup entry and no gate constraining their relationship.

The decomposition gates `s_decompose_abcd` (`compression_gates.rs` lines 64–71) and `s_decompose_efgh` (lines 136–143) include a `spread_check` of the form:

```
spread_a + spread_b * 2^4 + ... - spread_word_lo - spread_word_hi * 2^32 = 0
```

This constrains only the linear combination `spread_word_lo + spread_word_hi * 2^32`. It does not individually constrain either half. The same shift attack from Finding 3 applies: substituting `spread_word_lo + X * 2^32` and `spread_word_hi - X` leaves the sum invariant and the gate satisfied.

The downstream sigma and choice/majority gates (`s_upper_sigma_0`, `s_upper_sigma_1`, `s_ch`, `s_maj`) combine spread halves at specific bit offsets — not as a single aggregated sum. A shift in the half-split therefore misattributes bits across rotation boundaries and distorts the XOR computations for sigma, majority, and choice. This is an independent attack surface from Findings 1–3.

The re-submitted report adds nothing to what was examined in Round 2. The finding is confirmed.

### Conclusion

The absence of individual range constraints on spread halves via `SpreadVar::without_lookup` is confirmed. Confirmed High. Verdict unchanged.

---

## Finding 5: Sequential update() Calls Trigger a Deterministic Panic

**Auditor Severity**: Medium
**Prior Verdict (120626_2)**: Confirmed
**Round 3 Verdict**: Unchanged — Confirmed
**Revised Severity**: Medium

### Analysis

The code is unchanged. In `sha256.rs`, `Sha256::update()` handles a newly-full block by calling `self.chip.compress()` directly without a preceding `self.chip.initialization()`. The `compress()` return value sets `pieces` to `None` in both `RoundWordA::new_dense` and `RoundWordE::new_dense`.

When a second `update()` call delivers sufficient data to fill another block, `compress()` is invoked on this pieces-less state. Inside `compress()` -> `assign_round()`, the first round executes:

```rust
let sigma_1 = self.assign_upper_sigma_1(region, round_idx, e.pieces.clone().unwrap())?;
```

`e.pieces` is `None`; `.unwrap()` panics deterministically. Any caller streaming input in chunks where the combined prefix is a multiple of 512 bits will trigger this. The `Sha256::digest()` convenience wrapper avoids it by construction; only the multi-call streaming API is affected.

The re-submitted report adds nothing to what was examined in Round 2. The finding is confirmed.

### Conclusion

The panic is deterministic and directly traceable through the code path. Confirmed Medium. Verdict unchanged.

---

## Summary Table

| # | Finding | Prior Verdict | Round 3 Verdict | Severity |
|---|---------|--------------|-----------------|----------|
| 1 | Unconstrained carries (`h_prime_carry`, `a_new_carry`, `e_new_carry`) in compression gates | Confirmed | Unchanged — Confirmed | Critical |
| 2 | Message schedule generation row disconnected from decomposition row; no `constrain_equal` | Confirmed | Unchanged — Confirmed | Critical |
| 3 | w_lo/w_hi shift cancels exactly; carry range check provides zero protection; prior Round 1 defense was wrong | Confirmed (Round 1 retracted) | Unchanged — Confirmed, retraction stands | High |
| 4 | Spread halves unconstrained via `SpreadVar::without_lookup` | Confirmed | Unchanged — Confirmed | High |
| 5 | Sequential `update()` calls panic on `.unwrap()` of `None` pieces | Confirmed | Unchanged — Confirmed | Medium |

> This report is a verbatim re-submission. No new findings are raised, no prior finding is modified or withdrawn, and no new argument is offered. All Round 2 verdicts stand without further analysis required.
