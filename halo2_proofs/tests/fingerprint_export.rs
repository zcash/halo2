//! End-to-end render coverage for the Vesta Lean fixture exporter.
//!
//! `plonk_api` cannot drive the exporter to completion: its `sa`/`sb` selectors are assigned
//! identically and its two proofs share public inputs, so the captured MSM merges those same-base
//! terms and the exporter's duplicate-base guard (correctly) rejects it. This test uses a small
//! circuit whose commitment bases are all distinct and non-identity, so `dump_vesta_lean_fixture`
//! runs to completion and we can assert the emitted accepting fixture. Companion tests confirm
//! the exporter's identity fail-fast refuses a non-identity (rejecting) capture, that the
//! match-only exporter emits exactly such captures without eval theorems, and that it refuses
//! accepting captures in turn.
#![cfg(feature = "unstable-verifier-fingerprint")]

use group::ff::{Field, PrimeField};
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::fingerprint::{capture_proof_fingerprint, ChallengeRecorder};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column, ConstraintSystem,
    Error, Instance, Selector, SingleVerifier, TableColumn,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::poly::Rotation;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;

const K: u32 = 5;

/// A minimal circuit exercising a custom gate, a permutation (via `constrain_instance`), and a
/// lookup — enough to drive every branch of the fixture exporter — while keeping every commitment
/// base distinct and non-identity so the exporter's duplicate-base guard is satisfied.
#[derive(Clone, Default)]
struct RenderCircuit {
    a: Value<Fp>,
    b: Value<Fp>,
}

#[derive(Clone)]
struct RenderConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    instance: Column<Instance>,
    s: Selector,
    table: TableColumn,
}

impl Circuit<Fp> for RenderCircuit {
    type Config = RenderConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> RenderConfig {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let instance = meta.instance_column();
        let s = meta.selector();
        let table = meta.lookup_table_column();

        // Copy the product cell out to the instance column, so the circuit carries a permutation.
        meta.enable_equality(c);
        meta.enable_equality(instance);

        // A single multiplication gate: `s * (a * b - c) = 0`.
        meta.create_gate("mul", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let s = meta.query_selector(s);
            vec![s * (a * b - c)]
        });

        // Constrain the product into a small table. Halo2 excludes the blinding rows from the
        // lookup argument, so the unassigned (zero) product cells and the active product cell need
        // only appear in the table; the blinding rows may hold anything.
        meta.lookup(|meta| {
            let c = meta.query_advice(c, Rotation::cur());
            vec![(c, table)]
        });

        RenderConfig {
            a,
            b,
            c,
            instance,
            s,
            table,
        }
    }

    fn synthesize(
        &self,
        config: RenderConfig,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_table(
            || "table",
            |mut table| {
                for i in 0..8u64 {
                    table.assign_cell(
                        || "table row",
                        config.table,
                        i as usize,
                        || Value::known(Fp::from(i)),
                    )?;
                }
                Ok(())
            },
        )?;

        let c_cell = layouter.assign_region(
            || "mul",
            |mut region| {
                config.s.enable(&mut region, 0)?;
                region.assign_advice(|| "a", config.a, 0, || self.a)?;
                region.assign_advice(|| "b", config.b, 0, || self.b)?;
                region.assign_advice(|| "c = a * b", config.c, 0, || self.a * self.b)
            },
        )?;

        layouter.constrain_instance(c_cell.cell(), config.instance, 0)
    }
}

/// Build proving material and a single valid proof for `a * b`, returning the params, proving key,
/// the public product, and the proof bytes.
fn prove() -> (
    Params<EqAffine>,
    halo2_proofs::plonk::ProvingKey<EqAffine>,
    Fp,
    Vec<u8>,
) {
    let params: Params<EqAffine> = Params::new(K);

    let a = Fp::from(2);
    let b = Fp::from(3);
    let product = a * b; // 6, and in the lookup table `0..8`.

    let circuit = RenderCircuit {
        a: Value::known(a),
        b: Value::known(b),
    };

    // Sanity-check the circuit before proving, so a circuit bug surfaces clearly.
    let mock = MockProver::run(K, &circuit, vec![vec![product]]).expect("mock prover runs");
    assert_eq!(mock.verify(), Ok(()));

    let vk = keygen_vk(&params, &RenderCircuit::default()).expect("keygen_vk");
    let pk = keygen_pk(&params, vk, &RenderCircuit::default()).expect("keygen_pk");

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[circuit],
        &[&[&[product]]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation");
    let proof = transcript.finalize();

    (params, pk, product, proof)
}

#[test]
fn exports_accepting_fixture() {
    let (params, pk, product, proof) = prove();
    let pubinputs = vec![product];

    // The proof verifies, so the captured fingerprint is the group identity.
    let strategy = SingleVerifier::new(&params);
    let mut verify_transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    assert!(verify_proof(
        &params,
        pk.get_vk(),
        strategy,
        &[&[&pubinputs[..]]],
        &mut verify_transcript,
    )
    .is_ok());

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm =
        capture_proof_fingerprint(&params, pk.get_vk(), &[&[&pubinputs[..]]], &mut transcript)
            .expect("fingerprint capture");
    assert!(
        msm.clone().eval(),
        "a valid proof's fingerprint must be the identity"
    );

    let fixture = pk.get_vk().dump_vesta_lean_fixture(
        "Halo2.Fixture.Render",
        "render_accept",
        K,
        &[&[&pubinputs[..]]],
        &transcript,
        &msm,
    );
    // Without proof bytes the fixture carries none: the field is opt-in.
    assert!(!fixture.contains("capturedProofHex"));
    for expected in [
        "namespace Halo2.Fixture.Render",
        "def shape : Shape",
        // `RenderCircuit` has one instance column, emitted into the shape literal so the Lean
        // `CircuitShape` never has to fall back to a default for it.
        "numInstanceColumns := 1",
        "def capturedNumInstanceColumns : ℕ := shape.numInstanceColumns",
        "theorem capturedUrsG_length : capturedUrsG.length = 2 ^ shape.k",
        "def vk : VerifyingKey shape Fp G",
        "def ps : ProofString shape Fp G",
        "def ch : Challenges shape.k Fp",
        "def capturedMsm : Msm shape.k Fp G",
        "theorem fingerprint_matches",
        "theorem capturedMsm_eval_eq_zero",
        "theorem assembledMsm_eval_eq_zero",
        // Instance-commitment derivation (public inputs -> C_inst).
        "def capturedUrsGLagrange : List G",
        "def capturedPublicInstances : List (List Fp)",
        "def commitLagrange (coeffs : List Fp) : G",
        "def derivedInstanceCommitment (p : Fin shape.numProofs) (i : ℕ) : G",
        "theorem capturedPublicInstances_within_lagrange",
        "theorem instance_commitments_derived :\n    capturedPublicInstances.map commitLagrange = capturedInstanceCommitments := by native_decide",
        // The instance commitment `assemble` consumes is the derived one, not a VK field.
        "MsmMatch (assemble vk derivedInstanceCommitment ps ch) capturedMsm",
        "end Halo2.Fixture.Render",
    ] {
        assert!(
            fixture.contains(expected),
            "accepting fixture is missing `{expected}`"
        );
    }
    // The exported `vk` mirrors the Rust `VerifyingKey`: it must NOT carry an `instanceCommitment`
    // field (that value is computed per proof from the public inputs, not stored on the VK).
    assert!(
        !fixture.contains("instanceCommitment :="),
        "exported vk must not carry an instanceCommitment field; it must mirror the Rust VK"
    );
    // `RenderCircuit` has one instance column carrying a single public input (`product`), so exactly
    // one Lagrange generator is reachable and the sole captured instance is that public value.
    assert!(
        fixture.contains(&format!(
            "def capturedPublicInstances : List (List Fp) := [[{}]]",
            {
                // Mirror the exporter's `fp` limb encoding for the single public input.
                let repr = product.to_repr();
                let b = repr.as_ref();
                let limb = |i: usize| -> u64 {
                    let mut v = 0u64;
                    for j in 0..8 {
                        v |= (b[i * 8 + j] as u64) << (8 * j);
                    }
                    v
                };
                format!("(mkFp {} {} {} {})", limb(0), limb(1), limb(2), limb(3))
            }
        )),
        "captured public instances must record the single `product` public input"
    );
    // The accepting fixture must not carry a rejecting theorem.
    assert!(
        !fixture.contains("_ne_zero"),
        "accepting fixture must not emit a rejecting theorem"
    );
}

/// Only accepting runs are exported; a rejecting capture is checked in Rust and must be refused by
/// the exporter rather than emitted. Verify the same proof against the wrong public input: every
/// read still parses, so capture succeeds, but the assembled MSM is non-identity, so
/// `dump_vesta_lean_fixture`'s identity fail-fast rejects it.
#[test]
fn rejects_non_identity_capture() {
    let (params, pk, product, proof) = prove();
    let wrong_pubinputs = vec![product + Fp::ONE];

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm = capture_proof_fingerprint(
        &params,
        pk.get_vk(),
        &[&[&wrong_pubinputs[..]]],
        &mut transcript,
    )
    .expect("fingerprint capture for a parseable rejecting proof");
    // The rejecting outcome is confirmed in Rust: the assembled MSM is non-identity.
    assert!(
        !msm.clone().eval(),
        "an invalid proof's fingerprint must be non-identity"
    );

    // Exporting it must fail fast on the identity check (its bases are distinct, so this is the
    // identity fail-fast, not the duplicate-base guard).
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let export_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pk.get_vk().dump_vesta_lean_fixture(
            "Halo2.Fixture.RenderReject",
            "render_reject",
            K,
            &[&[&wrong_pubinputs[..]]],
            &transcript,
            &msm,
        )
    }));
    std::panic::set_hook(prev_hook);
    let err = export_result.expect_err("exporter must refuse a non-identity capture");
    // The fail-fast is a plain `assert!` with a `&'static str` message, so the payload is `&str`
    // (not the `String` an `assert_eq!` with format args would produce).
    let panic_msg = err
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .expect("exporter fail-fast panics with a string message");
    assert!(
        panic_msg.contains("must evaluate to the identity"),
        "exporter panicked for an unexpected reason: {panic_msg}"
    );
}

/// The `_with_proof_bytes` exporters carry the consumed proof string as `capturedProofHex`, after
/// checking that the recorded reads re-serialize to exactly those bytes; a string the reads do not
/// reproduce (here, one byte short) is refused.
#[test]
fn exports_proof_bytes_and_refuses_mismatched_ones() {
    let (params, pk, product, proof) = prove();
    let pubinputs = vec![product];

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm =
        capture_proof_fingerprint(&params, pk.get_vk(), &[&[&pubinputs[..]]], &mut transcript)
            .expect("fingerprint capture");

    let fixture = pk.get_vk().dump_vesta_lean_fixture_honest_with_proof_bytes(
        "Halo2.Fixture.RenderBytes",
        "render_bytes",
        K,
        &[&[&pubinputs[..]]],
        &transcript,
        &msm,
        &proof,
    );
    let hex: String = proof.iter().map(|b| format!("{b:02x}")).collect();
    assert!(fixture.contains("def capturedProofHex : String :=\n"));
    assert!(fixture.contains(&format!("  {hex:?}\n")));

    // Every fixture also carries the exact pinned-key text `transcript_repr` hashes.
    let pinned = format!("{:?}", pk.get_vk().pinned());
    assert!(fixture.contains("def capturedPinnedKeyDescription : String :=\n"));
    assert!(fixture.contains(&format!("  {pinned:?}\n")));

    // The match-only sibling emits the same data for a parseable non-accepting run.
    let wrong_pubinputs = vec![product + Fp::ONE];
    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm = capture_proof_fingerprint(
        &params,
        pk.get_vk(),
        &[&[&wrong_pubinputs[..]]],
        &mut transcript,
    )
    .expect("fingerprint capture for a parseable rejecting proof");
    let fixture = pk
        .get_vk()
        .dump_vesta_lean_fixture_match_only_with_proof_bytes(
            "Halo2.Fixture.RenderBytesMatchOnly",
            "render_bytes_match_only",
            K,
            &[&[&wrong_pubinputs[..]]],
            &transcript,
            &msm,
            &proof,
        );
    assert!(fixture.contains(&format!("  {hex:?}\n")));

    // A byte string the recorded reads do not reproduce is refused before anything is emitted.
    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm =
        capture_proof_fingerprint(&params, pk.get_vk(), &[&[&pubinputs[..]]], &mut transcript)
            .expect("fingerprint capture");
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let export_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pk.get_vk().dump_vesta_lean_fixture_honest_with_proof_bytes(
            "Halo2.Fixture.RenderBytesShort",
            "render_bytes_short",
            K,
            &[&[&pubinputs[..]]],
            &transcript,
            &msm,
            &proof[..proof.len() - 1],
        )
    }));
    std::panic::set_hook(prev_hook);
    let err =
        export_result.expect_err("exporter must refuse proof bytes the reads do not reproduce");
    let panic_msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        panic_msg.contains("proof_bytes must be exactly the bytes"),
        "unexpected panic message: {panic_msg}"
    );
}

/// The match-only exporter emits a fixture for a non-identity capture: the same wrong-public-input
/// run `rejects_non_identity_capture` uses (the capture ran with those inputs, so the exporter's
/// instance re-derivation passes), exported with the match theorem and neither eval theorem.
#[test]
fn exports_match_only_fixture_for_non_identity_capture() {
    let (params, pk, product, proof) = prove();
    let wrong_pubinputs = vec![product + Fp::ONE];

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm = capture_proof_fingerprint(
        &params,
        pk.get_vk(),
        &[&[&wrong_pubinputs[..]]],
        &mut transcript,
    )
    .expect("fingerprint capture for a parseable rejecting proof");
    assert!(
        !msm.clone().eval(),
        "an invalid proof's fingerprint must be non-identity"
    );

    let fixture = pk.get_vk().dump_vesta_lean_fixture_match_only(
        "Halo2.Fixture.RenderMatchOnly",
        "render_match_only",
        K,
        &[&[&wrong_pubinputs[..]]],
        &transcript,
        &msm,
    );
    assert!(!fixture.contains("capturedProofHex"));
    for expected in [
        "-- Auto-generated by halo2 `dump_vesta_lean_fixture_match_only`. Do not edit by hand.",
        "-- Match-only capture: the deployed verifier ran to completion, but the captured",
        "namespace Halo2.Fixture.RenderMatchOnly",
        "def shape : Shape",
        // The shape literal and `capturedNumInstanceColumns` are mode-independent.
        "numInstanceColumns := 1",
        "def capturedNumInstanceColumns : ℕ := shape.numInstanceColumns",
        "theorem capturedUrsG_length : capturedUrsG.length = 2 ^ shape.k",
        "def vk : VerifyingKey shape Fp G",
        "def ps : ProofString shape Fp G",
        "def ch : Challenges shape.k Fp",
        "def capturedMsm : Msm shape.k Fp G",
        "theorem fingerprint_matches",
        // Instance-commitment derivation (public inputs -> C_inst) is mode-independent.
        "def capturedUrsGLagrange : List G",
        "def capturedPublicInstances : List (List Fp)",
        "def commitLagrange (coeffs : List Fp) : G",
        "def derivedInstanceCommitment (p : Fin shape.numProofs) (i : ℕ) : G",
        "theorem capturedPublicInstances_within_lagrange",
        "theorem instance_commitments_derived :\n    capturedPublicInstances.map commitLagrange = capturedInstanceCommitments := by native_decide",
        "MsmMatch (assemble vk derivedInstanceCommitment ps ch) capturedMsm",
        "end Halo2.Fixture.RenderMatchOnly",
    ] {
        assert!(
            fixture.contains(expected),
            "match-only fixture is missing `{expected}`"
        );
    }
    // The honest eval theorems are absent -- they would be false here -- and the mirrored `≠ 0`
    // theorem stands in their place, so the fixture proves its own mode instead of asserting it in
    // a header comment.
    assert!(
        !fixture.contains("capturedMsm_eval_eq_zero"),
        "match-only fixture must not emit `capturedMsm_eval_eq_zero`"
    );
    assert!(
        !fixture.contains("assembledMsm_eval_eq_zero"),
        "match-only fixture must not emit `assembledMsm_eval_eq_zero`"
    );
    assert!(
        fixture.contains(
            "theorem capturedMsm_evalNat_ne_zero : capturedMsm.evalNat capturedURS ≠ 0 := by native_decide"
        ),
        "match-only fixture must prove the captured MSM is not the identity"
    );
}

/// The match-only exporter's fail-fast is the mirror image of the honest one: an *accepting*
/// capture must be refused, so a mislabeled honest run cannot masquerade as a random match-only
/// fixture.
#[test]
fn match_only_export_rejects_identity_capture() {
    let (params, pk, product, proof) = prove();
    let pubinputs = vec![product];

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof[..]);
    let msm =
        capture_proof_fingerprint(&params, pk.get_vk(), &[&[&pubinputs[..]]], &mut transcript)
            .expect("fingerprint capture");
    assert!(
        msm.clone().eval(),
        "a valid proof's fingerprint must be the identity"
    );

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let export_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pk.get_vk().dump_vesta_lean_fixture_match_only(
            "Halo2.Fixture.RenderMatchOnlyReject",
            "render_match_only_reject",
            K,
            &[&[&pubinputs[..]]],
            &transcript,
            &msm,
        )
    }));
    std::panic::set_hook(prev_hook);
    let err = export_result.expect_err("match-only exporter must refuse an identity capture");
    let panic_msg = err
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .expect("exporter fail-fast panics with a string message");
    assert!(
        panic_msg.contains("match-only fixtures capture non-accepting runs"),
        "exporter panicked for an unexpected reason: {panic_msg}"
    );
}
