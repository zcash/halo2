use group::ff::Field;
use pasta_curves::arithmetic::CurveAffine;
use rand_core::OsRng;

use super::{verify_proof, VerificationStrategy};
use crate::{
    multicore::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    plonk::{Error, VerifyingKey},
    poly::commitment::{Guard, Params, MSM},
    transcript::{Blake2bRead, EncodedChallenge},
};

/// A proof verification strategy that returns the proof's MSM.
///
/// `BatchVerifier` handles the accumulation of the MSMs for the batched proofs.
#[derive(Debug)]
struct BatchStrategy<C: CurveAffine> {
    msm: MSM<C>,
}

impl<C: CurveAffine> BatchStrategy<C> {
    fn new(n: u64) -> Self {
        BatchStrategy { msm: MSM::new(n) }
    }
}

impl<C: CurveAffine> VerificationStrategy<C> for BatchStrategy<C> {
    type Output = MSM<C>;

    fn process<E: EncodedChallenge<C>>(
        self,
        _: &Params<C>,
        f: impl FnOnce(MSM<C>) -> Result<Guard<C, E>, Error>,
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

    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    ///
    /// This uses [`OsRng`] internally instead of taking an `R: RngCore` argument, because
    /// the internal parallelization requires access to a RNG that is guaranteed to not
    /// clone its internal state when shared between threads.
    pub fn finalize(self, params: &Params<C>, vk: &VerifyingKey<C>) -> bool {
        fn accumulate_msm<C: CurveAffine>(mut acc: MSM<C>, msm: MSM<C>) -> MSM<C> {
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

                let strategy = BatchStrategy::new(params.n);
                let mut transcript = Blake2bRead::init(&item.proof[..]);
                verify_proof(params, vk, strategy, &instances, &mut transcript).map_err(|e| {
                    tracing::debug!("Batch item {} failed verification: {}", i, e);
                    e
                })
            })
            .try_fold(
                || params.empty_msm(),
                |msm, res| res.map(|proof_msm| accumulate_msm(msm, proof_msm)),
            )
            .try_reduce(|| params.empty_msm(), |a, b| Ok(accumulate_msm(a, b)));

        match final_msm {
            Ok(msm) => msm.eval(params),
            Err(_) => false,
        }
    }
}
