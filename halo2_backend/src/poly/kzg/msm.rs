use std::fmt::Debug;

use super::commitment::ParamsKZG;
use crate::{arithmetic::parallelize, poly::commitment::MSM};
use group::{Curve, Group};
use halo2_middleware::zal::traits::MsmAccel;
use halo2curves::{
    pairing::{Engine, MillerLoopResult, MultiMillerLoop},
    CurveAffine, CurveExt,
};

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Clone, Default, Debug)]
pub struct MSMKZG<E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) scalars: Vec<E::Fr>,
    pub(crate) bases: Vec<E::G1>,
}

impl<E: Engine> MSMKZG<E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    /// Create an empty MSM instance
    pub fn new() -> Self {
        MSMKZG {
            scalars: vec![],
            bases: vec![],
        }
    }

    /// Prepares all scalars in the MSM to linear combination
    pub fn combine_with_base(&mut self, base: E::Fr) {
        use halo2_middleware::ff::Field;
        let mut acc = E::Fr::ONE;
        if !self.scalars.is_empty() {
            for scalar in self.scalars.iter_mut().rev() {
                *scalar *= &acc;
                acc *= base;
            }
        }
    }
}

impl<E: Engine + Debug> MSM<E::G1Affine> for MSMKZG<E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    fn append_term(&mut self, scalar: E::Fr, point: E::G1) {
        self.scalars.push(scalar);
        self.bases.push(point);
    }

    fn add_msm(&mut self, other: &Self) {
        self.scalars.extend(other.scalars().iter());
        self.bases.extend(other.bases().iter());
    }

    fn scale(&mut self, factor: E::Fr) {
        if !self.scalars.is_empty() {
            parallelize(&mut self.scalars, |scalars, _| {
                for other_scalar in scalars {
                    *other_scalar *= &factor;
                }
            })
        }
    }

    fn check(&self, engine: &impl MsmAccel<E::G1Affine>) -> bool {
        bool::from(self.eval(engine).is_identity())
    }

    fn eval(&self, engine: &impl MsmAccel<E::G1Affine>) -> E::G1 {
        use group::prime::PrimeCurveAffine;
        let mut bases = vec![E::G1Affine::identity(); self.scalars.len()];
        E::G1::batch_normalize(&self.bases, &mut bases);
        engine.msm(&self.scalars, &bases)
    }

    fn bases(&self) -> Vec<E::G1> {
        self.bases.clone()
    }

    fn scalars(&self) -> Vec<E::Fr> {
        self.scalars.clone()
    }
}

/// A projective point collector
#[derive(Debug, Clone)]
pub(crate) struct PreMSM<E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    projectives_msms: Vec<MSMKZG<E>>,
}

impl<E: Engine + Debug> Default for PreMSM<E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    fn default() -> Self {
        PreMSM {
            projectives_msms: vec![],
        }
    }
}

impl<E: Engine + Debug> PreMSM<E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) fn normalize(self) -> MSMKZG<E> {
        let (scalars, bases) = self
            .projectives_msms
            .into_iter()
            .map(|msm| (msm.scalars, msm.bases))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        MSMKZG {
            scalars: scalars.into_iter().flatten().collect(),
            bases: bases.into_iter().flatten().collect(),
        }
    }

    pub(crate) fn add_msm(&mut self, other: MSMKZG<E>) {
        self.projectives_msms.push(other);
    }
}

impl<'params, E: MultiMillerLoop + Debug> From<&'params ParamsKZG<E>> for DualMSM<'params, E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    fn from(params: &'params ParamsKZG<E>) -> Self {
        DualMSM::new(params)
    }
}

/// Two channel MSM accumulator
#[derive(Debug, Clone)]
pub struct DualMSM<'a, E: Engine>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    pub(crate) params: &'a ParamsKZG<E>,
    pub(crate) left: MSMKZG<E>,
    pub(crate) right: MSMKZG<E>,
}

impl<'a, E: MultiMillerLoop + Debug> DualMSM<'a, E>
where
    E::G1Affine: CurveAffine<ScalarExt = <E as Engine>::Fr, CurveExt = <E as Engine>::G1>,
    E::G1: CurveExt<AffineExt = E::G1Affine>,
{
    /// Create a new two channel MSM accumulator instance
    pub fn new(params: &'a ParamsKZG<E>) -> Self {
        Self {
            params,
            left: MSMKZG::new(),
            right: MSMKZG::new(),
        }
    }

    /// Scale all scalars in the MSM by some scaling factor
    pub fn scale(&mut self, e: E::Fr) {
        self.left.scale(e);
        self.right.scale(e);
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: Self) {
        self.left.add_msm(&other.left);
        self.right.add_msm(&other.right);
    }

    /// Performs final pairing check with given verifier params and two channel linear combination
    pub fn check(self, engine: &impl MsmAccel<E::G1Affine>) -> bool {
        let s_g2_prepared = E::G2Prepared::from(self.params.s_g2);
        let n_g2_prepared = E::G2Prepared::from(-self.params.g2);

        let left = self.left.eval(engine);
        let right = self.right.eval(engine);

        let (term_1, term_2) = (
            (&left.into(), &s_g2_prepared),
            (&right.into(), &n_g2_prepared),
        );
        let terms = &[term_1, term_2];

        bool::from(
            E::multi_miller_loop(&terms[..])
                .final_exponentiation()
                .is_identity(),
        )
    }
}
