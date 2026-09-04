//! Lean fixture export for the verifier-fingerprint match (dev/test tooling).
//!
//! [`VerifyingKey::dump_vesta_lean_fixture`] emits a self-contained Lean file in the requested namespace,
//! capturing one real proof: the verifying key (gates, query layouts, lookups, permutation
//! chunking, domain), the proof string, the Fiat-Shamir challenges, and the assembled MSM, plus the
//! `MsmMatch (assemble vk ps ch) capturedMsm := by native_decide` theorem.
//!
//! Scalar and base field elements are emitted as four little-endian `u64` limbs (`mkFp`/`mkFq`).
//! Commitments and the complete verifier URS are emitted as validated concrete Vesta points, shared
//! across the VK, proof string, transcript, and MSM.
//!
//! Trust boundary: the fixture attests that the assembled MSM matches — and, for accepting
//! captures, evaluates to the identity — for *these* captured commitments and challenges. The
//! emitted `vk` mirrors halo2's
//! `VerifyingKey` field-for-field — circuit-fixed data only — so it stays faithful to the pinned Rust
//! key; the instance commitment is *not* a VK field. Instead the fixture re-derives it from the
//! public inputs (`commit_lagrange` of the zero-padded instance columns, exposed as
//! `derivedInstanceCommitment`), feeds *that* to `assemble`, and checks it against the captured
//! commitments (`instance_commitments_derived`), so instance commitments no longer enter as opaque
//! points nor masquerade as VK data. It does not reproduce Halo2's Blake2b transcript or pinned-key
//! serialization; those remain trusted from the Rust capture. Given the proof byte string the
//! verifier consumed ([`VerifyingKey::dump_vesta_lean_fixture_honest_with_proof_bytes`]), it checks that
//! re-serializing the recorded proof reads reproduces those bytes exactly and carries them in the
//! fixture as `capturedProofHex`, so a consumer can check its own proof-string decoder against the
//! bytes the deployed verifier read. Halo2's `MSM` also merges same-base
//! terms and drops identity bases, where the Lean assembly deliberately does neither; such a capture
//! is rejected at export (see [`VerifyingKey::dump_vesta_lean_fixture`]) rather than emitted.
//!
//! Two exporters share one emission path. [`VerifyingKey::dump_vesta_lean_fixture`] is the
//! accepting exporter: it verifies the captured MSM is the group identity before emitting
//! anything, so every honest fixture proves `capturedMsm.evalNat capturedURS = 0` alongside the
//! match theorem. [`VerifyingKey::dump_vesta_lean_fixture_match_only`] is its sibling for
//! non-accepting captures, of which random-input captures are the motivating case — the mode is
//! selected by the captured MSM, so non-acceptance is what the export checks and randomness is the
//! caller's business. The deployed verifier ran to completion, but the captured MSM is
//! deliberately *not* the identity — verified before emitting anything — so the fixture emits the
//! match theorem alongside `capturedMsm.evalNat capturedURS ≠ 0`, the mirror of the honest eval
//! theorem. Either way the fixture proves its own mode rather than asserting it in a header
//! comment. Rejecting runs of *honest* proofs remain Rust-only
//! checks (they are non-identity / rejected by the deployed verifier), never exported to Lean:
//! exporting a rejecting run as its own fixture was considered and dropped because it doubled the
//! exporter surface for a cross-check that a trivially-accepting Lean `assemble` would already
//! fail on the accepting fixtures.

use ff::{Field, FromUniformBytes, PrimeField};
use group::Curve;
use std::collections::{BTreeSet, HashMap};
use std::io::Read;

use super::{ChallengeRecorder, TranscriptEvent};
use crate::arithmetic::{Coordinates, CurveAffine};
use crate::pasta::{EqAffine, Fp, Fq};
use crate::poly::commitment::{Blind, MSM};
use crate::transcript::{Blake2bWrite, Challenge255, TranscriptWrite};

use super::super::circuit::{Any, Expression};
use super::super::VerifyingKey;

/// A field element as a Lean constructor call with four little-endian `u64` limbs.
fn field<F: PrimeField>(constructor: &str, x: F) -> String {
    let repr = x.to_repr();
    let b = repr.as_ref();
    debug_assert!(
        b.len() <= 32,
        "field repr is wider than the four u64 limbs emitted here; extra bytes would be silently dropped"
    );
    let limb = |i: usize| -> u64 {
        let mut v: u64 = 0;
        let mut j = 0;
        while j < 8 {
            let idx = i * 8 + j;
            if idx < b.len() {
                v |= (b[idx] as u64) << (8 * j);
            }
            j += 1;
        }
        v
    };
    format!(
        "({} {} {} {} {})",
        constructor,
        limb(0),
        limb(1),
        limb(2),
        limb(3)
    )
}

/// A scalar field element as a Lean `mkFp` call (four little-endian `u64` limbs).
fn fp(x: Fp) -> String {
    field("mkFp", x)
}

/// A Vesta base-field element as a Lean `mkFq` call (four little-endian `u64` limbs).
fn fq(x: Fq) -> String {
    field("mkFq", x)
}

/// Whether `segment` is a conservative ASCII subset of a Lean identifier: non-empty, starting with a
/// letter or `_`, and otherwise containing only ASCII alphanumerics and `_`. A digit-leading segment
/// (e.g. `123`) is rejected, so a dotted `lean_namespace` spliced verbatim into `namespace`/`end`
/// cannot emit a token Lean would reject.
fn is_lean_ident(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Coordinate key for a point's identity (its affine `(x, y)` repr; identity → sentinel).
fn point_key(p: EqAffine) -> Vec<u8> {
    let co: Option<Coordinates<EqAffine>> = p.coordinates().into();
    match co {
        Some(c) => {
            let mut k = c.x().to_repr().as_ref().to_vec();
            k.extend_from_slice(c.y().to_repr().as_ref());
            k
        }
        None => vec![0u8],
    }
}

/// Deduplicated concrete curve points used by the generated Lean fixture. The generated terms have
/// type `VestaG`; the numeric indices are only source compression for repeated coordinate literals.
struct PointTable {
    ids: HashMap<Vec<u8>, usize>,
    points: Vec<EqAffine>,
}

impl PointTable {
    fn new() -> Self {
        Self {
            ids: HashMap::new(),
            points: Vec::new(),
        }
    }

    fn id(&mut self, point: EqAffine) -> usize {
        let key = point_key(point);
        if let Some(id) = self.ids.get(&key) {
            *id
        } else {
            let id = self.points.len();
            self.ids.insert(key, id);
            self.points.push(point);
            id
        }
    }

    fn point_ref(&mut self, point: EqAffine) -> String {
        format!("capturedPoint {}", self.id(point))
    }

    fn point_refs(&mut self, points: &[EqAffine]) -> Vec<String> {
        points.iter().map(|point| self.point_ref(*point)).collect()
    }

    fn coordinate_literals(&self) -> Vec<String> {
        self.points
            .iter()
            .map(|point| {
                let coordinates: Option<Coordinates<EqAffine>> = point.coordinates().into();
                match coordinates {
                    Some(coordinates) => {
                        format!("({}, {})", fq(*coordinates.x()), fq(*coordinates.y()))
                    }
                    None => "(0, 0)".to_string(),
                }
            })
            .collect()
    }
}

fn transcript_point(points: &mut PointTable, point: EqAffine) -> String {
    format!("TranscriptElt.point ({})", points.point_ref(point))
}

fn transcript_scalar(scalar: Fp) -> String {
    format!("TranscriptElt.scalar {}", fp(scalar))
}

fn join<T: ToString>(xs: &[T]) -> String {
    xs.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_fps(xs: &[Fp]) -> String {
    xs.iter().map(|x| fp(*x)).collect::<Vec<_>>().join(", ")
}

/// Lowercase hex, two digits per byte, no separators.
fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(2 * bytes.len());
    for b in bytes {
        write!(out, "{b:02x}").expect("writing to a String cannot fail");
    }
    out
}

/// Re-serialize the proof reads the recorder observed, as the prover writes them: `write_point`
/// (the compressed encoding) for each `ReadPoint` and `write_scalar` for each `ReadScalar`, in
/// read order. Common inputs and squeezes consume no proof bytes.
fn reserialize_proof_reads(events: &[TranscriptEvent<EqAffine>]) -> Vec<u8> {
    let mut transcript = Blake2bWrite::<Vec<u8>, EqAffine, Challenge255<EqAffine>>::init(vec![]);
    for event in events {
        match event {
            TranscriptEvent::ReadPoint(point) => transcript
                .write_point(*point)
                .expect("writing to an in-memory transcript cannot fail"),
            TranscriptEvent::ReadScalar(scalar) => transcript
                .write_scalar(*scalar)
                .expect("writing to an in-memory transcript cannot fail"),
            TranscriptEvent::CommonPoint(_)
            | TranscriptEvent::CommonScalar(_)
            | TranscriptEvent::Squeeze(_) => {}
        }
    }
    transcript.finalize()
}

/// Serialize one gate / lookup `Expression` to a Lean `Expr Fp` literal (mirrors the verifier's
/// `Expression::evaluate`; virtual selectors are removed before verification).
fn expr_to_lean(e: &Expression<Fp>) -> String {
    e.evaluate(
        &|c| format!("(.constant {})", fp(c)),
        &|_| panic!("virtual selectors are removed before verification"),
        &|q| format!("(.fixed {})", q.index),
        &|q| format!("(.advice {})", q.index),
        &|q| format!("(.instance {})", q.index),
        &|a: String| format!("(.negated {})", a),
        &|a: String, b: String| format!("(.sum {} {})", a, b),
        &|a: String, b: String| format!("(.product {} {})", a, b),
        &|a: String, c| format!("(.scaled {} {})", a, fp(c)),
    )
}

/// Render the typed transcript prefix and each subsequent squeeze checkpoint. `init_len` is the
/// exact number of VK/instance events that precede verifier proof processing; it is intentionally
/// independent of when the first proof element is read, because valid circuits may have no advice
/// commitments before the first squeeze.
fn render_transcript_capture(
    points: &mut PointTable,
    transcript_events: &[TranscriptEvent<EqAffine>],
    init_len: usize,
) -> (Vec<String>, Vec<String>) {
    let mut captured_init = Vec::new();
    let mut prefix = Vec::new();
    let mut schedule_entries = Vec::new();

    for (event_index, event) in transcript_events.iter().enumerate() {
        let is_init = event_index < init_len;
        match *event {
            TranscriptEvent::CommonPoint(point) => {
                let elt = transcript_point(points, point);
                if is_init {
                    captured_init.push(elt.clone());
                }
                prefix.push(elt);
            }
            TranscriptEvent::CommonScalar(scalar) => {
                let elt = transcript_scalar(scalar);
                if is_init {
                    captured_init.push(elt.clone());
                }
                prefix.push(elt);
            }
            TranscriptEvent::ReadPoint(point) => {
                assert!(
                    !is_init,
                    "proof read appeared inside the VK/instance prefix"
                );
                prefix.push(transcript_point(points, point));
            }
            TranscriptEvent::ReadScalar(scalar) => {
                assert!(
                    !is_init,
                    "proof read appeared inside the VK/instance prefix"
                );
                prefix.push(transcript_scalar(scalar));
            }
            TranscriptEvent::Squeeze(challenge) => {
                assert!(
                    !is_init,
                    "challenge squeeze appeared inside the VK/instance prefix"
                );
                schedule_entries.push(format!("([{}], {})", prefix.join(", "), fp(challenge)));
                prefix.push(transcript_scalar(challenge));
            }
        }
    }

    (captured_init, schedule_entries)
}

/// Which theorem family the exporter emits, and which identity fail-fast it enforces.
///
/// Every dispatch on this is a `match` listing both variants, never an equality test with an
/// implicit fallthrough: a new mode must fail to compile at each site until its fail-fast, its
/// theorem set, and its header are all chosen. `#[non_exhaustive]` is inert while the enum is
/// private and records the same intent for the day it is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
enum FixtureMode {
    /// Accepting capture: the MSM must be the identity; the eval theorems are emitted.
    Honest,
    /// Non-accepting capture: the MSM must *not* be the identity, and the match theorem is
    /// emitted alongside the mirrored `≠ 0` eval theorem.
    MatchOnly,
}

impl VerifyingKey<EqAffine> {
    /// Emit the Vesta Lean fixture for one captured proof (see module docs).
    ///
    /// `recorder` supplies the ordered transcript events, the proof elements (points and scalars in
    /// read order), the instance commitments, and the squeezed challenges captured during the
    /// verifier run; `captured_msm` is the assembled verifier fingerprint together with its exact
    /// parameter set. Only *accepting* runs are exported: `captured_msm` is verified to be the group
    /// identity before anything is emitted, so the fixture always proves `capturedMsm.evalNat = 0`.
    ///
    /// `instances` are the public inputs fed to the verifier for this run (`proof → column →
    /// values`, matching the `instances` argument of [`super::capture_proof_fingerprint`]); their
    /// outer length is the proof count. The exporter re-derives each instance commitment from them
    /// (`commit_lagrange` of the zero-padded column) and fails fast unless it reproduces the
    /// corresponding captured commitment, so the emitted `instance_commitments_derived` theorem —
    /// the `public inputs → instance commitments` check — is always dischargeable.
    pub fn dump_vesta_lean_fixture<R: Read>(
        &self,
        lean_namespace: &str,
        circuit_id: &str,
        k: u32,
        instances: &[&[&[Fp]]],
        recorder: &ChallengeRecorder<R, EqAffine, Challenge255<EqAffine>>,
        captured_msm: &MSM<'_, EqAffine>,
    ) -> String {
        self.dump_vesta_lean_fixture_with_mode(
            FixtureMode::Honest,
            lean_namespace,
            circuit_id,
            k,
            instances,
            recorder,
            captured_msm,
            None,
        )
    }

    /// As [`VerifyingKey::dump_vesta_lean_fixture`], additionally given `proof_bytes`, the proof
    /// byte string the verifier consumed in this run, which the fixture then carries hex-encoded as
    /// `capturedProofHex`.
    ///
    /// The exporter fails fast unless re-serializing the recorder's proof reads (`write_point` for
    /// each point read, `write_scalar` for each scalar read, in read order) reproduces `proof_bytes`
    /// exactly: the bytes must be precisely what the verifier's reads consumed, with no trailing
    /// data, so the emitted typed proof `ps` is their canonical parse. A consumer can then check its
    /// own proof-string decoder against the bytes the deployed verifier read.
    #[allow(clippy::too_many_arguments)]
    pub fn dump_vesta_lean_fixture_honest_with_proof_bytes<R: Read>(
        &self,
        lean_namespace: &str,
        circuit_id: &str,
        k: u32,
        instances: &[&[&[Fp]]],
        recorder: &ChallengeRecorder<R, EqAffine, Challenge255<EqAffine>>,
        captured_msm: &MSM<'_, EqAffine>,
        proof_bytes: &[u8],
    ) -> String {
        self.dump_vesta_lean_fixture_with_mode(
            FixtureMode::Honest,
            lean_namespace,
            circuit_id,
            k,
            instances,
            recorder,
            captured_msm,
            Some(proof_bytes),
        )
    }

    /// Emit a *match-only* Vesta Lean fixture for one captured non-accepting run (see module docs).
    ///
    /// The sibling of [`VerifyingKey::dump_vesta_lean_fixture`] for non-accepting captures —
    /// random proof strings are why it exists, but randomness is not something the export can
    /// check, so non-acceptance is the contract. The deployed verifier ran to completion and the
    /// captured MSM is deliberately **not** the group identity, so the fixture emits
    /// `fingerprint_matches` — exact coefficient/term agreement of the Lean-assembled MSM with the
    /// capture — plus `capturedMsm_evalNat_ne_zero` in place of the honest eval theorems, so the
    /// fixture proves it is non-accepting. The export fails fast
    /// if the captured MSM *is* the identity: accepting captures must use the honest exporter so
    /// their fixtures keep proving `capturedMsm.evalNat = 0`. Everything else — coordinate
    /// ordering, on-curve validation, URS emission, instance-commitment re-derivation, schedule
    /// entries, and the slot-reconstruction and term-count guards — is identical to the honest
    /// exporter, so regeneration stays bit-reproducible across both modes.
    pub fn dump_vesta_lean_fixture_match_only<R: Read>(
        &self,
        lean_namespace: &str,
        circuit_id: &str,
        k: u32,
        instances: &[&[&[Fp]]],
        recorder: &ChallengeRecorder<R, EqAffine, Challenge255<EqAffine>>,
        captured_msm: &MSM<'_, EqAffine>,
    ) -> String {
        self.dump_vesta_lean_fixture_with_mode(
            FixtureMode::MatchOnly,
            lean_namespace,
            circuit_id,
            k,
            instances,
            recorder,
            captured_msm,
            None,
        )
    }

    /// As [`VerifyingKey::dump_vesta_lean_fixture_match_only`], additionally given `proof_bytes`,
    /// which the fixture carries hex-encoded as `capturedProofHex` after the same
    /// re-serialization check as
    /// [`VerifyingKey::dump_vesta_lean_fixture_honest_with_proof_bytes`].
    #[allow(clippy::too_many_arguments)]
    pub fn dump_vesta_lean_fixture_match_only_with_proof_bytes<R: Read>(
        &self,
        lean_namespace: &str,
        circuit_id: &str,
        k: u32,
        instances: &[&[&[Fp]]],
        recorder: &ChallengeRecorder<R, EqAffine, Challenge255<EqAffine>>,
        captured_msm: &MSM<'_, EqAffine>,
        proof_bytes: &[u8],
    ) -> String {
        self.dump_vesta_lean_fixture_with_mode(
            FixtureMode::MatchOnly,
            lean_namespace,
            circuit_id,
            k,
            instances,
            recorder,
            captured_msm,
            Some(proof_bytes),
        )
    }

    // The public wrappers each carry the exporter's six or seven inputs; threading `mode` and the
    // optional proof bytes through to the shared core adds two more past `&self`. Bundling them
    // into a struct would only move the same arity to a constructor, since every one is required.
    #[allow(clippy::too_many_arguments)]
    fn dump_vesta_lean_fixture_with_mode<R: Read>(
        &self,
        mode: FixtureMode,
        lean_namespace: &str,
        circuit_id: &str,
        k: u32,
        instances: &[&[&[Fp]]],
        recorder: &ChallengeRecorder<R, EqAffine, Challenge255<EqAffine>>,
        captured_msm: &MSM<'_, EqAffine>,
        proof_bytes: Option<&[u8]>,
    ) -> String {
        // `lean_namespace` is spliced verbatim into `namespace`/`end`, and `circuit_id` is emitted
        // via `{:?}`, whose Rust string escapes (`\u{...}`, ...) are not Lean's. Validate both up
        // front — in release builds too, since this runs once per export and a malformed name yields
        // an uncompilable fixture. [`is_lean_ident`] accepts a conservative ASCII subset of Lean
        // identifiers, so a digit-leading `lean_namespace` segment like `123` is rejected.
        assert!(
            lean_namespace.split('.').all(is_lean_ident),
            "lean_namespace must be a dot-separated path of ASCII Lean identifiers \
             (letter/underscore start): {lean_namespace:?}"
        );
        // `circuit_id` becomes a Lean *string literal*, not an identifier, so a leading digit is
        // fine; it only has to stay a plain ASCII slug so `{:?}` cannot emit a `\u{...}` escape Lean
        // would reject.
        assert!(
            !circuit_id.is_empty()
                && circuit_id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "circuit_id must be a non-empty ASCII slug (alphanumeric, `_`, `-`): {circuit_id:?}"
        );
        // The supplied proof bytes must be exactly what the verifier's reads consumed: re-serializing
        // the recorded reads as the prover writes them reproduces them byte for byte, so the typed
        // proof emitted below is their canonical parse and no trailing data went unread.
        if let Some(bytes) = proof_bytes {
            let reserialized = reserialize_proof_reads(&recorder.events);
            assert!(
                reserialized == bytes,
                "proof_bytes must be exactly the bytes the verifier's proof reads consume: \
                 re-serializing the recorded reads gives {} bytes, the supplied string has {}",
                reserialized.len(),
                bytes.len()
            );
        }

        // The public instances' outer dimension is the proof count, exactly as in `verify_proof`.
        let num_proofs = instances.len();

        let common_points = recorder.common_points.as_slice();
        let read_points = recorder.points.as_slice();
        let read_scalars = recorder.scalars.as_slice();
        let challenges = recorder.challenges.as_slice();
        let transcript_events = recorder.events.as_slice();

        let params = captured_msm.params;
        // The honest mode's `capturedMsm_eval_eq_zero` presupposes an accepting capture, and the
        // match-only mode's `capturedMsm_evalNat_ne_zero` presupposes the opposite; fail fast on a
        // mode/capture mismatch rather than export a fixture whose theorem cannot hold, or one that
        // mislabels an accepting run as match-only. This is the only thing that selects the mode:
        // whether the caller's input was a random proof string is neither checked nor recorded.
        // Rejecting runs of honest proofs are checked in Rust, never exported to Lean.
        match mode {
            FixtureMode::Honest => assert!(
                captured_msm.clone().eval(),
                "captured MSM must evaluate to the identity; the emitted fixture proves it"
            ),
            FixtureMode::MatchOnly => assert!(
                !captured_msm.clone().eval(),
                "match-only fixtures capture non-accepting runs, but this captured MSM evaluates \
                 to the identity; export accepting captures with `dump_vesta_lean_fixture`"
            ),
        }
        let (msm_g, msm_w, msm_u, msm_other) = captured_msm.fingerprint_terms();
        let msm_g = msm_g.expect("captured MSM must contain verifier-generator coefficients");
        let msm_w = msm_w.expect("captured MSM must contain a blinding-generator coefficient");
        let msm_u = msm_u.expect("captured MSM must contain an IPA-generator coefficient");
        let cs = &self.cs;
        let n_advice = cs.num_advice_columns;
        let n_inst_cols = cs.num_instance_columns;
        let n_lookups = cs.lookups.len();
        let chunk_len = self.cs_degree - 2;
        let perm_columns = cs.permutation.get_columns();
        // A zero chunk length is only consistent with there being no permutation columns; otherwise
        // `n_perm_sets` below and the chunk-export loop (which falls back to `chunk_len.max(1)`)
        // would disagree on how many sets exist.
        assert!(
            chunk_len > 0 || perm_columns.is_empty(),
            "permutation columns present but chunk length is zero"
        );
        let n_perm_sets = if chunk_len == 0 {
            0
        } else {
            perm_columns.chunks(chunk_len).count()
        };
        let n_perm_cols = self.permutation.commitments().len();
        let n_quotient = self.domain.get_quotient_poly_degree();
        let n_inst_q = cs.instance_queries.len();
        let n_adv_q = cs.advice_queries.len();
        let n_fixed_q = cs.fixed_queries.len();
        let blinding = cs.blinding_factors();
        let n: u64 = 1u64 << k;
        assert_eq!(params.k, k);
        assert_eq!(params.g.len(), n as usize);
        assert_eq!(msm_g.len(), n as usize);
        assert_eq!(common_points.len(), num_proofs * n_inst_cols);
        assert_eq!(challenges.len(), 11 + k as usize);

        // ---- instance-commitment derivation (public inputs -> C_inst) ----
        //
        // The captured `common_points` are the Rust verifier's instance commitments, which
        // otherwise enter Lean as opaque points. Re-derive each from the supplied public inputs
        // exactly as `verify_proof` does — `commit_lagrange(zeroPad(column), Blind::default())` —
        // and fail fast unless the supplied instances reproduce the captured commitments. This is
        // the Rust ground truth for the emitted `instance_commitments_derived` theorem, which
        // re-checks the same derivation independently in Lean (ironwood#65). `max_instance_len` is
        // the longest instance column across all proofs; only the leading `g_lagrange` generators up
        // to that length are reachable by a nonzero coefficient, so only they need exporting.
        let mut max_instance_len = 0usize;
        for (p, proof_instances) in instances.iter().enumerate() {
            assert_eq!(
                proof_instances.len(),
                n_inst_cols,
                "each proof must supply exactly `num_instance_columns` instance columns"
            );
            for (col, column) in proof_instances.iter().enumerate() {
                max_instance_len = max_instance_len.max(column.len());
                let mut poly = column.to_vec();
                poly.resize(params.n as usize, Fp::ZERO);
                let poly = self.domain.lagrange_from_vec(poly);
                let derived = params.commit_lagrange(&poly, Blind::default()).to_affine();
                assert_eq!(
                    derived,
                    common_points[p * n_inst_cols + col],
                    "supplied public instances do not reproduce the captured instance commitment \
                     for proof {p}, column {col}; the instance-commitment derivation could not hold"
                );
            }
        }

        let expected_read_points = num_proofs * n_advice
            + num_proofs * n_lookups * 2
            + num_proofs * n_perm_sets
            + num_proofs * n_lookups
            + 1
            + n_quotient
            + 1
            + 1
            + 2 * k as usize;
        assert_eq!(read_points.len(), expected_read_points);

        let minimum_read_scalars = num_proofs * n_inst_q
            + num_proofs * n_adv_q
            + n_fixed_q
            + 1
            + n_perm_cols
            + num_proofs
                * if n_perm_sets == 0 {
                    0
                } else {
                    3 * n_perm_sets - 1
                }
            + num_proofs * n_lookups * 5
            + 2;
        assert!(read_scalars.len() >= minimum_read_scalars);

        let event_common_points: Vec<_> = transcript_events
            .iter()
            .filter_map(|event| match event {
                TranscriptEvent::CommonPoint(point) => Some(*point),
                _ => None,
            })
            .collect();
        let event_read_points: Vec<_> = transcript_events
            .iter()
            .filter_map(|event| match event {
                TranscriptEvent::ReadPoint(point) => Some(*point),
                _ => None,
            })
            .collect();
        let event_read_scalars: Vec<_> = transcript_events
            .iter()
            .filter_map(|event| match event {
                TranscriptEvent::ReadScalar(scalar) => Some(*scalar),
                _ => None,
            })
            .collect();
        let event_challenges: Vec<_> = transcript_events
            .iter()
            .filter_map(|event| match event {
                TranscriptEvent::Squeeze(challenge) => Some(*challenge),
                _ => None,
            })
            .collect();
        assert_eq!(event_common_points, common_points);
        assert_eq!(event_read_points, read_points);
        assert_eq!(event_read_scalars, read_scalars);
        assert_eq!(event_challenges, challenges);

        let transcript_init_len = 1 + common_points.len();
        assert!(transcript_events.len() >= transcript_init_len);
        assert!(matches!(
            transcript_events.first(),
            Some(TranscriptEvent::CommonScalar(scalar)) if *scalar == self.transcript_repr
        ));
        for (event, expected_point) in transcript_events[1..transcript_init_len]
            .iter()
            .zip(common_points)
        {
            assert!(matches!(
                event,
                TranscriptEvent::CommonPoint(point) if point == expected_point
            ));
        }

        let mut points = PointTable::new();
        let urs_g = points.point_refs(&params.g);
        let urs_w = points.point_ref(params.w);
        let urs_u = points.point_ref(params.u);
        // Lagrange-basis generators actually reachable by a captured public instance: only the
        // leading `max_instance_len` matter, since `commit_lagrange` multiplies every later
        // generator by a zero (padding) coefficient. Exporting just this prefix keeps the fixture
        // small while remaining an exact mirror of `commit_lagrange` on the captured instances.
        let urs_g_lagrange = points.point_refs(&params.g_lagrange[..max_instance_len]);

        // The blocks below are a hand-maintained mirror of the verifier's read schedule; the
        // count asserts alone cannot catch an equal-length reordering. The Fiat-Shamir squeezes
        // delimit each phase of that schedule, so recover how many points/scalars had been read
        // before each squeeze and pin every block boundary that a squeeze delimits against it.
        // The challenges are, in squeeze order: θ, β, γ, y, x, x1, x2, x3, x4, ξ, z, then one per
        // IPA round.
        let (points_before_squeeze, scalars_before_squeeze): (Vec<usize>, Vec<usize>) = {
            let mut points_at = Vec::with_capacity(challenges.len());
            let mut scalars_at = Vec::with_capacity(challenges.len());
            let (mut np, mut ns) = (0usize, 0usize);
            for event in transcript_events {
                match event {
                    TranscriptEvent::ReadPoint(_) => np += 1,
                    TranscriptEvent::ReadScalar(_) => ns += 1,
                    TranscriptEvent::Squeeze(_) => {
                        points_at.push(np);
                        scalars_at.push(ns);
                    }
                    _ => {}
                }
            }
            (points_at, scalars_at)
        };
        assert_eq!(points_before_squeeze.len(), challenges.len());
        // Squeeze indices into the challenge/`*_before_squeeze` vectors.
        const THETA: usize = 0;
        const BETA: usize = 1;
        const GAMMA: usize = 2;
        const Y: usize = 3;
        const X: usize = 4;
        const X1: usize = 5;
        const X2: usize = 6;
        const X3: usize = 7;
        const X4: usize = 8;
        const XI: usize = 9;
        const Z: usize = 10;
        const IPA_ROUND_BASE: usize = 11;

        // ---- slice read points into blocks (verifier read order), pinning each squeeze-delimited
        //      boundary against `points_before_squeeze` ----
        let mut pi = 0usize;
        let advice_pts = &read_points[pi..pi + num_proofs * n_advice];
        pi += num_proofs * n_advice;
        // θ is squeezed once all advice commitments have been read.
        assert_eq!(pi, points_before_squeeze[THETA]);
        let lookup_perm_pts = &read_points[pi..pi + num_proofs * n_lookups * 2];
        pi += num_proofs * n_lookups * 2;
        // β, then γ with no reads between them, follow the lookup permuted commitments.
        assert_eq!(pi, points_before_squeeze[BETA]);
        assert_eq!(points_before_squeeze[GAMMA], points_before_squeeze[BETA]);
        let perm_prod_pts = &read_points[pi..pi + num_proofs * n_perm_sets];
        pi += num_proofs * n_perm_sets;
        let lookup_prod_pts = &read_points[pi..pi + num_proofs * n_lookups];
        pi += num_proofs * n_lookups;
        let vanishing_random = read_points[pi];
        pi += 1;
        // y is squeezed after the permutation/lookup products and the pre-y vanishing commitment.
        assert_eq!(pi, points_before_squeeze[Y]);
        let h_pts = &read_points[pi..pi + n_quotient];
        pi += n_quotient;
        // x is squeezed after the quotient (h) pieces; x1 and x2 follow with no points between.
        assert_eq!(pi, points_before_squeeze[X]);
        assert_eq!(points_before_squeeze[X1], points_before_squeeze[X]);
        assert_eq!(points_before_squeeze[X2], points_before_squeeze[X]);
        let q_prime = read_points[pi];
        pi += 1;
        // x3 is squeezed after q'; x4 follows with no points between (only the multiopen u scalars).
        assert_eq!(pi, points_before_squeeze[X3]);
        assert_eq!(points_before_squeeze[X4], points_before_squeeze[X3]);
        let ipa_s = read_points[pi];
        pi += 1;
        // ξ is squeezed after the IPA s commitment; z follows with no reads between.
        assert_eq!(pi, points_before_squeeze[XI]);
        assert_eq!(points_before_squeeze[Z], points_before_squeeze[XI]);
        let ipa_round_pts = &read_points[pi..]; // 2*k points (L, R per round)
        assert_eq!(ipa_round_pts.len(), 2 * k as usize);
        // Each IPA round reads its (L, R) pair immediately before squeezing that round's challenge.
        for round in 0..k as usize {
            assert_eq!(
                points_before_squeeze[IPA_ROUND_BASE + round],
                pi + 2 * (round + 1)
            );
        }

        // ---- slice read scalars into blocks, pinning each squeeze-delimited boundary against
        //      `scalars_before_squeeze` ----
        let mut si = 0usize;
        let instance_evals = &read_scalars[si..si + num_proofs * n_inst_q];
        si += num_proofs * n_inst_q;
        let advice_evals = &read_scalars[si..si + num_proofs * n_adv_q];
        si += num_proofs * n_adv_q;
        let fixed_evals = &read_scalars[si..si + n_fixed_q];
        si += n_fixed_q;
        let vanishing_eval = read_scalars[si];
        si += 1;
        let perm_common = &read_scalars[si..si + n_perm_cols];
        si += n_perm_cols;
        let perm_set_count = if n_perm_sets == 0 {
            0
        } else {
            3 * n_perm_sets - 1
        };
        let perm_set_evals = &read_scalars[si..si + num_proofs * perm_set_count];
        si += num_proofs * perm_set_count;
        let lookup_evals = &read_scalars[si..si + num_proofs * n_lookups * 5];
        si += num_proofs * n_lookups * 5;
        // All evaluations at x are read before x1 is squeezed; x2 and x3 follow with no scalars
        // between them (only the q' point read).
        assert_eq!(si, scalars_before_squeeze[X1]);
        assert_eq!(scalars_before_squeeze[X2], scalars_before_squeeze[X1]);
        assert_eq!(scalars_before_squeeze[X3], scalars_before_squeeze[X1]);
        assert!(read_scalars.len() >= si + 2);
        let n_point_sets = read_scalars.len() - 2 - si;
        let multiopen_u = &read_scalars[si..si + n_point_sets];
        si += n_point_sets;
        // x4 is squeezed after the multiopen u-evaluations; ξ, z, and every IPA-round squeeze then
        // precede the final c/f scalars, which are read only after the last squeeze.
        assert_eq!(si, scalars_before_squeeze[X4]);
        assert_eq!(scalars_before_squeeze[XI], si);
        assert_eq!(scalars_before_squeeze[Z], si);
        for round in 0..k as usize {
            assert_eq!(scalars_before_squeeze[IPA_ROUND_BASE + round], si);
        }
        let ipa_c = read_scalars[si];
        let ipa_f = read_scalars[si + 1];
        assert_eq!(si + 2, read_scalars.len());

        // ---- fail fast if the capture cannot possibly match in Lean ----
        //
        // Halo2's `MSM` keys terms by base coordinate and drops the identity, whereas the Lean
        // `assemble` appends one `other` term per protocol commitment slot and keeps every one. So
        // if two distinct slots share a base coordinate (e.g. two identically-assigned fixed
        // columns, or two proofs with equal instance commitments), or any slot is the identity, the
        // captured MSM carries strictly fewer `other` terms than `assemble`, and `fingerprint_matches`
        // — a permutation comparison with no merging — cannot hold. Diagnose that here instead of
        // deferring to an opaque Lean failure on a fixture that can never be discharged.
        //
        // The base slots are reconstructed from the same query layout the verifier consumes: one
        // slot per (proof, distinct queried instance/advice column), one per queried fixed column,
        // one per permutation-common / permutation-product / lookup / quotient (`h`) piece
        // commitment, plus the vanishing-random commitment, `q'`, the IPA `s`, and each IPA `L`/`R`.
        let distinct = |mut cols: Vec<usize>| {
            cols.sort_unstable();
            cols.dedup();
            cols
        };
        let inst_cols = distinct(cs.instance_queries.iter().map(|(c, _)| c.index()).collect());
        let adv_cols = distinct(cs.advice_queries.iter().map(|(c, _)| c.index()).collect());
        let fix_cols = distinct(cs.fixed_queries.iter().map(|(c, _)| c.index()).collect());
        let mut msm_base_slots: Vec<EqAffine> = Vec::new();
        for p in 0..num_proofs {
            for &col in &inst_cols {
                msm_base_slots.push(common_points[p * n_inst_cols + col]);
            }
            for &col in &adv_cols {
                msm_base_slots.push(advice_pts[p * n_advice + col]);
            }
        }
        for &col in &fix_cols {
            msm_base_slots.push(self.fixed_commitments[col]);
        }
        msm_base_slots.extend_from_slice(self.permutation.commitments());
        msm_base_slots.extend_from_slice(perm_prod_pts);
        msm_base_slots.extend_from_slice(lookup_perm_pts);
        msm_base_slots.extend_from_slice(lookup_prod_pts);
        msm_base_slots.push(vanishing_random);
        msm_base_slots.extend_from_slice(h_pts);
        msm_base_slots.push(q_prime);
        msm_base_slots.push(ipa_s);
        msm_base_slots.extend_from_slice(ipa_round_pts);

        let slot_coords: BTreeSet<Vec<u8>> = msm_base_slots
            .iter()
            .filter_map(|point| {
                let coords: Option<Coordinates<EqAffine>> = point.coordinates().into();
                // Halo2 drops the identity from the MSM, so it is not a captured base coordinate.
                coords.map(|_| point_key(*point))
            })
            .collect();
        let captured_coords: BTreeSet<Vec<u8>> = msm_other
            .iter()
            .map(|(_, x, y)| {
                let point: Option<EqAffine> = EqAffine::from_xy(*x, *y).into();
                point_key(point.expect("captured MSM base must be a valid affine point"))
            })
            .collect();
        // Self-check: the reconstructed slot bases must reproduce the captured MSM's actual bases.
        // A mismatch means the verifier read/query schedule assumed above has drifted from the
        // deployed verifier, so the reconstruction (and the guard below) can no longer be trusted.
        assert_eq!(
            slot_coords, captured_coords,
            "exporter's MSM base-slot reconstruction does not match the captured MSM bases; the \
             assumed verifier read/query schedule is stale"
        );
        // The guard: with all-distinct, non-identity bases the slot count equals the captured term
        // count. Any shortfall means Halo2 merged same-base terms or dropped an identity base.
        assert_eq!(
            msm_base_slots.len(),
            msm_other.len(),
            "captured MSM collapsed {} protocol commitment slots into {} terms by merging same-base \
             terms or dropping identity bases; the Lean assembly does neither, so the emitted \
             `fingerprint_matches` could not hold. Capture a proof whose commitments are all \
             distinct and non-identity.",
            msm_base_slots.len(),
            msm_other.len()
        );

        let mut out = String::new();

        // ---- Shape ----
        out.push_str(&format!(
            "def shape : Shape := {{ k := {}, numProofs := {}, numAdviceColumns := {}, numLookups := {}, numPermutationSets := {}, numPermutationColumns := {}, numQuotientPieces := {}, numInstanceColumns := {}, numInstanceQueries := {}, numAdviceQueries := {}, numFixedQueries := {}, numPointSets := {} }}\n\n",
            k, num_proofs, n_advice, n_lookups, n_perm_sets, n_perm_cols, n_quotient, n_inst_cols, n_inst_q, n_adv_q, n_fixed_q, n_point_sets,
        ));
        out.push_str(&format!(
            "def capturedUrsG : List G := [{}]\n\n",
            urs_g.join(", ")
        ));
        out.push_str(&format!(
            "def capturedURS : URS G := {{ k := {}, g := fun i => capturedUrsG.getD i.val 0, w := {}, u := {} }}\n\n",
            k, urs_w, urs_u
        ));
        // `capturedURS.g` indexes `capturedUrsG` with `getD`, which silently yields the identity for
        // any index past the list's end; pin the length so a truncated URS cannot pass unnoticed.
        out.push_str(
            "/-- The captured URS lists exactly the `2 ^ k` generators the MSM evaluates\n",
        );
        out.push_str("against, so `capturedURS.g`'s `getD` never substitutes the identity for a\n");
        out.push_str("missing generator. -/\n");
        out.push_str("theorem capturedUrsG_length : capturedUrsG.length = 2 ^ shape.k := by native_decide\n\n");
        out.push_str(&format!(
            "def capturedVkTranscriptRepr : Fp := {}\n\n",
            fp(self.transcript_repr)
        ));
        // The exact string `VerifyingKey::from_parts` hashed into `transcript_repr`: the compact
        // `Debug` rendering of the pinned key. Emitting it lets a consumer recompute the digest
        // and read the pinned fields, instead of trusting the scalar above. The exporter checks the
        // string is printable ASCII (so `{:?}` adds only the `\"` and `\\` escapes Lean shares)
        // and that hashing it reproduces `transcript_repr`, so the emitted text is the preimage.
        let pinned = format!("{:?}", self.pinned());
        assert!(
            pinned
                .chars()
                .all(|c| c.is_ascii() && !c.is_ascii_control()),
            "the pinned key description must be printable ASCII"
        );
        {
            let mut hasher = blake2b_simd::Params::new()
                .hash_length(64)
                .personal(b"Halo2-Verify-Key")
                .to_state();
            hasher.update(&(pinned.len() as u64).to_le_bytes());
            hasher.update(pinned.as_bytes());
            let recomputed = Fp::from_uniform_bytes(hasher.finalize().as_array());
            assert!(
                recomputed == self.transcript_repr,
                "hashing the pinned key description must reproduce transcript_repr"
            );
        }
        out.push_str("/-- The exact text `VerifyingKey::from_parts` hashed into `capturedVkTranscriptRepr`:\n");
        out.push_str("the compact `Debug` rendering of the pinned key (`Halo2-Verify-Key` BLAKE2b over its\n");
        out.push_str(
            "little-endian `u64` byte length then its bytes, reduced modulo `p`). The exporter\n",
        );
        out.push_str("re-hashed it and checked the scalar above before emitting. -/\n");
        out.push_str(&format!(
            "def capturedPinnedKeyDescription : String :=\n  {:?}\n\n",
            pinned
        ));

        // ---- gates ----
        let gates: Vec<String> = cs
            .gates
            .iter()
            .flat_map(|g| g.polynomials().iter().map(expr_to_lean))
            .collect();

        // ---- query layouts ----
        let inst_layout: Vec<String> = cs
            .instance_queries
            .iter()
            .map(|(c, r)| format!("({}, ({} : ℤ))", c.index(), r.0))
            .collect();
        let adv_layout: Vec<String> = cs
            .advice_queries
            .iter()
            .map(|(c, r)| format!("({}, ({} : ℤ))", c.index(), r.0))
            .collect();
        let fix_layout: Vec<String> = cs
            .fixed_queries
            .iter()
            .map(|(c, r)| format!("({}, ({} : ℤ))", c.index(), r.0))
            .collect();

        // ---- permutation chunks: (ColumnRef × ℕ common-eval index) ----
        let mut chunks: Vec<String> = Vec::new();
        let mut gidx = 0usize;
        for chunk in perm_columns.chunks(chunk_len.max(1)) {
            let mut entries: Vec<String> = Vec::new();
            for col in chunk {
                let qi = cs.get_any_query_index(*col);
                let cref = match col.column_type() {
                    Any::Advice => format!("(.advice {})", qi),
                    Any::Fixed => format!("(.fixed {})", qi),
                    Any::Instance => format!("(.instance {})", qi),
                };
                entries.push(format!("({}, {})", cref, gidx));
                gidx += 1;
            }
            chunks.push(format!("[{}]", entries.join(", ")));
        }

        // ---- lookup input/table expressions ----
        let lk_in: Vec<String> = cs
            .lookups
            .iter()
            .map(|a| {
                format!(
                    "[{}]",
                    a.input_expressions
                        .iter()
                        .map(expr_to_lean)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();
        let lk_tab: Vec<String> = cs
            .lookups
            .iter()
            .map(|a| {
                format!(
                    "[{}]",
                    a.table_expressions
                        .iter()
                        .map(expr_to_lean)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        // ---- VK commitments as concrete Vesta point references ----
        let fixed_points = points.point_refs(&self.fixed_commitments);
        let inst_comm_points = points.point_refs(common_points);
        let perm_comm_points = points.point_refs(self.permutation.commitments());

        // Defined through `shape` rather than re-emitting `n_inst_cols`, so the standalone name and
        // the shape field cannot drift: both now trace to the same captured Rust value.
        out.push_str("def capturedNumInstanceColumns : ℕ := shape.numInstanceColumns\n\n");
        out.push_str(&format!(
            "def capturedFixedCommitments : List G := [{}]\n\n",
            join(&fixed_points)
        ));
        out.push_str(&format!(
            "def capturedInstanceCommitments : List G := [{}]\n\n",
            join(&inst_comm_points)
        ));

        // ---- instance-commitment derivation: public inputs -> C_inst (ironwood#65) ----
        //
        // Export the captured public inputs and the Lagrange-basis prefix, then check in Lean that
        // `commit_lagrange` of each zero-padded column reproduces the captured commitment. This
        // closes the `public inputs -> instance commitments` gap that `capturedInstanceCommitments`
        // otherwise leaves as opaque points; the same derivation is asserted in Rust above.
        let public_instance_lits: Vec<String> = instances
            .iter()
            .flat_map(|proof_instances| {
                proof_instances
                    .iter()
                    .map(|column| format!("[{}]", join_fps(column)))
            })
            .collect();
        out.push_str(&format!(
            "def capturedUrsGLagrange : List G := [{}]\n\n",
            urs_g_lagrange.join(", ")
        ));
        out.push_str(
            "/-- Captured public inputs, flattened as `proof * capturedNumInstanceColumns + column`\n",
        );
        out.push_str(
            "(matching `capturedInstanceCommitments`); each entry is one instance column's\n",
        );
        out.push_str("values before zero-padding to the domain. -/\n");
        out.push_str(&format!(
            "def capturedPublicInstances : List (List Fp) := [{}]\n\n",
            public_instance_lits.join(", ")
        ));
        out.push_str(
            "/-- Commit to a zero-padded public-instance column against the Lagrange basis, exactly\n",
        );
        out.push_str("as halo2 `Params::commit_lagrange` with `Blind::default () = 1`:\n");
        out.push_str("`(∑ i in range coeffs.length, coeffs[i] • gLagrange[i]) + w`. `capturedUrsGLagrange`\n");
        out.push_str(
            "lists only the leading generators a captured instance can reach; every later\n",
        );
        out.push_str("generator carries a zero padding coefficient (`capturedPublicInstances_within_lagrange`). -/\n");
        out.push_str("def commitLagrange (coeffs : List Fp) : G :=\n");
        out.push_str("  ((List.range coeffs.length).map\n");
        out.push_str("    (fun i => (coeffs.getD i 0).val • capturedUrsGLagrange.getD i 0)).sum + capturedURS.w\n\n");
        out.push_str("/-- The instance commitment the verifier uses, computed per proof and column from the\n");
        out.push_str("public inputs — halo2 `verify_proof` derives this from its `instances` argument, not the\n");
        out.push_str("VK. `assemble` consumes this in place of the removed `vk.instanceCommitment` field; by\n");
        out.push_str("`instance_commitments_derived` it equals the captured commitment the deployed verifier used. -/\n");
        out.push_str("def derivedInstanceCommitment (p : Fin shape.numProofs) (i : ℕ) : G :=\n");
        out.push_str("  commitLagrange (capturedPublicInstances.getD (p.val * capturedNumInstanceColumns + i) [])\n\n");
        out.push_str("/-- Every captured public-instance column fits within the exported Lagrange prefix, so\n");
        out.push_str(
            "`commitLagrange`'s `getD` never substitutes the identity for a needed generator. -/\n",
        );
        out.push_str("theorem capturedPublicInstances_within_lagrange :\n");
        out.push_str("    capturedPublicInstances.all (fun c => decide (c.length ≤ capturedUrsGLagrange.length)) = true := by native_decide\n\n");
        out.push_str("/-- The captured instance commitments are the `commit_lagrange` of the captured public\n");
        out.push_str("inputs: this derives `public inputs → instance commitments` in Lean rather than trusting\n");
        out.push_str("`capturedInstanceCommitments` as opaque points (ironwood#65). -/\n");
        out.push_str("theorem instance_commitments_derived :\n");
        out.push_str("    capturedPublicInstances.map commitLagrange = capturedInstanceCommitments := by native_decide\n\n");

        out.push_str(&format!(
            "def capturedPermutationCommonCommitments : List G := [{}]\n\n",
            join(&perm_comm_points)
        ));

        // The `vk` literal mirrors halo2's `VerifyingKey` field-for-field: every entry below is read
        // from the Rust VK (`self.domain`/`self.cs`/`self.fixed_commitments`/`self.permutation`). The
        // instance commitment is deliberately *not* a field — halo2 computes it per proof inside
        // `verify_proof` from the public inputs, not from the VK — so it is emitted separately as
        // `derivedInstanceCommitment` and passed to `assemble`, keeping this `vk` faithful to the
        // pinned Rust key rather than carrying a value synthesized from the capture.
        out.push_str(
            "/-- The verifying key, mirroring halo2's `VerifyingKey` (circuit-fixed data only). The\n",
        );
        out.push_str(
            "instance commitment is intentionally absent: halo2 computes it per proof from the\n",
        );
        out.push_str(
            "public inputs (see `derivedInstanceCommitment`), so it is not a VK field. -/\n",
        );
        out.push_str("def vk : VerifyingKey shape Fp G := {\n");
        out.push_str(&format!("  omega := {},\n", fp(self.domain.get_omega())));
        out.push_str(&format!("  n := {},\n", n));
        out.push_str(&format!("  blindingFactors := {},\n", blinding));
        out.push_str(&format!("  delta := {},\n", fp(Fp::DELTA)));
        out.push_str(&format!("  chunkLen := {},\n", chunk_len));
        out.push_str(&format!("  gates := [{}],\n", gates.join(", ")));
        out.push_str(&format!(
            "  instanceQueryLayout := [{}],\n",
            inst_layout.join(", ")
        ));
        out.push_str(&format!(
            "  adviceQueryLayout := [{}],\n",
            adv_layout.join(", ")
        ));
        out.push_str(&format!(
            "  fixedQueryLayout := [{}],\n",
            fix_layout.join(", ")
        ));
        out.push_str("  fixedCommitment := fun i => capturedFixedCommitments.getD i 0,\n");
        out.push_str("  permutationCommonCommitment := fun i => capturedPermutationCommonCommitments.getD i.val 0,\n");
        out.push_str(&format!(
            "  permutationChunks := [{}],\n",
            chunks.join(", ")
        ));
        out.push_str(&format!(
            "  lookupInputExprs := fun l => ([{}] : List (List (Expr Fp))).getD l.val [],\n",
            lk_in.join(", ")
        ));
        out.push_str(&format!(
            "  lookupTableExprs := fun l => ([{}] : List (List (Expr Fp))).getD l.val [] }}\n\n",
            lk_tab.join(", ")
        ));

        // ---- proof string ----
        let advice_points = points.point_refs(advice_pts);
        let lk_input_points: Vec<String> = (0..num_proofs * n_lookups)
            .map(|i| points.point_ref(lookup_perm_pts[i * 2]))
            .collect();
        let lk_table_points: Vec<String> = (0..num_proofs * n_lookups)
            .map(|i| points.point_ref(lookup_perm_pts[i * 2 + 1]))
            .collect();
        let perm_prod_points = points.point_refs(perm_prod_pts);
        let lk_prod_points = points.point_refs(lookup_prod_pts);
        let h_points = points.point_refs(h_pts);
        let vanishing_random_point = points.point_ref(vanishing_random);
        let q_prime_point = points.point_ref(q_prime);
        let ipa_s_point = points.point_ref(ipa_s);
        let ipa_round_ids: Vec<String> = (0..ipa_round_pts.len() / 2)
            .map(|j| {
                format!(
                    "({}, {})",
                    points.point_ref(ipa_round_pts[2 * j]),
                    points.point_ref(ipa_round_pts[2 * j + 1])
                )
            })
            .collect();

        // permutation set evals: walk per proof / per set (eval, next, [last])
        let mut perm_set_lits: Vec<String> = Vec::new();
        {
            let mut idx = 0usize;
            for _p in 0..num_proofs {
                for s in 0..n_perm_sets {
                    let eval = perm_set_evals[idx];
                    idx += 1;
                    let next = perm_set_evals[idx];
                    idx += 1;
                    let last = if s + 1 < n_perm_sets {
                        let l = perm_set_evals[idx];
                        idx += 1;
                        format!("some {}", fp(l))
                    } else {
                        "none".to_string()
                    };
                    perm_set_lits.push(format!(
                        "{{ eval := {}, nextEval := {}, lastEval := {} }}",
                        fp(eval),
                        fp(next),
                        last
                    ));
                }
            }
        }
        let lookup_lits: Vec<String> = (0..num_proofs * n_lookups).map(|i| {
            let b = i * 5;
            format!("{{ productEval := {}, productNextEval := {}, permutedInputEval := {}, permutedInputInvEval := {}, permutedTableEval := {} }}",
                fp(lookup_evals[b]), fp(lookup_evals[b + 1]), fp(lookup_evals[b + 2]), fp(lookup_evals[b + 3]), fp(lookup_evals[b + 4]))
        }).collect();

        out.push_str(&format!(
            "def capturedAdviceCommitments : List G := [{}]\n\n",
            join(&advice_points)
        ));
        out.push_str(&format!(
            "def capturedLookupPermutedInput : List G := [{}]\n\n",
            join(&lk_input_points)
        ));
        out.push_str(&format!(
            "def capturedLookupPermutedTable : List G := [{}]\n\n",
            join(&lk_table_points)
        ));
        out.push_str(&format!(
            "def capturedPermutationProducts : List G := [{}]\n\n",
            join(&perm_prod_points)
        ));
        out.push_str(&format!(
            "def capturedLookupProducts : List G := [{}]\n\n",
            join(&lk_prod_points)
        ));
        out.push_str(&format!(
            "def capturedHPieces : List G := [{}]\n\n",
            join(&h_points)
        ));
        out.push_str(&format!(
            "def capturedInstanceEvals : List Fp := [{}]\n\n",
            join_fps(instance_evals)
        ));
        out.push_str(&format!(
            "def capturedAdviceEvals : List Fp := [{}]\n\n",
            join_fps(advice_evals)
        ));
        out.push_str(&format!(
            "def capturedFixedEvals : List Fp := [{}]\n\n",
            join_fps(fixed_evals)
        ));
        out.push_str(&format!(
            "def capturedPermutationCommonEvals : List Fp := [{}]\n\n",
            join_fps(perm_common)
        ));
        out.push_str(&format!(
            "def capturedPermutationSetEvals : List (PermSetEval Fp) := [{}]\n\n",
            perm_set_lits.join(", ")
        ));
        out.push_str(&format!(
            "def capturedLookupEvals : List (LookupEval Fp) := [{}]\n\n",
            lookup_lits.join(", ")
        ));
        out.push_str(&format!(
            "def capturedMultiopenU : List Fp := [{}]\n\n",
            join_fps(multiopen_u)
        ));
        out.push_str(&format!(
            "def capturedIpaRounds : List (G × G) := [{}]\n\n",
            ipa_round_ids.join(", ")
        ));

        out.push_str("def ps : ProofString shape Fp G := {\n");
        out.push_str(&format!("  adviceCommitments := fun p c => capturedAdviceCommitments.getD (p.val * {} + c.val) 0,\n", n_advice));
        out.push_str(&format!("  lookupPermutedInput := fun p l => capturedLookupPermutedInput.getD (p.val * {} + l.val) 0,\n", n_lookups));
        out.push_str(&format!("  lookupPermutedTable := fun p l => capturedLookupPermutedTable.getD (p.val * {} + l.val) 0,\n", n_lookups));
        out.push_str(&format!("  permutationProduct := fun p s => capturedPermutationProducts.getD (p.val * {} + s.val) 0,\n", n_perm_sets));
        out.push_str(&format!(
            "  lookupProduct := fun p l => capturedLookupProducts.getD (p.val * {} + l.val) 0,\n",
            n_lookups
        ));
        out.push_str(&format!(
            "  vanishingRandom := {},\n",
            vanishing_random_point
        ));
        out.push_str("  hPieces := fun i => capturedHPieces.getD i.val 0,\n");
        out.push_str(&format!(
            "  instanceEvals := fun p q => capturedInstanceEvals.getD (p.val * {} + q.val) 0,\n",
            n_inst_q
        ));
        out.push_str(&format!(
            "  adviceEvals := fun p q => capturedAdviceEvals.getD (p.val * {} + q.val) 0,\n",
            n_adv_q
        ));
        out.push_str("  fixedEvals := fun q => capturedFixedEvals.getD q.val 0,\n");
        out.push_str(&format!(
            "  vanishingRandomEval := {},\n",
            fp(vanishing_eval)
        ));
        out.push_str(
            "  permutationCommonEvals := fun i => capturedPermutationCommonEvals.getD i.val 0,\n",
        );
        out.push_str(&format!("  permutationSetEvals := fun p s => capturedPermutationSetEvals.getD (p.val * {} + s.val) {{ eval := 0, nextEval := 0, lastEval := none }},\n", n_perm_sets));
        out.push_str(&format!("  lookupEvals := fun p l => capturedLookupEvals.getD (p.val * {} + l.val) {{ productEval := 0, productNextEval := 0, permutedInputEval := 0, permutedInputInvEval := 0, permutedTableEval := 0 }},\n", n_lookups));
        out.push_str(&format!("  multiopenQPrime := {},\n", q_prime_point));
        out.push_str("  multiopenU := fun s => capturedMultiopenU.getD s.val 0,\n");
        out.push_str(&format!("  ipaS := {},\n", ipa_s_point));
        out.push_str("  ipaRounds := fun j => capturedIpaRounds.getD j.val (0, 0),\n");
        out.push_str(&format!("  ipaC := {},\n", fp(ipa_c)));
        out.push_str(&format!("  ipaF := {} }}\n\n", fp(ipa_f)));

        // ---- challenges ----
        let ipa_ch: Vec<Fp> = challenges[11..].to_vec();
        out.push_str("def ch : Challenges shape.k Fp := {\n");
        out.push_str(&format!(
            "  theta := {}, beta := {}, gamma := {}, y := {}, x := {},\n",
            fp(challenges[0]),
            fp(challenges[1]),
            fp(challenges[2]),
            fp(challenges[3]),
            fp(challenges[4])
        ));
        out.push_str(&format!(
            "  x1 := {}, x2 := {}, x3 := {}, x4 := {},\n",
            fp(challenges[5]),
            fp(challenges[6]),
            fp(challenges[7]),
            fp(challenges[8])
        ));
        out.push_str(&format!(
            "  xi := {}, z := {},\n",
            fp(challenges[9]),
            fp(challenges[10])
        ));
        out.push_str(&format!(
            "  ipaRound := fun j => ([{}] : List Fp).getD j.val 0 }}\n\n",
            join_fps(&ipa_ch)
        ));

        // ---- captured Fiat-Shamir prelude and schedule ----
        //
        // The recorder observes the full typed deployed transcript after Blake2b initialization:
        // the verification-key transcript scalar, instance commitments, proof reads, and squeezes.
        // `capturedInit` contains the exact VK/instance prefix. After each later squeeze we append
        // the captured challenge as a scalar because `deriveChallenges` uses that abstract separator
        // between consecutive squeezes; the deployed Blake2b transcript instead uses a domain-prefix
        // byte.
        let (captured_init, schedule_entries) =
            render_transcript_capture(&mut points, transcript_events, transcript_init_len);
        out.push_str("/-- Captured verifier Fiat-Shamir prefix before the proof-derived transcript suffix.\n");
        out.push_str("Generated from `ChallengeRecorder`'s verification-key and instance absorb events. -/\n");
        out.push_str("def capturedInit : List (TranscriptElt Fp G) :=\n");
        out.push_str(&format!("  [{}]\n\n", captured_init.join(", ")));
        out.push_str("theorem capturedInit_startsWith_vkTranscriptRepr :\n");
        out.push_str("    capturedInit.head? = some (TranscriptElt.scalar capturedVkTranscriptRepr) := by native_decide\n\n");
        out.push_str(
            "/-- Captured verifier Fiat-Shamir schedule, including the captured prefix.\n",
        );
        out.push_str(
            "Generated from `ChallengeRecorder`'s ordered absorb/squeeze events. After each\n",
        );
        out.push_str(
            "squeeze the captured challenge is appended as an abstract separator (mirroring\n",
        );
        out.push_str(
            "`deriveChallenges`); the deployed Blake2b transcript uses a domain-prefix byte. -/\n",
        );
        out.push_str("def capturedScheduleEntries : List (List (TranscriptElt Fp G) × Fp) :=\n");
        out.push_str(&format!("  [{}]\n\n", schedule_entries.join(", ")));

        // ---- captured proof bytes ----
        if let Some(bytes) = proof_bytes {
            out.push_str(
                "/-- The proof string the verifier consumed in this capture, hex-encoded. The\n",
            );
            out.push_str(
                "exporter checked that re-serializing the recorded proof reads (`write_point`,\n",
            );
            out.push_str(
                "`write_scalar`, in read order) reproduces these bytes exactly, so `ps` is their\n",
            );
            out.push_str("canonical parse. -/\n");
            out.push_str(&format!(
                "def capturedProofHex : String :=\n  {:?}\n\n",
                hex_lower(bytes)
            ));
        }

        // ---- captured MSM ----
        let other_lits: Vec<String> = msm_other
            .iter()
            .map(|(s, x, y)| {
                let point: Option<EqAffine> = EqAffine::from_xy(*x, *y).into();
                let point = point.expect("captured MSM base must be a valid affine point");
                format!("({}, {})", fp(*s), points.point_ref(point))
            })
            .collect();
        out.push_str("def capturedMsm : Msm shape.k Fp G := {\n");
        out.push_str(&format!(
            "  gScalars := fun i => (#[{}] : Array Fp)[i.val]!,\n",
            join_fps(&msm_g)
        ));
        out.push_str(&format!("  wScalar := {},\n", fp(msm_w)));
        out.push_str(&format!("  uScalar := {},\n", fp(msm_u)));
        out.push_str(&format!("  other := [{}] }}\n\n", other_lits.join(", ")));

        out.push_str("theorem fingerprint_matches : MsmMatch (assemble vk derivedInstanceCommitment ps ch) capturedMsm := by native_decide\n\n");
        match mode {
            FixtureMode::Honest => {
                out.push_str("theorem capturedMsm_eval_eq_zero : capturedMsm.evalNat capturedURS = 0 := by native_decide\n\n");
                out.push_str(
                    "/-- Meaningful jointly with `fingerprint_matches`: `assemble`'s zero-MSM rejection\n",
                );
                out.push_str(
                    "fallback also evaluates to zero, so this statement alone is not acceptance. -/\n",
                );
                out.push_str("theorem assembledMsm_eval_eq_zero : (assemble vk derivedInstanceCommitment ps ch).evalNat capturedURS = 0 := by\n");
                out.push_str("  rw [msmMatch_evalNat capturedURS fingerprint_matches]\n");
                out.push_str("  exact capturedMsm_eval_eq_zero\n\n");
            }
            // The mirror of the honest eval theorem. `= 0` would be false here, but stating the
            // negation keeps the fixture self-certifying about its own mode — a mislabeled
            // accepting capture or a dead export cannot pass for a non-accepting one — and gives
            // the generated file the aliveness check the honest families get from their eval
            // theorems. The Rust export already fail-fasts on an identity capture; this is the
            // same fact, checked again by Lean against the emitted data.
            FixtureMode::MatchOnly => {
                out.push_str(
                    "/-- The captured MSM does not evaluate to the identity: this capture is\n",
                );
                out.push_str("genuinely non-accepting, so a mislabeled accepting capture or dead export plumbing\n");
                out.push_str("cannot pass for it. -/\n");
                out.push_str("theorem capturedMsm_evalNat_ne_zero : capturedMsm.evalNat capturedURS ≠ 0 := by native_decide\n\n");
            }
        }
        out.push_str(&format!("end {}\n", lean_namespace));

        let body = out;
        let point_coordinates = points.coordinate_literals();
        let mut out = String::new();
        match mode {
            FixtureMode::Honest => out.push_str(
                "-- Auto-generated by halo2 `dump_vesta_lean_fixture`. Do not edit by hand.\n",
            ),
            FixtureMode::MatchOnly => {
                out.push_str("-- Auto-generated by halo2 `dump_vesta_lean_fixture_match_only`. Do not edit by hand.\n");
                out.push_str("-- Match-only capture: the deployed verifier ran to completion, but the captured\n");
                out.push_str(
                    "-- MSM is deliberately NOT the identity (a non-accepting run), as proved below\n",
                );
                out.push_str(
                    "-- by `capturedMsm_evalNat_ne_zero`. This fixture witnesses coefficient\n",
                );
                out.push_str(
                    "-- agreement; what made the run non-accepting is the caller's business and is\n",
                );
                out.push_str("-- not attested here.\n");
            }
        }
        // `maxRecDepth` is raised for the deeply-nested literals (the 2048-element URS and
        // g-scalar arrays, point-coordinate validation, and gate Expr trees) that `native_decide`
        // compiles.
        out.push_str(&format!(
            "import Zcash.Snark\n\nset_option maxRecDepth 1000000\n\nnamespace {}\n\nopen Zcash.Snark\nopen CompElliptic.CurveForms.ShortWeierstrass\nopen CompElliptic.Curves.Pasta\nopen CompElliptic.Fields.Pasta\n\n",
            lean_namespace
        ));
        out.push_str("/-- Scalar field element from four little-endian u64 limbs. -/\n");
        out.push_str("def mkFp (a b c d : ℕ) : Fp := (a : Fp) + (b : Fp) * (2 : Fp) ^ 64 + (c : Fp) * (2 : Fp) ^ 128 + (d : Fp) * (2 : Fp) ^ 192\n\n");
        out.push_str("abbrev Fq := VestaBaseField\n\n");
        out.push_str("/-- Vesta base field element from four little-endian u64 limbs. -/\n");
        out.push_str("def mkFq (a b c d : ℕ) : Fq := (a : Fq) + (b : Fq) * (2 : Fq) ^ 64 + (c : Fq) * (2 : Fq) ^ 128 + (d : Fq) * (2 : Fq) ^ 192\n\n");
        out.push_str("abbrev G := VestaG\n\n");
        out.push_str("instance : Inhabited G := ⟨0⟩\n\n");
        out.push_str(&format!(
            "def capturedCircuitId : String := {:?}\n\n",
            circuit_id
        ));
        out.push_str("/-- Canonical affine coordinates for every distinct Vesta point used by this fixture. -/\n");
        out.push_str(&format!(
            "def capturedPointCoordinates : List (Fq × Fq) := [{}]\n\n",
            point_coordinates.join(", ")
        ));
        out.push_str("def capturedPointCoordinatesValid : Bool :=\n");
        out.push_str(
            "  capturedPointCoordinates.all fun p => decide (OnCurve Vesta.a Vesta.b p) || decide (p = (0, 0))\n\n",
        );
        out.push_str("theorem capturedPointCoordinatesValid_eq_true : capturedPointCoordinatesValid = true := by native_decide\n\n");
        out.push_str("def mkVestaPoint (p : Fq × Fq) : G :=\n");
        out.push_str("  if h : OnCurve Vesta.a Vesta.b p then ⟨p.1, p.2, Or.inl h⟩\n");
        out.push_str("  else if h0 : p = (0, 0) then ⟨p.1, p.2, Or.inr h0⟩ else 0\n\n");
        out.push_str(
            "def capturedPoints : List G := capturedPointCoordinates.map mkVestaPoint\n\n",
        );
        out.push_str("def capturedPoint (i : ℕ) : G := capturedPoints.getD i 0\n\n");
        out.push_str(&body);
        out
    }
}

#[cfg(test)]
mod tests {
    use group::prime::PrimeCurveAffine;

    use super::*;

    #[test]
    fn transcript_schedule_records_squeeze_before_first_proof_read() {
        let vk_repr = Fp::from(1);
        let theta = Fp::from(2);
        let beta = Fp::from(3);
        let events = [
            TranscriptEvent::CommonScalar(vk_repr),
            // A zero-advice circuit squeezes theta without first reading an advice commitment.
            TranscriptEvent::Squeeze(theta),
            TranscriptEvent::ReadPoint(EqAffine::generator()),
            TranscriptEvent::Squeeze(beta),
        ];
        let mut points = PointTable::new();

        let (captured_init, schedule_entries) = render_transcript_capture(&mut points, &events, 1);

        assert_eq!(captured_init, vec![transcript_scalar(vk_repr)]);
        assert_eq!(schedule_entries.len(), 2);
        assert!(schedule_entries[0].ends_with(&format!(", {})", fp(theta))));
        assert!(schedule_entries[1].ends_with(&format!(", {})", fp(beta))));
    }

    #[test]
    fn proof_reads_reserialize_as_the_prover_writes() {
        let point = EqAffine::generator();
        let scalar = Fp::from(7);
        let events = [
            TranscriptEvent::CommonScalar(Fp::from(1)),
            TranscriptEvent::ReadPoint(point),
            TranscriptEvent::Squeeze(Fp::from(2)),
            TranscriptEvent::ReadScalar(scalar),
        ];

        let mut expected = Blake2bWrite::<Vec<u8>, EqAffine, Challenge255<EqAffine>>::init(vec![]);
        expected.write_point(point).unwrap();
        expected.write_scalar(scalar).unwrap();
        let expected = expected.finalize();

        assert_eq!(reserialize_proof_reads(&events), expected);
        assert_eq!(expected.len(), 64);
        assert_eq!(hex_lower(&[0x00, 0x6e, 0xff]), "006eff");
    }

    #[test]
    fn lean_identifier_grammar() {
        // Accepted: letter/underscore start, ASCII alphanumerics and `_` thereafter.
        assert!(is_lean_ident("Halo2"));
        assert!(is_lean_ident("_private"));
        assert!(is_lean_ident("Fixture0"));
        // Rejected: empty, digit-leading (the case release builds previously waved through), and
        // any non-`_` punctuation (a dotted path is validated segment-by-segment by the caller).
        assert!(!is_lean_ident(""));
        assert!(!is_lean_ident("123"));
        assert!(!is_lean_ident("0abc"));
        assert!(!is_lean_ident("has-hyphen"));
        assert!(!is_lean_ident("has.dot"));
        assert!(!is_lean_ident("has space"));
    }
}
