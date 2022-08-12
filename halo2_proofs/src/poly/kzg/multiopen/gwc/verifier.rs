use std::fmt::Debug;
use std::io::Read;
use std::marker::PhantomData;

use super::{construct_intermediate_sets, ChallengeU, ChallengeV};
use crate::arithmetic::{eval_polynomial, lagrange_interpolate, powers, CurveAffine, FieldExt};

use crate::poly::commitment::Verifier;
use crate::poly::commitment::MSM;
use crate::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use crate::poly::kzg::msm::{DualMSM, MSMKZG};
use crate::poly::kzg::strategy::{AccumulatorStrategy, GuardKZG, SingleStrategy};
use crate::poly::query::Query;
use crate::poly::query::{CommitmentReference, VerifierQuery};
use crate::poly::strategy::VerificationStrategy;
use crate::poly::{
    commitment::{Params, ParamsVerifier},
    Error,
};
use crate::transcript::{EncodedChallenge, TranscriptRead};

use ff::Field;
use group::Group;
use halo2curves::pairing::{Engine, MillerLoopResult, MultiMillerLoop};
use rand_core::OsRng;

#[derive(Debug)]
/// Concrete KZG verifier with GWC variant
pub struct VerifierGWC<'params, E: Engine> {
    params: &'params ParamsKZG<E>,
}

impl<'params, E: MultiMillerLoop + Debug> Verifier<'params, KZGCommitmentScheme<E>>
    for VerifierGWC<'params, E>
{
    type Guard = GuardKZG<'params, E>;
    type MSMAccumulator = DualMSM<'params, E>;

    fn new(params: &'params ParamsKZG<E>) -> Self {
        Self { params }
    }

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
        let v: ChallengeV<_> = transcript.squeeze_challenge_scalar();

        let commitment_data = construct_intermediate_sets(queries);

        let w: Vec<E::G1Affine> = (0..commitment_data.len())
            .map(|_| transcript.read_point().map_err(|_| Error::SamplingError))
            .collect::<Result<Vec<E::G1Affine>, Error>>()?;

        let u: ChallengeU<_> = transcript.squeeze_challenge_scalar();

        let mut commitment_multi = MSMKZG::<E>::new();
        let mut eval_multi = E::Scalar::zero();

        let mut witness = MSMKZG::<E>::new();
        let mut witness_with_aux = MSMKZG::<E>::new();

        for ((commitment_at_a_point, wi), power_of_u) in
            commitment_data.iter().zip(w.into_iter()).zip(powers(*u))
        {
            assert!(!commitment_at_a_point.queries.is_empty());
            let z = commitment_at_a_point.point;

            let (mut commitment_batch, eval_batch) = commitment_at_a_point
                .queries
                .iter()
                .zip(powers(*v))
                .map(|(query, power_of_v)| {
                    assert_eq!(query.get_point(), z);

                    let commitment = match query.get_commitment() {
                        CommitmentReference::Commitment(c) => {
                            let mut msm = MSMKZG::<E>::new();
                            msm.append_term(power_of_v, (*c).into());
                            msm
                        }
                        CommitmentReference::MSM(msm) => {
                            let mut msm = msm.clone();
                            msm.scale(power_of_v);
                            msm
                        }
                    };
                    let eval = power_of_v * query.get_eval();

                    (commitment, eval)
                })
                .reduce(|(mut commitment_acc, eval_acc), (commitment, eval)| {
                    commitment_acc.add_msm(&commitment);
                    (commitment_acc, eval_acc + eval)
                })
                .unwrap();

            commitment_batch.scale(power_of_u);
            commitment_multi.add_msm(&commitment_batch);
            eval_multi += power_of_u * eval_batch;

            witness_with_aux.append_term(power_of_u * z, wi.into());
            witness.append_term(power_of_u, wi.into());
        }

        msm_accumulator.left.add_msm(&witness);

        msm_accumulator.right.add_msm(&witness_with_aux);
        msm_accumulator.right.add_msm(&commitment_multi);
        let g0: E::G1 = self.params.g[0].into();
        msm_accumulator.right.append_term(eval_multi, -g0);

        Ok(Self::Guard::new(msm_accumulator))
    }
}
