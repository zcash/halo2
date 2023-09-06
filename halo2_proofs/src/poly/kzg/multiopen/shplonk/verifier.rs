use std::fmt::Debug;

use super::ChallengeY;
use super::{construct_intermediate_sets, ChallengeU, ChallengeV};
use crate::arithmetic::{
    eval_polynomial, evaluate_vanishing_polynomial, lagrange_interpolate, powers,
};
use crate::helpers::SerdeCurveAffine;
use crate::poly::commitment::Verifier;
use crate::poly::commitment::MSM;
use crate::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use crate::poly::kzg::msm::DualMSM;
use crate::poly::kzg::msm::{PreMSM, MSMKZG};
use crate::poly::kzg::strategy::GuardKZG;
use crate::poly::query::{CommitmentReference, VerifierQuery};
use crate::poly::Error;
use crate::transcript::{EncodedChallenge, TranscriptRead};
use ff::{Field, PrimeField};
use halo2curves::pairing::{Engine, MultiMillerLoop};
use std::ops::MulAssign;

/// Concrete KZG multiopen verifier with SHPLONK variant
#[derive(Debug)]
pub struct VerifierSHPLONK<'params, E: Engine> {
    params: &'params ParamsKZG<E>,
}

impl<'params, E> Verifier<'params, KZGCommitmentScheme<E>> for VerifierSHPLONK<'params, E>
where
    E: MultiMillerLoop + Debug,
    E::Scalar: PrimeField + Ord,
    E::G1Affine: SerdeCurveAffine,
    E::G2Affine: SerdeCurveAffine,
{
    type Guard = GuardKZG<'params, E>;
    type MSMAccumulator = DualMSM<'params, E>;

    const QUERY_INSTANCE: bool = false;

    fn new(params: &'params ParamsKZG<E>) -> Self {
        Self { params }
    }

    /// Verify a multi-opening proof
    fn verify_proof<
        'com,
        Ch: EncodedChallenge<E::G1Affine>,
        T: TranscriptRead<E::G1Affine, Ch>,
        I,
    >(
        &self,
        transcript: &mut T,
        queries: I,
        mut msm_accumulator: DualMSM<'params, E>,
    ) -> Result<Self::Guard, Error>
    where
        I: IntoIterator<Item = VerifierQuery<'com, E::G1Affine, MSMKZG<E>>> + Clone,
    {
        let intermediate_sets = construct_intermediate_sets(queries);
        let (rotation_sets, super_point_set) = (
            intermediate_sets.rotation_sets,
            intermediate_sets.super_point_set,
        );

        let y: ChallengeY<_> = transcript.squeeze_challenge_scalar();
        let v: ChallengeV<_> = transcript.squeeze_challenge_scalar();

        let h1 = transcript.read_point().map_err(|_| Error::SamplingError)?;
        let u: ChallengeU<_> = transcript.squeeze_challenge_scalar();
        let h2 = transcript.read_point().map_err(|_| Error::SamplingError)?;

        let (mut z_0_diff_inverse, mut z_0) = (E::Scalar::ZERO, E::Scalar::ZERO);
        let (mut outer_msm, mut r_outer_acc) = (PreMSM::<E>::new(), E::Scalar::ZERO);
        for (i, (rotation_set, power_of_v)) in rotation_sets.iter().zip(powers(*v)).enumerate() {
            let diffs: Vec<E::Scalar> = super_point_set
                .iter()
                .filter(|point| !rotation_set.points.contains(point))
                .copied()
                .collect();
            let mut z_diff_i = evaluate_vanishing_polynomial(&diffs[..], *u);

            // normalize coefficients by the coefficient of the first commitment
            if i == 0 {
                z_0 = evaluate_vanishing_polynomial(&rotation_set.points[..], *u);
                z_0_diff_inverse = z_diff_i.invert().unwrap();
                z_diff_i = E::Scalar::ONE;
            } else {
                z_diff_i.mul_assign(z_0_diff_inverse);
            }

            let (mut inner_msm, r_inner_acc) = rotation_set
                .commitments
                .iter()
                .zip(powers(*y))
                .map(|(commitment_data, power_of_y)| {
                    // calculate low degree equivalent
                    let r_x = lagrange_interpolate(
                        &rotation_set.points[..],
                        &commitment_data.evals()[..],
                    );
                    let r_eval = power_of_y * eval_polynomial(&r_x[..], *u);
                    let msm = match commitment_data.get() {
                        CommitmentReference::Commitment(c) => {
                            let mut msm = MSMKZG::<E>::new();
                            msm.append_term(power_of_y, (*c).into());
                            msm
                        }
                        CommitmentReference::MSM(msm) => {
                            let mut msm = msm.clone();
                            msm.scale(power_of_y);
                            msm
                        }
                    };
                    (msm, r_eval)
                })
                .reduce(|(mut msm_acc, r_eval_acc), (msm, r_eval)| {
                    msm_acc.add_msm(&msm);
                    (msm_acc, r_eval_acc + r_eval)
                })
                .unwrap();

            inner_msm.scale(power_of_v * z_diff_i);
            outer_msm.add_msm(inner_msm);
            r_outer_acc += power_of_v * r_inner_acc * z_diff_i;
        }
        let mut outer_msm = outer_msm.normalize();
        let g1: E::G1 = self.params.g[0].into();
        outer_msm.append_term(-r_outer_acc, g1);
        outer_msm.append_term(-z_0, h1.into());
        outer_msm.append_term(*u, h2.into());

        msm_accumulator.left.append_term(E::Scalar::ONE, h2.into());

        msm_accumulator.right.add_msm(&outer_msm);

        Ok(Self::Guard::new(msm_accumulator))
    }
}
