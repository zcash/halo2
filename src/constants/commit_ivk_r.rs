use super::{OrchardFixedBase, OrchardFixedBases, COMMIT_IVK_PERSONALIZATION};
use halo2::arithmetic::{CurveAffine, FieldExt};

pub const PERSONALIZATION: &str = COMMIT_IVK_PERSONALIZATION;

/// Generator used in SinsemillaCommit randomness for IVK commitment
pub const GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        148, 238, 176, 134, 155, 115, 130, 32, 71, 108, 43, 81, 122, 0, 18, 183, 189, 78, 168, 64,
        186, 250, 116, 181, 109, 239, 205, 50, 185, 198, 91, 14,
    ],
    [
        250, 170, 248, 240, 127, 69, 3, 58, 247, 119, 172, 189, 199, 234, 17, 244, 34, 155, 121,
        180, 206, 194, 252, 134, 55, 107, 139, 116, 163, 117, 220, 15,
    ],
);

/// Full-width z-values for GENERATOR
pub const Z: [u64; super::NUM_WINDOWS] = [
    1640, 16319, 75535, 213644, 22431, 77718, 73598, 44704, 58426, 90793, 51317, 35788, 62987,
    39128, 29961, 196204, 23144, 4960, 31792, 67688, 156889, 128199, 394678, 1391, 49801, 69085,
    177001, 27216, 17637, 12069, 8898, 134862, 137982, 35001, 261172, 3219, 171891, 6532, 93082,
    27872, 44212, 66355, 4768, 96884, 4793, 37757, 26619, 5486, 1315, 15325, 48605, 9168, 2511,
    84012, 73415, 74774, 224831, 26856, 4179, 82322, 39504, 32139, 75335, 14373, 63220, 39155,
    29901, 33099, 758, 27784, 6442, 252, 142824, 106033, 24247, 47057, 170067, 30302, 304042,
    163259, 49391, 34561, 350373, 139177, 147760,
];

/// Short signed z-values for GENERATOR
pub const Z_SHORT: [u64; super::NUM_WINDOWS_SHORT] = [
    1640, 16319, 75535, 213644, 22431, 77718, 73598, 44704, 58426, 90793, 51317, 35788, 62987,
    39128, 29961, 196204, 23144, 4960, 31792, 67688, 156889, 11429,
];

pub fn generator<C: CurveAffine>() -> OrchardFixedBases<C> {
    OrchardFixedBases::CommitIvkR(OrchardFixedBase::<C>::new(
        C::from_xy(
            C::Base::from_bytes(&GENERATOR.0).unwrap(),
            C::Base::from_bytes(&GENERATOR.1).unwrap(),
        )
        .unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{TestFixedBase, L_VALUE, NUM_WINDOWS, NUM_WINDOWS_SHORT};
    use super::*;
    use crate::primitives::sinsemilla::CommitDomain;
    use ff::PrimeField;
    use group::Curve;
    use halo2::{
        arithmetic::{CurveAffine, FieldExt},
        pasta::pallas,
    };

    #[test]
    fn generator() {
        let domain = CommitDomain::new(PERSONALIZATION);
        let point = domain.R();
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(x, pallas::Base::from_bytes(&GENERATOR.0).unwrap());
        assert_eq!(y, pallas::Base::from_bytes(&GENERATOR.1).unwrap());
    }

    #[test]
    fn lagrange_coeffs() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::CommitIvkR(inner) => inner.test_lagrange_coeffs(
                pallas::Scalar::rand(),
                pallas::Scalar::NUM_BITS as usize,
                NUM_WINDOWS,
            ),
            _ => unreachable!(),
        }
    }

    #[test]
    fn lagrange_coeffs_short() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::CommitIvkR(inner) => {
                let scalar = pallas::Scalar::from_u64(rand::random::<u64>());
                inner.test_lagrange_coeffs(scalar, L_VALUE, NUM_WINDOWS_SHORT)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn z() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::CommitIvkR(inner) => inner.test_z(&Z, NUM_WINDOWS),
            _ => unreachable!(),
        }
    }

    #[test]
    fn z_short() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::CommitIvkR(inner) => inner.test_z(&Z_SHORT, NUM_WINDOWS_SHORT),
            _ => unreachable!(),
        }
    }
}
