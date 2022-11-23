use ark_std::{end_timer, start_timer};
use ff::Field;
use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

pub fn random_field_tests<F: Field>(type_name: String) {
    let mut rng = XorShiftRng::from_seed([
        0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06, 0xbc,
        0xe5,
    ]);

    random_multiplication_tests::<F, _>(&mut rng, type_name.clone());
    random_addition_tests::<F, _>(&mut rng, type_name.clone());
    random_subtraction_tests::<F, _>(&mut rng, type_name.clone());
    random_negation_tests::<F, _>(&mut rng, type_name.clone());
    random_doubling_tests::<F, _>(&mut rng, type_name.clone());
    random_squaring_tests::<F, _>(&mut rng, type_name.clone());
    random_inversion_tests::<F, _>(&mut rng, type_name.clone());
    random_expansion_tests::<F, _>(&mut rng, type_name);

    assert_eq!(F::zero().is_zero().unwrap_u8(), 1);
    {
        let mut z = F::zero();
        z = z.neg();
        assert_eq!(z.is_zero().unwrap_u8(), 1);
    }

    assert!(bool::from(F::zero().invert().is_none()));

    // Multiplication by zero
    {
        let mut a = F::random(&mut rng);
        a.mul_assign(&F::zero());
        assert_eq!(a.is_zero().unwrap_u8(), 1);
    }

    // Addition by zero
    {
        let mut a = F::random(&mut rng);
        let copy = a;
        a.add_assign(&F::zero());
        assert_eq!(a, copy);
    }
}

fn random_multiplication_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("multiplication {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let a = F::random(&mut rng);
        let b = F::random(&mut rng);
        let c = F::random(&mut rng);

        let mut t0 = a; // (a * b) * c
        t0.mul_assign(&b);
        t0.mul_assign(&c);

        let mut t1 = a; // (a * c) * b
        t1.mul_assign(&c);
        t1.mul_assign(&b);

        let mut t2 = b; // (b * c) * a
        t2.mul_assign(&c);
        t2.mul_assign(&a);

        assert_eq!(t0, t1);
        assert_eq!(t1, t2);
    }
    end_timer!(start);
}

fn random_addition_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("addition {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let a = F::random(&mut rng);
        let b = F::random(&mut rng);
        let c = F::random(&mut rng);

        let mut t0 = a; // (a + b) + c
        t0.add_assign(&b);
        t0.add_assign(&c);

        let mut t1 = a; // (a + c) + b
        t1.add_assign(&c);
        t1.add_assign(&b);

        let mut t2 = b; // (b + c) + a
        t2.add_assign(&c);
        t2.add_assign(&a);

        assert_eq!(t0, t1);
        assert_eq!(t1, t2);
    }
    end_timer!(start);
}

fn random_subtraction_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("subtraction {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let a = F::random(&mut rng);
        let b = F::random(&mut rng);

        let mut t0 = a; // (a - b)
        t0.sub_assign(&b);

        let mut t1 = b; // (b - a)
        t1.sub_assign(&a);

        let mut t2 = t0; // (a - b) + (b - a) = 0
        t2.add_assign(&t1);

        assert_eq!(t2.is_zero().unwrap_u8(), 1);
    }
    end_timer!(start);
}

fn random_negation_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("negation {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let a = F::random(&mut rng);
        let mut b = a;
        b = b.neg();
        b.add_assign(&a);

        assert_eq!(b.is_zero().unwrap_u8(), 1);
    }
    end_timer!(start);
}

fn random_doubling_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("doubling {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let mut a = F::random(&mut rng);
        let mut b = a;
        a.add_assign(&b);
        b = b.double();

        assert_eq!(a, b);
    }
    end_timer!(start);
}

fn random_squaring_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("squaring {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let mut a = F::random(&mut rng);
        let mut b = a;
        a.mul_assign(&b);
        b = b.square();

        assert_eq!(a, b);
    }
    end_timer!(start);
}

fn random_inversion_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    assert!(bool::from(F::zero().invert().is_none()));

    let message = format!("inversion {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        let mut a = F::random(&mut rng);
        let b = a.invert().unwrap(); // probablistically nonzero
        a.mul_assign(&b);

        assert_eq!(a, F::one());
    }
    end_timer!(start);
}

fn random_expansion_tests<F: Field, R: RngCore>(mut rng: R, type_name: String) {
    let message = format!("expansion {}", type_name);
    let start = start_timer!(|| message);
    for _ in 0..1000000 {
        // Compare (a + b)(c + d) and (a*c + b*c + a*d + b*d)

        let a = F::random(&mut rng);
        let b = F::random(&mut rng);
        let c = F::random(&mut rng);
        let d = F::random(&mut rng);

        let mut t0 = a;
        t0.add_assign(&b);
        let mut t1 = c;
        t1.add_assign(&d);
        t0.mul_assign(&t1);

        let mut t2 = a;
        t2.mul_assign(&c);
        let mut t3 = b;
        t3.mul_assign(&c);
        let mut t4 = a;
        t4.mul_assign(&d);
        let mut t5 = b;
        t5.mul_assign(&d);

        t2.add_assign(&t3);
        t2.add_assign(&t4);
        t2.add_assign(&t5);

        assert_eq!(t0, t2);
    }
    end_timer!(start);
}
