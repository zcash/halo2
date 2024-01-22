use super::{
    commitment::{KZGCommitmentScheme, ParamsKZG},
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
use ff::Field;
use halo2curves::{
    pairing::{Engine, MultiMillerLoop},
    CurveAffine, CurveExt,
};
use rand_core::OsRng;
use std::fmt::Debug;

/// Wrapper for linear verification accumulator
#[derive(Debug, Clone)]
pub struct GuardKZG<'params, E: MultiMillerLoop + Debug>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm_accumulator: DualMSM<'params, E>,
}

/// Define accumulator type as `DualMSM`
impl<'params, E> Guard<KZGCommitmentScheme<E>> for GuardKZG<'params, E>
where
    E: MultiMillerLoop + Debug,
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
{
    type MSMAccumulator = DualMSM<'params, E>;
}

/// KZG specific operations
impl<'params, E: MultiMillerLoop + Debug> GuardKZG<'params, E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) fn new(msm_accumulator: DualMSM<'params, E>) -> Self {
        Self { msm_accumulator }
    }
}

/// A verifier that checks multiple proofs in a batch
#[derive(Clone, Debug)]
pub struct AccumulatorStrategy<'params, E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm_accumulator: DualMSM<'params, E>,
}

impl<'params, E: MultiMillerLoop + Debug> AccumulatorStrategy<'params, E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
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
pub struct SingleStrategy<'params, E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) msm: DualMSM<'params, E>,
}

impl<'params, E: MultiMillerLoop + Debug> SingleStrategy<'params, E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
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
where
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
{
    type Output = Self;

    fn new(params: &'params ParamsKZG<E>) -> Self {
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
where
    E::G1Affine: SerdeCurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
    E::G2Affine: SerdeCurveAffine,
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
