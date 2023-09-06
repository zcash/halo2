use super::{construct_intermediate_sets, ChallengeV, Query};
use crate::arithmetic::{kate_division, powers};
use crate::helpers::SerdeCurveAffine;
use crate::poly::commitment::ParamsProver;
use crate::poly::commitment::Prover;
use crate::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use crate::poly::query::ProverQuery;
use crate::poly::{commitment::Blind, Polynomial};
use crate::transcript::{EncodedChallenge, TranscriptWrite};

use ff::PrimeField;
use group::Curve;
use halo2curves::pairing::Engine;
use rand_core::RngCore;
use std::fmt::Debug;
use std::io;
use std::marker::PhantomData;

/// Concrete KZG prover with GWC variant
#[derive(Debug)]
pub struct ProverGWC<'params, E: Engine> {
    params: &'params ParamsKZG<E>,
}

/// Create a multi-opening proof
impl<'params, E: Engine + Debug> Prover<'params, KZGCommitmentScheme<E>> for ProverGWC<'params, E>
where
    E::Scalar: PrimeField,
    E::G1Affine: SerdeCurveAffine,
    E::G2Affine: SerdeCurveAffine,
{
    const QUERY_INSTANCE: bool = false;

    fn new(params: &'params ParamsKZG<E>) -> Self {
        Self { params }
    }

    /// Create a multi-opening proof
    fn create_proof<
        'com,
        Ch: EncodedChallenge<E::G1Affine>,
        T: TranscriptWrite<E::G1Affine, Ch>,
        R,
        I,
    >(
        &self,
        _: R,
        transcript: &mut T,
        queries: I,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = ProverQuery<'com, E::G1Affine>> + Clone,
        R: RngCore,
    {
        let v: ChallengeV<_> = transcript.squeeze_challenge_scalar();
        let commitment_data = construct_intermediate_sets(queries);

        for commitment_at_a_point in commitment_data.iter() {
            let z = commitment_at_a_point.point;
            let (poly_batch, eval_batch) = commitment_at_a_point
                .queries
                .iter()
                .zip(powers(*v))
                .map(|(query, power_of_v)| {
                    assert_eq!(query.get_point(), z);

                    let poly = query.get_commitment().poly;
                    let eval = query.get_eval();

                    (poly.clone() * power_of_v, eval * power_of_v)
                })
                .reduce(|(poly_acc, eval_acc), (poly, eval)| (poly_acc + &poly, eval_acc + eval))
                .unwrap();

            let poly_batch = &poly_batch - eval_batch;
            let witness_poly = Polynomial {
                values: kate_division(&poly_batch.values, z),
                _marker: PhantomData,
            };
            let w = self
                .params
                .commit(&witness_poly, Blind::default())
                .to_affine();

            transcript.write_point(w)?;
        }
        Ok(())
    }
}
