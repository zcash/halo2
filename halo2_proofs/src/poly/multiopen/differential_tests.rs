use super::{construct_intermediate_sets, CommitmentData, IntermediateSets, Query};
use group::ff::{Field, FromUniformBytes};
use indexmap::IndexMap;
use pasta_curves::Fp;
use proptest::{collection::vec, prelude::*};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};

#[derive(Debug, Clone)]
struct MyQuery<F> {
    point: F,
    eval: F,
    commitment: usize,
}

impl Query<Fp> for MyQuery<Fp> {
    type Commitment = usize;
    type Eval = Fp;

    fn get_point(&self) -> Fp {
        self.point
    }
    fn get_eval(&self) -> Fp {
        self.eval
    }
    fn get_commitment(&self) -> usize {
        self.commitment
    }
}

prop_compose! {
    fn arb_point()(bytes in vec(any::<u8>(), 64)) -> Fp {
        Fp::from_uniform_bytes(&<[u8; 64]>::try_from(bytes).unwrap())
    }
}
// The implementation at the pinned parent revision, unchanged as an oracle.
fn original_sets<F: Field + Ord, I, Q: Query<F>>(queries: I) -> Option<IntermediateSets<F, Q>>
where
    I: IntoIterator<Item = Q> + Clone,
{
    let mut commitment_map: IndexMap<Q::Commitment, CommitmentData<Option<Q::Eval>, ()>> =
        IndexMap::new();
    let mut point_index_map = BTreeMap::new();
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
    let mut inverse_point_index_map = BTreeMap::new();
    for (&point, &point_index) in point_index_map.iter() {
        inverse_point_index_map.insert(point_index, point);
    }
    let mut point_idx_sets = BTreeMap::new();
    let mut commitment_set_map = IndexMap::new();
    for (commitment, commitment_data) in commitment_map.iter_mut() {
        let mut point_index_set = BTreeSet::new();
        for &point_index in commitment_data.point_indices.iter() {
            point_index_set.insert(point_index);
        }
        let len = point_index_set.len();
        commitment_set_map.insert(*commitment, point_index_set.clone());
        let num_sets = point_idx_sets.len();
        point_idx_sets.entry(point_index_set).or_insert(num_sets);
        commitment_data.evals = vec![None; len];
    }
    for query in queries {
        let commitment_data = commitment_map
            .get_mut(&query.get_commitment())
            .expect("present by construction");
        let point_index = point_index_map.get(&query.get_point()).unwrap();
        let point_index_set = commitment_set_map
            .get(&query.get_commitment())
            .expect("present by construction");
        let set_index = point_idx_sets.get(point_index_set).unwrap();
        commitment_data.set_index = *set_index;
        let point_index_set: Vec<usize> = point_index_set.iter().cloned().collect();
        let point_index_in_set = point_index_set
            .iter()
            .position(|i| i == point_index)
            .unwrap();
        let eval = commitment_data
            .evals
            .get_mut(point_index_in_set)
            .expect("valid index");
        if eval.is_none() {
            *eval = Some(query.get_eval());
        } else {
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
    let mut point_sets: Vec<Vec<F>> = vec![Vec::new(); point_idx_sets.len()];
    for (point_idx_set, &set_idx) in point_idx_sets.iter() {
        for &point_idx in point_idx_set.iter() {
            let point = inverse_point_index_map.get(&point_idx).unwrap();
            point_sets[set_idx].push(*point);
        }
    }
    Some((commitment_map, point_sets))
}
#[allow(clippy::type_complexity)]
fn describe(
    sets: Option<IntermediateSets<Fp, MyQuery<Fp>>>,
) -> Option<(Vec<(usize, usize, Vec<usize>, Vec<Fp>)>, Vec<Vec<Fp>>)> {
    sets.map(|(data, points)| {
        (
            data.into_iter()
                .map(|item| {
                    (
                        item.commitment,
                        item.set_index,
                        item.point_indices,
                        item.evals,
                    )
                })
                .collect(),
            points,
        )
    })
}

#[test]
fn empty_and_duplicate_queries_match_pinned_parent() {
    let empty = Vec::<MyQuery<Fp>>::new();
    assert_eq!(
        describe(construct_intermediate_sets(empty.clone())),
        describe(original_sets(empty)),
    );
    let query = MyQuery {
        commitment: 0,
        point: Fp::from(7),
        eval: Fp::from(11),
    };
    for duplicate in [
        query.clone(),
        MyQuery {
            eval: Fp::from(12),
            ..query.clone()
        },
    ] {
        let queries = vec![query.clone(), duplicate];
        assert_eq!(
            describe(construct_intermediate_sets(queries.clone())),
            describe(original_sets(queries)),
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20_000))]
    #[test]
    fn matches_original_construction(
        mapping in vec((0usize..8, 0usize..8, any::<u64>()), 0..32),
        points in vec(arb_point(), 8),
        mutation in 0u8..3,
    ) {
        let mut queries = mapping.into_iter().map(|(commitment, index, eval)| MyQuery {
            commitment,
            point: points[index],
            eval: Fp::from(eval),
        }).collect::<Vec<_>>();
        if let Some(first) = queries.first().cloned() {
            match mutation {
                1 => queries.push(first),
                2 => queries.push(MyQuery { eval: first.eval + Fp::ONE, ..first }),
                _ => (),
            }
        }
        prop_assert_eq!(
            describe(construct_intermediate_sets(queries.clone())),
            describe(original_sets(queries)),
        );
    }
}
