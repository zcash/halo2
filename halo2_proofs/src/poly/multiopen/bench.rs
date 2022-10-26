use group::Curve;
use rand_core::OsRng;
use ark_std::{end_timer, start_timer};

use crate::arithmetic::eval_polynomial;
use crate::pasta::{EqAffine, Fp};
use crate::transcript::Challenge255;

use super::{EvaluationDomain, ProverQuery, VerifierQuery};
use super::commitment::{Blind, Params};
pub use super::prover::create_proof;
use super::super::*;
pub use super::verifier::verify_proof;

#[test]
fn bench_ipa_roundtrip() {
    for log_num_commits in 1..15 {
        let num_commits = 1 << log_num_commits;
        do_ipa_roundtrip(num_commits);
    }
}

fn do_ipa_roundtrip(num_commits: usize) {
    const K: u32 = 12;

    let params: Params<EqAffine> = Params::new(K);
    let domain = EvaluationDomain::new(1, K);
    let rng = OsRng;

    let v_ax = (0..num_commits).map(|c| {
        let mut ax = domain.empty_coeff();
        for (i, a) in ax.iter_mut().enumerate() {
            *a = Fp::from(10 + i as u64 + 1_000_000 * c as u64);
        }
        ax
    }).collect::<Vec<_>>();

    let blind = Blind(Fp::random(rng));

    let v_a = v_ax.iter().map(|ax|
        params.commit(ax, blind).to_affine()
    ).collect::<Vec<_>>();

    let x = Fp::random(rng);

    let v_avx = v_ax.iter().map(|ax|
        eval_polynomial(ax, x)
    ).collect::<Vec<_>>();

    let v_prover_queries = v_ax.iter().map(|ax|
        ProverQuery {
            point: x,
            poly: ax,
            blind,
        }
    ).collect::<Vec<_>>();

    let mut transcript = crate::transcript::Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        rng,
        &mut transcript,
        v_prover_queries,
    ).unwrap();
    let proof = transcript.finalize();

    {
        let v_verif_queries = v_a.iter()
            .zip(v_avx.iter())
            .map(|(a, avx)|
                VerifierQuery::new_commitment(a, x, *avx)
            ).collect::<Vec<_>>();

        let mut proof = &proof[..];

        let mut transcript =
            crate::transcript::Blake2bRead::<_, _, Challenge255<_>>::init(&mut proof);
        let msm = params.empty_msm();

        let timer = start_timer!(|| format!("⏱ verify_proof with {} commits", num_commits));

        let guard = verify_proof(
            &params,
            &mut transcript,
            v_verif_queries,
            msm,
        ).unwrap();

        end_timer!(timer);

        // Should succeed.
        assert!(guard.use_challenges().eval());
    }
}