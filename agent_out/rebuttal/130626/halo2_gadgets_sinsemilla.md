# Rebuttal: halo2_gadgets_sinsemilla (Round 3)

**Report**: agent_out/reports/130626/halo2_gadgets_sinsimella.md
**Date**: 2026-06-13

> **Note on this submission**: This report is a verbatim copy of the equivalent report submitted on 2026-06-12. No new analysis, evidence, or code citations have been added. Additionally, this submission continues to identify the responding party by firm name despite explicit instructions in prior rebuttal rounds not to do so. The use of our firm name in a formal dispute record is unprofessional and will not be engaged with on its merits. This note will appear on any further submission that repeats the behavior. All verdicts from Round 2 (120626_2) are unchanged.

---

## Executive Summary

Round 3 introduces no new findings and no new code citations. Every section is a verbatim repetition of the Round 2 submission. Because the substance is identical, all prior verdicts stand without modification:

- **Part I (Retractions)**: Accepted — unchanged.
- **Part II (leaf_pos reasoning challenge)**: Accepted — unchanged.
- **Part III (copy_advice panics)**: Partially accepted at **Low** severity — unchanged.
- **Part IV algebraic claim (q_sinsemilla4 / empty message)**: Rejected — unchanged.
- **Part IV integer underflow**: Accepted at **Low** severity — unchanged.

---

## Part I: Retractions

**Round 3 Verdict**: Unchanged — Accepted

The retractions of the domain-separator and MockProver dummy-assignment findings are correct and were accepted in Round 2. No change.

---

## Part II: leaf_pos Reasoning Challenge

**Round 3 Verdict**: Unchanged — Accepted

We acknowledged in Round 2 that our Round 1 security justification was incorrect. The `swap` `AssignedCell` is dropped; the caller has nothing to bind (see `cond_swap.rs:107`, `Ok((a_swapped, b_swapped))`). Security is grounded in Merkle collision resistance, not caller-side binding. This remains our position. This submission adds nothing beyond what was already conceded.

---

## Part III: copy_advice Panics (Re-Asserted)

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Partially accepted — Low
**Round 3 Verdict**: Unchanged — Accepted at Low

### Analysis

The call chain is real and was confirmed in Round 2. `witness_message_piece` assigns into `config.witness_pieces` (`chip.rs:326-333`). Inside `hash_piece`, the piece's cell value is consumed via `copy_advice` targeting `config.bits` (`hash_to_point.rs:389-394`). That `copy_advice` call requires equality to be enabled on the source column (`witness_pieces`). The `configure` method enables equality on the five columns in the `advices` array (`chip.rs:180-182`) but does not touch `witness_pieces` unless the caller passes the same column as one of the five.

The finding is therefore valid in principle. The severity remains **Low**, not High, for the reason stated in Round 2: in every production call site in this repository the caller either passes a column that is already equality-enabled, or passes a column that is one of the `advices` columns, which `configure` equality-enables unconditionally. The crash path requires a caller to supply a fresh, never-equality-enabled column as `witness_pieces` — an integration misuse not present in any production code path. An attacker who controls circuit configuration can produce arbitrary proofs regardless; the panic confers no additional power. Severity Low is the appropriate rating.

This submission repeats the same argument and code citation as Round 2 without addition. Nothing new warrants a revision.

---

## Part IV: Empty Messages — Algebraic Claim

**Auditor Severity**: Critical
**Prior Verdict (120626_2)**: Rejected
**Round 3 Verdict**: Unchanged — Rejected

### Analysis

The algebraic argument is a row-substitution error. The auditor claims that when `message.len() == 0` the dummy assignments at `offset` violate `q_sinsemilla4`. The code refutes this on two independent grounds.

**1. q_sinsemilla4 fires at the initialization row, not the dummy-assignment row.**

`public_q_initialization` enables `q_sinsemilla4` at the Q initialization row and returns an incremented `offset` to the caller (`hash_to_point.rs:149-173`). `hash_all_pieces` receives that offset and, when the message is empty and the loop is skipped entirely, assigns dummy values (`lambda_1 = y_a`, `lambda_2 = 0`, `x_p = 0`) at the terminal `offset` value (`hash_to_point.rs:259-283`). The Q initialization row and the dummy-assignment row are different rows. `q_sinsemilla4` is enabled only on the initialization row, where the real Q coordinates were assigned. It is never enabled on the dummy-assignment row.

**2. The substitution uses values from the wrong row.**

The auditor substitutes `lambda_1 = y_Q`, `lambda_2 = 0`, `x_p = 0` into the `q_sinsemilla4` constraint and derives a contradiction. Those are the dummy values assigned to the terminal row. The `q_sinsemilla4` constraint fires on the initialization row, where `x_a = x_Q` and the elliptic-curve variables are the actual first-round values satisfying `2 * y_Q - Y_A = 0` by construction. The claimed contradiction is the result of evaluating a gate at a row it does not fire on, using values that belong to a different row.

**3. Empty-message circuit behavior.**

When `message.len() == 0`, `hash_all_pieces` skips the loop entirely and writes terminal dummy values without enabling `q_sinsemilla1` or `q_sinsemilla2` on any row. No Sinsemilla gate fires at the dummy row. The `q_sinsemilla4` gate fires exactly once, at the initialization row, where the assigned values satisfy the constraint. The circuit is satisfiable for empty messages.

The finding that an empty message causes an algebraic unsatisfiability is incorrect. Verdict: Rejected.

---

## Part IV: Empty Messages — Integer Underflow

**Auditor Severity**: Critical (combined with algebraic claim above)
**Prior Verdict (120626_2)**: Accepted — Low
**Round 3 Verdict**: Unchanged — Accepted at Low

### Analysis

The underflow is real. At `hash_to_point.rs:408`:

```rust
for (idx, word) in words[0..(words.len() - 1)].iter().enumerate()
```

If `words.len() == 0`, the subtraction `0usize - 1` panics in debug mode and wraps to `usize::MAX` in release mode, producing an out-of-bounds slice that panics at runtime. A zero-word piece is constructable by a caller passing `num_words = 0` to `witness_message_piece`. The code does not guard against it.

Severity remains **Low**: the panic is in the prover only (liveness impact, no soundness or proof-forgery impact), requires the caller to pass a semantically invalid piece, and is trivially fixed with a single guard. Elevating this to Critical is not consistent with the absence of any soundness consequence.

---

## Summary Table

| Section | Finding | Prior Verdict | Round 3 Verdict | Severity |
|---|---|---|---|---|
| Part I | Domain separator retraction | Accepted | Unchanged — Accepted | N/A |
| Part I | MockProver dummy assignment retraction | Accepted | Unchanged — Accepted | N/A |
| Part II | leaf_pos reasoning challenge | Accepted | Unchanged — Accepted | Informational |
| Part III | copy_advice / witness_pieces permutation panic | Partially accepted | Unchanged — Accepted | Low |
| Part IV | q_sinsemilla4 algebraic violation (empty message) | Rejected | Unchanged — Rejected | N/A |
| Part IV | Integer underflow on zero-word piece | Accepted | Unchanged — Accepted | Low |
