//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;
use group::ff::{Field, FromUniformBytes, PrimeField};

use crate::arithmetic::CurveAffine;
use crate::poly::{
    Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, PinnedEvaluationDomain,
    Polynomial,
};
use crate::transcript::{ChallengeScalar, EncodedChallenge, Transcript};

mod assigned;
mod circuit;
mod error;
mod keygen;
mod lookup;
pub(crate) mod permutation;
mod vanishing;

mod prover;
mod verifier;

#[cfg(feature = "unstable-verifier-fingerprint")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-verifier-fingerprint")))]
pub mod fingerprint;

pub use assigned::*;
pub use circuit::*;
pub use error::*;
pub use keygen::*;
pub use prover::*;
pub use verifier::*;

use std::io;
#[cfg(feature = "batch")]
use std::sync::Arc;

/// This is a verifying key which allows for the verification of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    permutation: permutation::VerifyingKey<C>,
    cs: ConstraintSystem<C::Scalar>,
    /// Cached maximum degree of `cs` (which doesn't change after construction).
    cs_degree: usize,
    /// The representative of this `VerifyingKey` in transcripts.
    transcript_repr: C::Scalar,
    /// Where proofs' points are, learned by the batch verifier; shared by
    /// clones and not part of [`Self::pinned`].
    #[cfg(feature = "batch")]
    proof_point_layouts: Arc<ProofPointLayouts>,
}

/// The byte ranges of a proof's point runs, by the number of circuit instances
/// the proof covers.
///
/// A proof's sequence of point and scalar reads is fixed by the verifying key
/// and its circuit-instance count, so the runs recorded while one proof of
/// a shape is read completely hold for every proof of that shape, valid or
/// not (validity is decided later, by the batch's final check). The batch
/// verifier uses them to decode a proof's points together before reading it;
/// a wrong layout would only cost speed, since a point read at an offset that
/// was not decoded ahead decodes as usual.
#[cfg(feature = "batch")]
#[derive(Debug, Default)]
pub(crate) struct ProofPointLayouts(std::sync::Mutex<ProofPointLayoutCache>);

/// Fixed slots replaced in insertion order, so early shapes cannot pin the
/// cache for the verifying key's lifetime.
#[cfg(feature = "batch")]
#[derive(Debug)]
struct ProofPointLayoutCache {
    layouts: [Option<ProofPointLayout>; ProofPointLayouts::MAX_SHAPES],
    next: usize,
}

#[cfg(feature = "batch")]
#[derive(Debug)]
struct ProofPointLayout {
    circuits: usize,
    ranges: Arc<[std::ops::Range<usize>]>,
}

#[cfg(feature = "batch")]
impl Default for ProofPointLayoutCache {
    fn default() -> Self {
        Self {
            layouts: [const { None }; ProofPointLayouts::MAX_SHAPES],
            next: 0,
        }
    }
}

#[cfg(feature = "batch")]
impl<C: CurveAffine> VerifyingKey<C> {
    pub(crate) fn proof_point_layouts(&self) -> &ProofPointLayouts {
        &self.proof_point_layouts
    }
}

#[cfg(feature = "batch")]
impl ProofPointLayouts {
    /// At most this many shapes are kept; each has at most
    /// [`crate::MAX_CACHED_PROOF_POINT_RUNS`] runs. Orchard proofs differ only
    /// in their action count.
    const MAX_SHAPES: usize = 256;

    pub(crate) fn get(&self, circuits: usize) -> Option<Arc<[std::ops::Range<usize>]>> {
        let layouts = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        layouts
            .layouts
            .iter()
            .flatten()
            .find(|layout| layout.circuits == circuits)
            .map(|layout| Arc::clone(&layout.ranges))
    }

    pub(crate) fn insert(&self, circuits: usize, ranges: Vec<std::ops::Range<usize>>) {
        if ranges.len() > crate::MAX_CACHED_PROOF_POINT_RUNS {
            return;
        }
        let mut layouts = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if layouts
            .layouts
            .iter()
            .flatten()
            .any(|layout| layout.circuits == circuits)
        {
            return;
        }
        let next = layouts.next;
        layouts.layouts[next] = Some(ProofPointLayout {
            circuits,
            ranges: ranges.into(),
        });
        layouts.next = (next + 1) % Self::MAX_SHAPES;
    }
}

#[cfg(all(test, feature = "batch"))]
mod proof_point_layout_tests {
    use super::ProofPointLayouts;

    #[test]
    fn replacement_does_not_change_in_flight_layouts() {
        let layouts = ProofPointLayouts::default();
        layouts.insert(0, vec![0..32, 64..96]);
        let in_flight = layouts.get(0).unwrap();
        // Re-learning the same shape must neither replace it nor advance
        // the replacement cursor.
        layouts.insert(0, std::iter::once(32..64).collect());
        for shape in 1..ProofPointLayouts::MAX_SHAPES {
            layouts.insert(shape, std::iter::once(shape..shape + 1).collect());
        }
        assert_eq!(&*layouts.get(0).unwrap(), &[0..32, 64..96]);
        layouts.insert(
            ProofPointLayouts::MAX_SHAPES,
            std::iter::once(128..160).collect(),
        );
        assert!(layouts.get(0).is_none());
        assert_eq!(&*in_flight, &[0..32, 64..96]);
        let replacement = layouts.get(ProofPointLayouts::MAX_SHAPES).unwrap();
        assert_eq!(replacement.len(), 1);
        assert_eq!(replacement[0], 128..160);
        assert!(layouts.get(1).is_some());
    }

    #[test]
    fn oversized_layouts_do_not_displace_cached_shapes() {
        let layouts = ProofPointLayouts::default();
        let limit = crate::MAX_CACHED_PROOF_POINT_RUNS;
        let ranges = |len| (0..len).map(|i| i * 64..i * 64 + 32).collect();
        layouts.insert(0, ranges(limit));
        assert_eq!(layouts.get(0).unwrap().len(), limit);
        for shape in 1..=ProofPointLayouts::MAX_SHAPES {
            layouts.insert(shape, ranges(limit + 1));
            assert!(layouts.get(shape).is_none());
        }
        assert_eq!(layouts.get(0).unwrap().len(), limit);
    }
}

impl<C: CurveAffine> VerifyingKey<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    fn from_parts(
        domain: EvaluationDomain<C::Scalar>,
        fixed_commitments: Vec<C>,
        permutation: permutation::VerifyingKey<C>,
        cs: ConstraintSystem<C::Scalar>,
    ) -> Self {
        // Compute cached values.
        let cs_degree = cs.degree();

        let mut vk = Self {
            domain,
            fixed_commitments,
            permutation,
            cs,
            cs_degree,
            // Temporary, this is not pinned.
            transcript_repr: C::Scalar::ZERO,
            #[cfg(feature = "batch")]
            proof_point_layouts: Arc::default(),
        };

        let mut hasher = Blake2bParams::new()
            .hash_length(64)
            .personal(b"Halo2-Verify-Key")
            .to_state();

        let s = format!("{:?}", vk.pinned());

        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());

        // Hash in final Blake2bState
        vk.transcript_repr = C::Scalar::from_uniform_bytes(hasher.finalize().as_array());

        vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Hashes a verification key into a transcript.
    pub fn hash_into<E: EncodedChallenge<C>, T: Transcript<C, E>>(
        &self,
        transcript: &mut T,
    ) -> io::Result<()> {
        transcript.common_scalar(self.transcript_repr)?;

        Ok(())
    }

    /// Obtains a pinned representation of this verification key that contains
    /// the minimal information necessary to reconstruct the verification key.
    pub fn pinned(&self) -> PinnedVerificationKey<'_, C> {
        PinnedVerificationKey {
            base_modulus: C::Base::MODULUS,
            scalar_modulus: C::Scalar::MODULUS,
            domain: self.domain.pinned(),
            fixed_commitments: &self.fixed_commitments,
            permutation: &self.permutation,
            cs: self.cs.pinned(),
        }
    }
}

/// Minimal representation of a verification key that can be used to identify
/// its active contents.
#[allow(dead_code)]
#[derive(Debug)]
pub struct PinnedVerificationKey<'a, C: CurveAffine> {
    base_modulus: &'static str,
    scalar_modulus: &'static str,
    domain: PinnedEvaluationDomain<'a, C::Scalar>,
    cs: PinnedConstraintSystem<'a, C::Scalar>,
    fixed_commitments: &'a Vec<C>,
    permutation: &'a permutation::VerifyingKey<C>,
}
/// This is a proving key which allows for the creation of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct ProvingKey<C: CurveAffine> {
    vk: VerifyingKey<C>,
    l0: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_blind: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_last: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation: permutation::ProvingKey<C>,
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Get the underlying [`VerifyingKey`].
    pub fn get_vk(&self) -> &VerifyingKey<C> {
        &self.vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Get the underlying [`EvaluationDomain`].
    pub fn get_domain(&self) -> &EvaluationDomain<C::Scalar> {
        &self.domain
    }
}

#[derive(Clone, Copy, Debug)]
struct Theta;
type ChallengeTheta<F> = ChallengeScalar<F, Theta>;

#[derive(Clone, Copy, Debug)]
struct Beta;
type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
struct Gamma;
type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
struct Y;
type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
struct X;
type ChallengeX<F> = ChallengeScalar<F, X>;
