use super::super::{
    commitment::{self, Blind, Params},
    Coeff, Polynomial,
};
use super::{
    construct_intermediate_sets, ChallengeX1, ChallengeX2, ChallengeX3, ChallengeX4, ProverQuery,
    Query,
};

use crate::arithmetic::{eval_polynomial, kate_division, CurveAffine, FieldExt};
use crate::transcript::TranscriptWrite;

use ff::Field;
use group::Curve;
use std::io;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
struct CommitmentData<C: CurveAffine> {
    set_index: usize,
    blind: Blind<C::Scalar>,
    point_indices: Vec<usize>,
    evals: Vec<C::Scalar>,
}

/// Create a multi-opening proof
pub fn create_proof<'a, I, C: CurveAffine, T: TranscriptWrite<C>>(
    params: &Params<C>,
    transcript: &mut T,
    queries: I,
) -> io::Result<()>
where
    I: IntoIterator<Item = ProverQuery<'a, C>> + Clone,
{
    let x_1 = ChallengeX1::get(transcript);
    let x_2 = ChallengeX2::get(transcript);

    let (poly_map, point_sets) = construct_intermediate_sets(queries);

    // Collapse openings at same point sets together into single openings using
    // x_1 challenge.
    let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> = vec![None; point_sets.len()];
    let mut q_blinds = vec![Blind(C::Scalar::zero()); point_sets.len()];

    {
        let mut accumulate =
            |set_idx: usize, new_poly: &Polynomial<C::Scalar, Coeff>, blind: Blind<C::Scalar>| {
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

    let f_poly = point_sets
        .iter()
        .zip(q_polys.iter())
        .fold(None, |f_poly, (points, poly)| {
            let mut poly = points
                .iter()
                .fold(poly.clone().unwrap().values, |poly, point| {
                    kate_division(&poly, *point)
                });
            poly.resize(params.n as usize, C::Scalar::zero());
            let poly = Polynomial {
                values: poly,
                _marker: PhantomData,
            };

            if f_poly.is_none() {
                Some(poly)
            } else {
                f_poly.map(|f_poly| f_poly * *x_2 + &poly)
            }
        })
        .unwrap();

    let f_blind = Blind(C::Scalar::rand());
    let f_commitment = params.commit(&f_poly, f_blind).to_affine();

    transcript.write_point(f_commitment)?;

    let x_3 = ChallengeX3::get(transcript);

    let q_evals: Vec<C::Scalar> = q_polys
        .iter()
        .map(|poly| eval_polynomial(poly.as_ref().unwrap(), *x_3))
        .collect();

    for eval in q_evals.iter() {
        transcript.write_scalar(*eval)?;
    }

    let x_4 = ChallengeX4::get(transcript);

    let (f_poly, f_blind_try) = q_polys.iter().zip(q_blinds.iter()).fold(
        (f_poly, f_blind),
        |(f_poly, f_blind), (poly, blind)| {
            (
                f_poly * *x_4 + poly.as_ref().unwrap(),
                Blind((f_blind.0 * &(*x_4)) + &blind.0),
            )
        },
    );

    commitment::create_proof(&params, transcript, &f_poly, f_blind_try, *x_3)
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
