# Rebuttal: halo2_proofs_circuits_dev (Round 3)

**Report**: agent_out/reports/130626/halo2_proofs_circuits_dev.md
**Date**: 2026-06-13

> **Note on this submission**: This report is a verbatim copy of the halo2_proofs_circuit_sys.md
> report submitted on 2026-06-12. Only the filename has changed. No new analysis, no new
> evidence, no new code citations, no responses to the Round 2 rebuttals. All five verdicts are
> unchanged from Round 2.

---

## Executive Summary

This submission is a rename, not a resubmission. Every finding is word-for-word identical to
the prior round. No attempt has been made to engage with the Round 2 rebuttals, address the
rejected adversary model for F1, contest the severity downgrades for F3, F4, or F5, or provide
new evidence for any finding. Accordingly, all verdicts carry forward without modification.

| # | Finding | Prior Verdict | Round 3 Verdict | Severity |
|---|---------|---------------|-----------------|----------|
| 1 | Silent Soundness Hole via Untracked Table Columns | Rejected — exploit framing requires developer error | Rejected | Informational |
| 2 | Missing RAII Guard — Backend Poisoning on Synthesis Error | Confirmed | Confirmed | High |
| 3 | No Constant Deduplication | Confirmed (DoS, not soundness) | Confirmed | Medium |
| 4 | VirtualCells Drops Lookup Cell Tracking | Confirmed (tooling gap) | Confirmed | Low |
| 5 | query_any Panic for Fixed at Non-Zero Rotation | Confirmed (documented behavior, API design) | Confirmed | Low |

---

## Finding 1: Silent Soundness Hole via Untracked Table Columns

**Auditor Severity**: Critical
**Prior Verdict**: Rejected — Informational
**Round 3 Verdict**: Rejected — Informational

### Analysis

No new argument has been offered. The Round 2 rebuttal stands in full.

The exploit framing remains wrong. An external adversary does not run synthesis. The scenario
requires a circuit developer to allocate a `TableColumn`, wire it into a lookup argument, and
then never call `assign_table`. The resulting defective prover produces an unsound proof of its
own making — not a proof that an external party forged past a legitimate verifier. Critical
severity requires adversarial reachability; this finding has none.

The underlying engineering gap — no post-synthesis cross-check that every column appearing in
`self.lookups[i].table_expressions` was assigned during synthesis — remains real. The
recommendation to add a defensive check or a prominent documentation warning stands. That
recommendation does not require upgrading the severity.

**No change. Informational.**

---

## Finding 2: Missing RAII Guard — Backend Poisoning on Synthesis Error

**Auditor Severity**: High
**Prior Verdict**: Confirmed at High
**Round 3 Verdict**: Confirmed at High

### Analysis

No new argument has been offered. The Round 2 confirmation stands in full.

Both `v1.rs` and `single_pass.rs` apply `?` directly after the assignment closure, bypassing
`exit_region()` on any `Err` return. The `Assignment` trait explicitly documents that
`enter_region` panics if already in a region. A single failing gadget therefore corrupts the
`ConstraintSystem` state for all subsequent regions. In test contexts where gadget errors are
expected to be recoverable, this turns a soft error into a hard panic. The fix — capture the
result, call `exit_region()` unconditionally, then propagate — is mechanical.

**No change. Confirmed at High.**

---

## Finding 3: No Constant Deduplication

**Auditor Severity**: High
**Prior Verdict**: Confirmed at Medium
**Round 3 Verdict**: Confirmed at Medium

### Analysis

No new argument has been offered. The Round 2 confirmation and downgrade stand in full.

The lack of deduplication in both `single_pass.rs` and `v1.rs` is real. High-repetition
circuits that use `assign_advice_from_constant` for common values will exhaust the row budget
and receive `NotEnoughColumnsForConstants`. This is a resource exhaustion issue at compile time,
not a soundness issue. A circuit that fails to fit does not produce an unsound proof; it fails
to produce one. The manual workaround via `constrain_equal` is underdocumented but functional.
High severity is not warranted for a compile-time DoS with a known workaround.

**No change. Confirmed at Medium.**

---

## Finding 4: VirtualCells Drops Lookup Cell Tracking

**Auditor Severity**: Gotcha
**Prior Verdict**: Confirmed at Low
**Round 3 Verdict**: Confirmed at Low

### Analysis

No new argument has been offered. The Round 2 confirmation stands in full.

`ConstraintSystem::lookup` constructs a `VirtualCells` context, queries columns to build the
lookup argument, then drops `cells` without saving `queried_cells` or `queried_selectors` into
the `lookup::Argument` struct — unlike `create_gate`, which saves both into the `Gate` struct.
The mathematical argument is correctly formed; columns are registered in `fixed_queries` and
embedded in `table_expressions`. The gap is purely in the tooling layer: dev-graph and routing
analysis tools that read `gate.queried_cells()` are blind to lookup column dependencies.

**No change. Confirmed at Low (Gotcha).**

---

## Finding 5: query_any Panic for Fixed at Non-Zero Rotation

**Auditor Severity**: Gotcha
**Prior Verdict**: Confirmed at Low
**Round 3 Verdict**: Confirmed at Low

### Analysis

No new argument has been offered. The Round 2 confirmation stands in full.

`VirtualCells::query_any` accepts `at: Rotation` uniformly for all column types but panics
when given `Any::Fixed` with a non-`Rotation::cur()` argument. The panic is documented in the
docstring. The problem is that the signature does not express the restriction — a generic gadget
parameterized over `Column<Any>` compiles and passes tests with Advice and Instance columns,
then panics at runtime when configured with Fixed. The correct fix is a type-level restriction
or a `Result`-returning API. This is an API design footgun, not a security vulnerability.

**No change. Confirmed at Low (Gotcha).**

---

## Summary Table

| # | Finding | Prior Verdict | Round 3 Verdict | Severity |
|---|---------|---------------|-----------------|----------|
| 1 | Silent Soundness Hole via Untracked Table Columns | Rejected — developer error, not adversarial input | Rejected | Informational |
| 2 | Missing RAII Guard — Backend Poisoning on Synthesis Error | Confirmed | Confirmed | High |
| 3 | No Constant Deduplication | Confirmed (downgraded from High) | Confirmed | Medium |
| 4 | VirtualCells Drops Lookup Cell Tracking | Confirmed | Confirmed | Low |
| 5 | query_any Panic for Fixed at Non-Zero Rotation | Confirmed | Confirmed | Low |
