use group::{ff::PrimeField, Group as _};
pub use pasta_curves::arithmetic::*;

pub(crate) fn multiexp_serial<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) -> C::Curve {
    let c = if bases.len() < 4 {
        1
    } else if bases.len() < 32 {
        3
    } else {
        (bases.len() as f64).ln().ceil() as usize
    };
    let mut buckets: Vec<Vec<Bucket<C>>> = vec![vec![Bucket::None; (1 << c) - 1]; (256 / c) + 1];

    buckets
        .iter_mut()
        .enumerate()
        .rev()
        .map(|(i, bucket)| {
            for (coeff, base) in coeffs.iter().zip(bases.iter()) {
                let seg = get_at::<C::Scalar>(i, c, &coeff.to_repr());
                if seg != 0 {
                    bucket[seg - 1].add_assign(base);
                }
            }
            bucket
        })
        .fold(C::Curve::identity(), |mut sum, bucket| {
            for _ in 0..c {
                sum = sum.double();
            }
            // Summation by parts
            // e.g. 3a + 2b + 1c = a +
            //                    (a) + b +
            //                    ((a) + b) + c
            let mut running_sum = C::Curve::identity();
            bucket.iter().rev().for_each(|exp| {
                running_sum = exp.add(running_sum);
                sum += &running_sum;
            });
            sum
        })
}

/// Performs a small multi-exponentiation operation.
/// Uses the double-and-add algorithm with doublings shared across points.
pub(crate) fn small_multiexp<C: CurveAffine>(coeffs: &[C::Scalar], bases: &[C]) -> C::Curve {
    let coeffs: Vec<_> = coeffs.iter().map(|a| a.to_repr()).collect();
    let mut acc = C::Curve::identity();

    // for byte idx
    for byte_idx in (0..32).rev() {
        // for bit idx
        for bit_idx in (0..8).rev() {
            acc = acc.double();
            // for each coeff
            for coeff_idx in 0..coeffs.len() {
                let byte = coeffs[coeff_idx].as_ref()[byte_idx];
                if ((byte >> bit_idx) & 1) != 0 {
                    acc += bases[coeff_idx];
                }
            }
        }
    }

    acc
}

#[derive(Clone, Copy)]
enum Bucket<C: CurveAffine> {
    None,
    Affine(C),
    Projective(C::Curve),
}

impl<C: CurveAffine> Bucket<C> {
    fn add_assign(&mut self, other: &C) {
        *self = match *self {
            Bucket::None => Bucket::Affine(*other),
            Bucket::Affine(a) => Bucket::Projective(a + *other),
            Bucket::Projective(mut a) => {
                a += *other;
                Bucket::Projective(a)
            }
        }
    }

    fn add(self, mut other: C::Curve) -> C::Curve {
        match self {
            Bucket::None => other,
            Bucket::Affine(a) => {
                other += a;
                other
            }
            Bucket::Projective(a) => other + &a,
        }
    }
}

fn get_at<F: PrimeField>(segment: usize, c: usize, bytes: &F::Repr) -> usize {
    let skip_bits = segment * c;
    let skip_bytes = skip_bits / 8;

    if skip_bytes >= 32 {
        return 0;
    }

    let mut v = [0; 8];
    for (v, o) in v.iter_mut().zip(bytes.as_ref()[skip_bytes..].iter()) {
        *v = *o;
    }

    let mut tmp = u64::from_le_bytes(v);
    tmp >>= skip_bits - (skip_bytes * 8);
    (tmp % (1 << c)) as usize
}

#[cfg(test)]
mod test {
    use super::{multiexp_serial, small_multiexp};
    use crate::pasta::EqAffine;
    use crate::pasta::Fp;
    use crate::poly::commitment::Params;
    use ff::Field;
    use proptest::{collection::vec, prelude::*};
    use rand_core::OsRng;

    fn arb_poly(k: usize, rng: OsRng) -> Vec<Fp> {
        (0..(1 << k)).map(|_| Fp::random(rng)).collect::<Vec<_>>()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        #[test]
        fn test_multiexp(k in 3usize..10) {
            let coeffs = arb_poly(k, OsRng);
            let params: Params<EqAffine> = Params::new(k as u32);
            let g_a = &mut params.get_g();
            let g_b = &mut params.get_g();

            let point_a = multiexp_serial(&coeffs, g_a);
            let point_b = small_multiexp(&coeffs, g_b);

            assert_eq!(point_a, point_b);
        }
    }
}
