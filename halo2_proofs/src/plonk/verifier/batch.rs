use group::ff::{Field, FromUniformBytes};
use pasta_curves::arithmetic::CurveAffine;
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

use super::{verify_proof_with_instance_commitments, InstanceCommitments, VerificationStrategy};
use crate::{
    plonk::{Error, VerifyingKey},
    poly::commitment::{Guard, Params, MSM},
    transcript::{Blake2bRead, EncodedChallenge},
};

use maybe_rayon::iter::IntoParallelIterator;
#[cfg(feature = "multicore")]
use maybe_rayon::iter::{IndexedParallelIterator, ParallelIterator};

#[cfg(test)]
mod tests;

/// A proof verification strategy that returns the proof's MSM.
///
/// `BatchVerifier` handles the accumulation of the MSMs for the batched proofs.
#[derive(Debug)]
struct BatchStrategy<'params, C: CurveAffine> {
    msm: MSM<'params, C>,
    batching_scalar: C::Scalar,
}

impl<'params, C: CurveAffine> BatchStrategy<'params, C> {
    fn new(params: &'params Params<C>, batching_scalar: C::Scalar) -> Self {
        BatchStrategy {
            msm: MSM::new(params),
            batching_scalar,
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
        Ok(guard.use_challenges_with_scale(self.batching_scalar))
    }
}

#[derive(Debug)]
struct BatchItem<C: CurveAffine> {
    instances: Vec<Vec<Vec<C::Scalar>>>,
    proof: Vec<u8>,
}

/// `iter.try_fold().try_reduce()` for a parallel iterator of results, or
/// `Iterator::try_fold` without the `multicore` feature. The item type must be
/// `Result<T, E>`: `fold_op` must satisfy both rayon's `try_fold` and
/// `try_reduce`.
trait TryFoldAndReduce<T, E> {
    fn try_fold_and_reduce(
        self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(T, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E>;
}

#[cfg(feature = "multicore")]
impl<T, E, I> TryFoldAndReduce<T, E> for I
where
    T: Send + Sync,
    E: Send + Sync,
    I: maybe_rayon::iter::ParallelIterator<Item = Result<T, E>>,
{
    fn try_fold_and_reduce(
        self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(T, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E> {
        self.try_fold(&identity, &fold_op)
            .try_reduce(&identity, |a, b| fold_op(a, Ok(b)))
    }
}

#[cfg(not(feature = "multicore"))]
impl<T, E, I> TryFoldAndReduce<T, E> for I
where
    I: std::iter::Iterator<Item = Result<T, E>>,
{
    fn try_fold_and_reduce(
        mut self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(T, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E> {
        self.try_fold(identity(), fold_op)
    }
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
            acc.add_msm_batch(msm);
            acc
        }

        let instances = self
            .items
            .iter()
            .map(|item| &item.instances)
            .collect::<Vec<_>>();
        let instance_commitments = match InstanceCommitments::for_proofs(params, vk, &instances) {
            Ok(instance_commitments) => instance_commitments,
            Err(e) => {
                tracing::debug!("Batch failed instance validation: {}", e);
                return false;
            }
        };

        let final_msm = self
            .items
            .into_par_iter()
            .zip(instance_commitments.into_par_iter())
            .enumerate()
            .map(|(i, (item, instance_commitments))| {
                // All proof equations are fixed before sampling these independent
                // weights. Fix the first weight at one; randomize every later
                // equation so invalid proofs cannot deliberately cancel.
                let rho = if i == 0 {
                    C::Scalar::ONE
                } else {
                    C::Scalar::random(&mut UnwrapErr(SysRng))
                };
                let strategy = BatchStrategy::new(params, rho);
                let mut transcript = Blake2bRead::init(&item.proof[..]);
                verify_proof_with_instance_commitments(
                    params,
                    vk,
                    strategy,
                    instance_commitments,
                    &mut transcript,
                )
                .map_err(|e| {
                    tracing::debug!("Batch item {} failed verification: {}", i, e);
                    e
                })
            })
            .try_fold_and_reduce(
                || params.empty_msm(),
                |acc, res| res.map(|proof_msm| accumulate_msm(acc, proof_msm)),
            );

        match final_msm {
            Ok(msm) => msm.eval(),
            Err(_) => false,
        }
    }
}
