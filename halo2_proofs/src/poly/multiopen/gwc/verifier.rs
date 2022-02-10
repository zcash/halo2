use super::{construct_intermediate_sets, ChallengeU, ChallengeV};
use crate::arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt};
use crate::poly::Rotation;
use crate::poly::{
    commitment::{Params, ParamsVerifier},
    multiopen::{CommitmentReference, Query, VerifierQuery},
    Error, PairMSM, MSM,
};
use crate::transcript::{EncodedChallenge, TranscriptRead};

use ff::Field;
use group::Group;
use pairing::arithmetic::{MillerLoopResult, MultiMillerLoop};
use subtle::Choice;

/// Verify a multi-opening proof
pub fn verify_proof<
    'r,
    'params: 'r,
    I,
    C: MultiMillerLoop,
    E: EncodedChallenge<C::G1Affine>,
    T: TranscriptRead<C::G1Affine, E>,
>(
    params: &'params ParamsVerifier<C>,
    transcript: &mut T,
    queries: I,
) -> Result<PairMSM<C::G1Affine>, Error>
where
    I: IntoIterator<Item = VerifierQuery<'r, C::G1Affine>> + Clone,
{
    let v: ChallengeV<_> = transcript.squeeze_challenge_scalar();
    let u: ChallengeU<_> = transcript.squeeze_challenge_scalar();

    let commitment_data = construct_intermediate_sets(queries);

    let mut commitment_multi = params.empty_msm();
    let mut eval_multi = C::Scalar::zero();

    let mut witness = params.empty_msm();
    let mut witness_with_aux = params.empty_msm();

    for commitment_at_a_point in commitment_data.iter() {
        assert!(!commitment_at_a_point.queries.is_empty());
        let z = commitment_at_a_point.point;

        let wi = transcript.read_point().map_err(|_| Error::SamplingError)?;

        witness_with_aux.scale(*u);
        witness_with_aux.append_term(z, wi);
        witness.scale(*u);
        witness.append_term(C::Scalar::one(), wi);
        commitment_multi.scale(*u);
        eval_multi = eval_multi * *u;

        let mut commitment_batch = params.empty_msm();
        let mut eval_batch = C::Scalar::zero();

        for query in commitment_at_a_point.queries.iter() {
            assert_eq!(query.get_point(), z);

            let commitment = query.get_commitment();
            let eval = query.get_eval();

            commitment_batch.scale(*v);
            match commitment {
                CommitmentReference::Commitment(c) => {
                    commitment_batch.append_term(C::Scalar::one(), *c);
                }
                CommitmentReference::MSM(msm) => {
                    commitment_batch.add_msm(msm);
                }
            }

            eval_batch = eval_batch * *v + eval;
        }

        commitment_multi.add_msm(&commitment_batch);
        eval_multi += eval_batch;
    }

    let mut left = params.empty_msm();
    left.add_msm(&witness);

    let mut right = params.empty_msm();
    right.add_msm(&witness_with_aux);
    right.add_msm(&commitment_multi);
    right.append_term(eval_multi, -params.g1);

    Ok(PairMSM::with(left, right))
}

impl<'a, 'b, C: CurveAffine> Query<C::Scalar> for VerifierQuery<'a, C> {
    type Commitment = CommitmentReference<'a, C>;

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_rotation(&self) -> Rotation {
        self.rotation
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        self.commitment
    }
}
