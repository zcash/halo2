use super::*;
use crate::arithmetic::{eval_polynomial, lagrange_interpolate};
use pasta_curves::{Fp, Fq};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

fn reference<F: VartimeField>(
    point_sets: &[Vec<F>],
    eval_sets: &[Vec<F>],
    proof_evals: &[F],
    x_2: F,
    x_3: F,
) -> F {
    point_sets.iter().zip(eval_sets).zip(proof_evals).fold(
        F::ZERO,
        |acc, ((points, evals), proof_eval)| {
            let r_eval = eval_polynomial(&lagrange_interpolate(points, evals), x_3);
            let eval = points.iter().fold(*proof_eval - r_eval, |eval, point| {
                eval * (x_3 - point).invert_vartime().unwrap()
            });
            acc * x_2 + eval
        },
    )
}

#[test]
fn batched_denominators_match_polynomial_interpolation() {
    fn check<F: VartimeField + From<u64>>() {
        let mut rng = UnwrapErr(SysRng);
        assert_eq!(
            compute_msm_eval::<F>(&[], &[], &[], F::ZERO, F::ONE),
            F::ZERO
        );
        let points = [F::ZERO, F::ONE, -F::ONE, F::from(2)];
        for count in 0..=4 {
            let sets = (0..=count)
                .map(|n| points[..n].to_vec())
                .collect::<Vec<_>>();
            for _ in 0..16 {
                let evals = sets
                    .iter()
                    .map(|points| points.iter().map(|_| F::random(&mut rng)).collect())
                    .collect::<Vec<Vec<_>>>();
                let proof_evals = sets.iter().map(|_| F::random(&mut rng)).collect::<Vec<_>>();
                for x_2 in [F::ZERO, F::ONE, -F::ONE, F::random(&mut rng)] {
                    let mut x_3 = F::random(&mut rng);
                    while points.contains(&x_3) {
                        x_3 = F::random(&mut rng);
                    }
                    assert_eq!(
                        compute_msm_eval(&sets, &evals, &proof_evals, x_2, x_3),
                        reference(&sets, &evals, &proof_evals, x_2, x_3),
                    );
                }
            }
        }
    }
    check::<Fp>();
    check::<Fq>();
}

#[test]
#[should_panic(expected = "verifier challenge must differ")]
fn challenge_at_query_point_preserves_zero_denominator_failure() {
    compute_msm_eval(
        &[vec![Fp::ONE]],
        &[vec![Fp::ONE]],
        &[Fp::ONE],
        Fp::ONE,
        Fp::ONE,
    );
}
