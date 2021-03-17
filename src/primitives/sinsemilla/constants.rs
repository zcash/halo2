//! Sinsemilla generators
use halo2::arithmetic::{CurveAffine, CurveExt};

/// Number of bits of each message piece in $\mathsf{SinsemillaHashToPoint}$
pub const K: usize = 10;

/// The largest integer such that $2^c \leq (r_P - 1) / 2$, where $r_P$ is the order
/// of Pallas.
pub const C: usize = 253;

// Sinsemilla Q generators

/// SWU hash-to-curve personalization for Sinsemilla $Q$ generators.
pub const Q_PERSONALIZATION: &str = "z.cash:SinsemillaQ";

/// Generator used in SinsemillaHashToPoint for note commitment
pub const Q_NOTE_COMMITMENT_M_GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        17, 166, 94, 204, 113, 234, 240, 126, 87, 121, 119, 126, 2, 201, 212, 93, 41, 34, 212, 208,
        68, 169, 141, 7, 220, 238, 38, 95, 90, 247, 70, 18,
    ],
    [
        112, 75, 165, 87, 136, 232, 105, 167, 146, 87, 199, 38, 162, 29, 25, 74, 210, 48, 46, 194,
        238, 187, 31, 185, 170, 183, 90, 145, 96, 225, 82, 11,
    ],
);

/// Generator used in SinsemillaHashToPoint for IVK commitment
pub const Q_COMMIT_IVK_M_GENERATOR: ([u8; 32], [u8; 32]) = (
    [
        15, 244, 194, 152, 48, 102, 16, 30, 201, 92, 40, 155, 68, 183, 67, 44, 99, 163, 152, 38,
        99, 82, 136, 230, 79, 7, 246, 126, 5, 115, 236, 38,
    ],
    [
        111, 190, 31, 34, 22, 64, 206, 247, 250, 75, 120, 48, 132, 183, 190, 222, 242, 10, 244,
        189, 244, 158, 82, 19, 17, 77, 71, 93, 148, 240, 120, 16,
    ],
);

/// Generator used in SinsemillaHashToPoint for Merkle collision-resistant hash
pub const Q_MERKLE_CRH: ([u8; 32], [u8; 32]) = (
    [
        109, 131, 41, 145, 131, 167, 124, 146, 255, 59, 69, 88, 173, 99, 176, 39, 6, 29, 234, 237,
        189, 119, 140, 28, 209, 251, 3, 251, 133, 240, 159, 32,
    ],
    [
        203, 59, 20, 136, 4, 179, 213, 0, 24, 204, 101, 110, 131, 91, 228, 86, 81, 18, 56, 67, 12,
        153, 160, 95, 190, 61, 129, 107, 108, 54, 79, 41,
    ],
);

// Sinsemilla S generators

/// SWU hash-to-curve personalization for Sinsemilla $S$ generators.
pub const S_PERSONALIZATION: &str = "z.cash:SinsemillaS";

/// Creates the Sinsemilla S generators used in each round of the Sinsemilla hash
// TODO: inline the Sinsemilla S generators used in each round of the Sinsemilla hash
pub fn sinsemilla_s_generators<C: CurveAffine>() -> Vec<C::CurveExt> {
    let hasher = C::CurveExt::hash_to_curve(S_PERSONALIZATION);
    (0..(1 << K))
        .map(|j| hasher(&(j as usize).to_le_bytes()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::{CommitDomain, HashDomain};
    use super::*;
    use crate::constants::{
        COMMIT_IVK_PERSONALIZATION, MERKLE_CRH_PERSONALIZATION, NOTE_COMMITMENT_PERSONALIZATION,
    };
    use group::Curve;
    use halo2::arithmetic::FieldExt;
    use halo2::pasta::pallas;

    #[test]
    fn q_note_commitment_m() {
        let domain = CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
        let point = domain.M.Q;
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(
            x,
            pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.0).unwrap()
        );
        assert_eq!(
            y,
            pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.1).unwrap()
        );
    }

    #[test]
    fn q_commit_ivk_m() {
        let domain = CommitDomain::new(COMMIT_IVK_PERSONALIZATION);
        let point = domain.M.Q;
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(
            x,
            pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.0).unwrap()
        );
        assert_eq!(
            y,
            pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.1).unwrap()
        );
    }

    #[test]
    fn q_merkle_crh() {
        let domain = HashDomain::new(MERKLE_CRH_PERSONALIZATION);
        let point = domain.Q;
        let (x, y) = point.to_affine().get_xy().unwrap();

        assert_eq!(x, pallas::Base::from_bytes(&Q_MERKLE_CRH.0).unwrap());
        assert_eq!(y, pallas::Base::from_bytes(&Q_MERKLE_CRH.1).unwrap());
    }
}
