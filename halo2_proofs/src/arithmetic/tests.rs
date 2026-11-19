use super::*;
use group::Group;

pub(super) fn assert_public_multiexp_matches_naive<C: CurveAffine>(
    coeffs: &[C::Scalar],
    bases: &[C],
    expected: C::Curve,
) {
    assert_eq!(best_multiexp_public(coeffs, bases), expected);
}

#[cfg(test)]
fn check_multiexp_hook<C: CurveAffine>() {
    let mut rng = UnwrapErr(SysRng);
    let coeffs = (0..5000)
        .map(|_| C::Scalar::random(&mut rng))
        .collect::<Vec<_>>();
    let bases = (0..coeffs.len())
        .map(|_| C::Curve::random(&mut rng).to_affine_vartime())
        .collect::<Vec<_>>();
    let sizes = [
        31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256, 257, 511, 512, 513, 1023, 1024, 1025,
        2047, 2048, 2049, 4095, 4096, 4097, 5000,
    ];
    for len in (0..=17).chain(sizes) {
        assert_eq!(
            best_multiexp_public(&coeffs[..len], &bases[..len]),
            best_multiexp(&coeffs[..len], &bases[..len]),
            "random MSM of length {len}",
        );
    }

    let mut split_bit = C::Scalar::ONE;
    for _ in 0..128 {
        split_bit = split_bit.double();
    }
    let mut high_bit = C::Scalar::ONE;
    for _ in 1..C::Scalar::NUM_BITS {
        high_bit = high_bit.double();
    }
    let edge_coeffs = [
        C::Scalar::ZERO,
        C::Scalar::ONE,
        -C::Scalar::ONE,
        split_bit - C::Scalar::ONE,
        split_bit,
        split_bit + C::Scalar::ONE,
        high_bit,
        -high_bit,
    ];
    let point = bases[0];
    let edge_bases = [C::identity(), point, point, -point, point];
    let coeffs = edge_coeffs
        .into_iter()
        .cycle()
        .take(5000)
        .collect::<Vec<_>>();
    let bases = edge_bases
        .into_iter()
        .cycle()
        .take(5000)
        .collect::<Vec<_>>();
    for len in (0..=17).chain(sizes) {
        // The hook may decline any size; the public path must still match the
        // generic fallback for identities, repeated bases and boundary scalars.
        assert_eq!(
            best_multiexp_public(&coeffs[..len], &bases[..len]),
            best_multiexp(&coeffs[..len], &bases[..len]),
            "edge MSM of length {len}",
        );
    }
    assert_eq!(
        best_multiexp_public(&vec![C::Scalar::ZERO; bases.len()], &bases),
        C::Curve::identity(),
    );
    assert_eq!(
        best_multiexp_public(&coeffs, &vec![C::identity(); coeffs.len()]),
        C::Curve::identity(),
    );
    assert_multiexp_matches_naive(&[C::Scalar::ONE, C::Scalar::ONE], &[point, -point]);
    assert_multiexp_matches_naive(&[C::Scalar::ONE, -C::Scalar::ONE], &[point, point]);
}

#[test]
fn test_multiexp_hook_pallas() {
    check_multiexp_hook::<EpAffine>();
}

#[test]
fn test_multiexp_hook_vesta() {
    check_multiexp_hook::<EqAffine>();
}

#[test]
#[should_panic]
fn test_multiexp_missing_base() {
    best_multiexp_public::<EqAffine>(&[Fp::ONE], &[]);
}

#[test]
#[should_panic]
fn test_multiexp_missing_coefficient() {
    best_multiexp_public::<EpAffine>(&[], &[EpAffine::from(Ep::generator())]);
}
