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
    /// Bounded public proof layouts, shared by clones and excluded from the pinned key.
    #[cfg(feature = "batch")]
    proof_point_layouts: Arc<ProofPointLayouts>,
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

/// Point runs indexed by circuit-instance count for this verifying key.
///
/// The verifier's sequence of point and scalar reads depends only on the key
/// and the circuit-instance count. Only a successfully verified batch installs
/// its recorded layouts. A missing or mismatched layout falls back to ordinary
/// decoding at every offset that was not prefetched.
#[cfg(feature = "batch")]
#[derive(Debug, Default)]
pub(crate) struct ProofPointLayouts(std::sync::Mutex<ProofPointLayoutCache>);

/// Fixed slots replaced in insertion order, so early shapes cannot pin the cache.
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
    /// Each retained shape has at most [`crate::MAX_CACHED_PROOF_POINT_RUNS`] runs.
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
        if ranges.len() <= crate::MAX_CACHED_PROOF_POINT_RUNS {
            self.insert_layout(ProofPointLayout {
                circuits,
                ranges: ranges.into(),
            });
        }
    }

    fn insert_layout(&self, layout: ProofPointLayout) {
        let mut layouts = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if layouts
            .layouts
            .iter()
            .flatten()
            .any(|existing| existing.circuits == layout.circuits)
        {
            return;
        }
        let next = layouts.next;
        layouts.layouts[next] = Some(layout);
        layouts.next = (next + 1) % Self::MAX_SHAPES;
    }

    /// Publishes bounded candidate layouts only after the batch's final MSM check.
    pub(crate) fn install(self, destination: &Self) {
        let layouts = self
            .0
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for layout in layouts.layouts.into_iter().flatten() {
            destination.insert_layout(layout);
        }
    }
}
