mod prover;
mod verifier;

use super::Query;
use crate::{
    arithmetic::{eval_polynomial, CurveAffine, FieldExt},
    poly::{
        commitment::{Params, ParamsVerifier},
        msm::MSM,
        Coeff, Polynomial,
    },
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

struct CommitmentData<F: FieldExt, Q: Query<F>> {
    queries: Vec<Q>,
    point: F,
    _marker: PhantomData<F>,
}

fn construct_intermediate_sets<F: FieldExt, I, Q: Query<F>>(queries: I) -> Vec<CommitmentData<F, Q>>
where
    I: IntoIterator<Item = Q> + Clone,
{
    let mut point_query_map: BTreeMap<F, Vec<Q>> = BTreeMap::new();
    for query in queries.clone() {
        if let Some(queries) = point_query_map.get_mut(&query.get_point()) {
            queries.push(query);
        } else {
            point_query_map.insert(query.get_point(), vec![query]);
        }
    }

    point_query_map
        .keys()
        .map(|point| {
            let queries = point_query_map.get(point).unwrap();
            CommitmentData {
                queries: queries.clone(),
                point: *point,
                _marker: PhantomData,
            }
        })
        .collect()
}
