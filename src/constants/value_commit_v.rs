use super::{OrchardFixedBase, OrchardFixedBases, VALUE_COMMITMENT_PERSONALIZATION};
use halo2::arithmetic::{CurveAffine, FieldExt};

pub const PERSONALIZATION: &str = VALUE_COMMITMENT_PERSONALIZATION;

/// The value commitment is used to check balance between inputs and outputs. The value is
/// placed over this generator.
pub const GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        146, 134, 252, 1, 6, 122, 71, 38, 242, 210, 12, 65, 214, 129, 99, 228, 216, 165, 217, 139,
        4, 159, 130, 201, 115, 100, 204, 172, 241, 221, 192, 57,
    ],
    [
        104, 21, 198, 148, 56, 181, 122, 135, 95, 147, 179, 108, 174, 10, 183, 200, 243, 25, 27,
        248, 167, 68, 37, 105, 11, 87, 167, 253, 72, 189, 248, 33,
    ],
);

/// z-values for GENERATOR
pub const Z: [u64; 85] = [
    12093, 20558, 3369, 22650, 43666, 81863, 2960, 131095, 84, 117033, 7349, 122998, 47884, 43451,
    22237, 3461, 71521, 147314, 31021, 70554, 47822, 44159, 45362, 7756, 19977, 41666, 82714,
    21407, 16731, 48013, 173284, 356652, 3027, 9756, 10560, 1554, 40272, 131726, 32724, 6152,
    67912, 2642, 100128, 8950, 20487, 58314, 7440, 63032, 586, 32770, 37328, 21775, 4186, 172635,
    111256, 35867, 23903, 137179, 16694, 43650, 32899, 40670, 55501, 44805, 20211, 207309, 2718,
    63301, 145483, 5584, 55596, 349346, 30535, 112990, 44821, 48471, 107386, 16232, 16492, 88498,
    33976, 106405, 11043, 44897, 98652,
];

pub fn generator<C: CurveAffine>() -> OrchardFixedBases<C> {
    OrchardFixedBases::ValueCommitV(OrchardFixedBase::<C>::new(
        C::from_xy(
            C::Base::from_bytes(&GENERATOR.0).unwrap(),
            C::Base::from_bytes(&GENERATOR.1).unwrap(),
        )
        .unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::TestFixedBase;
    use super::*;
    use group::Curve;
    use halo2::{
        arithmetic::{CurveAffine, CurveExt, FieldExt},
        pasta::pallas,
    };

    #[test]
    fn generator() {
        let hasher = pallas::Point::hash_to_curve(PERSONALIZATION);
        let point = hasher(b"-v");
        let (x, y) = point.to_affine().get_xy().unwrap();

        println!("{:?}", x.to_bytes());
        println!("{:?}", y.to_bytes());

        assert_eq!(x, pallas::Base::from_bytes(&GENERATOR.0).unwrap());
        assert_eq!(y, pallas::Base::from_bytes(&GENERATOR.1).unwrap());
    }

    #[test]
    fn lagrange_coeffs() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::ValueCommitV(inner) => inner.test_lagrange_coeffs(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn z() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::ValueCommitV(inner) => inner.test_z(&Z),
            _ => unreachable!(),
        }
    }
}
