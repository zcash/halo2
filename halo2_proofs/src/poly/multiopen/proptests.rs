use group::ff::FromUniformBytes;
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    sample::select,
};

use super::{construct_intermediate_sets, CommitmentData, Field, IntermediateSets, Query};
use indexmap::IndexMap;
use pasta_curves::Fp;
use std::collections::{BTreeMap, BTreeSet};

use std::{cell::Cell, convert::TryFrom};

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

#[test]
fn intermediate_sets_match_reference_in_one_traversal() {
    // Repeated points, interleaved commitments, and shared point sets.
    let queries = [
        (7, 2, 72),
        (3, 0, 30),
        (11, 1, 111),
        (7, 1, 71),
        (9, 0, 90),
        (3, 1, 31),
        (11, 2, 112),
    ]
    .map(|(commitment, point, eval)| MyQuery {
        point: Fp::from(point),
        eval: Fp::from(eval),
        commitment,
    });

    // Vary the first-seen commitment, point, and point-set orders.
    for offset in 0..queries.len() {
        for reverse in [false, true] {
            let mut ordered = queries.to_vec();
            ordered.rotate_left(offset);
            if reverse {
                ordered.reverse();
            }
            let (expected_commitments, expected_sets) =
                construct_intermediate_sets_reference(ordered.clone())
                    .expect("queries are distinct");
            let traversed = Cell::new(0);
            let (commitments, point_sets) =
                construct_intermediate_sets(ordered.into_iter().inspect(|_| {
                    traversed.set(traversed.get() + 1);
                }))
                .expect("queries are distinct");

            assert_eq!(traversed.get(), queries.len());
            assert_eq!(point_sets, expected_sets);
            assert_eq!(commitments.len(), expected_commitments.len());
            for (actual, expected) in commitments.iter().zip(&expected_commitments) {
                assert_eq!(actual.commitment, expected.commitment);
                assert_eq!(actual.set_index, expected.set_index);
                assert_eq!(actual.point_indices, expected.point_indices);
                assert_eq!(actual.evals, expected.evals);
            }
        }
    }
}

#[test]
fn intermediate_sets_reject_duplicate_queries() {
    let query = MyQuery {
        point: Fp::from(1),
        eval: Fp::from(2),
        commitment: 3,
    };

    for eval in [query.eval, Fp::from(4)] {
        for duplicate_index in [1, 3] {
            let mut queries = vec![
                query.clone(),
                MyQuery {
                    commitment: 7,
                    ..query.clone()
                },
                MyQuery {
                    point: Fp::from(2),
                    ..query.clone()
                },
            ];
            queries.insert(
                duplicate_index,
                MyQuery {
                    eval,
                    ..query.clone()
                },
            );
            assert!(construct_intermediate_sets_reference(queries.clone()).is_none());
            assert!(construct_intermediate_sets(queries).is_none());
        }
    }
}

#[test]
fn intermediate_sets_accept_empty_queries() {
    for result in [
        construct_intermediate_sets(std::iter::empty::<MyQuery<Fp>>()),
        construct_intermediate_sets_reference(std::iter::empty::<MyQuery<Fp>>()),
    ] {
        let (commitments, point_sets) = result.expect("empty queries remain accepted");
        assert!(commitments.is_empty());
        assert!(point_sets.is_empty());
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
        // Use a HashSet to ensure we sample distinct (column, point) queries.
        queries in hash_set(
            (
                select((0..num_cols).collect::<Vec<_>>()),
                select((0..num_points).collect::<Vec<_>>()),
            ),
            num_queries,
        )
    ) -> Vec<(usize, usize)> {
        queries.into_iter().collect()
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
        let (commitment_data, _point_sets) = construct_intermediate_sets(queries_1)
            .ok_or_else(|| TestCaseError::Fail("mismatched evals".into()))?;
        let set_indices = commitment_data.iter().map(|data| data.set_index).collect::<Vec<_>>();
        let point_indices = commitment_data.iter().map(|data| data.point_indices.clone()).collect::<Vec<_>>();

        // It shouldn't matter what the point or eval values are; we should get
        // the same exact point set indices and point indices again.
        let (new_commitment_data, _new_point_sets) = construct_intermediate_sets(queries_2)
            .ok_or_else(|| TestCaseError::Fail("mismatched evals".into()))?;
        let new_set_indices = new_commitment_data.iter().map(|data| data.set_index).collect::<Vec<_>>();
        let new_point_indices = new_commitment_data.iter().map(|data| data.point_indices.clone()).collect::<Vec<_>>();

        assert_eq!(set_indices, new_set_indices);
        assert_eq!(point_indices, new_point_indices);
    }
}

fn construct_intermediate_sets_reference<F: Field + Ord, I, Q: Query<F>>(
    queries: I,
) -> Option<IntermediateSets<F, Q>>
where
    I: IntoIterator<Item = Q> + Clone,
{
    // Construct sets of unique commitments and corresponding information about
    // their queries.
    let mut commitment_map: IndexMap<Q::Commitment, CommitmentData<Option<Q::Eval>, ()>> =
        IndexMap::new();

    // Also construct mapping from a unique point to a point_index. This defines
    // an ordering on the points.
    let mut point_index_map = BTreeMap::new();

    // Iterate over all of the queries, computing the ordering of the points
    // while also creating new commitment data.
    for query in queries.clone() {
        let num_points = point_index_map.len();
        let point_idx = point_index_map
            .entry(query.get_point())
            .or_insert(num_points);

        commitment_map
            .entry(query.get_commitment())
            .or_insert_with(|| CommitmentData::new(()))
            .point_indices
            .push(*point_idx);
    }

    // Also construct inverse mapping from point_index to the point
    let mut inverse_point_index_map = BTreeMap::new();
    for (&point, &point_index) in point_index_map.iter() {
        inverse_point_index_map.insert(point_index, point);
    }

    // Construct map of unique ordered point_idx_sets to their set_idx
    let mut point_idx_sets = BTreeMap::new();
    // Also construct mapping from commitment to point_idx_set
    let mut commitment_set_map = IndexMap::new();

    for (commitment, commitment_data) in commitment_map.iter_mut() {
        let mut point_index_set = BTreeSet::new();
        // Note that point_index_set is ordered, unlike point_indices
        for &point_index in commitment_data.point_indices.iter() {
            point_index_set.insert(point_index);
        }
        let len = point_index_set.len();

        // Push point_index_set to CommitmentData for the relevant commitment
        commitment_set_map.insert(*commitment, point_index_set.clone());

        let num_sets = point_idx_sets.len();
        point_idx_sets.entry(point_index_set).or_insert(num_sets);

        // Initialise empty evals vec for each unique commitment
        commitment_data.evals = vec![None; len];
    }

    // Populate set_index, evals and points for each commitment using point_idx_sets
    for query in queries {
        // An entry for every query's commitment was added to `commitment_map` the first
        // time we iterated through `queries`.
        let commitment_data = commitment_map
            .get_mut(&query.get_commitment())
            .expect("present by construction");

        // The index of the point at which the commitment is queried
        let point_index = point_index_map.get(&query.get_point()).unwrap();

        // The point_index_set at which the commitment was queried
        let point_index_set = commitment_set_map
            .get(&query.get_commitment())
            .expect("present by construction");

        // The set_index of the point_index_set
        let set_index = point_idx_sets.get(point_index_set).unwrap();
        commitment_data.set_index = *set_index;
        let point_index_set: Vec<usize> = point_index_set.iter().cloned().collect();

        // The offset of the point_index in the point_index_set
        let point_index_in_set = point_index_set
            .iter()
            .position(|i| i == point_index)
            .unwrap();

        // Insert the eval using the ordering of the point_index_set
        let eval = commitment_data
            .evals
            .get_mut(point_index_in_set)
            .expect("valid index");
        if eval.is_none() {
            *eval = Some(query.get_eval());
        } else {
            // Caller tried to provide two different evaluations for the same
            // commitment. Permitting this would be unsound.
            return None;
        }
    }

    let commitment_map = commitment_map
        .into_iter()
        .map(|(commitment, commitment_data)| CommitmentData {
            commitment,
            set_index: commitment_data.set_index,
            point_indices: commitment_data.point_indices,
            evals: commitment_data
                .evals
                .into_iter()
                .map(|eval| eval.expect("logic ensures all evals are set"))
                .collect(),
        })
        .collect();

    // Get actual points in each point set
    let mut point_sets: Vec<Vec<F>> = vec![Vec::new(); point_idx_sets.len()];
    for (point_idx_set, &set_idx) in point_idx_sets.iter() {
        for &point_idx in point_idx_set.iter() {
            let point = inverse_point_index_map.get(&point_idx).unwrap();
            point_sets[set_idx].push(*point);
        }
    }

    Some((commitment_map, point_sets))
}
