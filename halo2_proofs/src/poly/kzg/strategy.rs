use std::{fmt::Debug, marker::PhantomData};

use super::{
    commitment::{KZGCommitmentScheme, ParamsKZG},
    msm::{DualMSM, MSMKZG},
    multiopen::VerifierGWC,
};
use crate::{
    plonk::Error,
    poly::{
        commitment::{Verifier, MSM},
        ipa::msm::MSMIPA,
        strategy::{Guard, VerificationStrategy},
    },
    transcript::{EncodedChallenge, TranscriptRead},
};
use ff::Field;
use group::Group;
use halo2curves::{
    pairing::{Engine, MillerLoopResult, MultiMillerLoop},
    CurveAffine,
};
use rand_core::OsRng;

/// Wrapper for linear verification accumulator
#[derive(Debug, Clone)]
pub struct GuardKZG<'params, E: MultiMillerLoop + Debug> {
    pub(crate) msm_accumulator: DualMSM<'params, E>,
}

/// Define accumulator type as `DualMSM`
impl<'params, E: MultiMillerLoop + Debug> Guard<KZGCommitmentScheme<E>> for GuardKZG<'params, E> {
    type MSMAccumulator = DualMSM<'params, E>;
}

/// KZG specific operations
impl<'params, E: MultiMillerLoop + Debug> GuardKZG<'params, E> {
    pub(crate) fn new(msm_accumulator: DualMSM<'params, E>) -> Self {
        Self { msm_accumulator }
    }
}

/// A verifier that checks multiple proofs in a batch
#[derive(Clone, Debug)]
pub struct AccumulatorStrategy<'params, E: Engine> {
    pub(crate) msm_accumulator: DualMSM<'params, E>,
}

impl<'params, E: MultiMillerLoop + Debug> AccumulatorStrategy<'params, E> {
    /// Constructs an empty batch verifier
    pub fn new(params: &'params ParamsKZG<E>) -> Self {
        AccumulatorStrategy {
            msm_accumulator: DualMSM::new(params),
        }
    }

    /// Constructs and initialized new batch verifier
    pub fn with(msm_accumulator: DualMSM<'params, E>) -> Self {
        AccumulatorStrategy { msm_accumulator }
    }
}

/// A verifier that checks a single proof
#[derive(Clone, Debug)]
pub struct SingleStrategy<'params, E: Engine> {
    pub(crate) msm: DualMSM<'params, E>,
}

impl<'params, E: MultiMillerLoop + Debug> SingleStrategy<'params, E> {
    /// Constructs an empty batch verifier
    pub fn new(params: &'params ParamsKZG<E>) -> Self {
        SingleStrategy {
            msm: DualMSM::new(params),
        }
    }
}

impl<
        'params,
        E: MultiMillerLoop + Debug,
        V: Verifier<
            'params,
            KZGCommitmentScheme<E>,
            MSMAccumulator = DualMSM<'params, E>,
            Guard = GuardKZG<'params, E>,
        >,
    > VerificationStrategy<'params, KZGCommitmentScheme<E>, V> for AccumulatorStrategy<'params, E>
{
    type Output = Self;

    fn new(params: &'params ParamsKZG<E>) -> Self {
        AccumulatorStrategy::new(params)
    }

    fn process(
        mut self,
        f: impl FnOnce(V::MSMAccumulator) -> Result<V::Guard, Error>,
    ) -> Result<Self::Output, Error> {
        self.msm_accumulator.scale(E::Scalar::random(OsRng));

        // Guard is updated with new msm contributions
        let guard = f(self.msm_accumulator)?;
        Ok(Self {
            msm_accumulator: guard.msm_accumulator,
        })
    }

    fn finalize(self) -> bool {
        self.msm_accumulator.check()
    }
}

impl<
        'params,
        E: MultiMillerLoop + Debug,
        V: Verifier<
            'params,
            KZGCommitmentScheme<E>,
            MSMAccumulator = DualMSM<'params, E>,
            Guard = GuardKZG<'params, E>,
        >,
    > VerificationStrategy<'params, KZGCommitmentScheme<E>, V> for SingleStrategy<'params, E>
{
    type Output = ();

    fn new(params: &'params ParamsKZG<E>) -> Self {
        Self::new(params)
    }

    fn process(
        self,
        f: impl FnOnce(V::MSMAccumulator) -> Result<V::Guard, Error>,
    ) -> Result<Self::Output, Error> {
        // Guard is updated with new msm contributions
        let guard = f(self.msm)?;
        let msm = guard.msm_accumulator;
        if msm.check() {
            Ok(())
        } else {
            Err(Error::ConstraintSystemFailure)
        }
    }

    fn finalize(self) -> bool {
        unreachable!();
    }
}
