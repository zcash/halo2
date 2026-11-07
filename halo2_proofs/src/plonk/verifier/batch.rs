use group::ff::{Field, FromUniformBytes};
use pasta_curves::arithmetic::CurveAffine;
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

use super::{verify_proof, VerificationStrategy};
use crate::{
    multicore::{IntoParallelIterator, TryFoldAndReduce},
    plonk::{Error, ProofPointLayouts, VerifyingKey},
    poly::commitment::{Guard, Params, MSM},
    transcript::{EncodedChallenge, PrefetchedBlake2bRead},
};

#[cfg(feature = "multicore")]
use crate::multicore::{IndexedParallelIterator, ParallelIterator};

/// A proof verification strategy that returns the proof's MSM.
///
/// `BatchVerifier` handles the accumulation of the MSMs for the batched proofs.
#[derive(Debug)]
struct BatchStrategy<'params, C: CurveAffine> {
    msm: MSM<'params, C>,
}

impl<'params, C: CurveAffine> BatchStrategy<'params, C> {
    fn new(params: &'params Params<C>) -> Self {
        BatchStrategy {
            msm: MSM::new(params),
        }
    }
}

impl<'params, C: CurveAffine> VerificationStrategy<'params, C> for BatchStrategy<'params, C> {
    type Output = MSM<'params, C>;

    fn process<E: EncodedChallenge<C>>(
        self,
        f: impl FnOnce(MSM<'params, C>) -> Result<Guard<'params, C, E>, Error>,
    ) -> Result<Self::Output, Error> {
        let guard = f(self.msm)?;
        Ok(guard.use_challenges())
    }
}

#[derive(Debug)]
struct BatchItem<C: CurveAffine> {
    instances: Vec<Vec<Vec<C::Scalar>>>,
    proof: Vec<u8>,
}

/// Decodes complete encodings only; transcript reads retain truncation errors.
fn prefetch_points<C: CurveAffine>(
    proof: &[u8],
    ranges: &[std::ops::Range<usize>],
) -> Vec<(usize, Option<C>)> {
    let len = C::Repr::default().as_ref().len();
    if len == 0 {
        return Vec::new();
    }
    let (offsets, encodings): (Vec<usize>, Vec<C::Repr>) = ranges
        .iter()
        .filter_map(|range| {
            let bytes = proof.get(range.start..range.end.min(proof.len()))?;
            Some((range.start, bytes))
        })
        .flat_map(|(start, bytes)| {
            bytes.chunks_exact(len).enumerate().map(move |(i, bytes)| {
                let mut repr = C::Repr::default();
                repr.as_mut().copy_from_slice(bytes);
                (start + i * len, repr)
            })
        })
        .unzip();
    offsets
        .into_iter()
        .zip(C::batch_decode(&encodings))
        .collect()
}

/// A verifier that checks multiple proofs in a batch. **This requires the
/// `batch` crate feature to be enabled.**
#[derive(Debug, Default)]
pub struct BatchVerifier<C: CurveAffine> {
    items: Vec<BatchItem<C>>,
}

impl<C: CurveAffine> BatchVerifier<C> {
    /// Constructs a new batch verifier.
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Adds a proof to the batch.
    pub fn add_proof(&mut self, instances: Vec<Vec<Vec<C::Scalar>>>, proof: Vec<u8>) {
        self.items.push(BatchItem { instances, proof })
    }
}

impl<C: CurveAffine> BatchVerifier<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    ///
    /// This uses [`SysRng`] internally instead of taking an `R: Rng` argument, because
    /// the internal parallelization requires access to a RNG that is guaranteed to not
    /// clone its internal state when shared between threads.
    pub fn finalize(self, params: &Params<C>, vk: &VerifyingKey<C>) -> bool {
        fn accumulate_msm<'params, C: CurveAffine>(
            mut acc: MSM<'params, C>,
            msm: MSM<'params, C>,
        ) -> MSM<'params, C> {
            // Scale the MSM by a random factor to ensure that if the existing MSM has
            // `is_zero() == false` then this argument won't be able to interfere with it
            // to make it true, with high probability.
            acc.scale(C::Scalar::random(&mut UnwrapErr(SysRng)));

            acc.add_msm(&msm);
            acc
        }

        // Parsing success alone does not establish proof validity. Keep candidates
        // bounded and private until the batch's final MSM check succeeds.
        let learned_layouts = ProofPointLayouts::default();
        let final_msm = self
            .items
            .into_par_iter()
            .enumerate()
            .map(|(i, item)| {
                let instances: Vec<Vec<_>> = item
                    .instances
                    .iter()
                    .map(|i| i.iter().map(|c| &c[..]).collect())
                    .collect();
                let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

                let strategy = BatchStrategy::new(params);
                let circuits = item.instances.len();
                let mut transcript = match vk.proof_point_layouts().get(circuits) {
                    Some(ranges) => PrefetchedBlake2bRead::with_prefetched(
                        &item.proof,
                        prefetch_points(&item.proof, &ranges),
                    ),
                    None => PrefetchedBlake2bRead::recording(&item.proof),
                };
                let result = verify_proof(params, vk, strategy, &instances, &mut transcript);
                if result.is_ok() {
                    if let Some(ranges) = transcript.into_recorded() {
                        learned_layouts.insert(circuits, ranges);
                    }
                }
                result.map_err(|e| {
                    tracing::debug!("Batch item {} failed verification: {}", i, e);
                    e
                })
            })
            .try_fold_and_reduce(
                || params.empty_msm(),
                |acc, res| res.map(|proof_msm| accumulate_msm(acc, proof_msm)),
            );

        let valid = final_msm.is_ok_and(|msm| msm.eval());
        if valid {
            learned_layouts.install(vk.proof_point_layouts());
        }
        valid
    }
}

#[cfg(test)]
mod tests;
