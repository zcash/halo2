use super::super::{
    commitment::{self, Blind, Params},
    Coeff, Error, Polynomial,
};
use super::{Proof, ProverQuery};

use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, interpolate, kate_division, parallelize, Challenge,
    Curve, CurveAffine, Field,
};
use crate::plonk::hash_point;
use crate::transcript::Hasher;
use std::collections::{BTreeMap, BTreeSet};
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
        transcript: &mut HBase,
        transcript_scalar: &mut HScalar,
        queries: I,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = ProverQuery<'a, C>> + Clone,
    {
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let (poly_map, point_sets) = construct_intermediate_sets::<'a, C, I>(queries);

        // Collapse openings at same point sets together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> = vec![None; point_sets.len()];
        let mut q_blinds = vec![Blind(C::Scalar::zero()); point_sets.len()];
        let mut q_eval_sets: Vec<Vec<_>> = vec![Vec::new(); point_sets.len()];
        for (set_idx, point_set) in point_sets.iter().enumerate() {
            q_eval_sets[set_idx] = vec![C::Scalar::zero(); point_set.len()];
        }

        {
            let mut accumulate = |set_idx: usize,
                                  new_poly: &Polynomial<C::Scalar, Coeff>,
                                  blind: Blind<C::Scalar>,
                                  evals: Vec<C::Scalar>| {
                q_polys[set_idx]
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
                        q_polys[set_idx] = Some(new_poly.clone());
                        Some(())
                    });
                q_blinds[set_idx] *= x_4;
                q_blinds[set_idx] += blind;
                for (eval_idx, &eval) in evals.iter().enumerate() {
                    q_eval_sets[set_idx][eval_idx] *= &x_4;
                    q_eval_sets[set_idx][eval_idx] += &eval;
                }
            };

            for (poly, commitment_data) in poly_map {
                accumulate(
                    commitment_data.set_index,      // set_idx,
                    &poly,                          // poly,
                    commitment_data.blind,          // blind,
                    commitment_data.evals.to_vec(), // evals
                );
            }
        }

        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let f_poly = point_sets
            .iter()
            .zip(q_eval_sets.iter())
            .zip(q_polys.iter())
            .fold(None, |f_poly, ((points, evals), poly)| {
                let mut poly = poly.clone()?.values;
                // TODO: makes implicit asssumption that poly degree is smaller than interpolation poly degree
                for (p, r) in poly.iter_mut().zip(interpolate(points, evals)) {
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
            });

        let f_poly = f_poly.unwrap();
        let mut f_blind = Blind(C::Scalar::random());
        let mut f_commitment = params.commit(&f_poly, f_blind).to_affine();

        let (opening, q_evals) = loop {
            let mut transcript = transcript.clone();
            let mut transcript_scalar = transcript_scalar.clone();
            hash_point(&mut transcript, &f_commitment).unwrap();

            let x_6: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let q_evals: Vec<C::Scalar> = q_polys
                .iter()
                .map(|poly| eval_polynomial(poly.as_ref().unwrap(), x_6))
                .collect();

            for eval in q_evals.iter() {
                transcript_scalar.absorb(*eval);
            }

            let transcript_scalar_point =
                C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
            transcript.absorb(transcript_scalar_point);

            let x_7: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let (f_poly, f_blind_try) = q_polys.iter().zip(q_blinds.iter()).fold(
                (f_poly.clone(), f_blind),
                |(f_poly, f_blind), (poly, blind)| {
                    (
                        f_poly * x_7 + &poly.clone().unwrap(),
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

// For multiopen prover: Construct intermediate representations relating polynomials to sets of points by index
fn construct_intermediate_sets<'a, C: CurveAffine, I>(
    queries: I,
) -> (
    Vec<(&'a Polynomial<C::Scalar, Coeff>, CommitmentData<C>)>, // poly_map
    Vec<Vec<C::Scalar>>,                                        // point_sets
)
where
    I: IntoIterator<Item = ProverQuery<'a, C>> + Clone,
{
    // Construct vec of unique polynomials and corresponding information about their queries
    let mut poly_map: Vec<(&'a Polynomial<C::Scalar, Coeff>, CommitmentData<C>)> = Vec::new();

    // Also construct mapping from a unique point to a point_index
    let mut point_index_map: BTreeMap<C::Scalar, usize> = BTreeMap::new();

    // Construct point_indices which each polynomial is queried at
    for query in queries.clone() {
        let num_points = point_index_map.len();
        let point_idx = point_index_map.entry(query.point).or_insert(num_points);

        let mut exists = false;
        for (existing_poly, existing_commitment_data) in poly_map.iter_mut() {
            // Add to CommitmentData for existing commitment in commitment_map
            if std::ptr::eq(query.poly, *existing_poly) {
                exists = true;
                existing_commitment_data.point_indices.push(*point_idx);
            }
        }

        // Add new poly and CommitmentData to poly_map
        if !exists {
            let commitment_data = CommitmentData {
                set_index: 0,
                blind: query.blind,
                point_indices: vec![*point_idx],
                evals: vec![],
            };
            poly_map.push((query.poly, commitment_data));
        }
    }

    // Also construct inverse mapping from point_index to the point
    let mut inverse_point_index_map: BTreeMap<usize, C::Scalar> = BTreeMap::new();
    for (&point, &point_index) in point_index_map.iter() {
        inverse_point_index_map.insert(point_index, point);
    }

    // Construct map of unique ordered point_idx_sets to their set_idx
    let mut point_idx_sets: BTreeMap<BTreeSet<usize>, usize> = BTreeMap::new();
    // Also construct mapping from poly to point_idx_set
    let mut poly_set_map: Vec<(&Polynomial<C::Scalar, Coeff>, BTreeSet<usize>)> = Vec::new();

    for (poly, commitment_data) in poly_map.iter_mut() {
        let mut point_index_set = BTreeSet::new();
        // Note that point_index_set is ordered, unlike point_indices
        for &point_index in commitment_data.point_indices.iter() {
            point_index_set.insert(point_index);
        }

        // Push point_index_set to CommitmentData for the relevant poly
        poly_set_map.push((poly, point_index_set.clone()));

        let num_sets = point_idx_sets.len();
        point_idx_sets
            .entry(point_index_set.clone())
            .or_insert(num_sets);
    }

    // Initialise empty evals vec for each unique poly
    for (_, commitment_data) in poly_map.iter_mut() {
        let len = commitment_data.point_indices.len();
        commitment_data.evals = vec![C::Scalar::zero(); len];
    }

    // Populate set_index, evals and points for each poly using point_idx_sets
    for query in queries.clone() {
        // The index of the point at which the poly is queried
        let point_index = point_index_map.get(&query.point).unwrap();

        // The point_index_set at which the poly was queried
        let mut point_index_set = BTreeSet::new();
        for (poly, point_idx_set) in poly_set_map.iter() {
            if std::ptr::eq(query.poly, *poly) {
                point_index_set = point_idx_set.clone();
            }
        }

        // The set_index of the point_index_set
        let set_index = point_idx_sets.get(&point_index_set).unwrap();
        for (poly, commitment_data) in poly_map.iter_mut() {
            if std::ptr::eq(query.poly, *poly) {
                commitment_data.set_index = *set_index;
            }
        }

        let point_index_set: Vec<usize> = point_index_set.iter().cloned().collect();

        // The offset of the point_index in the point_index_set
        let point_index_in_set = point_index_set
            .iter()
            .position(|i| i == point_index)
            .unwrap();

        for (poly, commitment_data) in poly_map.iter_mut() {
            if std::ptr::eq(query.poly, *poly) {
                // Insert the eval using the ordering of the point_index_set
                commitment_data.evals[point_index_in_set] = query.eval;
            }
        }
    }

    // Get actual points in each point set
    let mut point_sets: Vec<Vec<C::Scalar>> = vec![Vec::new(); point_idx_sets.len()];
    for (point_idx_set, &set_idx) in point_idx_sets.iter() {
        for &point_idx in point_idx_set.iter() {
            let point = inverse_point_index_map.get(&point_idx).unwrap();
            point_sets[set_idx].push(*point);
        }
    }

    (poly_map, point_sets)
}
