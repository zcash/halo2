use ff::Field;
use group::prime::PrimeCurveAffine;
use pasta_curves::arithmetic::{CurveAffine, CurveExt};
use rand_core::OsRng;

pub fn curve_tests<G: CurveExt>() {
    is_on_curve::<G>();
    equality::<G>();
    projective_to_affine_affine_to_projective::<G>();
    projective_addition::<G>();
    mixed_addition::<G>();
    multiplication::<G>();
    batch_normalize::<G>();
}

fn is_on_curve<G: CurveExt>() {
    assert!(bool::from(G::identity().is_on_curve()));
    assert!(bool::from(G::generator().is_on_curve()));
    assert!(bool::from(G::identity().is_on_curve()));
    assert!(bool::from(G::generator().is_on_curve()));

    for _ in 0..100 {
        let point = G::random(OsRng);
        assert!(bool::from(point.is_on_curve()));
        let affine_point: G::AffineExt = point.into();
        assert!(bool::from(affine_point.is_on_curve()));
    }
}

fn equality<G: CurveExt>() {
    let a = G::generator();
    let b = G::identity();

    assert!(a == a);
    assert!(b == b);
    assert!(a != b);
    assert!(b != a);

    for _ in 0..100 {
        let a = G::random(OsRng);
        let b = G::random(OsRng);

        assert!(a == a);
        assert!(b == b);
        assert!(a != b);
        assert!(b != a);

        let a: G::AffineExt = a.into();
        let b: G::AffineExt = b.into();

        assert!(a == a);
        assert!(b == b);
        assert!(a != b);
        assert!(b != a);
    }
}

fn projective_to_affine_affine_to_projective<G: CurveExt>() {
    let a = G::generator();
    let b = G::identity();

    assert!(bool::from(G::AffineExt::from(a).is_on_curve()));
    assert!(!bool::from(G::AffineExt::from(a).is_identity()));
    assert!(bool::from(G::AffineExt::from(b).is_on_curve()));
    assert!(bool::from(G::AffineExt::from(b).is_identity()));

    let a = G::AffineExt::generator();
    let b = G::AffineExt::identity();

    assert!(bool::from(G::from(a).is_on_curve()));
    assert!(!bool::from(G::from(a).is_identity()));
    assert!(bool::from(G::from(b).is_on_curve()));
    assert!(bool::from(G::from(b).is_identity()));
}

fn projective_addition<G: CurveExt>() {
    let a = G::identity();
    let b = G::identity();
    let c = a + b;
    assert!(bool::from(c.is_identity()));
    assert!(bool::from(c.is_on_curve()));
    let c = a - b;
    assert!(bool::from(c.is_identity()));
    assert!(bool::from(c.is_on_curve()));

    let a = G::identity();
    let a = -a;
    assert!(bool::from(a.is_on_curve()));
    assert!(bool::from(a.is_identity()));

    let a = G::random(OsRng);
    assert!(a == a + G::identity());
    assert!(a == G::identity() + a);
    assert!(-a == G::identity() - a);

    let a = G::identity();
    let a = a.double();
    assert!(bool::from(c.is_on_curve()));
    assert!(bool::from(a.is_identity()));

    let a = G::generator();
    let a = a.double();
    assert!(bool::from(c.is_on_curve()));
    assert_eq!(a, G::generator() + G::generator());

    let a = G::random(OsRng);
    assert!(a.double() - a == a);

    let a = G::random(OsRng);
    let b = G::random(OsRng);
    let c = G::random(OsRng);
    assert!(a + b == b + a);
    assert!(a - b == -(b - a));
    assert!(c + (a + b) == a + (c + b));
    assert!((a - b) - c == (a - c) - b);

    let a = G::generator().double().double(); // 4P
    let b = G::generator().double(); // 2P
    let c = a + b;

    let mut d = G::generator();
    for _ in 0..5 {
        d += G::generator();
    }

    assert!(c == d);
    assert!(!bool::from(c.is_identity()));
    assert!(bool::from(c.is_on_curve()));
    assert!(!bool::from(d.is_identity()));
    assert!(bool::from(d.is_on_curve()));
}

fn mixed_addition<G: CurveExt>() {
    let a = G::identity();
    let b = G::AffineRepr::identity();
    let c = a + b;
    assert!(bool::from(c.is_identity()));
    assert!(bool::from(c.is_on_curve()));
    let c = a - b;
    assert!(bool::from(c.is_identity()));
    assert!(bool::from(c.is_on_curve()));

    let a = G::identity();
    let a = -a;
    assert!(bool::from(a.is_on_curve()));
    assert!(bool::from(a.is_identity()));
    let a = G::AffineExt::identity();
    let a = -a;
    assert!(bool::from(a.is_on_curve()));
    assert!(bool::from(a.is_identity()));

    let a: G::AffineExt = G::random(OsRng).into();
    assert!(a.to_curve() == a + G::AffineExt::identity());

    let a = G::random(OsRng);
    assert!(a.double() - a == a);

    let a = G::random(OsRng);
    let b: G::AffineExt = G::random(OsRng).into();
    let c0 = a + b;
    let c1 = a + G::from(b);
    assert_eq!(c0, c1);
}

fn batch_normalize<G: CurveExt>() {
    let a = G::generator().double();
    let b = a.double();
    let c = b.double();

    for a_identity in (0..1).map(|n| n == 1) {
        for b_identity in (0..1).map(|n| n == 1) {
            for c_identity in (0..1).map(|n| n == 1) {
                let mut v = [a, b, c];
                if a_identity {
                    v[0] = G::identity()
                }
                if b_identity {
                    v[1] = G::identity()
                }
                if c_identity {
                    v[2] = G::identity()
                }

                let mut t = [
                    G::AffineExt::identity(),
                    G::AffineExt::identity(),
                    G::AffineExt::identity(),
                ];
                let expected = [
                    G::AffineExt::from(v[0]),
                    G::AffineExt::from(v[1]),
                    G::AffineExt::from(v[2]),
                ];

                G::batch_normalize(&v[..], &mut t[..]);

                assert_eq!(&t[..], &expected[..]);
            }
        }
    }
}

fn multiplication<G: CurveExt>() {
    for _ in 1..1000 {
        let s1 = G::ScalarExt::random(OsRng);
        let s2 = G::ScalarExt::random(OsRng);

        let t0 = G::identity() * s1;
        assert!(bool::from(t0.is_identity()));

        let a = G::random(OsRng);
        let t0 = a * G::ScalarExt::one();
        assert_eq!(a, t0);

        let t0 = a * G::ScalarExt::zero();
        assert!(bool::from(t0.is_identity()));

        let t0 = a * s1 + a * s2;

        let s3 = s1 + s2;
        let t1 = a * s3;

        assert_eq!(t0, t1);

        let mut t0 = a * s1;
        let mut t1 = a * s2;
        t0 += t1;
        let s3 = s1 + s2;
        t1 = a * s3;
        assert_eq!(t0, t1);
    }
}
