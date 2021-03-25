use super::{OrchardFixedBase, OrchardFixedBases, VALUE_COMMITMENT_PERSONALIZATION};
use halo2::arithmetic::{CurveAffine, FieldExt};

pub const PERSONALIZATION: &str = VALUE_COMMITMENT_PERSONALIZATION;

/// The value commitment is used to check balance between inputs and outputs. The value is
/// placed over this generator.
pub const GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        199, 209, 64, 100, 24, 110, 206, 85, 59, 77, 6, 42, 67, 76, 118, 116, 253, 250, 208, 71,
        184, 191, 140, 13, 93, 79, 56, 33, 94, 39, 101, 10,
    ],
    [
        76, 105, 226, 202, 61, 74, 65, 200, 220, 56, 136, 63, 71, 255, 94, 246, 153, 160, 21, 162,
        106, 112, 47, 120, 165, 32, 53, 96, 19, 181, 110, 36,
    ],
);

/// Full-width z-values for GENERATOR
pub const Z: [u64; super::NUM_WINDOWS] = [
    287008, 5261, 10541, 67788, 1084, 31201, 1662, 32921, 2652, 52006, 3486, 82692, 7295, 40007,
    37754, 44773, 3021, 171863, 33315, 8829, 67034, 50428, 40391, 6615, 40340, 238, 199437, 50234,
    899, 27825, 139735, 36053, 194684, 28229, 31719, 66166, 100600, 59796, 52804, 10221, 159298,
    32923, 158, 40332, 100062, 8923, 23819, 96460, 44805, 2951, 50005, 134465, 44269, 51778, 73741,
    11413, 19391, 84631, 96003, 71276, 61444, 49575, 154646, 229521, 4555, 313045, 30544, 15466,
    7134, 12520, 164127, 29119, 11279, 103167, 63033, 13765, 35197, 71168, 10379, 9560, 54432,
    132537, 189703, 29967, 9941,
];

/// Short signed z-values for GENERATOR
pub const Z_SHORT: [u64; super::NUM_WINDOWS_SHORT] = [
    287008, 5261, 10541, 67788, 1084, 31201, 1662, 32921, 2652, 52006, 3486, 82692, 7295, 40007,
    37754, 44773, 3021, 171863, 33315, 8829, 67034, 16641,
];

pub fn generator<C: CurveAffine>() -> OrchardFixedBases<C> {
    OrchardFixedBases::ValueCommitR(OrchardFixedBase::<C>::new(
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
    use ff::PrimeField;
    use group::Curve;
    use halo2::{
        arithmetic::{CurveAffine, CurveExt, FieldExt},
        pasta::pallas,
    };

    #[test]
    fn generator() {
        let hasher = pallas::Point::hash_to_curve(PERSONALIZATION);
        let point = hasher(b"-r");
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(x, pallas::Base::from_bytes(&GENERATOR.0).unwrap());
        assert_eq!(y, pallas::Base::from_bytes(&GENERATOR.1).unwrap());
    }

    #[test]
    fn lagrange_coeffs() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::ValueCommitR(inner) => inner.test_lagrange_coeffs(
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
            OrchardFixedBases::ValueCommitR(inner) => {
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
            OrchardFixedBases::ValueCommitR(inner) => inner.test_z(&Z, NUM_WINDOWS),
            _ => unreachable!(),
        }
    }

    #[test]
    fn z_short() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::ValueCommitR(inner) => inner.test_z(&Z_SHORT, NUM_WINDOWS_SHORT),
            _ => unreachable!(),
        }
    }
}
