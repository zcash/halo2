//! TODO

use crate::arithmetic::{CurveAffine, FieldExt};
use crate::plonk::{
    lookup, permutation, vanishing, ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX,
    ChallengeY,
};
use crate::poly::{commitment::Accumulator, multiopen};
use crate::transcript::Challenge255;

/// TODO
#[derive(Debug)]
pub struct RecursiveProof<Native: CurveAffine, Remote: CurveAffine> {
    // Make this a bytearray
    proof: Proof<Native>,
    native_accumulator: Accumulator<Native, Challenge255<Native>>,
    remote_accumulator: Accumulator<Remote, Challenge255<Remote>>,
    deferred: Deferred<Remote>,
}

/// TODO: documentation
#[derive(Default, Debug)]
pub struct Proof<C: CurveAffine> {
    pub(crate) fixed_evals: Vec<C::Scalar>,
    pub(crate) instances: Vec<InstanceProof<C>>,
    pub(crate) vanishing: vanishing::Proof<C>,
    pub(crate) multiopen: multiopen::Proof<C>,
    pub(crate) common_perm_evals: Vec<C::Scalar>,
}

/// TODO: documentation
#[derive(Default, Debug, Clone)]
pub struct InstanceProof<C: CurveAffine> {
    pub(crate) advice_commitments: Vec<C>,
    pub(crate) instance_evals: Vec<C::Scalar>,
    pub(crate) advice_evals: Vec<C::Scalar>,
    pub(crate) lookups: Vec<lookup::Proof<C>>,
    pub(crate) permutation: permutation::Proof<C>,
}

/// TODO: documentation
#[derive(Default, Debug, Clone)]
pub struct Challenges<C: CurveAffine> {
    pub(crate) theta: ChallengeTheta<C>,
    pub(crate) beta: ChallengeBeta<C>,
    pub(crate) gamma: ChallengeGamma<C>,
    pub(crate) y: ChallengeY<C>,
    pub(crate) x: ChallengeX<C>,
    pub(crate) multiopen: multiopen::Challenges<C>,
}

/// TODO: Documentation
#[derive(Debug)]
pub struct Deferred<C: CurveAffine> {
    fixed_evals: Vec<C::Scalar>,
    instances: Vec<InstanceDeferred<C::Scalar>>,
    vanishing: C::Scalar,
    multiopen_q_evals: Vec<C::Scalar>,
    common_perm_evals: Vec<C::Scalar>,
    challenges: Challenges<C>,
}

impl<C: CurveAffine> From<(Proof<C>, Challenges<C>)> for Deferred<C> {
    fn from(proof_challenges: (Proof<C>, Challenges<C>)) -> Self {
        let (proof, challenges) = proof_challenges;

        Self {
            fixed_evals: proof.fixed_evals,
            instances: proof
                .instances
                .into_iter()
                .map(|lookup| lookup.into())
                .collect(),
            vanishing: proof.vanishing.random_eval,
            multiopen_q_evals: proof.multiopen.q_evals,
            common_perm_evals: proof.common_perm_evals.clone(),
            challenges,
        }
    }
}

#[derive(Debug)]
struct InstanceDeferred<F: FieldExt> {
    instance: Vec<F>,
    advice: Vec<F>,
    lookups: Vec<lookup::Evals<F>>,
    permutation_set_evals: Vec<permutation::SetEvals<F>>,
}

impl<C: CurveAffine> From<InstanceProof<C>> for InstanceDeferred<C::Scalar> {
    fn from(proof: InstanceProof<C>) -> Self {
        Self {
            instance: proof.instance_evals,
            advice: proof.advice_evals,
            lookups: proof.lookups.iter().map(|lookup| lookup.evals).collect(),
            permutation_set_evals: proof.permutation.set_evals,
        }
    }
}
