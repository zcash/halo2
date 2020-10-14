use super::super::{
    commitment::{self, Blind, Params},
    Coeff, Error, Polynomial,
};
use super::{construct_intermediate_sets, Proof, ProverQuery, Query};

use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, kate_division, lagrange_interpolate, Challenge, Curve,
    CurveAffine, Field,
};
use crate::transcript::{Hasher, Transcript};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
struct CommitmentData<C: CurveAffine> {
    set_index: usize,
    blind: Blind<C::Scalar>,
    point_indices: Vec<usize>,
    evals: Vec<C::Scalar>,
}

impl<C: CurveAffine> Proof<C> {
    /// Create a multi-opening proof
    pub fn create<'a, I, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        params: &Params<C>,
        transcript: &mut Transcript<C, HBase, HScalar>,
        queries: I,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = ProverQuery<'a, C>> + Clone,
    {
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));
        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let (poly_map, point_sets) = construct_intermediate_sets(queries);

        // Collapse openings at same point sets together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> = vec![None; point_sets.len()];
        let mut q_blinds = vec![Blind(C::Scalar::zero()); point_sets.len()];

        // A vec of vecs of evals. The outer vec corresponds to the point set,
        // while the inner vec corresponds to the points in a particular set.
        let mut q_eval_sets = Vec::with_capacity(point_sets.len());
        for point_set in point_sets.iter() {
            q_eval_sets.push(vec![C::Scalar::zero(); point_set.len()]);
        }

        {
            let mut accumulate = |set_idx: usize,
                                  new_poly: &Polynomial<C::Scalar, Coeff>,
                                  blind: Blind<C::Scalar>,
                                  evals: Vec<C::Scalar>| {
                if let Some(poly) = &q_polys[set_idx] {
                    q_polys[set_idx] = Some(poly.clone() * x_4 + new_poly);
                } else {
                    q_polys[set_idx] = Some(new_poly.clone());
                }
                q_blinds[set_idx] *= x_4;
                q_blinds[set_idx] += blind;
                // Each polynomial is evaluated at a set of points. For each set,
                // we collapse each polynomial's evals pointwise.
                for (eval, set_eval) in evals.iter().zip(q_eval_sets[set_idx].iter_mut()) {
                    *set_eval *= &x_4;
                    *set_eval += eval;
                }
            };

            for commitment_data in poly_map.into_iter() {
                accumulate(
                    commitment_data.set_index,        // set_idx,
                    commitment_data.commitment.poly,  // poly,
                    commitment_data.commitment.blind, // blind,
                    commitment_data.evals,            // evals
                );
            }
        }

        let f_poly = point_sets
            .iter()
            .zip(q_eval_sets.iter())
            .zip(q_polys.iter())
            .fold(None, |f_poly, ((points, evals), poly)| {
                let mut poly = poly.clone().unwrap().values;
                // TODO: makes implicit asssumption that poly degree is smaller than interpolation poly degree
                for (p, r) in poly.iter_mut().zip(lagrange_interpolate(points, evals)) {
                    *p -= &r;
                }
                let mut poly = points
                    .iter()
                    .fold(poly, |poly, point| kate_division(&poly, *point));
                poly.resize(params.n as usize, C::Scalar::zero());
                let poly = Polynomial {
                    values: poly,
                    _marker: PhantomData,
                };

                if f_poly.is_none() {
                    Some(poly)
                } else {
                    f_poly.map(|f_poly| f_poly * x_5 + &poly)
                }
            })
            .unwrap();

        let mut f_blind = Blind(C::Scalar::random());
        let mut f_commitment = params.commit(&f_poly, f_blind).to_affine();

        let (opening, q_evals) = loop {
            let mut transcript = transcript.clone();
            transcript
                .absorb_point(&f_commitment)
                .map_err(|_| Error::SamplingError)?;

            let x_6: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let q_evals: Vec<C::Scalar> = q_polys
                .iter()
                .map(|poly| eval_polynomial(poly.as_ref().unwrap(), x_6))
                .collect();

            for eval in q_evals.iter() {
                transcript.absorb_scalar(*eval);
            }

            let x_7: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let (f_poly, f_blind_try) = q_polys.iter().zip(q_blinds.iter()).fold(
                (f_poly.clone(), f_blind),
                |(f_poly, f_blind), (poly, blind)| {
                    (
                        f_poly * x_7 + poly.as_ref().unwrap(),
                        Blind((f_blind.0 * &x_7) + &blind.0),
                    )
                },
            );

            if let Ok(opening) =
                commitment::Proof::create(&params, &mut transcript, &f_poly, f_blind_try, x_6)
            {
                break (opening, q_evals);
            } else {
                f_blind += C::Scalar::one();
                f_commitment = (f_commitment + params.h).to_affine();
            }
        };

        Ok(Proof {
            q_evals,
            f_commitment,
            opening,
        })
    }
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

    fn get_point(&self) -> C::Scalar {
        self.point
    }
    fn get_eval(&self) -> C::Scalar {
        self.eval
    }
    fn get_commitment(&self) -> Self::Commitment {
        PolynomialPointer {
            poly: self.poly,
            blind: self.blind,
        }
    }
}
