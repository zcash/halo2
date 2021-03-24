use crate::constants::{OrchardFixedBase, OrchardFixedBases, ORCHARD_PERSONALIZATION};
use halo2::arithmetic::{CurveAffine, FieldExt};

pub const PERSONALIZATION: &str = ORCHARD_PERSONALIZATION;

pub const GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        74, 166, 88, 164, 116, 16, 207, 20, 93, 0, 62, 45, 168, 59, 130, 172, 228, 79, 239, 35, 33,
        244, 20, 8, 126, 126, 100, 17, 248, 88, 183, 52,
    ],
    [
        141, 155, 6, 243, 162, 111, 188, 180, 253, 188, 17, 96, 117, 217, 25, 246, 206, 193, 176,
        192, 64, 196, 91, 252, 21, 22, 204, 177, 62, 197, 187, 44,
    ],
);

/// Full-width z-values for GENERATOR
pub const Z: [u64; super::NUM_WINDOWS] = [
    32517, 3118, 55842, 5295, 2252, 43091, 193188, 73424, 27335, 55867, 11015, 46382, 29066, 69577,
    2838, 245429, 25519, 172913, 25762, 138009, 11170, 132216, 114997, 52870, 52313, 102066, 5989,
    365, 73950, 74675, 191463, 34356, 16506, 63389, 4652, 81717, 108428, 120446, 80918, 25398,
    75806, 116193, 63775, 97332, 2183, 43473, 92592, 38016, 47712, 30288, 25445, 10737, 211404,
    26095, 72119, 25953, 3730, 19087, 28678, 11891, 69181, 214129, 2050, 72933, 124047, 16956,
    16977, 37315, 74647, 49184, 75499, 30521, 12997, 11908, 108937, 37055, 47165, 40492, 22849,
    89930, 69888, 193158, 105211, 27681, 32387,
];

/// Short signed z-values for GENERATOR
pub const Z_SHORT: [u64; super::NUM_WINDOWS_SHORT] = [
    32517, 3118, 55842, 5295, 2252, 43091, 193188, 73424, 27335, 55867, 11015, 46382, 29066, 69577,
    2838, 245429, 25519, 172913, 25762, 138009, 11170, 5770,
];

pub fn generator<C: CurveAffine>() -> OrchardFixedBases<C> {
    OrchardFixedBases::NullifierK(OrchardFixedBase::<C>::new(
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
    use rand;

    #[test]
    fn generator() {
        let hasher = pallas::Point::hash_to_curve(PERSONALIZATION);
        let point = hasher(b"K");
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(x, pallas::Base::from_bytes(&GENERATOR.0).unwrap());
        assert_eq!(y, pallas::Base::from_bytes(&GENERATOR.1).unwrap());
    }

    #[test]
    fn lagrange_coeffs() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::NullifierK(inner) => inner.test_lagrange_coeffs(
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
            OrchardFixedBases::NullifierK(inner) => {
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
            OrchardFixedBases::NullifierK(inner) => inner.test_z(&Z, NUM_WINDOWS),
            _ => unreachable!(),
        }
    }

    #[test]
    fn z_short() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::NullifierK(inner) => inner.test_z(&Z_SHORT, NUM_WINDOWS_SHORT),
            _ => unreachable!(),
        }
    }
}
