mod prover;
mod verifier;

use super::Query;
use crate::{
    arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt},
    poly::{msm::MSM, Coeff, Polynomial},
    transcript::ChallengeScalar,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

pub use prover::create_proof;
pub use verifier::verify_proof;

#[derive(Clone, Copy, Debug)]
struct U {}
type ChallengeU<F> = ChallengeScalar<F, U>;

#[derive(Clone, Copy, Debug)]
struct V {}
type ChallengeV<F> = ChallengeScalar<F, V>;

#[derive(Clone, Copy, Debug)]
struct Y {}
type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone)]
struct Commitment<F: FieldExt, T: PartialEq + Clone>((T, Vec<F>));

impl<F: FieldExt, T: PartialEq + Clone> Commitment<F, T> {
    fn get(&self) -> T {
        self.0 .0.clone()
    }

    fn evals(&self) -> Vec<F> {
        self.0 .1.clone()
    }
}

struct RotationSet<F: FieldExt, T: PartialEq + Clone> {
    commitments: Vec<Commitment<F, T>>,
    points: Vec<F>,
    diffs: Vec<F>,
}

struct IntermediateSets<F: FieldExt, Q: Query<F>> {
    rotation_sets: Vec<RotationSet<F, Q::Commitment>>,
    super_point_set: Vec<F>,
}

fn construct_intermediate_sets<F: FieldExt, I, Q: Query<F>>(queries: I) -> IntermediateSets<F, Q>
where
    I: IntoIterator<Item = Q> + Clone,
{
    // 1. collect points to construct vanishing polynomial of all points
    let mut super_point_set = BTreeSet::new();
    // point_set = { p_0, p_1, ... }
    for query in queries.clone() {
        super_point_set.insert(query.get_point());
    }

    // 2. identify commitments
    // commitment_ids = [(1, c_0), (2, c_1), ... ]
    let mut commitment_ids = vec![];
    // id index starts from 1
    let mut id = 1usize;
    for query in queries.clone() {
        let mut found = false;
        for (_, commitment) in commitment_ids.iter() {
            if *commitment == query.get_commitment() {
                found = true;
                break;
            }
        }
        if !found {
            commitment_ids.push((id, query.get_commitment()));
            id += 1;
        }
    }

    let get_commitment = |id: usize| -> Q::Commitment {
        for (_id, commitment) in commitment_ids.clone().into_iter() {
            if _id == id {
                return commitment;
            }
        }
        panic!("must find a commitment");
    };

    let get_commitment_id = |query: &Q| -> usize {
        let mut id = 0;
        for (_id, commitment) in commitment_ids.iter() {
            if query.get_commitment() == *commitment {
                id = *_id;
            } else {
                continue;
            }
        }
        // an id must be found
        assert_ne!(id, 0);

        id
    };

    // 3.a. map points to commitments
    // commitment_id_point_map = { c_1: { p_0, p_1, ... }, c_2: { p_1, p_2, ... }, ... }
    let mut commitment_id_point_map = BTreeMap::new();
    // 3.b. map points to commitments
    // commitment_id_point_eval_map_map = { c_1: { p_0: e_0, p_1: e_1, ... }, c_2: { p_1: e_1, p_2: e_2, ... }, ... }
    let mut commitment_id_point_eval_map_map = BTreeMap::new();
    for query in queries.clone() {
        let id = get_commitment_id(&query);

        commitment_id_point_map
            .entry(id)
            // create new set for points
            .or_insert_with(BTreeSet::new)
            // insert the point, there won't be duplicates
            .insert(query.get_point());

        commitment_id_point_eval_map_map
            .entry(id)
            // create new map for point to eval map
            .or_insert_with(BTreeMap::new)
            // insert point eval key values
            .insert(query.get_point(), query.get_eval());
    }

    // 4. find diff points
    // commitment_id_diff_points_map = { c_1:  { p_2, p_3, ... }, c_2: { p_0 }, ... }
    let mut commitment_id_diff_points_map = BTreeMap::new();
    for query in queries.clone() {
        let id = get_commitment_id(&query);
        let commitment_point_set = commitment_id_point_map.get(&id).unwrap();

        // diff_set = super_point_set \ commitment_point_set
        let diff_set: BTreeSet<F> = super_point_set
            .difference(commitment_point_set)
            .cloned()
            .collect();
        commitment_id_diff_points_map.insert(id, diff_set);
    }

    assert_eq!(
        commitment_id_point_map.len(),
        commitment_id_diff_points_map.len()
    );

    // 5. construct point_set to commitment map
    // counterwise of commitment_id_diff_points_map
    let mut points_commitment_id_map = BTreeMap::new();
    for (commitment_id, point_set) in commitment_id_point_map.iter() {
        points_commitment_id_map
            .entry(point_set)
            // create new set for commitment ids
            .or_insert_with(BTreeSet::new)
            // insert the id, there won't be duplicates
            .insert(*commitment_id);
    }

    // 6. finally construct intermediate sets
    let mut rotation_sets = vec![];
    for (points, commitment_ids) in points_commitment_id_map {
        assert!(!commitment_ids.is_empty());
        let commitment_ids: Vec<usize> = commitment_ids.iter().cloned().collect();

        let commitments: Vec<Commitment<F, Q::Commitment>> = commitment_ids
            .iter()
            .map(|id| {
                let commitment = get_commitment(*id);

                let point_eval_map = commitment_id_point_eval_map_map.get(id).unwrap();
                let evals: Vec<F> = points
                    .iter()
                    .map(|point| *point_eval_map.get(point).unwrap())
                    .collect();
                Commitment((commitment, evals))
            })
            .collect();

        let some_id = commitment_ids[0];
        let diffs: Vec<F> = commitment_id_diff_points_map
            .get(&some_id)
            .unwrap()
            .iter()
            .cloned()
            .collect();
        let points: Vec<F> = points.iter().cloned().collect();

        let rotation_set = RotationSet {
            commitments,
            points,
            diffs,
        };

        rotation_sets.push(rotation_set);
    }

    IntermediateSets {
        rotation_sets,
        super_point_set: super_point_set.iter().cloned().collect(),
    }
}

#[cfg(test)]
mod tests {

    use super::construct_intermediate_sets;
    use crate::arithmetic::{eval_polynomial, FieldExt};
    use crate::pairing::bn256::{Bn256, Fr, G1Affine};
    use crate::poly::{
        commitment::{Params, ParamsVerifier},
        multiopen::{create_proof, verify_proof},
        multiopen::{ProverQuery, Query, VerifierQuery},
        Coeff, Polynomial,
    };
    use crate::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, ChallengeScalar, Transcript, TranscriptRead,
        TranscriptWrite,
    };

    use ff::Field;
    use rand::RngCore;
    use rand_core::OsRng;
    use std::collections::BTreeSet;
    use std::marker::PhantomData;

    #[derive(Clone)]
    pub(super) struct MyQuery<F> {
        commitment: usize,
        point: F,
        eval: F,
    }

    impl<F: FieldExt> Query<F> for MyQuery<F> {
        type Commitment = usize;

        fn get_point(&self) -> F {
            self.point
        }
        fn get_eval(&self) -> F {
            self.eval
        }
        fn get_commitment(&self) -> Self::Commitment {
            self.commitment
        }
    }

    fn rotation_set(points: Vec<u64>) -> Vec<Fr> {
        points.into_iter().map(Fr::from).collect()
    }

    fn make_query(commitment: usize, point: Fr) -> MyQuery<Fr> {
        MyQuery {
            commitment,
            point,
            eval: point + Fr::from(commitment as u64),
        }
    }

    #[test]
    fn test_intermediate_sets() {
        fn vec_to_set<T: Ord>(v: Vec<T>) -> BTreeSet<T> {
            let mut set = BTreeSet::new();
            for el in v {
                set.insert(el);
            }
            set
        }

        let rotation_sets_init = vec![vec![1u64, 2, 3], vec![2, 3, 4], vec![4, 5, 6, 7], vec![8]];
        let number_of_sets = rotation_sets_init.len();
        let rotation_sets: Vec<Vec<Fr>> = rotation_sets_init
            .clone()
            .into_iter()
            .map(rotation_set)
            .collect();
        let mut super_point_set: Vec<Fr> = rotation_sets.clone().into_iter().flatten().collect();
        super_point_set.sort();
        super_point_set.dedup();

        let commitment_per_set = 3;
        let number_of_commitments = commitment_per_set * rotation_sets.len();

        let mut queries: Vec<MyQuery<Fr>> = vec![];

        for i in 0..number_of_commitments {
            let rotation_set = &rotation_sets[i % rotation_sets.len()];
            for point in rotation_set.iter() {
                let query = make_query(i, *point);
                queries.push(query);
            }
        }

        let intermediate_sets = construct_intermediate_sets(queries);
        assert_eq!(intermediate_sets.rotation_sets.len(), rotation_sets.len());
        assert_eq!(intermediate_sets.super_point_set, super_point_set);

        let (rotation_sets, super_point_set) = (
            intermediate_sets.rotation_sets,
            intermediate_sets.super_point_set,
        );

        for (i, rotation_set) in rotation_sets.iter().enumerate() {
            let commitments = rotation_set.commitments.clone();
            assert_eq!(commitments.len(), commitment_per_set);
            for (j, commitment) in commitments.iter().enumerate() {
                let commitment_id: usize = commitment.get();
                assert_eq!(commitment_id, number_of_sets * j + i);
            }

            let points: Vec<Fr> = rotation_set.points.clone();
            let diffs: Vec<Fr> = rotation_set.diffs.clone();

            assert_eq!(points.len(), rotation_sets_init[i].len());

            let points = vec_to_set(points);
            let diffs = vec_to_set(diffs);
            let intersection: Vec<Fr> = points.intersection(&diffs).cloned().collect();
            assert_eq!(intersection.len(), 0);
            let union: Vec<Fr> = points.union(&diffs).cloned().collect();
            assert_eq!(union, super_point_set);
        }
    }
}
