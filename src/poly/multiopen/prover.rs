use std::marker::PhantomData;

use super::super::{
    commitment::{self, Blind, Params},
    Coeff, Error, Polynomial,
};
use super::Proof;

use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, kate_division, parallelize, Challenge, Curve,
    CurveAffine, Field,
};
use crate::plonk::hash_point;
use crate::transcript::Hasher;

impl<C: CurveAffine> Proof<C> {
    /// Create a multi-opening proof
    pub fn create<I, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        params: &Params<C>,
        transcript: &mut HBase,
        transcript_scalar: &mut HScalar,
        points: Vec<C::Scalar>,
        instances: I,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<
                Item = (
                    usize,
                    Polynomial<C::Scalar, Coeff>,
                    Blind<C::Scalar>,
                    C::Scalar,
                ),
            > + Clone,
    {
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Collapse openings at same points together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> = vec![None; points.len()];
        let mut q_blinds = vec![Blind(C::Scalar::zero()); points.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); points.len()];
        {
            let mut accumulate =
                |point_index: usize, new_poly: Polynomial<C::Scalar, Coeff>, blind, eval| {
                    q_polys[point_index]
                        .as_mut()
                        .map(|poly| {
                            parallelize(poly, |q, start| {
                                for (q, a) in q.iter_mut().zip(new_poly[start..].iter()) {
                                    *q *= &x_4;
                                    *q += a;
                                }
                            });
                        })
                        .or_else(|| {
                            q_polys[point_index] = Some(new_poly.clone());
                            Some(())
                        });
                    q_blinds[point_index] *= x_4;
                    q_blinds[point_index] += blind;
                    q_evals[point_index] *= &x_4;
                    q_evals[point_index] += &eval;
                };

            for instance in instances.clone() {
                accumulate(
                    instance.0, // point_index,
                    instance.1, // poly,
                    instance.2, // blind,
                    instance.3, // eval
                );
            }
        }

        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut f_poly: Option<Polynomial<C::Scalar, Coeff>> = None;
        for (point_index, &point) in points.iter().enumerate() {
            let mut poly = q_polys[point_index].as_ref().unwrap().clone();
            poly[0] -= &q_evals[point_index];
            // TODO: change kate_division interface?
            let mut poly = kate_division(&poly[..], point);
            poly.push(C::Scalar::zero());
            let poly = Polynomial {
                values: poly,
                _marker: PhantomData,
            };

            f_poly = f_poly
                .map(|mut f_poly| {
                    parallelize(&mut f_poly, |q, start| {
                        for (q, a) in q.iter_mut().zip(poly[start..].iter()) {
                            *q *= &x_5;
                            *q += a;
                        }
                    });
                    f_poly
                })
                .or_else(|| Some(poly));
        }

        let f_poly = f_poly.unwrap();
        let mut f_blind = Blind(C::Scalar::random());
        let mut f_commitment = params.commit(&f_poly, f_blind).to_affine();

        let (opening, q_evals) = loop {
            let mut transcript = transcript.clone();
            let mut transcript_scalar = transcript_scalar.clone();
            hash_point(&mut transcript, &f_commitment).unwrap();

            let x_6: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let mut q_evals = vec![C::Scalar::zero(); points.len()];

            for (point_index, _) in points.iter().enumerate() {
                q_evals[point_index] =
                    eval_polynomial(&q_polys[point_index].as_ref().unwrap(), x_6);
            }

            for eval in q_evals.iter() {
                transcript_scalar.absorb(*eval);
            }

            let transcript_scalar_point =
                C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
            transcript.absorb(transcript_scalar_point);

            let x_7: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let mut f_blind_dup = f_blind;
            let mut f_poly = f_poly.clone();
            for (point_index, _) in points.iter().enumerate() {
                f_blind_dup *= x_7;
                f_blind_dup += q_blinds[point_index];

                parallelize(&mut f_poly, |f, start| {
                    for (f, a) in f
                        .iter_mut()
                        .zip(q_polys[point_index].as_ref().unwrap()[start..].iter())
                    {
                        *f *= &x_7;
                        *f += a;
                    }
                });
            }

            if let Ok(opening) =
                commitment::Proof::create(&params, &mut transcript, &f_poly, f_blind_dup, x_6)
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
