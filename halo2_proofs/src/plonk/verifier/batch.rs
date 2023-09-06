use ff::FromUniformBytes;
use group::ff::Field;
use halo2curves::CurveAffine;
use rand_core::OsRng;

use super::{verify_proof, VerificationStrategy};
use crate::{
    multicore::{IntoParallelIterator, TryFoldAndReduce},
    plonk::{Error, VerifyingKey},
    poly::{
        commitment::{Params, MSM},
        ipa::{
            commitment::{IPACommitmentScheme, ParamsVerifierIPA},
            msm::MSMIPA,
            multiopen::VerifierIPA,
            strategy::GuardIPA,
        },
    },
    transcript::{Blake2bRead, TranscriptReadBuffer},
};

#[cfg(feature = "multicore")]
use crate::multicore::{IndexedParallelIterator, ParallelIterator};

/// A proof verification strategy that returns the proof's MSM.
///
/// `BatchVerifier` handles the accumulation of the MSMs for the batched proofs.
#[derive(Debug)]
struct BatchStrategy<'params, C: CurveAffine> {
    msm: MSMIPA<'params, C>,
}

impl<'params, C: CurveAffine>
    VerificationStrategy<'params, IPACommitmentScheme<C>, VerifierIPA<'params, C>>
    for BatchStrategy<'params, C>
{
    type Output = MSMIPA<'params, C>;

    fn new(params: &'params ParamsVerifierIPA<C>) -> Self {
        BatchStrategy {
            msm: MSMIPA::new(params),
        }
    }

    fn process(
        self,
        f: impl FnOnce(MSMIPA<'params, C>) -> Result<GuardIPA<'params, C>, Error>,
    ) -> Result<Self::Output, Error> {
        let guard = f(self.msm)?;
        Ok(guard.use_challenges())
    }

    fn finalize(self) -> bool {
        unreachable!()
    }
}

#[derive(Debug)]
struct BatchItem<C: CurveAffine> {
    instances: Vec<Vec<Vec<C::ScalarExt>>>,
    proof: Vec<u8>,
}

/// A verifier that checks multiple proofs in a batch. **This requires the
/// `batch` crate feature to be enabled.**
#[derive(Debug, Default)]
pub struct BatchVerifier<C: CurveAffine> {
    items: Vec<BatchItem<C>>,
}

impl<C: CurveAffine> BatchVerifier<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    /// Constructs a new batch verifier.
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Adds a proof to the batch.
    pub fn add_proof(&mut self, instances: Vec<Vec<Vec<C::Scalar>>>, proof: Vec<u8>) {
        self.items.push(BatchItem { instances, proof })
    }

    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    ///
    /// This uses [`OsRng`] internally instead of taking an `R: RngCore` argument, because
    /// the internal parallelization requires access to a RNG that is guaranteed to not
    /// clone its internal state when shared between threads.
    pub fn finalize(self, params: &ParamsVerifierIPA<C>, vk: &VerifyingKey<C>) -> bool {
        fn accumulate_msm<'params, C: CurveAffine>(
            mut acc: MSMIPA<'params, C>,
            msm: MSMIPA<'params, C>,
        ) -> MSMIPA<'params, C> {
            // Scale the MSM by a random factor to ensure that if the existing MSM has
            // `is_zero() == false` then this argument won't be able to interfere with it
            // to make it true, with high probability.
            acc.scale(C::Scalar::random(OsRng));

            acc.add_msm(&msm);
            acc
        }

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
                let mut transcript = Blake2bRead::init(&item.proof[..]);
                verify_proof(params, vk, strategy, &instances, &mut transcript).map_err(|e| {
                    tracing::debug!("Batch item {} failed verification: {}", i, e);
                    e
                })
            })
            .try_fold_and_reduce(
                || params.empty_msm(),
                |acc, res| res.map(|proof_msm| accumulate_msm(acc, proof_msm)),
            );

        match final_msm {
            Ok(msm) => msm.check(),
            Err(_) => false,
        }
    }
}
