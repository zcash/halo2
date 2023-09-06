mod prover;
mod verifier;

use crate::multicore::IntoParallelIterator;
use crate::{poly::query::Query, transcript::ChallengeScalar};
use ff::Field;
pub use prover::ProverSHPLONK;
use std::collections::BTreeSet;
pub use verifier::VerifierSHPLONK;

#[cfg(feature = "multicore")]
use crate::multicore::ParallelIterator;

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
struct Commitment<F: Field, T: PartialEq + Clone>((T, Vec<F>));

impl<F: Field, T: PartialEq + Clone> Commitment<F, T> {
    fn get(&self) -> T {
        self.0 .0.clone()
    }

    fn evals(&self) -> Vec<F> {
        self.0 .1.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RotationSet<F: Field, T: PartialEq + Clone> {
    commitments: Vec<Commitment<F, T>>,
    points: Vec<F>,
}

#[derive(Debug, PartialEq)]
struct IntermediateSets<F: Field, Q: Query<F>> {
    rotation_sets: Vec<RotationSet<F, Q::Commitment>>,
    super_point_set: BTreeSet<F>,
}

fn construct_intermediate_sets<F: Field + Ord, I, Q: Query<F, Eval = F>>(
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

    // All points that appear in queries
    let mut super_point_set = BTreeSet::new();

    // Collect rotation sets for each commitment
    // Example elements in the vector:
    // (C_0, {r_5}),
    // (C_1, {r_1, r_2, r_3}),
    // (C_2, {r_2, r_3, r_4}),
    // (C_3, {r_2, r_3, r_4}),
    // ...
    let mut commitment_rotation_set_map: Vec<(Q::Commitment, BTreeSet<F>)> = vec![];
    for query in queries.iter() {
        let rotation = query.get_point();
        super_point_set.insert(rotation);
        if let Some(commitment_rotation_set) = commitment_rotation_set_map
            .iter_mut()
            .find(|(commitment, _)| *commitment == query.get_commitment())
        {
            let (_, rotation_set) = commitment_rotation_set;
            rotation_set.insert(rotation);
        } else {
            commitment_rotation_set_map.push((
                query.get_commitment(),
                BTreeSet::from_iter(std::iter::once(rotation)),
            ));
        };
    }

    // Flatten rotation sets and collect commitments that opens against each commitment set
    // Example elements in the vector:
    // {r_5}: [C_0],
    // {r_1, r_2, r_3} : [C_1]
    // {r_2, r_3, r_4} : [C_2, C_3],
    // ...
    // NOTE: we want to make the order of the collection of rotation sets independent of the opening points, to ease the verifier computation
    let mut rotation_set_commitment_map: Vec<(BTreeSet<F>, Vec<Q::Commitment>)> = vec![];
    for (commitment, rotation_set) in commitment_rotation_set_map.into_iter() {
        if let Some(rotation_set_commitment) = rotation_set_commitment_map
            .iter_mut()
            .find(|(set, _)| set == &rotation_set)
        {
            let (_, commitments) = rotation_set_commitment;
            commitments.push(commitment);
        } else {
            rotation_set_commitment_map.push((rotation_set, vec![commitment]));
        };
    }

    let rotation_sets = rotation_set_commitment_map
        .into_par_iter()
        .map(|(rotations, commitments)| {
            let rotations_vec = rotations.iter().collect::<Vec<_>>();
            let commitments: Vec<Commitment<F, Q::Commitment>> = commitments
                .into_par_iter()
                .map(|commitment| {
                    let evals: Vec<F> = rotations_vec
                        .as_slice()
                        .into_par_iter()
                        .map(|&&rotation| get_eval(commitment, rotation))
                        .collect();
                    Commitment((commitment, evals))
                })
                .collect();

            RotationSet {
                commitments,
                points: rotations.into_iter().collect(),
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
    use super::{construct_intermediate_sets, Commitment, IntermediateSets};
    use ff::FromUniformBytes;
    use halo2curves::pasta::Fp;
    use proptest::{collection::vec, prelude::*, sample::select};
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
            Fp::from_uniform_bytes(&<[u8; 64]>::try_from(bytes).unwrap())
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
