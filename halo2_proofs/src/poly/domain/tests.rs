use super::{EvaluationDomain, Field, Rotation};

#[test]
fn test_rotate_omega_fast_paths_match_exponentiation() {
    use crate::pasta::pallas::Base;
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    let mut rng = UnwrapErr(SysRng);
    for k in [3, 11] {
        let domain = EvaluationDomain::<Base>::new(3, k);
        for value in [
            Base::ZERO,
            Base::ONE,
            Base::random(&mut rng),
            Base::random(&mut rng),
        ] {
            for rotation in [i32::MIN, -17, -7, -6, -2, -1, 0, 1, 2, 7, 17, i32::MAX] {
                let exponent = u64::from(rotation.unsigned_abs());
                let expected = if rotation >= 0 {
                    value * domain.get_omega().pow_vartime([exponent])
                } else {
                    value * domain.get_omega_inv().pow_vartime([exponent])
                };
                assert_eq!(
                    domain.rotate_omega(value, Rotation(rotation)),
                    expected,
                    "k={k}, rotation={rotation}"
                );
            }
        }
    }
}
