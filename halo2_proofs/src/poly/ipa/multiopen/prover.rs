use super::{construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4};
use crate::arithmetic::{eval_polynomial, kate_division, CurveAffine};
use crate::poly::commitment::ParamsProver;
use crate::poly::commitment::{Blind, Prover};
use crate::poly::ipa::commitment::{self, IPACommitmentScheme, ParamsIPA};
use crate::poly::query::ProverQuery;
use crate::poly::{Coeff, Polynomial};
use crate::transcript::{EncodedChallenge, TranscriptWrite};

use ff::Field;
use group::Curve;
use rand_core::RngCore;
use std::io;
use std::marker::PhantomData;

/// IPA multi-open prover
#[derive(Debug)]
pub struct ProverIPA<'params, C: CurveAffine> {
    pub(crate) params: &'params ParamsIPA<C>,
}

impl<'params, C: CurveAffine> Prover<'params, IPACommitmentScheme<C>> for ProverIPA<'params, C> {
    const QUERY_INSTANCE: bool = true;

    fn new(params: &'params ParamsIPA<C>) -> Self {
        Self { params }
    }

    /// Create a multi-opening proof
    fn create_proof<'com, Z: EncodedChallenge<C>, T: TranscriptWrite<C, Z>, R, I>(
        &self,
        mut rng: R,
        transcript: &mut T,
        queries: I,
    ) -> io::Result<()>
    where
        I: IntoIterator<Item = ProverQuery<'com, C>> + Clone,
        R: RngCore,
    {
        let x_1: ChallengeX1<_> = transcript.squeeze_challenge_scalar();
        let x_2: ChallengeX2<_> = transcript.squeeze_challenge_scalar();

        let (poly_map, point_sets) = construct_intermediate_sets(queries);

        // Collapse openings at same point sets together into single openings using
        // x_1 challenge.
        let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> = vec![None; point_sets.len()];
        let mut q_blinds = vec![Blind(C::Scalar::ZERO); point_sets.len()];

        {
            let mut accumulate = |set_idx: usize,
                                  new_poly: &Polynomial<C::Scalar, Coeff>,
                                  blind: Blind<C::Scalar>| {
                if let Some(poly) = &q_polys[set_idx] {
                    q_polys[set_idx] = Some(poly.clone() * *x_1 + new_poly);
                } else {
                    q_polys[set_idx] = Some(new_poly.clone());
                }
                q_blinds[set_idx] *= *x_1;
                q_blinds[set_idx] += blind;
            };

            for commitment_data in poly_map.into_iter() {
                accumulate(
                    commitment_data.set_index,        // set_idx,
                    commitment_data.commitment.poly,  // poly,
                    commitment_data.commitment.blind, // blind,
                );
            }
        }

        let q_prime_poly = point_sets
            .iter()
            .zip(q_polys.iter())
            .fold(None, |q_prime_poly, (points, poly)| {
                let mut poly = points
                    .iter()
                    .fold(poly.clone().unwrap().values, |poly, point| {
                        kate_division(&poly, *point)
                    });
                poly.resize(self.params.n as usize, C::Scalar::ZERO);
                let poly = Polynomial {
                    values: poly,
                    _marker: PhantomData,
                };

                if q_prime_poly.is_none() {
                    Some(poly)
                } else {
                    q_prime_poly.map(|q_prime_poly| q_prime_poly * *x_2 + &poly)
                }
            })
            .unwrap();

        let q_prime_blind = Blind(C::Scalar::random(&mut rng));
        let q_prime_commitment = self.params.commit(&q_prime_poly, q_prime_blind).to_affine();

        transcript.write_point(q_prime_commitment)?;

        let x_3: ChallengeX3<_> = transcript.squeeze_challenge_scalar();

        // Prover sends u_i for all i, which correspond to the evaluation
        // of each Q polynomial commitment at x_3.
        for q_i_poly in &q_polys {
            transcript.write_scalar(eval_polynomial(q_i_poly.as_ref().unwrap(), *x_3))?;
        }

        let x_4: ChallengeX4<_> = transcript.squeeze_challenge_scalar();

        let (p_poly, p_poly_blind) = q_polys.into_iter().zip(q_blinds.into_iter()).fold(
            (q_prime_poly, q_prime_blind),
            |(q_prime_poly, q_prime_blind), (poly, blind)| {
                (
                    q_prime_poly * *x_4 + &poly.unwrap(),
                    Blind((q_prime_blind.0 * &(*x_4)) + &blind.0),
                )
            },
        );

        commitment::create_proof(self.params, rng, transcript, &p_poly, p_poly_blind, *x_3)
    }
}
