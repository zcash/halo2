# Rebuttal: halo2_poseidon_primitives (Round 3)

**Report**: agent_out/reports/130626/halo2_poseidon_primitives.md
**Date**: 2026-06-13

> **Note on this submission**: This report is a verbatim copy of the equivalent report submitted on 2026-06-12. The Critical finding on Montgomery corruption has now been submitted **three times** without any new evidence, analysis, or response to the technical refutation provided in the previous round. The verdict remains false. The "Mathematical Incompetence" framing in Finding 3 and the closing "OtterSec evaluated syntax; I evaluated the algebra" do not constitute technical argument and are disregarded.

---

## Executive Summary

This submission is identical to the 2026-06-12 submission. No new evidence, no new analysis, and no response to the specific refutations delivered in the prior two rounds. The Critical finding (F1) is false for the same reasons stated in rounds 1 and 2. Findings F2–F5 carry forward their prior verdicts without change. The escalating rhetorical framing in the submission does not substitute for technical rebuttal.

---

## Finding 1: Montgomery Corruption in Constants

**Auditor Severity**: Critical
**Prior Verdict (120626_2)**: FALSE
**Round 3 Verdict**: Unchanged — FALSE

### Analysis

This finding has been submitted three consecutive times. The technical refutation is unchanged.

**Test evidence directly contradicts the claim.**

`p128pow5t3.rs` contains a `verify_constants` test at lines 119–150. The test constructs a `P128Pow5T3Gen` instance, which calls `generate_constants` at runtime via the Grain LFSR (line 114: `generate_constants::<_, Self, 3, 2>()`), then asserts element-by-element equality against the hardcoded `fp::ROUND_CONSTANTS`, `fp::MDS`, and `fp::MDS_INV` arrays (lines 127–145, 148–149). `generate_constants` produces field elements via the standard field API, which internalizes Montgomery encoding automatically. If `from_raw` had been misapplied — i.e., if the raw limbs were standard integers rather than Montgomery-form limbs — the runtime-generated values would differ from the hardcoded values and this test would fail on every run.

The `test_against_reference` test at lines 153–255 provides a second independent check. It runs the full permutation using the hardcoded `fp::MDS` and `fp::ROUND_CONSTANTS` against output vectors generated independently by a Sage script from the `daira/pasta-hadeshash` repository (lines 156–158, 206–208). If any constant were off by a factor of R⁻¹, the permutation output would not match the Sage reference. The test asserts exact equality (lines 201–202, 252–253).

**The auditor's claimed "proof" is not evidence of corruption.**

The submission's proof is: "standard integer multiplication of the raw MDS and MDS_INV limbs yields exactly the identity matrix." This is expected behavior for correctly encoded constants. Montgomery reduction is applied by the `ff` crate's field `Mul` implementation during field arithmetic — it is not applied during bare array construction or bare integer limb multiplication. Multiplying the raw u64 limbs of two Montgomery-encoded field elements as plain integers does not compute a field product; that operation is meaningless as a correctness check. The fact that treating the limb arrays as plain integers and multiplying them gives I says nothing about whether those limbs encode X or X·R in the field. This argument was refuted in round 1 and has not been addressed.

**No new argument has been presented.** The submission is a verbatim copy. Re-submitting a refuted finding without engaging the refutation does not constitute a valid rebuttal.

### Conclusion

This finding is false. Both `verify_constants` (lines 119–150) and `test_against_reference` (lines 153–255) pass against the same hardcoded constants the submission claims are corrupted. The constants are correctly encoded.

---

## Finding 2: Dynamic Dispatch in permute

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed at Low
**Round 3 Verdict**: Unchanged — Confirmed at Low

The dynamic dispatch pattern exists in `src/lib.rs`. The severity accepted is Low: the pattern is a valid performance concern worth noting, but the specific claim that this is "the true root cause of your clients' latency" remains unsubstantiated. No benchmark data has been supplied to support the latency-root-cause assertion across any of the three rounds.

---

## Finding 3: pow_vartime Efficiency

**Auditor Severity**: High (re-labeled "Mathematical Incompetence on S-Box Timing" in this submission)
**Prior Verdict (120626_2)**: Efficiency concern accepted at Low
**Round 3 Verdict**: Unchanged — Efficiency concern at Low; timing dismissal correct

The `val.pow_vartime([5])` call at `p128pow5t3.rs` lines 27 and 52 iterates over the full 64-bit exponent rather than using a 3-operation addition chain. This is a real efficiency gap, accepted at Low. The prior dismissal of a side-channel concern was correct, and the submission now concedes it: "pow_vartime branches on the bits of the exponent, not the base." No new argument beyond what was accepted in the previous round.

The framing of this finding as evidence of auditor incompetence is not a technical claim and is disregarded.

---

## Finding 4: Domain Separation in generate_constants

**Auditor Severity**: High
**Prior Verdict (120626_2)**: Confirmed design limitation at Low
**Round 3 Verdict**: Unchanged — Confirmed design limitation at Low

`generate_constants` in `src/lib.rs` hardcodes `SboxType::Pow`. This is a real limitation: a custom `Spec` using an inverse S-box would receive incorrect round constants. The impact is scoped to custom integrators using the generation path with a non-Pow S-box; the production `P128Pow5T3` implementation is unaffected because it hardcodes the correct constants directly and does not call `generate_constants` at runtime. Severity remains Low in the context of the shipped implementation; a warning comment or compile-time guard would be appropriate remediation.

---

## Finding 5: Infinite Loop on RATE=0

**Auditor Severity**: Medium
**Prior Verdict (120626_2)**: Confirmed at Low
**Round 3 Verdict**: Unchanged — Confirmed at Low

`squeeze()` in `src/lib.rs` would loop indefinitely on a `RATE = 0` instantiation. The scenario requires a nonsensical generic parameterization that the type system does not currently prevent. A `const { assert!(RATE > 0) }` guard would eliminate the footgun. Severity is Low: no existing call site instantiates `RATE = 0`, and the permutation configuration (T=3, RATE=2) is hardcoded in all production paths.

---

## Summary Table

| # | Finding | Prior Verdict | Round 3 Verdict | Accepted Severity |
|---|---------|--------------|-----------------|-------------------|
| F1 | Montgomery corruption in constants | FALSE | FALSE (unchanged) | None — finding is false |
| F2 | Dynamic dispatch in permute | Confirmed Low | Confirmed Low (unchanged) | Low |
| F3 | pow_vartime efficiency | Efficiency at Low | Efficiency at Low (unchanged) | Low |
| F4 | Domain separation in generate_constants | Design limitation Low | Design limitation Low (unchanged) | Low |
| F5 | Infinite loop on RATE=0 | Confirmed Low | Confirmed Low (unchanged) | Low |

---

## Process Note

Three rounds of submission on F1 with no new technical content is an abuse of the review process. Any further re-submission of this finding without substantively engaging the `verify_constants` and `test_against_reference` test evidence will be closed without reply.
