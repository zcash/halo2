use super::{
    commitment::{KZGCommitmentScheme, ParamsVerifierKZG},
    msm::DualMSM,
};
use crate::{
    helpers::SerdeCurveAffine,
    plonk::Error,
    poly::{
        commitment::Verifier,
        strategy::{Guard, VerificationStrategy},
    },
};
use halo2_middleware::ff::Field;
use halo2_middleware::zal::impls::H2cEngine;
use halo2curves::{
    pairing::{Engine, MultiMillerLoop},
    CurveAffine, CurveExt,
};
use rand_core::OsRng;
use std::fmt::Debug;

/// Wrapper for linear verification accumulator
#[derive(Debug, Clone)]
pub struct GuardKZG<E: MultiMillerLoop>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm_accumulator: DualMSM<E>,
}

/// Define accumulator type as `DualMSM`
impl<E> Guard<KZGCommitmentScheme<E>> for GuardKZG<E>
where
    E: MultiMillerLoop + Debug,
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
{
    type MSMAccumulator = DualMSM<E>;
}

/// KZG specific operations
impl<E> GuardKZG<E>
where
    E: MultiMillerLoop,
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) fn new(msm_accumulator: DualMSM<E>) -> Self {
        Self { msm_accumulator }
    }
}

/// A verifier that checks multiple proofs in a batch
#[derive(Clone, Debug)]
pub struct AccumulatorStrategy<E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm_accumulator: DualMSM<E>,
    params: ParamsVerifierKZG<E>,
}

impl<E> AccumulatorStrategy<E>
where
    E: MultiMillerLoop,
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    /// Constructs an empty batch verifier
    pub fn new(params: &ParamsVerifierKZG<E>) -> Self {
        AccumulatorStrategy {
            msm_accumulator: DualMSM::new(),
            params: params.clone(),
        }
    }

    /// Constructs and initialized new batch verifier
    pub fn with(msm_accumulator: DualMSM<E>, params: &ParamsVerifierKZG<E>) -> Self {
        AccumulatorStrategy {
            msm_accumulator,
            params: params.clone(),
        }
    }
}

/// A verifier that checks a single proof
#[derive(Clone, Debug)]
pub struct SingleStrategy<E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm: DualMSM<E>,
    params: ParamsVerifierKZG<E>,
}

impl<'params, E: MultiMillerLoop + Debug> SingleStrategy<E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    /// Constructs an empty batch verifier
    pub fn new(params: &'params ParamsVerifierKZG<E>) -> Self {
        SingleStrategy {
            msm: DualMSM::new(),
            params: params.clone(),
        }
    }
}

impl<'params, E, V> VerificationStrategy<'params, KZGCommitmentScheme<E>, V>
    for AccumulatorStrategy<E>
where
    E: MultiMillerLoop + Debug,
    V: Verifier<'params, KZGCommitmentScheme<E>, MSMAccumulator = DualMSM<E>, Guard = GuardKZG<E>>,
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
{
    type Output = Self;

    fn new(params: &'params ParamsVerifierKZG<E>) -> Self {
        AccumulatorStrategy::new(params)
    }

    fn process(
        mut self,
        f: impl FnOnce(V::MSMAccumulator) -> Result<V::Guard, Error>,
    ) -> Result<Self::Output, Error> {
        self.msm_accumulator.scale(E::Fr::random(OsRng));

        // Guard is updated with new msm contributions
        let guard = f(self.msm_accumulator)?;
        Ok(Self {
            msm_accumulator: guard.msm_accumulator,
            params: self.params,
        })
    }

    fn finalize(self) -> bool {
        // ZAL: Verification is (supposedly) cheap, hence we don't use an accelerator engine
        let default_engine = H2cEngine::new();
        self.msm_accumulator.check(&default_engine, &self.params)
    }
}

impl<'params, E, V> VerificationStrategy<'params, KZGCommitmentScheme<E>, V> for SingleStrategy<E>
where
    E: MultiMillerLoop + Debug,
    V: Verifier<'params, KZGCommitmentScheme<E>, MSMAccumulator = DualMSM<E>, Guard = GuardKZG<E>>,
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
{
    type Output = ();

    fn new(params: &'params ParamsVerifierKZG<E>) -> Self {
        Self::new(params)
    }

    fn process(
        self,
        f: impl FnOnce(V::MSMAccumulator) -> Result<V::Guard, Error>,
    ) -> Result<Self::Output, Error> {
        // Guard is updated with new msm contributions
        let guard = f(self.msm)?;
        let msm = guard.msm_accumulator;
        // Verification is (supposedly) cheap, hence we don't use an accelerator engine
        let default_engine = H2cEngine::new();
        if msm.check(&default_engine, &self.params) {
            Ok(())
        } else {
            Err(Error::ConstraintSystemFailure)
        }
    }

    fn finalize(self) -> bool {
        unreachable!();
    }
}
