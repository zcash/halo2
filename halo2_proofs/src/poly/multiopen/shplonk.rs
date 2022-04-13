mod prover;
mod verifier;

use super::Query;
use crate::{
    arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt},
    poly::{msm::MSM, Coeff, Polynomial},
    transcript::ChallengeScalar,
};

use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    marker::PhantomData,
};

use crate::poly::Rotation;
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

#[derive(Debug, Clone, PartialEq)]
struct Commitment<F: FieldExt, T: PartialEq + Clone>((T, Vec<F>));

impl<F: FieldExt, T: PartialEq + Clone> Commitment<F, T> {
    fn get(&self) -> T {
        self.0 .0.clone()
    }

    fn evals(&self) -> Vec<F> {
        self.0 .1.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RotationSet<F: FieldExt, T: PartialEq + Clone> {
    commitments: Vec<Commitment<F, T>>,
    points: Vec<F>,
}

#[derive(Debug, PartialEq)]
struct IntermediateSets<F: FieldExt, Q: Query<F>> {
    rotation_sets: Vec<RotationSet<F, Q::Commitment>>,
    super_point_set: Vec<F>,
}

fn construct_intermediate_sets<F: FieldExt, I, Q: Query<F>>(queries: I) -> IntermediateSets<F, Q>
where
    I: IntoIterator<Item = Q> + Clone,
{
    let queries = queries.into_iter().collect::<Vec<_>>();

    // Find evaluation of a commitment at a rotation
    let get_eval = |commitment: Q::Commitment, rotation: Rotation| -> F {
        queries
            .iter()
            .find(|query| query.get_commitment() == commitment && query.get_rotation() == rotation)
            .unwrap()
            .get_eval()
    };

    // Order points according to their rotation
    let mut rotation_point_map = BTreeMap::new();
    for query in queries.clone() {
        let point = rotation_point_map
            .entry(query.get_rotation())
            .or_insert_with(|| query.get_point());

        // Assert rotation point matching consistency
        assert_eq!(*point, query.get_point());
    }
    // All points appear in queries
    let super_point_set: Vec<F> = rotation_point_map.values().cloned().collect();

    // Collect rotation sets for each commitment
    // Example elements in the vector:
    // (C_0, {r_5}),
    // (C_1, {r_1, r_2, r_3}),
    // (C_2, {r_2, r_3, r_4}),
    // (C_3, {r_2, r_3, r_4}),
    // ...
    let mut commitment_rotation_set_map: Vec<(Q::Commitment, BTreeSet<Rotation>)> = vec![];
    for query in queries.clone() {
        let rotation = query.get_rotation();
        if let Some(pos) = commitment_rotation_set_map
            .iter()
            .position(|(commitment, _)| *commitment == query.get_commitment())
        {
            let (_, rotation_set) = &mut commitment_rotation_set_map[pos];
            rotation_set.insert(rotation);
        } else {
            let rotation_set = BTreeSet::from([rotation]);
            commitment_rotation_set_map.push((query.get_commitment(), rotation_set));
        };
    }

    // Flatten rotation sets and collect commitments that opens against each commitment set
    // Example elements in the vector:
    // {r_5}: [C_0],
    // {r_1, r_2, r_3} : [C_1]
    // {r_2, r_3, r_4} : [C_2, C_3],
    // ...
    let mut rotation_set_commitment_map = BTreeMap::<BTreeSet<_>, Vec<Q::Commitment>>::new();
    for (commitment, rotation_set) in commitment_rotation_set_map.iter() {
        let commitments = rotation_set_commitment_map
            .entry(rotation_set.clone())
            .or_insert_with(Vec::new);
        if !commitments.contains(commitment) {
            commitments.push(commitment.clone());
        }
    }

    let rotation_sets = rotation_set_commitment_map
        .into_iter()
        .map(|(rotation_set, commitments)| {
            let rotations: Vec<Rotation> = rotation_set.iter().cloned().collect();

            let commitments: Vec<Commitment<F, Q::Commitment>> = commitments
                .iter()
                .map(|commitment| {
                    let evals: Vec<F> = rotations
                        .iter()
                        .map(|rotation| get_eval(commitment.clone(), *rotation))
                        .collect();
                    Commitment((commitment.clone(), evals))
                })
                .collect();

            RotationSet {
                commitments,
                points: rotations
                    .iter()
                    .map(|rotation| *rotation_point_map.get(rotation).unwrap())
                    .collect(),
            }
        })
        .collect::<Vec<RotationSet<_, _>>>();

    IntermediateSets {
        rotation_sets,
        super_point_set,
    }
}

#[cfg(test)]
mod tests {

    use super::construct_intermediate_sets;
    use crate::arithmetic::{eval_polynomial, FieldExt};
    use crate::pairing::bn256::{Bn256, Fr, G1Affine};
    use crate::poly::multiopen::shplonk::{IntermediateSets, RotationSet};
    use crate::poly::{
        commitment::{Params, ParamsVerifier},
        multiopen::{create_proof, verify_proof},
        multiopen::{ProverQuery, Query, VerifierQuery},
        Coeff, Polynomial, Rotation,
    };
    use crate::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, ChallengeScalar, Transcript, TranscriptRead,
        TranscriptWrite,
    };

    use ff::Field;
    use rand::RngCore;
    use rand_core::OsRng;
    use std::collections::{BTreeMap, BTreeSet};
    use std::marker::PhantomData;

    #[derive(Clone, Debug, PartialEq)]
    pub(super) struct MyQuery<F> {
        commitment: usize,
        point: F,
        rotation: Rotation,
        eval: F,
    }

    impl<F: FieldExt> Query<F> for MyQuery<F> {
        type Commitment = usize;

        fn get_rotation(&self) -> Rotation {
            self.rotation
        }
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

    #[test]
    fn test_intermediate_sets() {
        let mut rng = OsRng;
        use rand::seq::SliceRandom;

        for _ in 0..100 {
            let rotation_range = -4..5;
            let number_of_queries = 16;
            let number_of_commitments = 8;
            let mut point_rotation_map = BTreeMap::new();
            let rotations: Vec<(Fr, Rotation, usize)> = rotation_range
                .enumerate()
                .map(|(index_of_rot, rot)| {
                    let point = Fr::random(&mut rng);
                    let rotation = Rotation(rot);
                    point_rotation_map.insert(point, rotation);
                    (point, rotation, index_of_rot)
                })
                .collect();

            let mut evals = BTreeMap::new();
            for (point, _, _) in rotations.iter() {
                for i in 0..number_of_commitments {
                    evals.insert((i, *point), Fr::random(&mut rng));
                }
            }

            let check_evals = |rotation_sets: Vec<RotationSet<Fr, usize>>| {
                for rotation_set in rotation_sets.iter() {
                    let points = rotation_set.points.clone();
                    for commitment in rotation_set.commitments.iter() {
                        // let evals = commitment.evals();
                        let com = commitment.get();
                        for (eval_0, point) in commitment.evals().iter().zip(points.iter()) {
                            let eval_1 = evals.get(&(com, *point)).unwrap();
                            assert_eq!(eval_0, eval_1);
                        }
                    }
                }
            };

            let queries_0: Vec<MyQuery<_>> = (0usize..number_of_queries)
                .map(|_| {
                    let commitments: Vec<usize> = (0..number_of_commitments).collect();
                    let commitment = commitments.choose(&mut rng).unwrap();
                    let rotation = rotations.choose(&mut rng).unwrap();
                    let eval = *evals.get(&(*commitment, rotation.0)).unwrap();
                    MyQuery {
                        commitment: *commitment,
                        point: rotation.0,
                        rotation: rotation.1,
                        eval,
                    }
                })
                .collect();

            let IntermediateSets {
                rotation_sets: rotation_sets_0,
                super_point_set: _,
            } = construct_intermediate_sets(queries_0.clone());
            check_evals(rotation_sets_0.clone());

            // Change points and check partial equality
            let mut queries_1 = queries_0.clone();
            let e = Fr::random(&mut rng);
            for q in queries_1.iter_mut() {
                q.point = e * Fr::from(q.rotation.0 as u64);
            }
            let IntermediateSets {
                rotation_sets: rotation_sets_1,
                super_point_set: _,
            } = construct_intermediate_sets(queries_1.clone());

            for (rotation_set_0, rotation_set_1) in
                rotation_sets_0.iter().zip(rotation_sets_1.iter())
            {
                assert_eq!(rotation_set_0.commitments, rotation_set_1.commitments);
            }

            let make_queries = |rotation_sets: Vec<RotationSet<Fr, usize>>| -> Vec<MyQuery<Fr>> {
                let mut queries: Vec<MyQuery<Fr>> = vec![];
                for rotation_set in rotation_sets.iter() {
                    let points = rotation_set.points.clone();
                    let rotations: Vec<Rotation> = points
                        .iter()
                        .map(|point| *point_rotation_map.get(point).unwrap())
                        .collect();
                    for commitment in rotation_set.commitments.iter() {
                        for ((point, rotation), eval) in points
                            .iter()
                            .zip(rotations.iter())
                            .zip(commitment.evals().iter())
                        {
                            queries.push(MyQuery {
                                commitment: commitment.get(),
                                point: *point,
                                rotation: *rotation,
                                eval: *eval,
                            })
                        }
                    }
                }
                queries
            };

            // Construct queries from rotation sets
            let queries_1 = make_queries(rotation_sets_0);
            for q in queries_0.iter() {
                assert!(queries_1.contains(q));
            }
            for q in queries_1.iter() {
                assert!(queries_0.contains(q));
            }

            // Shuffle queries and compare results
            let mut queries_1 = queries_0.clone();
            queries_1.shuffle(&mut rng);
            let IntermediateSets {
                rotation_sets: rotation_sets_1,
                super_point_set: _,
            } = construct_intermediate_sets(queries_1.clone());
            check_evals(rotation_sets_1.clone());

            let queries_1 = make_queries(rotation_sets_1);
            for q in queries_0.iter() {
                assert!(queries_1.contains(q));
            }
            for q in queries_1.iter() {
                assert!(queries_0.contains(q));
            }
        }
    }
}
