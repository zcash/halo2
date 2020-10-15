use super::super::{
    commitment::{Guard, Params, MSM},
    Error,
};
use super::{Proof, VerifierQuery};
use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, lagrange_interpolate, Challenge, CurveAffine, Field,
};
use crate::plonk::hash_point;
use crate::transcript::Hasher;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct CommitmentData<C: CurveAffine> {
    set_index: usize,
    point_indices: Vec<usize>,
    evals: Vec<C::Scalar>,
}

impl<C: CurveAffine> Proof<C> {
    /// Verify a multi-opening proof
    pub fn verify<'a, I, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        params: &'a Params<C>,
        transcript: &mut HBase,
        transcript_scalar: &mut HScalar,
        queries: I,
        mut msm: MSM<'a, C>,
    ) -> Result<Guard<'a, C>, Error>
    where
        I: IntoIterator<Item = VerifierQuery<'a, C>> + Clone,
    {
        // Scale the MSM by a random factor to ensure that if the existing MSM
        // has is_zero() == false then this argument won't be able to interfere
        // with it to make it true, with high probability.
        msm.scale(C::Scalar::random());

        // Sample x_4 for compressing openings at the same points together
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let (commitment_map, point_sets) = construct_intermediate_sets::<'a, C, I>(queries.clone());

        // Compress the commitments and expected evaluations at x_3 together.
        // using the challenge x_4
        let mut q_commitments: Vec<_> = vec![params.empty_msm(); point_sets.len()];

        // A vec of vecs of evals. The outer vec corresponds to the point set,
        // while the inner vec corresponds to the points in a particular set.
        let mut q_eval_sets: Vec<Vec<C::Scalar>> = vec![Vec::new(); point_sets.len()];
        for (set_idx, point_set) in point_sets.iter().enumerate() {
            q_eval_sets[set_idx] = vec![C::Scalar::zero(); point_set.len()];
        }
        {
            let mut accumulate = |set_idx: usize, new_commitment, evals: Vec<C::Scalar>| {
                q_commitments[set_idx].scale(x_4);
                q_commitments[set_idx].add_term(C::Scalar::one(), new_commitment);
                for (eval, set_eval) in evals.iter().zip(q_eval_sets[set_idx].iter_mut()) {
                    *set_eval *= &x_4;
                    *set_eval += eval;
                }
            };

            // Each commitment corresponds to evaluations at a set of points.
            // For each set, we collapse each commitment's evals pointwise.
            for (commitment, commitment_data) in commitment_map.into_iter() {
                accumulate(
                    commitment_data.set_index, // set_idx,
                    *commitment,               // commitment,
                    commitment_data.evals,     // evals
                );
            }
        }

        // Sample a challenge x_5 for keeping the multi-point quotient
        // polynomial terms linearly independent.
        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain the commitment to the multi-point quotient polynomial f(X).
        hash_point(transcript, &self.f_commitment).unwrap();

        // Sample a challenge x_6 for checking that f(X) was committed to
        // correctly.
        let x_6: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        for eval in self.q_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        // We can compute the expected msm_eval at x_6 using the q_evals provided
        // by the prover and from x_5
        let msm_eval = point_sets
            .iter()
            .zip(q_eval_sets.iter())
            .zip(self.q_evals.iter())
            .fold(
                C::Scalar::zero(),
                |msm_eval, ((points, evals), proof_eval)| {
                    let r_poly = lagrange_interpolate(points, evals);
                    let r_eval = eval_polynomial(&r_poly, x_6);
                    let eval = points.iter().fold(*proof_eval - &r_eval, |eval, point| {
                        eval * &(x_6 - &point).invert().unwrap()
                    });
                    msm_eval * &x_5 + &eval
                },
            );

        // Sample a challenge x_7 that we will use to collapse the openings of
        // the various remaining polynomials at x_6 together.
        let x_7: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compute the final commitment that has to be opened
        let mut commitment_msm = params.empty_msm();
        commitment_msm.add_term(C::Scalar::one(), self.f_commitment);
        let (commitment_msm, msm_eval) = q_commitments.iter().zip(self.q_evals.iter()).fold(
            (commitment_msm, msm_eval),
            |(mut commitment_msm, msm_eval), (q_commitment, q_eval)| {
                commitment_msm.scale(x_7);
                commitment_msm.add_msm(&q_commitment);
                (commitment_msm, msm_eval * &x_7 + &q_eval)
            },
        );

        // Verify the opening proof
        self.opening
            .verify(params, msm, transcript, x_6, commitment_msm, msm_eval)
    }
}

// For multiopen verifier: Construct intermediate representations relating commitments to sets of points by index
fn construct_intermediate_sets<'a, C: CurveAffine, I>(
    queries: I,
) -> (
    Vec<(&'a C, CommitmentData<C>)>, // commitment_map
    Vec<Vec<C::Scalar>>,             // point_sets
)
where
    I: IntoIterator<Item = VerifierQuery<'a, C>> + Clone,
{
    // Construct sets of unique commitments and corresponding information about their queries
    let mut commitment_map: Vec<(&'a C, CommitmentData<C>)> = Vec::new();

    // Also construct mapping from a unique point to a point_index. This defines an ordering on the points.
    let mut point_index_map: BTreeMap<C::Scalar, usize> = BTreeMap::new();

    // Construct point_indices which each commitment is queried at
    for query in queries.clone() {
        let num_points = point_index_map.len();
        let point_idx = point_index_map.entry(query.point).or_insert(num_points);

        let mut exists = false;
        for (existing_commitment, existing_commitment_data) in commitment_map.iter_mut() {
            // Add to CommitmentData for existing commitment in commitment_map
            if std::ptr::eq(query.commitment, *existing_commitment) {
                exists = true;
                existing_commitment_data.point_indices.push(*point_idx);
            }
        }

        // Add new commitment and CommitmentData to commitment_map
        if !exists {
            let commitment_data = CommitmentData {
                set_index: 0,
                point_indices: vec![*point_idx],
                evals: vec![],
            };
            commitment_map.push((query.commitment, commitment_data));
        }
    }

    // Also construct inverse mapping from point_index to the point
    let mut inverse_point_index_map: BTreeMap<usize, C::Scalar> = BTreeMap::new();
    for (&point, &point_index) in point_index_map.iter() {
        inverse_point_index_map.insert(point_index, point);
    }

    // Construct map of unique ordered point_idx_sets to their set_idx
    let mut point_idx_sets: BTreeMap<BTreeSet<usize>, usize> = BTreeMap::new();
    // Also construct mapping from commitment to point_idx_set
    let mut commitment_set_map: Vec<(&'a C, BTreeSet<usize>)> = Vec::new();

    for (commitment, commitment_data) in commitment_map.iter_mut() {
        let mut point_index_set = BTreeSet::new();
        // Note that point_index_set is ordered, unlike point_indices
        for &point_index in commitment_data.point_indices.iter() {
            point_index_set.insert(point_index);
        }

        // Push point_index_set to CommitmentData for the relevant commitment
        commitment_set_map.push((commitment, point_index_set.clone()));

        let num_sets = point_idx_sets.len();
        point_idx_sets
            .entry(point_index_set.clone())
            .or_insert(num_sets);
    }

    // Initialise empty evals vec for each unique commitment
    for (_, commitment_data) in commitment_map.iter_mut() {
        let len = commitment_data.point_indices.len();
        commitment_data.evals = vec![C::Scalar::zero(); len];
    }

    // Populate set_index, evals and points for each commitment using point_idx_sets
    for query in queries.clone() {
        // The index of the point at which the commitment is queried
        let point_index = point_index_map.get(&query.point).unwrap();

        // The point_index_set at which the commitment was queried
        let mut point_index_set = BTreeSet::new();
        for (commitment, point_idx_set) in commitment_set_map.iter() {
            if std::ptr::eq(query.commitment, *commitment) {
                point_index_set = point_idx_set.clone();
            }
        }
        // The set_index of the point_index_set
        let set_index = point_idx_sets.get(&point_index_set).unwrap();
        for (commitment, commitment_data) in commitment_map.iter_mut() {
            if std::ptr::eq(query.commitment, *commitment) {
                commitment_data.set_index = *set_index;
            }
        }
        let point_index_set: Vec<usize> = point_index_set.iter().cloned().collect();

        // The offset of the point_index in the point_index_set
        let point_index_in_set = point_index_set
            .iter()
            .position(|i| i == point_index)
            .unwrap();

        for (commitment, commitment_data) in commitment_map.iter_mut() {
            if std::ptr::eq(query.commitment, *commitment) {
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

    (commitment_map, point_sets)
}
