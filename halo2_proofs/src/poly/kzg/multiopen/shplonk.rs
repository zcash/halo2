mod prover;
mod verifier;

pub use prover::ProverSHPLONK;
pub use verifier::VerifierSHPLONK;

use crate::{
    arithmetic::{eval_polynomial, lagrange_interpolate, CurveAffine, FieldExt},
    poly::{query::Query, Coeff, Polynomial},
    transcript::ChallengeScalar,
};

use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    marker::PhantomData,
};

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

fn construct_intermediate_sets<F: FieldExt, I, Q: Query<F, Eval = F>>(
    queries: I,
) -> IntermediateSets<F, Q>
where
    I: IntoIterator<Item = Q> + Clone,
{
    let queries = queries.into_iter().collect::<Vec<_>>();

    // Find evaluation of a commitment at a rotation
    let get_eval = |commitment: Q::Commitment, rotation: F| -> F {
        queries
            .iter()
            .find(|query| query.get_commitment() == commitment && query.get_point() == rotation)
            .unwrap()
            .get_eval()
    };

    // Order points according to their rotation
    let mut rotation_point_map = BTreeMap::new();
    for query in queries.clone() {
        let point = rotation_point_map
            .entry(query.get_point())
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
    let mut commitment_rotation_set_map: Vec<(Q::Commitment, Vec<F>)> = vec![];
    for query in queries.clone() {
        let rotation = query.get_point();
        if let Some(pos) = commitment_rotation_set_map
            .iter()
            .position(|(commitment, _)| *commitment == query.get_commitment())
        {
            let (_, rotation_set) = &mut commitment_rotation_set_map[pos];
            if !rotation_set.contains(&rotation) {
                rotation_set.push(rotation);
            }
        } else {
            commitment_rotation_set_map.push((query.get_commitment(), vec![rotation]));
        };
    }

    // Flatten rotation sets and collect commitments that opens against each commitment set
    // Example elements in the vector:
    // {r_5}: [C_0],
    // {r_1, r_2, r_3} : [C_1]
    // {r_2, r_3, r_4} : [C_2, C_3],
    // ...
    let mut rotation_set_commitment_map = Vec::<(Vec<_>, Vec<Q::Commitment>)>::new();
    for (commitment, rotation_set) in commitment_rotation_set_map.iter() {
        if let Some(pos) = rotation_set_commitment_map.iter().position(|(set, _)| {
            BTreeSet::<F>::from_iter(set.iter().cloned())
                == BTreeSet::<F>::from_iter(rotation_set.iter().cloned())
        }) {
            let (_, commitments) = &mut rotation_set_commitment_map[pos];
            if !commitments.contains(commitment) {
                commitments.push(*commitment);
            }
        } else {
            rotation_set_commitment_map.push((rotation_set.clone(), vec![*commitment]))
        }
    }

    let rotation_sets = rotation_set_commitment_map
        .into_iter()
        .map(|(rotations, commitments)| {
            let commitments: Vec<Commitment<F, Q::Commitment>> = commitments
                .iter()
                .map(|commitment| {
                    let evals: Vec<F> = rotations
                        .iter()
                        .map(|rotation| get_eval(*commitment, *rotation))
                        .collect();
                    Commitment((*commitment, evals))
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
mod proptests {
    use proptest::{
        collection::vec,
        prelude::*,
        sample::{select, subsequence},
        strategy::Strategy,
    };

    use super::{construct_intermediate_sets, Commitment, IntermediateSets};
    use crate::poly::Rotation;
    use halo2curves::{pasta::Fp, FieldExt};

    use std::collections::BTreeMap;
    use std::convert::TryFrom;

    #[derive(Debug, Clone)]
    struct MyQuery<F> {
        point: F,
        eval: F,
        commitment: usize,
    }

    impl super::Query<Fp> for MyQuery<Fp> {
        type Commitment = usize;
        type Eval = Fp;

        fn get_point(&self) -> Fp {
            self.point
        }

        fn get_eval(&self) -> Self::Eval {
            self.eval
        }

        fn get_commitment(&self) -> Self::Commitment {
            self.commitment
        }
    }

    prop_compose! {
        fn arb_point()(
            bytes in vec(any::<u8>(), 64)
        ) -> Fp {
            Fp::from_bytes_wide(&<[u8; 64]>::try_from(bytes).unwrap())
        }
    }

    prop_compose! {
        fn arb_query(commitment: usize, point: Fp)(
            eval in arb_point()
        ) -> MyQuery<Fp> {
            MyQuery {
                point,
                eval,
                commitment
            }
        }
    }

    prop_compose! {
        // Mapping from column index to point index.
        fn arb_queries_inner(num_points: usize, num_cols: usize, num_queries: usize)(
            col_indices in vec(select((0..num_cols).collect::<Vec<_>>()), num_queries),
            point_indices in vec(select((0..num_points).collect::<Vec<_>>()), num_queries)
        ) -> Vec<(usize, usize)> {
            col_indices.into_iter().zip(point_indices.into_iter()).collect()
        }
    }

    prop_compose! {
        fn compare_queries(
            num_points: usize,
            num_cols: usize,
            num_queries: usize,
        )(
            points_1 in vec(arb_point(), num_points),
            points_2 in vec(arb_point(), num_points),
            mapping in arb_queries_inner(num_points, num_cols, num_queries)
        )(
            queries_1 in mapping.iter().map(|(commitment, point_idx)| arb_query(*commitment, points_1[*point_idx])).collect::<Vec<_>>(),
            queries_2 in mapping.iter().map(|(commitment, point_idx)| arb_query(*commitment, points_2[*point_idx])).collect::<Vec<_>>(),
        ) -> (
            Vec<MyQuery<Fp>>,
            Vec<MyQuery<Fp>>
        ) {
            (
                queries_1,
                queries_2,
            )
        }
    }

    proptest! {
        #[test]
        fn test_intermediate_sets(
            (queries_1, queries_2) in compare_queries(8, 8, 16)
        ) {
            let IntermediateSets { rotation_sets, .. } = construct_intermediate_sets(queries_1);
            let commitment_sets = rotation_sets.iter().map(|data|
                data.commitments.iter().map(Commitment::get).collect::<Vec<_>>()
            ).collect::<Vec<_>>();

            // It shouldn't matter what the point or eval values are; we should get
            // the same exact point set indices and point indices again.
            let IntermediateSets { rotation_sets: new_rotation_sets, .. } = construct_intermediate_sets(queries_2);
            let new_commitment_sets = new_rotation_sets.iter().map(|data|
                data.commitments.iter().map(Commitment::get).collect::<Vec<_>>()
            ).collect::<Vec<_>>();

            assert_eq!(commitment_sets, new_commitment_sets);
        }
    }
}
