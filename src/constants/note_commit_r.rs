use super::{OrchardFixedBase, OrchardFixedBases, NOTE_COMMITMENT_PERSONALIZATION};
use halo2::arithmetic::{CurveAffine, FieldExt};

pub const PERSONALIZATION: &str = NOTE_COMMITMENT_PERSONALIZATION;

/// Generator used in SinsemillaCommit randomness for note commitment
pub const GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        27, 85, 45, 121, 90, 101, 108, 254, 2, 206, 235, 232, 42, 16, 38, 207, 116, 73, 104, 35,
        246, 164, 144, 15, 230, 159, 210, 102, 11, 44, 179, 36,
    ],
    [
        184, 34, 111, 203, 218, 243, 248, 43, 50, 219, 6, 34, 114, 46, 198, 187, 54, 179, 135, 26,
        92, 178, 197, 64, 187, 63, 245, 129, 53, 180, 231, 25,
    ],
);

/// z-values for GENERATOR
pub const Z: [u64; 85] = [
    10213, 84688, 5015, 29076, 5250, 12480, 1589, 21978, 40626, 116200, 36680, 56513, 80295, 1371,
    36801, 26527, 11103, 61032, 199301, 33177, 49711, 167190, 1448, 51069, 40410, 171413, 82827,
    15451, 53663, 4202, 47840, 93100, 44310, 10271, 27499, 76928, 39695, 59189, 70288, 24401,
    33207, 3472, 13911, 8835, 193349, 259, 41151, 2318, 33540, 21052, 14435, 18358, 49426, 52169,
    96418, 52931, 85348, 392973, 85905, 92539, 22878, 26933, 41387, 22788, 89854, 54883, 18584,
    19451, 4488, 283677, 74400, 56046, 20644, 5330, 27521, 99158, 9360, 10834, 78610, 7963, 19984,
    149297, 10335, 32061, 214389,
];

pub fn generator<C: CurveAffine>() -> OrchardFixedBases<C> {
    OrchardFixedBases::NoteCommitR(OrchardFixedBase::<C>::new(
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
    use crate::primitives::sinsemilla::CommitDomain;
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
            OrchardFixedBases::NoteCommitR(inner) => inner.test_lagrange_coeffs(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn z() {
        let base = super::generator::<pallas::Affine>();
        match base {
            OrchardFixedBases::NoteCommitR(inner) => inner.test_z(&Z),
            _ => unreachable!(),
        }
    }
}
