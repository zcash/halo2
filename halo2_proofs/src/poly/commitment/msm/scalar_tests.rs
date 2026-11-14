use super::CanonicalFieldKey;
use crate::poly::commitment::Params;
use ff::{Field, PrimeField};
use group::Group;
use pasta_curves::{
    arithmetic::{CurveAffine, CurveExt},
    EpAffine, EqAffine, Fp, Fq,
};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

#[test]
fn cached_keys_preserve_pasta_order() {
    fn check<F: PrimeField + Ord>() {
        let mut rng = UnwrapErr(SysRng);
        let mut values = vec![F::ZERO, F::ONE, -F::ONE];
        values.extend((0..32).map(|_| F::random(&mut rng)));
        for a in &values {
            for b in &values {
                assert_eq!(
                    CanonicalFieldKey::new(*a).cmp(&CanonicalFieldKey::new(*b)),
                    a.cmp(b)
                );
            }
        }
    }
    check::<Fp>();
    check::<Fq>();
}

#[test]
fn buffered_owned_msms_match_naive_sums() {
    fn check<C: CurveAffine>() {
        let params = Params::<C>::new(3);
        let mut rng = UnwrapErr(SysRng);
        for count in [0, 1, 2, 17] {
            let mut incremental = params.empty_msm();
            let mut batched = params.empty_msm();
            let mut expected = C::Curve::identity();
            for _ in 0..count {
                let mut msm = params.empty_msm();
                let scalars = (0..params.n)
                    .map(|_| C::Scalar::random(&mut rng))
                    .collect::<Vec<_>>();
                for (base, scalar) in params.g.iter().zip(&scalars) {
                    expected += *base * scalar;
                }
                let allocation = scalars.as_ptr();
                msm.add_owned_g_scalars(scalars);
                assert_eq!(msm.g_scalars.as_ref().unwrap().as_ptr(), allocation);
                msm.add_to_w_scalar(C::Scalar::ONE);
                msm.add_to_u_scalar(-C::Scalar::ONE);
                expected += params.w - params.u;
                for point in [
                    C::identity(),
                    params.g[1],
                    params.g[1],
                    -params.g[1],
                    params.g[2],
                ] {
                    for scalar in [
                        C::Scalar::ZERO,
                        C::Scalar::ONE,
                        -C::Scalar::ONE,
                        C::Scalar::random(&mut rng),
                    ] {
                        msm.append_term(scalar, point);
                        expected += point * scalar;
                    }
                }
                incremental.add_msm(&msm);
                batched.add_msm_batch(msm);
            }
            // Exercise both sources and destinations containing buffered terms.
            let mut nested = params.empty_msm();
            nested.add_msm_batch(batched.clone());
            let mut borrowed = params.empty_msm();
            borrowed.add_msm(&nested);
            for scale in [
                C::Scalar::ZERO,
                C::Scalar::ONE,
                -C::Scalar::ONE,
                C::Scalar::random(&mut rng),
            ] {
                for mut msm in [
                    incremental.clone(),
                    batched.clone(),
                    nested.clone(),
                    borrowed.clone(),
                ] {
                    msm.scale(scale);
                    msm.append_term(-C::Scalar::ONE, (expected * scale).to_affine_vartime());
                    assert!(msm.clone().eval());
                    msm.append_term(C::Scalar::ONE, params.g[0]);
                    assert!(!msm.eval());
                }
            }
        }
    }
    check::<EpAffine>();
    check::<EqAffine>();
}
