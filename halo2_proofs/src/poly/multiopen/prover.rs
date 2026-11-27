use super::super::{
    commitment::{self, Blind, Params},
    Coeff, Polynomial,
};
use super::{
    construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4, ProverQuery,
    Query,
};

use crate::arithmetic::{evaluate_batch, kate_division_parallel, CurveAffine};
use crate::transcript::{EncodedChallenge, TranscriptWrite};

use ff::Field;
use maybe_rayon::prelude::*;
use pasta_curves::arithmetic::CurveExt;
use rand_core::Rng;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

/// Create a multi-opening proof
pub fn create_proof<
    'a,
    I,
    C: CurveAffine,
    E: EncodedChallenge<C>,
    R: Rng,
    T: TranscriptWrite<C, E>,
>(
    params: &Params<C>,
    mut rng: R,
    transcript: &mut T,
    queries: I,
) -> io::Result<()>
where
    I: IntoIterator<Item = ProverQuery<'a, C>> + Clone,
{
    let x_1: ChallengeX1<_> = transcript.squeeze_challenge_scalar();
    let x_2: ChallengeX2<_> = transcript.squeeze_challenge_scalar();

    let (poly_map, point_sets) = construct_intermediate_sets(queries).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "queries iterator contains mismatching evaluations",
        )
    })?;

    // Collapse openings at same point sets together into single openings using
    // x_1 challenge. The sets are independent, so they are collapsed in
    // parallel; within a set, the polynomials keep their query order.
    let mut by_set: Vec<Vec<PolynomialPointer<'a, C>>> = vec![vec![]; point_sets.len()];
    for commitment_data in poly_map.into_iter() {
        by_set[commitment_data.set_index].push(commitment_data.commitment);
    }
    let (q_polys, q_blinds): (Vec<_>, Vec<_>) = by_set
        .into_par_iter()
        .map(|commitments| {
            let (first, rest) = commitments
                .split_first()
                .expect("every point set has at least one polynomial");
            rest.iter().fold(
                (first.poly.clone(), first.blind),
                |(poly, blind), commitment| {
                    (
                        poly * *x_1 + commitment.poly,
                        Blind(blind.0 * *x_1 + commitment.blind.0),
                    )
                },
            )
        })
        .unzip();

    // Divide each collapsed polynomial by the vanishing polynomial of its
    // point set, in parallel across sets, then combine them with x_2.
    let divided: Vec<Vec<C::Scalar>> = point_sets
        .iter()
        .zip(q_polys.iter())
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(
            |(points, poly): (&Vec<C::Scalar>, &Polynomial<C::Scalar, Coeff>)| {
                let mut poly = points
                    .iter()
                    .fold(poly.values.clone(), |poly: Vec<C::Scalar>, point| {
                        kate_division_parallel(&poly, *point)
                    });
                poly.resize(params.n as usize, C::Scalar::ZERO);
                poly
            },
        )
        .collect();
    let q_prime_poly = divided
        .into_iter()
        .map(|values| Polynomial {
            values,
            _marker: PhantomData,
        })
        .reduce(|q_prime_poly, poly| q_prime_poly * *x_2 + &poly)
        .unwrap();

    let q_prime_blind = Blind(C::Scalar::random(&mut rng));
    let q_prime_commitment = params
        .commit(&q_prime_poly, q_prime_blind)
        .to_affine_vartime();

    transcript.write_point(q_prime_commitment)?;

    let x_3: ChallengeX3<_> = transcript.squeeze_challenge_scalar();

    // Prover sends u_i for all i, which correspond to the evaluation
    // of each Q polynomial commitment at x_3.
    let queries: Vec<(&[C::Scalar], C::Scalar)> =
        q_polys.iter().map(|poly| (&poly[..], *x_3)).collect();
    for eval in evaluate_batch(&queries) {
        transcript.write_scalar(eval)?;
    }

    let x_4: ChallengeX4<_> = transcript.squeeze_challenge_scalar();

    let (p_poly, p_poly_blind) = q_polys.into_iter().zip(q_blinds).fold(
        (q_prime_poly, q_prime_blind),
        |(q_prime_poly, q_prime_blind), (poly, blind)| {
            (
                q_prime_poly * *x_4 + &poly,
                Blind((q_prime_blind.0 * &(*x_4)) + &blind.0),
            )
        },
    );

    commitment::create_proof(params, rng, transcript, &p_poly, p_poly_blind, *x_3)
}

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct PolynomialPointer<'a, C: CurveAffine> {
    poly: &'a Polynomial<C::Scalar, Coeff>,
    blind: commitment::Blind<C::Scalar>,
}

impl<'a, C: CurveAffine> PartialEq for PolynomialPointer<'a, C> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.poly, other.poly)
    }
}

impl<'a, C: CurveAffine> Eq for PolynomialPointer<'a, C> {}

impl<'a, C: CurveAffine> Hash for PolynomialPointer<'a, C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.poly, state)
    }
}

impl<'a, C: CurveAffine> Query<C::Scalar> for ProverQuery<'a, C> {
    type Commitment = PolynomialPointer<'a, C>;
    type Eval = ();

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) {}
    fn get_commitment(&self) -> Self::Commitment {
        PolynomialPointer {
            poly: self.poly,
            blind: self.blind,
        }
    }
}
