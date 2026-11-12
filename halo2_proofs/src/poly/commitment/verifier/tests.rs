use super::*;
use crate::transcript::Challenge255;
use ff::FromUniformBytes;
use group::Group;
use pasta_curves::{EpAffine, EqAffine, Fp, Fq};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

fn coefficient_products<F: Field>(u: &[F], init: F) -> Vec<F> {
    (0..1usize << u.len())
        .map(|index| {
            u.iter()
                .rev()
                .enumerate()
                .fold(init, |value, (bit, challenge)| {
                    if index & (1 << bit) == 0 {
                        value
                    } else {
                        value * challenge
                    }
                })
        })
        .collect()
}

#[test]
fn coefficients_and_scaled_inverses_match_reference() {
    fn check<F: VartimeField>() {
        let mut rng = UnwrapErr(SysRng);
        for rounds in 0..=10 {
            let u = (0..rounds).map(|_| F::random(&mut rng)).collect::<Vec<_>>();
            for scale in [F::ZERO, F::ONE, -F::ONE, F::random(&mut rng)] {
                let mut u = u.clone();
                for zero in 0..=rounds {
                    if zero > 0 {
                        u[zero - 1] = F::ZERO;
                    }
                    let mut actual = u.clone();
                    batch_inverse_and_scale(&mut actual, scale);
                    let mut expected = u.clone();
                    expected.iter_mut().batch_invert_vartime();
                    for value in &mut expected {
                        *value *= scale;
                    }
                    assert_eq!(actual, expected);
                    if rounds > 0 {
                        assert_eq!(compute_s(&u, scale), coefficient_products(&u, scale));
                    }
                }
            }
        }
    }
    check::<Fp>();
    check::<Fq>();
}

#[test]
#[should_panic]
fn empty_challenges_preserve_existing_failure() {
    compute_s::<Fp>(&[], Fp::ONE);
}

fn assert_sum<C: CurveAffine>(mut msm: MSM<'_, C>, expected: C::Curve) {
    msm.append_term(-C::Scalar::ONE, expected.to_affine_vartime());
    assert!(msm.eval());
}

#[test]
fn every_guard_consumption_preserves_the_verifier_equation() {
    fn check<C: CurveAffine>()
    where
        C::Scalar: FromUniformBytes<64>,
    {
        let params = Params::<C>::new(4);
        let mut rng = UnwrapErr(SysRng);
        for zero_challenge in [false, true] {
            let mut u = (0..4)
                .map(|_| C::Scalar::random(&mut rng))
                .collect::<Vec<_>>();
            if zero_challenge {
                u[1] = C::Scalar::ZERO;
            }
            let rounds = vec![
                (params.g[0], -params.g[0]),
                (C::identity(), params.g[1]),
                (params.g[2], params.g[2]),
                (-params.g[2], C::identity()),
            ];
            let neg_c = C::Scalar::random(&mut rng);
            let g_0_scalar = C::Scalar::random(&mut rng);
            for existing in [false, true] {
                let mut msm = params.empty_msm();
                msm.append_term(C::Scalar::ONE, params.g[3]);
                msm.add_to_u_scalar(C::Scalar::ONE);
                msm.add_to_w_scalar(-C::Scalar::ONE);
                let mut expected = params.g[3].to_curve() + params.u - params.w;
                if existing {
                    let scalars = (0..params.n)
                        .map(|_| C::Scalar::random(&mut rng))
                        .collect::<Vec<_>>();
                    msm.add_to_g_scalars(&scalars);
                    for (base, scalar) in params.g.iter().zip(scalars) {
                        expected += *base * scalar;
                    }
                }
                let guard = Guard::<C, Challenge255<C>> {
                    msm,
                    neg_c,
                    g_0_scalar,
                    rounds: rounds.clone(),
                    u: u.clone(),
                    u_packed: vec![],
                };
                for ((l, r), challenge) in rounds.iter().zip(&u) {
                    expected +=
                        *l * challenge.invert_vartime().unwrap_or(C::Scalar::ZERO) + *r * challenge;
                }
                expected += params.g[0] * g_0_scalar;
                let g = params
                    .g
                    .iter()
                    .zip(coefficient_products(&u, C::Scalar::ONE))
                    .fold(C::Curve::identity(), |sum, (base, scalar)| {
                        sum + *base * scalar
                    });
                assert_eq!(guard.compute_g(), g.to_affine_vartime());
                assert_sum(guard.clone().use_challenges(), expected + g * neg_c);
                for scale in [
                    C::Scalar::ZERO,
                    C::Scalar::ONE,
                    -C::Scalar::ONE,
                    C::Scalar::random(&mut rng),
                ] {
                    assert_sum(
                        guard.clone().use_challenges_with_scale(scale),
                        (expected + g * neg_c) * scale,
                    );
                }
                let supplied = params.g[2];
                let (msm, accumulator) = guard.use_g(supplied);
                assert_eq!(accumulator.g, supplied);
                assert_sum(msm, expected + supplied * neg_c);
            }
        }
    }
    check::<EpAffine>();
    check::<EqAffine>();
}
