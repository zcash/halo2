use crate::{arithmetic::best_multiexp, poly::commitment::Params};
use ff::Field;
use group::Group;
use pasta_curves::{
    arithmetic::{CurveAffine, CurveExt},
    EpAffine, EqAffine,
};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

#[test]
fn full_width_identity_matches_ordinary_msm() {
    fn check<C: CurveAffine>() {
        let params = Params::<C>::new(4);
        let mut rng = UnwrapErr(SysRng);
        assert!(params.empty_msm().eval());
        for count in [1, 2, 8, 16, 65] {
            let mut scalars = Vec::new();
            let mut bases = Vec::new();
            let mut msm = params.empty_msm();
            for i in 0..count {
                let base = match i % 4 {
                    0 => C::identity(),
                    1 => params.g[i % params.g.len()],
                    2 => -params.g[i % params.g.len()],
                    _ => params.w,
                };
                let scalar = match i % 4 {
                    0 => C::Scalar::ZERO,
                    1 => -C::Scalar::ONE,
                    _ => C::Scalar::random(&mut rng),
                };
                scalars.push(scalar);
                bases.push(base);
                msm.append_term(scalar, base);
            }
            let expected = best_multiexp(&scalars, &bases);
            assert_eq!(msm.clone().eval(), bool::from(expected.is_identity()));
            msm.append_term(-C::Scalar::ONE, expected.to_affine_vartime());
            assert!(msm.clone().eval());
            msm.append_term(C::Scalar::ONE, params.g[0]);
            assert!(!msm.eval());
        }
        // Include the distinct generator, blinding and auxiliary term sources.
        let scalars = vec![-C::Scalar::ONE; params.g.len()];
        let expected = best_multiexp(&scalars, &params.g) + params.w - params.u;
        let mut msm = params.empty_msm();
        msm.add_to_g_scalars(&scalars);
        msm.add_to_w_scalar(C::Scalar::ONE);
        msm.add_to_u_scalar(-C::Scalar::ONE);
        msm.append_term(-C::Scalar::ONE, expected.to_affine_vartime());
        assert!(msm.eval());
    }
    check::<EpAffine>();
    check::<EqAffine>();
}
