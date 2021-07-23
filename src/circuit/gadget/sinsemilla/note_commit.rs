use group::GroupEncoding;
use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

use crate::{
    circuit::gadget::{
        ecc::{
            chip::{EccChip, EccPoint},
            Point,
        },
        utilities::{bitrange_subset, bool_check, copy, CellValue, Var},
    },
    constants::T_P,
};

use super::{
    chip::{SinsemillaChip, SinsemillaCommitDomains, SinsemillaConfig},
    CommitDomain, Message, MessagePiece,
};

/*
    <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
    We need to hash g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi,
    where
        - g★_d is the representation of the point g_d, with 255 bits used for the
          x-coordinate and 1 bit used for the y-coordinate;
        - pk★_d is the representation of the point pk_d, with 255 bits used for the
          x-coordinate and 1 bit used for the y-coordinate;
        - v is a 64-bit value;
        - rho is a base field element (255 bits); and
        - psi is a base field element (255 bits).

    All bit ranges are inclusive.
    a (250 bits) = bits 0..=249 of x(g_d)
    b (10 bits)  = b_0 || b_1 || b_2 || b_3
                 = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
    c (250 bits) = bits 4..=253 of pk★_d
    d (60 bits)  = d_0 || d_1 || d_2 || d_3
                 = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (0..=7 of v) || (8..=57 of v)
    e (10 bits)  = e_0 || e_1
                 = (bits 58..=63 of v) || (bits 0..=3 of rho)
    f (250 bits) = bits 4..=253 inclusive of rho
    g (250 bits) = g_0 || g_1 || g_2
                 = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
    h (10 bits)  = h_0 || h_1 || h_2
                 = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
*/

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct NoteCommitConfig {
    q_canon: Column<Fixed>,
    advices: [Column<Advice>; 10],
    sinsemilla_config: SinsemillaConfig,
}

impl NoteCommitConfig {
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        sinsemilla_config: SinsemillaConfig,
    ) -> Self {
        let q_canon = meta.fixed_column();

        let config = Self {
            q_canon,
            advices,
            sinsemilla_config,
        };

        // Useful constants
        let two = pallas::Base::from_u64(2);
        let two_pow_2 = pallas::Base::from_u64(1 << 2);
        let two_pow_4 = two_pow_2.square();
        let two_pow_5 = two_pow_4 * two;
        let two_pow_6 = two_pow_5 * two;
        let two_pow_8 = two_pow_4.square();
        let two_pow_9 = two_pow_8 * two;
        let two_pow_10 = two_pow_9 * two;
        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        let two_pow_130 = Expression::Constant(pallas::Base::from_u128(1 << 65).square());
        let two_pow_140 = Expression::Constant(pallas::Base::from_u128(1 << 70).square());
        let t_p = Expression::Constant(pallas::Base::from_u128(T_P));

        meta.create_gate("NoteCommit decomposition check", |meta| {
            /*
                a (250 bits) = bits 0..=249 of x(g_d)
                b (10 bits)  = b_0 || b_1 || b_2 || b_3
                             = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
                c (250 bits) = bits 4..=253 of pk★_d
                d (60 bits)  = d_0 || d_1 || d_2 || d_3
                             = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (0..=7 of v) || (8..=57 of v)
                e (10 bits)  = e_0 || e_1
                             = (bits 58..=63 of v) || (bits 0..=3 of rho)
                f (250 bits) = bits 4..=253 inclusive of rho
                g (250 bits) = g_0 || g_1 || g_2
                             = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
                h (10 bits)  = h_0 || h_1 || h_2
                             = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits

                |         A_0         |          A_1           |         A_2            |          A_3            |  A_4  |  A_5  |  A_6  |  A_7   |  A_8  |   A_9   |   q_canon   |
                -------------------------------------------------------------------------------------------------------------------------------------------------------------------
                |       a_prime       |       b3_c_prime       |       e1_f_prime       |       g1_g2_prime       |   a   |   b   |  b_2  |   b_3  |   c   |    d    |             |
                |         d_1         |           d_2          |          z1_d          |           e             |  e_0  |  e_1  |   f   |    g   |  g_1  |    h    |      1      |
                |         h_0         |           h_1          |         x(g_d)         |        x(pk_d)          | value |  b_0  |  b_1  |   d_0  |  g_0  |   z1_g  |      2      |
                |a_prime_decomposition|b3_c_prime_decomposition|e1_f_prime_decomposition|g1_g2_prime_decomposition| z13_a | z13_c | z13_f | z13_g  |  psi  |   rho   |             |

             q_canon_is_one checks that:
              - piece decomposition:
                  - b = b_0 + (2^4) b_1 + (2^5) b_2 + (2^6) b_3
                    - b_1 is boolean
                    - b_2 is boolean
                  - d = d_0 + (2) d_1 + (2^2) d_2 + (2^10) d_3
                    - d_0 is boolean
                    - d_1 is boolean
                  - e = e_0 + (2^6) e_1
                  - g = g_0 + (2) g_1 + (2^10) g_2
                    - g_0 is boolean
                  - h = h_0 + (2^5) h_1
                    - h_1 is boolean
              - field element decomposition:
                  - x(g_d) = a + (2^250) b_0 + (2^254) b_1
                  - x(pk_d) = b_3 + (2^4) c + (2^254) d_0
                  - value = d_2 + (2^8) d_3 + (2^58) e_0
               - *_prime derivations:
                  - a_prime = a + 2^130 - t_P
                  - b3_c_prime = b_3 + (2^4)c + 2^140 - t_P
                  - e1_f_prime = e_1 + (2^4)g + 2^140 - t_P
                  - g1_g2_prime = g_1 + (2^9) g_2 + 2^140 - t_P
            */
            let q_canon_is_one = {
                let two = Expression::Constant(pallas::Base::from_u64(2));
                let q_canon = meta.query_fixed(config.q_canon, Rotation::cur());
                q_canon.clone() * (two - q_canon)
            };

            // Offset prev
            let a_prime = meta.query_advice(config.advices[0], Rotation::prev());
            let b3_c_prime = meta.query_advice(config.advices[1], Rotation::prev());
            let e1_f_prime = meta.query_advice(config.advices[2], Rotation::prev());
            let g1_g2_prime = meta.query_advice(config.advices[3], Rotation::prev());
            // `a` has been constrained to 250 bits by the Sinsemilla hash.
            let a = meta.query_advice(config.advices[4], Rotation::prev());
            // `b` has been constrained to 10 bits by the Sinsemilla hash.
            let b_whole = meta.query_advice(config.advices[5], Rotation::prev());
            // This gate constrains b_2 to be boolean.
            let b_2 = meta.query_advice(config.advices[6], Rotation::prev());
            // `b_3` has been constrained to 4 bits outside this gate.
            let b_3 = meta.query_advice(config.advices[7], Rotation::prev());
            // `c` has been constrained to 250 bits by the Sinsemilla hash.
            let c = meta.query_advice(config.advices[8], Rotation::prev());
            // `d` has been constrained to 10 bits by the Sinsemilla hash.
            let d_whole = meta.query_advice(config.advices[9], Rotation::prev());

            // Offset cur
            // This gate constrains d_1 to be boolean.
            let d_1 = meta.query_advice(config.advices[0], Rotation::cur());
            // `d_2` has been constrained to 8 bits outside this gate.
            let d_2 = meta.query_advice(config.advices[1], Rotation::cur());
            // `z1_d` has been constrained to 50 bits by the Sinsemilla hash.
            let z1_d = meta.query_advice(config.advices[2], Rotation::cur());
            let d_3 = z1_d;
            // `e` has been constrained to 10 bits by the Sinsemilla hash.
            let e_whole = meta.query_advice(config.advices[3], Rotation::cur());
            // `e_0` has been constrained to 6 bits outside this gate.
            let e_0 = meta.query_advice(config.advices[4], Rotation::cur());
            // `e_1` has been constrained to 4 bits outside this gate.
            let e_1 = meta.query_advice(config.advices[5], Rotation::cur());
            // `f` has been constrained to 250 bits by the Sinsemilla hash.
            let f = meta.query_advice(config.advices[6], Rotation::cur());
            // `g` has been constrained to 250 bits by the Sinsemilla hash.
            let g_whole = meta.query_advice(config.advices[7], Rotation::cur());
            // `g_1` has been constrained to 9 bits outside this gate.
            let g_1 = meta.query_advice(config.advices[8], Rotation::cur());
            // `h` has been constrained to 10 bits by the Sinsemilla hash.
            let h_whole = meta.query_advice(config.advices[9], Rotation::cur());

            // Offset next
            // h_0 has been constrained to be 5 bits outside this gate.
            let h_0 = meta.query_advice(config.advices[0], Rotation::next());
            // This gate constrains h_1 to be boolean.
            let h_1 = meta.query_advice(config.advices[1], Rotation::next());
            let gd_x = meta.query_advice(config.advices[2], Rotation::next());
            let pkd_x = meta.query_advice(config.advices[3], Rotation::next());
            let value = meta.query_advice(config.advices[4], Rotation::next());
            // b_0 has been constrained to be 4 bits outside this gate.
            let b_0 = meta.query_advice(config.advices[5], Rotation::next());
            // This gate constrains b_1 to be boolean.
            let b_1 = meta.query_advice(config.advices[6], Rotation::next());
            // This gate constrains d_0 to be boolean.
            let d_0 = meta.query_advice(config.advices[7], Rotation::next());
            // This gate constrains g_0 to be boolean.
            let g_0 = meta.query_advice(config.advices[8], Rotation::next());
            // z1_g has been constrained to 240 bits by the Sinsemilla hash.
            let z1_g = meta.query_advice(config.advices[9], Rotation::next());
            let g_2 = z1_g;

            // Boolean checks on 1-bit pieces.
            let boolean_checks = std::iter::empty()
                .chain(Some(("bool_check b_1", bool_check(b_1.clone()))))
                .chain(Some(("bool_check b_2", bool_check(b_2.clone()))))
                .chain(Some(("bool_check d_0", bool_check(d_0.clone()))))
                .chain(Some(("bool_check d_1", bool_check(d_1.clone()))))
                .chain(Some(("bool_check g_0", bool_check(g_0.clone()))))
                .chain(Some(("bool_check h_1", bool_check(h_1.clone()))));

            // b = b_0 + (2^4) b_1 + (2^5) b_2 + (2^6) b_3
            let b_check = b_whole
                - (b_0.clone()
                    + b_1.clone() * two_pow_4
                    + b_2 * two_pow_5
                    + b_3.clone() * two_pow_6);
            // d = d_0 + (2) d_1 + (2^2) d_2 + (2^10) d_3
            let d_check = d_whole
                - (d_0.clone() + d_1 * two + d_2.clone() * two_pow_2 + d_3.clone() * two_pow_10);
            // e = e_0 + (2^6) e_1
            let e_check = e_whole - (e_0.clone() + e_1.clone() * two_pow_6);
            // g = g_0 + (2) g_1 + (2^10) g_2
            let g_check = g_whole - (g_0 + g_1.clone() * two + g_2.clone() * two_pow_10);
            // h = h_0 + (2^5) h_1
            let h_check = h_whole - (h_0 + h_1 * two_pow_5);

            // Check that *_prime pieces were correctly derived.
            // a_prime = a + 2^130 - t_P
            let a_prime_check = a.clone() + two_pow_130 - t_p.clone() - a_prime;

            // b3_c_prime = b_3 + (2^4)c + 2^140 - t_P
            let b3_c_prime_check = b_3.clone() + (c.clone() * two_pow_4) + two_pow_140.clone()
                - t_p.clone()
                - b3_c_prime;

            // e1_f_prime = e_1 + (2^4)f + 2^140 - t_P
            let e1_f_prime_check =
                e_1 + (f * two_pow_4) + two_pow_140.clone() - t_p.clone() - e1_f_prime;

            // g1_g2_prime = g_1 + (2^9)g_2 + 2^140 - t_P
            let g1_g2_prime_check = {
                let two_pow_9 = two_pow_4 * two_pow_5;
                g_1 + (g_2 * two_pow_9) + two_pow_140.clone() - t_p.clone() - g1_g2_prime
            };

            // x(g_d) = a + (2^250)b_0 + (2^254)b_1
            let gd_x_check = {
                let two_pow_250 = pallas::Base::from_u128(1 << 125).square();
                let sum = a + b_0 * two_pow_250 + b_1 * two_pow_254;
                sum - gd_x
            };

            // x(pk_d) = b_3 + (2^4)c + (2^254)d_0
            let pkd_x_check = {
                let sum = b_3 + c * two_pow_4 + d_0 * two_pow_254;
                sum - pkd_x
            };

            // value = d_2 + (2^8)d_3 + (2^58)e_0
            let value_check = {
                let two_pow_8 = pallas::Base::from_u64(1 << 8);
                let two_pow_58 = pallas::Base::from_u64(1 << 58);
                d_2 + d_3 * two_pow_8 + e_0 * two_pow_58 - value
            };

            std::iter::empty()
                .chain(boolean_checks)
                .chain(Some(("a_prime_check", a_prime_check)))
                .chain(Some(("b3_c_prime_check", b3_c_prime_check)))
                .chain(Some(("e1_f_prime_check", e1_f_prime_check)))
                .chain(Some(("g1_g2_prime_check", g1_g2_prime_check)))
                .chain(Some(("b_check", b_check)))
                .chain(Some(("d_check", d_check)))
                .chain(Some(("e_check", e_check)))
                .chain(Some(("g_check", g_check)))
                .chain(Some(("h_check", h_check)))
                .chain(Some(("gd_x_check", gd_x_check)))
                .chain(Some(("pkd_x_check", pkd_x_check)))
                .chain(Some(("value_check", value_check)))
                .map(move |(name, poly)| (name, q_canon_is_one.clone() * poly))
        });

        meta.create_gate("Canonicity checks", |meta| {
            /*
                a (250 bits) = bits 0..=249 of x(g_d)
                b (10 bits)  = b_0 || b_1 || b_2 || b_3
                            = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
                c (250 bits) = bits 4..=253 of pk★_d
                d (60 bits)  = d_0 || d_1 || d_2 || d_3
                            = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (0..=7 of v) || (8..=57 of v)
                e (10 bits)  = e_0 || e_1
                            = (bits 58..=63 of v) || (bits 0..=3 of rho)
                f (250 bits) = bits 4..=253 inclusive of rho
                g (250 bits) = g_0 || g_1 || g_2
                            = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
                h (10 bits)  = h_0 || h_1 || h_2
                            = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits

                |         A_0         |          A_1           |         A_2            |          A_3            |  A_4  |  A_5  |  A_6  |  A_7   |  A_8  |   A_9   |   q_canon   |
                -------------------------------------------------------------------------------------------------------------------------------------------------------------------
                |       a_prime       |       b3_c_prime       |       e1_f_prime       |       g1_g2_prime       |   a   |   b   |  b_2  |   b_3  |   c   |    d    |             |
                |         d_1         |           d_2          |          z1_d          |           e             |  e_0  |  e_1  |   f   |    g   |  g_1  |    h    |      1      |
                |         h_0         |           h_1          |         x(g_d)         |        x(pk_d)          | value |  b_0  |  b_1  |   d_0  |  g_0  |   z1_g  |      2      |
                |a_prime_decomposition|b3_c_prime_decomposition|e1_f_prime_decomposition|g1_g2_prime_decomposition| z13_a | z13_c | z13_f | z13_g  |  psi  |   rho   |             |
            */

            // q_canon_is_two checks that:
            //   - field element decomposition:
            //      - rho = e_1 + (2^4) g + (2^254) h_0
            //      - psi = h_1 + (2^9) i + (2^249) j_0 + (2^254) j_1
            //   - canonicity:
            //      - b_1 = 0 => b_0 = 0
            //                && z13_a = 0
            //                && a_prime_decomposition = 0
            //      - d_0 = 0 => z13_c = 0
            //                && b3_c_prime_decomposition = 0
            //      - h_0 = 0 => z13_f = 0
            //                && e1_f_prime_decomposition = 0
            //      - j_1 = 0 => j_0 = 0
            //                && g1_g2_prime_decomposition = 0

            let q_canon_is_two = {
                let one = Expression::Constant(pallas::Base::one());
                let q_canon = meta.query_fixed(config.q_canon, Rotation::cur());
                q_canon.clone() * (one - q_canon)
            };

            // Offset prev
            let e_1 = meta.query_advice(config.advices[5], Rotation::prev());
            let f = meta.query_advice(config.advices[6], Rotation::prev());
            let g_1 = meta.query_advice(config.advices[8], Rotation::prev());

            // Offset cur
            let h_0 = meta.query_advice(config.advices[0], Rotation::cur());
            let h_1 = meta.query_advice(config.advices[1], Rotation::cur());
            let b_0 = meta.query_advice(config.advices[5], Rotation::cur());
            let b_1 = meta.query_advice(config.advices[6], Rotation::cur());
            let d_0 = meta.query_advice(config.advices[7], Rotation::cur());
            let g_0 = meta.query_advice(config.advices[8], Rotation::cur());
            let z1_g = meta.query_advice(config.advices[9], Rotation::cur());
            let g_2 = z1_g;

            // Offset next
            let a_prime_decomposition = meta.query_advice(config.advices[0], Rotation::next());
            let b3_c_prime_decomposition = meta.query_advice(config.advices[1], Rotation::next());
            let e1_f_prime_decomposition = meta.query_advice(config.advices[2], Rotation::next());
            let g1_g2_prime_decomposition = meta.query_advice(config.advices[3], Rotation::next());
            let z13_a = meta.query_advice(config.advices[4], Rotation::next());
            let z13_c = meta.query_advice(config.advices[5], Rotation::next());
            let z13_f = meta.query_advice(config.advices[6], Rotation::next());
            let z13_g = meta.query_advice(config.advices[7], Rotation::next());
            let psi = meta.query_advice(config.advices[8], Rotation::next());
            let rho = meta.query_advice(config.advices[9], Rotation::next());

            // rho = e_1 + (2^4) f + (2^254) g_0
            let rho_decomposition_check = {
                let sum = e_1 + f * two_pow_4 + g_0.clone() * two_pow_254;
                sum - rho
            };

            // psi = g_1 + (2^9) g_2 + (2^249) h_0 + (2^254) h_1
            let psi_decomposition_check = {
                let two_pow_249 =
                    pallas::Base::from_u128(1 << 124).square() * pallas::Base::from_u128(2);
                let sum = g_1
                    + g_2 * pallas::Base::from_u64(1 << 9)
                    + h_0.clone() * two_pow_249
                    + h_1.clone() * two_pow_254;
                sum - psi
            };

            // The gd_x_canonicity_checks are enforced if and only if `b_1` = 1.
            // x(g_d) = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let gd_x_canonicity_checks = std::iter::empty()
                .chain(Some(b_0))
                .chain(Some(z13_a))
                .chain(Some(a_prime_decomposition))
                .map(move |poly| b_1.clone() * poly);

            // The pkd_x_canonicity_checks are enforced if and only if `d_0` = 1.
            // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
            let pkd_x_canonicity_checks = std::iter::empty()
                .chain(Some(z13_c))
                .chain(Some(b3_c_prime_decomposition))
                .map(move |poly| d_0.clone() * poly);

            // The rho_canonicity_checks are enforced if and only if `g_0` = 1.
            // rho = e_1 (4 bits) || f (250 bits) || g_0 (1 bit)
            let rho_canonicity_checks = std::iter::empty()
                .chain(Some(z13_f))
                .chain(Some(e1_f_prime_decomposition))
                .map(move |poly| g_0.clone() * poly);

            // The psi_canonicity_checks are enforced if and only if `i_1` = 1.
            // `psi` = `g_1 (9 bits) || g_2 (240 bits) || h_0 (5 bits) || h_1 (1 bit)`
            let psi_canonicity_checks = std::iter::empty()
                .chain(Some(h_0))
                .chain(Some(z13_g))
                .chain(Some(g1_g2_prime_decomposition))
                .map(move |poly| h_1.clone() * poly);

            std::iter::empty()
                .chain(Some(rho_decomposition_check))
                .chain(Some(psi_decomposition_check))
                .chain(gd_x_canonicity_checks)
                .chain(pkd_x_canonicity_checks)
                .chain(rho_canonicity_checks)
                .chain(psi_canonicity_checks)
                .map(move |poly| q_canon_is_two.clone() * poly)
        });

        config
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::circuit) fn assign_region(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        chip: SinsemillaChip,
        ecc_chip: EccChip,
        g_d: &EccPoint,
        pk_d: &EccPoint,
        value: CellValue<pallas::Base>,
        rho: CellValue<pallas::Base>,
        psi: CellValue<pallas::Base>,
        rcm: Option<pallas::Scalar>,
    ) -> Result<Point<pallas::Affine, EccChip>, Error> {
        let (gd_x, gd_y) = point_repr(g_d.point());
        let (pkd_x, pkd_y) = point_repr(pk_d.point());
        let value_val = value.value();
        let rho_val = rho.value();
        let psi_val = psi.value();

        // `a` = bits 0..=249 of `x(g_d)`
        let a = {
            let a = gd_x.map(|gd_x| bitrange_subset(gd_x, 0..250));
            MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "a"), a, 25)?
        };

        // b = b_0 || b_1 || b_2 || b_3
        //   = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
        let (b_0, b_1, b_2, b_3, b) =
            {
                let b_0 = gd_x.map(|gd_x| bitrange_subset(gd_x, 250..254));
                let b_1 = gd_x.map(|gd_x| bitrange_subset(gd_x, 254..255));
                let b_2 = gd_y.map(|gd_y| bitrange_subset(gd_y, 0..1));
                let b_3 = pkd_x.map(|pkd_x| bitrange_subset(pkd_x, 0..4));

                // Constrain b_0 to be 4 bits
                let b_0 = self.sinsemilla_config.lookup_config.witness_short_check(
                    layouter.namespace(|| "b_0 is 4 bits"),
                    b_0,
                    4,
                )?;

                // Constrain b_3 to be 4 bits
                let b_3 = self.sinsemilla_config.lookup_config.witness_short_check(
                    layouter.namespace(|| "b_3 is 4 bits"),
                    b_3,
                    4,
                )?;

                // b_1, b_2 will be boolean-constrained in the gate.

                let b = b_0.value().zip(b_1).zip(b_2).zip(b_3.value()).map(
                    |(((b_0, b_1), b_2), b_3)| {
                        let b1_shifted = b_1 * pallas::Base::from_u64(1 << 4);
                        let b2_shifted = b_2 * pallas::Base::from_u64(1 << 5);
                        let b3_shifted = b_3 * pallas::Base::from_u64(1 << 6);
                        b_0 + b1_shifted + b2_shifted + b3_shifted
                    },
                );

                let b =
                    MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "b"), b, 1)?;

                (b_0, b_1, b_2, b_3, b)
            };

        // c = bits 4..=253 of pk★_d
        let c = {
            let c = pkd_x.map(|pkd_x| bitrange_subset(pkd_x, 4..254));
            MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "c"), c, 25)?
        };

        // d = d_0 || d_1 || d_2 || d_3
        //   = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (bits 0..=7 of v) || (bits 8..=57 of v)
        let (d_0, d_1, d_2, d) = {
            let d_0 = pkd_x.map(|pkd_x| bitrange_subset(pkd_x, 254..255));
            let d_1 = pkd_y.map(|pkd_y| bitrange_subset(pkd_y, 0..1));
            let d_2 = value_val.map(|value| bitrange_subset(value, 0..8));
            let d_3 = value_val.map(|value| bitrange_subset(value, 8..58));

            // Constrain d_2 to be 8 bits
            let d_2 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "d_2 is 8 bits"),
                d_2,
                8,
            )?;

            // d_0, d_1 will be boolean-constrained in the gate.
            // d_3 = z1_d from the SinsemillaHash(d) running sum output.

            let d = d_0
                .zip(d_1)
                .zip(d_2.value())
                .zip(d_3)
                .map(|(((d_0, d_1), d_2), d_3)| {
                    let d1_shifted = d_1 * pallas::Base::from_u64(2);
                    let d2_shifted = d_2 * pallas::Base::from_u64(1 << 2);
                    let d3_shifted = d_3 * pallas::Base::from_u64(1 << 10);
                    d_0 + d1_shifted + d2_shifted + d3_shifted
                });

            let d = MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "d"), d, 6)?;

            (d_0, d_1, d_2, d)
        };

        // e = e_0 || e_1 = (bits 58..=63 of v) || (bits 0..=3 of rho)
        let (e_0, e_1, e) = {
            let e_0 = value_val.map(|value| bitrange_subset(value, 58..64));
            let e_1 = rho_val.map(|rho| bitrange_subset(rho, 0..4));

            // Constrain e_0 to be 6 bits.
            let e_0 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "e_0 is 6 bits"),
                e_0,
                6,
            )?;

            // Constrain e_1 to be 4 bits.
            let e_1 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "e_1 is 4 bits"),
                e_1,
                4,
            )?;

            let e = e_0
                .value()
                .zip(e_1.value())
                .map(|(e_0, e_1)| e_0 + e_1 * pallas::Base::from_u64(1 << 6));
            let e = MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "e"), e, 1)?;

            (e_0, e_1, e)
        };

        // f = bits 4..=253 inclusive of rho
        let f = {
            let f = rho_val.map(|rho| bitrange_subset(rho, 4..254));
            MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "f"), f, 25)?
        };

        // g = g_0 || g_1 || g_2
        //   = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
        let (g_0, g_1, g) = {
            let g_0 = rho_val.map(|rho| bitrange_subset(rho, 254..255));
            let g_1 = psi_val.map(|psi| bitrange_subset(psi, 0..9));
            let g_2 = psi_val.map(|psi| bitrange_subset(psi, 9..249));

            // Constrain g_1 to be 9 bits.
            let g_1 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "g_1 is 9 bits"),
                g_1,
                9,
            )?;

            // g_0 will be boolean-constrained in the gate.
            // g_2 = z1_g from the SinsemillaHash(g) running sum output.

            let g = g_0.zip(g_1.value()).zip(g_2).map(|((g_0, g_1), g_2)| {
                let g1_shifted = g_1 * pallas::Base::from_u64(2);
                let g2_shifted = g_2 * pallas::Base::from_u64(1 << 10);
                g_0 + g1_shifted + g2_shifted
            });
            let g = MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "g"), g, 25)?;

            (g_0, g_1, g)
        };

        // h = h_0 || h_1 || h_2
        //   = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
        let (h_0, h_1, h) = {
            let h_0 = psi_val.map(|psi| bitrange_subset(psi, 249..254));
            let h_1 = psi_val.map(|psi| bitrange_subset(psi, 254..255));

            // Constrain h_0 to be 5 bits.
            let h_0 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "h_0 is 5 bits"),
                h_0,
                5,
            )?;

            // h_1 will be boolean-constrained in the gate.

            let h = h_0
                .value()
                .zip(h_1)
                .map(|(h_0, h_1)| h_0 + h_1 * pallas::Base::from_u64(1 << 5));
            let h = MessagePiece::from_field_elem(chip.clone(), layouter.namespace(|| "h"), h, 1)?;

            (h_0, h_1, h)
        };

        let (cm, zs) = {
            let message = Message::from_pieces(
                chip.clone(),
                vec![
                    a.clone(),
                    b.clone(),
                    c.clone(),
                    d.clone(),
                    e.clone(),
                    f.clone(),
                    g.clone(),
                    h.clone(),
                ],
            );
            let domain = CommitDomain::new(chip, ecc_chip, &SinsemillaCommitDomains::NoteCommit);
            domain.commit(
                layouter.namespace(|| "Process NoteCommit inputs"),
                message,
                rcm,
            )?
        };

        let z13_a = zs[0][13];
        let z13_c = zs[2][13];
        let z1_d = zs[3][1];
        let z13_f = zs[5][13];
        let z1_g = zs[6][1];
        let g_2 = z1_g;
        let z13_g = zs[6][13];

        let (a_prime, a_prime_decomposition) = self.gd_x_canonicity(
            layouter.namespace(|| "x(g_d) canonicity"),
            a.inner().cell_value(),
        )?;

        let (b3_c_prime, b3_c_prime_decomposition) = self.pkd_x_canonicity(
            layouter.namespace(|| "x(pk_d) canonicity"),
            b_3,
            c.inner().cell_value(),
        )?;

        let (e1_f_prime, e1_f_prime_decomposition) = self.rho_canonicity(
            layouter.namespace(|| "rho canonicity"),
            e_1,
            f.inner().cell_value(),
        )?;

        let (g1_g2_prime, g1_g2_prime_decomposition) =
            self.psi_canonicity(layouter.namespace(|| "psi canonicity"), g_1, g_2)?;

        let gate_cells = GateCells {
            a: a.inner().cell_value(),
            b: b.inner().cell_value(),
            b_0,
            b_1,
            b_2,
            b_3,
            c: c.inner().cell_value(),
            d: d.inner().cell_value(),
            d_0,
            d_1,
            d_2,
            z1_d,
            e: e.inner().cell_value(),
            e_0,
            e_1,
            f: f.inner().cell_value(),
            g: g.inner().cell_value(),
            g_0,
            g_1,
            z1_g,
            h: h.inner().cell_value(),
            h_0,
            h_1,
            gd_x: g_d.x(),
            pkd_x: pk_d.x(),
            value,
            rho,
            psi,
            a_prime,
            b3_c_prime,
            e1_f_prime,
            g1_g2_prime,
            a_prime_decomposition,
            b3_c_prime_decomposition,
            e1_f_prime_decomposition,
            g1_g2_prime_decomposition,
            z13_a,
            z13_c,
            z13_f,
            z13_g,
        };

        self.assign_gate(layouter.namespace(|| "Assign gate cells"), gate_cells)?;

        Ok(cm)
    }

    #[allow(clippy::type_complexity)]
    // Check canonicity of `x(g_d)` encoding
    fn gd_x_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        a: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `x(g_d)` = `a (250 bits) || b_0 (4 bits) || b_1 (1 bit)`
        // - b_1 = 1 => b_0 = 0
        // - b_1 = 1 => a < t_P
        //     - 0 ≤ a < 2^130 (z_13 of SinsemillaHash(a))
        //     - 0 ≤ a + 2^130 - t_P < 2^130 (thirteen 10-bit lookups)

        // Decompose the low 130 bits of a_prime = a + 2^130 - t_P, and output
        // the running sum at the end of it. If a_prime < 2^130, the running sum
        // will be 0.
        let a_prime = a.value().map(|a| {
            let two_pow_130 = pallas::Base::from_u128(1u128 << 65).square();
            let t_p = pallas::Base::from_u128(T_P);
            a + two_pow_130 - t_p
        });
        let (a_prime, zs) = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 130 bits of (a + 2^130 - t_P)"),
            a_prime,
            13,
            false,
        )?;
        assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z_13]

        Ok((a_prime, zs[13]))
    }

    // Check canonicity of `x(pk_d)` encoding
    fn pkd_x_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        b_3: CellValue<pallas::Base>,
        c: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
        // - d_0 = 1 => b_3 + 2^4 c < t_P
        //     - 0 ≤ b_3 + 2^4 c < 2^134
        //         - b_3 is part of the Sinsemilla message piece
        //           b = b_0 (4 bits) || b_1 (1 bit) || b_2 (1 bit) || b_3 (4 bits)
        //         - b_3 is individually constrained to be 4 bits.
        //         - z_13 of SinsemillaHash(c) == 0 constrains bits 4..=253 of pkd_x
        //           to 130 bits. z13_c is directly checked in the gate.
        //     - 0 ≤ b_3 + 2^4 c + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Decompose the low 140 bits of b3_c_prime = b_3 + 2^4 c + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If b3_c_prime < 2^140, the running sum will be 0.
        let b3_c_prime = b_3.value().zip(c.value()).map(|(b_3, c)| {
            let two_pow_4 = pallas::Base::from_u64(1u64 << 4);
            let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
            let t_p = pallas::Base::from_u128(T_P);
            b_3 + (two_pow_4 * c) + two_pow_140 - t_p
        });

        let (b3_c_prime, zs) = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (b_3 + 2^4 c + 2^140 - t_P)"),
            b3_c_prime,
            14,
            false,
        )?;
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]

        Ok((b3_c_prime, zs[14]))
    }

    #[allow(clippy::type_complexity)]
    // Check canonicity of `rho` encoding
    fn rho_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        e_1: CellValue<pallas::Base>,
        f: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `rho` = `e_1 (4 bits) || f (250 bits) || g_0 (1 bit)`
        // - g_0 = 1 => e_1 + 2^4 f < t_P
        // - 0 ≤ e_1 + 2^4 f < 2^134
        //     - e_1 is part of the Sinsemilla message piece
        //       e = e_0 (56 bits) || e_1 (4 bits)
        //     - e_1 is individually constrained to be 4 bits.
        //     - z_13 of SinsemillaHash(f) == 0 constrains bits 4..=253 of rho
        //       to 130 bits. z13_f == 0 is directly checked in the gate.
        // - 0 ≤ e_1 + 2^4 f + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        let e1_f_prime = e_1.value().zip(f.value()).map(|(e_1, f)| {
            let two_pow_4 = pallas::Base::from_u64(1u64 << 4);
            let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
            let t_p = pallas::Base::from_u128(T_P);
            e_1 + (two_pow_4 * f) + two_pow_140 - t_p
        });

        // Decompose the low 140 bits of e1_f_prime = e_1 + 2^4 f + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If e1_f_prime < 2^140, the running sum will be 0.
        let (e1_f_prime, zs) = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (e_1 + 2^4 f + 2^140 - t_P)"),
            e1_f_prime,
            14,
            false,
        )?;
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]

        Ok((e1_f_prime, zs[14]))
    }

    // Check canonicity of `psi` encoding
    fn psi_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        g_1: CellValue<pallas::Base>,
        g_2: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `psi` = `g_1 (9 bits) || g_2 (240 bits) || h_0 (5 bits) || h_1 (1 bit)`
        // - h_1 = 1 => (h_0 = 0) ∧ (g_1 + 2^9 g_2 < t_P)
        // - 0 ≤ g_1 + 2^9 g_2 < 2^139
        //     - g_1 is individually constrained to be 9 bits
        //     - z_13 of SinsemillaHash(g) == 0 constrains bits 0..=248 of psi
        //       to 130 bits. z13_g == 0 is directly checked in the gate.
        // - 0 ≤ g_1 + (2^9)g_2 + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Decompose the low 140 bits of g1_g2_prime = g_1 + (2^9)g_2 + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If g1_g2_prime < 2^140, the running sum will be 0.
        let g1_g2_prime = g_1.value().zip(g_2.value()).map(|(g_1, g_2)| {
            let two_pow_9 = pallas::Base::from_u64(1u64 << 9);
            let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
            let t_p = pallas::Base::from_u128(T_P);
            g_1 + (two_pow_9 * g_2) + two_pow_140 - t_p
        });

        let (g1_g2_prime, zs) = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (g_1 + (2^9)g_2 + 2^140 - t_P)"),
            g1_g2_prime,
            14,
            false,
        )?;
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]

        Ok((g1_g2_prime, zs[14]))
    }

    fn assign_gate(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        gate_cells: GateCells,
    ) -> Result<(), Error> {
        /*
            The pieces are witnessed in the below configuration, such that no gate has to query an
            offset greater than +/- 1 from its relative row.

            |         A_0         |          A_1           |         A_2            |          A_3            |  A_4  |  A_5  |  A_6  |  A_7   |  A_8  |   A_9   |   q_canon   |
            -------------------------------------------------------------------------------------------------------------------------------------------------------------------
            |       a_prime       |       b3_c_prime       |       e1_f_prime       |       g1_g2_prime       |   a   |   b   |  b_2  |   b_3  |   c   |    d    |             |
            |         d_1         |           d_2          |          z1_d          |           e             |  e_0  |  e_1  |   f   |    g   |  g_1  |    h    |      1      |
            |         h_0         |           h_1          |         x(g_d)         |        x(pk_d)          | value |  b_0  |  b_1  |   d_0  |  g_0  |   z1_g  |      2      |
            |a_prime_decomposition|b3_c_prime_decomposition|e1_f_prime_decomposition|g1_g2_prime_decomposition| z13_a | z13_c | z13_f | z13_g  |  psi  |   rho   |             |

        */
        layouter.assign_region(
            || "Assign gate cells",
            |mut region| {
                // Assign fixed column the correct values
                region.assign_fixed(
                    || "q_canon = 1",
                    self.q_canon,
                    1,
                    || Ok(pallas::Base::one()),
                )?;
                region.assign_fixed(
                    || "q_canon = 2",
                    self.q_canon,
                    2,
                    || Ok(pallas::Base::from_u64(2)),
                )?;

                // Offset 0
                {
                    let offset = 0;

                    // advices[0]
                    copy(
                        &mut region,
                        || "a_prime",
                        self.advices[0],
                        offset,
                        &gate_cells.a_prime,
                    )?;

                    // advices[1]
                    copy(
                        &mut region,
                        || "b3_c_prime",
                        self.advices[1],
                        offset,
                        &gate_cells.b3_c_prime,
                    )?;

                    // advices[2]
                    copy(
                        &mut region,
                        || "e1_f_prime",
                        self.advices[2],
                        offset,
                        &gate_cells.e1_f_prime,
                    )?;

                    // advices[3]
                    copy(
                        &mut region,
                        || "g1_g2_prime",
                        self.advices[3],
                        offset,
                        &gate_cells.g1_g2_prime,
                    )?;

                    // advices[4]
                    copy(&mut region, || "a", self.advices[4], offset, &gate_cells.a)?;

                    // advices[5]
                    copy(&mut region, || "b", self.advices[5], offset, &gate_cells.b)?;

                    // advices[6]
                    region.assign_advice(
                        || "b_2",
                        self.advices[6],
                        offset,
                        || gate_cells.b_2.ok_or(Error::SynthesisError),
                    )?;

                    // advices[7]
                    copy(
                        &mut region,
                        || "b_3",
                        self.advices[7],
                        offset,
                        &gate_cells.b_3,
                    )?;

                    // advices[8]
                    copy(&mut region, || "c", self.advices[8], offset, &gate_cells.c)?;

                    // advices[9]
                    copy(&mut region, || "d", self.advices[9], offset, &gate_cells.d)?;
                }

                // Offset 1
                {
                    let offset = 1;

                    // advices[0]
                    region.assign_advice(
                        || "d_1",
                        self.advices[0],
                        offset,
                        || gate_cells.d_1.ok_or(Error::SynthesisError),
                    )?;

                    // advices[1]
                    copy(
                        &mut region,
                        || "d_2",
                        self.advices[1],
                        offset,
                        &gate_cells.d_2,
                    )?;

                    // advices[2]
                    copy(
                        &mut region,
                        || "z1_d",
                        self.advices[2],
                        offset,
                        &gate_cells.z1_d,
                    )?;

                    // advices[3]
                    copy(&mut region, || "e", self.advices[3], offset, &gate_cells.e)?;

                    // advices[4]
                    copy(
                        &mut region,
                        || "e_0",
                        self.advices[4],
                        offset,
                        &gate_cells.e_0,
                    )?;

                    // advices[5]
                    copy(
                        &mut region,
                        || "e_1",
                        self.advices[5],
                        offset,
                        &gate_cells.e_1,
                    )?;

                    // advices[6]
                    copy(&mut region, || "f", self.advices[6], offset, &gate_cells.f)?;

                    // advices[7]
                    copy(&mut region, || "g", self.advices[7], offset, &gate_cells.g)?;

                    // advices[8]
                    copy(
                        &mut region,
                        || "g_1",
                        self.advices[8],
                        offset,
                        &gate_cells.g_1,
                    )?;

                    // advices[9]
                    copy(&mut region, || "h", self.advices[9], offset, &gate_cells.h)?;
                }

                // Offset 2
                {
                    let offset = 2;

                    // advices[0]
                    copy(
                        &mut region,
                        || "h_0",
                        self.advices[0],
                        offset,
                        &gate_cells.h_0,
                    )?;

                    // advices[1]
                    region.assign_advice(
                        || "h_1",
                        self.advices[1],
                        offset,
                        || gate_cells.h_1.ok_or(Error::SynthesisError),
                    )?;

                    // advices[2]
                    copy(
                        &mut region,
                        || "gd_x",
                        self.advices[2],
                        offset,
                        &gate_cells.gd_x,
                    )?;

                    // advices[3]
                    copy(
                        &mut region,
                        || "pkd_x",
                        self.advices[3],
                        offset,
                        &gate_cells.pkd_x,
                    )?;

                    // advices[4]
                    copy(
                        &mut region,
                        || "value",
                        self.advices[4],
                        offset,
                        &gate_cells.value,
                    )?;

                    // advices[5]
                    copy(
                        &mut region,
                        || "b_0",
                        self.advices[5],
                        offset,
                        &gate_cells.b_0,
                    )?;

                    // advices[6]
                    region.assign_advice(
                        || "b_1",
                        self.advices[6],
                        offset,
                        || gate_cells.b_1.ok_or(Error::SynthesisError),
                    )?;

                    // advices[7]
                    region.assign_advice(
                        || "d_0",
                        self.advices[7],
                        offset,
                        || gate_cells.d_0.ok_or(Error::SynthesisError),
                    )?;

                    // advices[8]
                    region.assign_advice(
                        || "g_0",
                        self.advices[8],
                        offset,
                        || gate_cells.g_0.ok_or(Error::SynthesisError),
                    )?;

                    // advices[9]
                    copy(
                        &mut region,
                        || "z1_g",
                        self.advices[9],
                        offset,
                        &gate_cells.z1_g,
                    )?;
                }

                // Offset 3
                {
                    let offset = 3;

                    // advices[0]
                    copy(
                        &mut region,
                        || "a_prime_decomposition",
                        self.advices[0],
                        offset,
                        &gate_cells.a_prime_decomposition,
                    )?;

                    // advices[1]
                    copy(
                        &mut region,
                        || "b3_c_prime_decomposition",
                        self.advices[1],
                        offset,
                        &gate_cells.b3_c_prime_decomposition,
                    )?;

                    // advices[2]
                    copy(
                        &mut region,
                        || "e1_f_prime_decomposition",
                        self.advices[2],
                        offset,
                        &gate_cells.e1_f_prime_decomposition,
                    )?;

                    // advices[3]
                    copy(
                        &mut region,
                        || "g1_g2_prime_decomposition",
                        self.advices[3],
                        offset,
                        &gate_cells.g1_g2_prime_decomposition,
                    )?;

                    // advices[4]
                    copy(
                        &mut region,
                        || "z13_a",
                        self.advices[4],
                        offset,
                        &gate_cells.z13_a,
                    )?;

                    // advices[5]
                    copy(
                        &mut region,
                        || "z13_c",
                        self.advices[5],
                        offset,
                        &gate_cells.z13_c,
                    )?;

                    // advices[6]
                    copy(
                        &mut region,
                        || "z13_f",
                        self.advices[6],
                        offset,
                        &gate_cells.z13_f,
                    )?;

                    // advices[7]
                    copy(
                        &mut region,
                        || "z13_g",
                        self.advices[7],
                        offset,
                        &gate_cells.z13_g,
                    )?;

                    // advices[8]
                    copy(
                        &mut region,
                        || "psi",
                        self.advices[8],
                        offset,
                        &gate_cells.psi,
                    )?;

                    // advices[9]
                    copy(
                        &mut region,
                        || "rho",
                        self.advices[9],
                        offset,
                        &gate_cells.rho,
                    )?;
                }

                Ok(())
            },
        )
    }
}

fn point_repr(point: Option<pallas::Affine>) -> (Option<pallas::Base>, Option<pallas::Base>) {
    let x: Option<pallas::Base> = point.map(|point| *point.coordinates().unwrap().x());
    let y: Option<pallas::Base> = point.map(|point| {
        let last_byte: u8 = point.to_bytes().as_ref()[31];
        let last_bit = (last_byte >> 7) % 2;
        pallas::Base::from_u64(last_bit as u64)
    });

    (x, y)
}

struct GateCells {
    a: CellValue<pallas::Base>,
    b: CellValue<pallas::Base>,
    b_0: CellValue<pallas::Base>,
    b_1: Option<pallas::Base>,
    b_2: Option<pallas::Base>,
    b_3: CellValue<pallas::Base>,
    c: CellValue<pallas::Base>,
    d: CellValue<pallas::Base>,
    d_0: Option<pallas::Base>,
    d_1: Option<pallas::Base>,
    d_2: CellValue<pallas::Base>,
    z1_d: CellValue<pallas::Base>,
    e: CellValue<pallas::Base>,
    e_0: CellValue<pallas::Base>,
    e_1: CellValue<pallas::Base>,
    f: CellValue<pallas::Base>,
    g: CellValue<pallas::Base>,
    g_0: Option<pallas::Base>,
    g_1: CellValue<pallas::Base>,
    z1_g: CellValue<pallas::Base>,
    h: CellValue<pallas::Base>,
    h_0: CellValue<pallas::Base>,
    h_1: Option<pallas::Base>,
    gd_x: CellValue<pallas::Base>,
    pkd_x: CellValue<pallas::Base>,
    value: CellValue<pallas::Base>,
    rho: CellValue<pallas::Base>,
    psi: CellValue<pallas::Base>,
    a_prime: CellValue<pallas::Base>,
    b3_c_prime: CellValue<pallas::Base>,
    e1_f_prime: CellValue<pallas::Base>,
    g1_g2_prime: CellValue<pallas::Base>,
    a_prime_decomposition: CellValue<pallas::Base>,
    b3_c_prime_decomposition: CellValue<pallas::Base>,
    e1_f_prime_decomposition: CellValue<pallas::Base>,
    g1_g2_prime_decomposition: CellValue<pallas::Base>,
    z13_a: CellValue<pallas::Base>,
    z13_c: CellValue<pallas::Base>,
    z13_f: CellValue<pallas::Base>,
    z13_g: CellValue<pallas::Base>,
}

#[cfg(test)]
mod tests {
    use super::NoteCommitConfig;
    use crate::{
        circuit::gadget::{
            ecc::{
                chip::{EccChip, EccConfig},
                Point,
            },
            sinsemilla::chip::SinsemillaChip,
            utilities::{
                lookup_range_check::LookupRangeCheckConfig, CellValue, UtilitiesInstructions,
            },
        },
        constants::T_Q,
    };

    use ff::Field;
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{
        arithmetic::{CurveAffine, FieldExt},
        pallas,
    };

    use rand::{rngs::OsRng, RngCore};
    use std::convert::TryInto;

    #[test]
    fn note_commit() {
        #[derive(Default)]
        struct MyCircuit {
            gd_x: Option<pallas::Base>,
            pkd_x: Option<pallas::Base>,
            rho: Option<pallas::Base>,
            psi: Option<pallas::Base>,
        }

        impl UtilitiesInstructions<pallas::Base> for MyCircuit {
            type Var = CellValue<pallas::Base>;
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = (NoteCommitConfig, EccConfig);
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
                let advices = [
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                ];

                // Shared fixed column for loading constants.
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                for advice in advices.iter() {
                    meta.enable_equality((*advice).into());
                }

                let table_idx = meta.fixed_column();
                let lookup = (table_idx, meta.fixed_column(), meta.fixed_column());
                let lagrange_coeffs = [
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                ];

                let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);
                let sinsemilla_config = SinsemillaChip::configure(
                    meta,
                    advices[..5].try_into().unwrap(),
                    advices[2],
                    lagrange_coeffs[0],
                    lookup,
                    range_check.clone(),
                );
                let note_commit_config =
                    NoteCommitConfig::configure(meta, advices, sinsemilla_config);

                let ecc_config = EccChip::configure(meta, advices, lagrange_coeffs, range_check);

                (note_commit_config, ecc_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let (note_commit_config, ecc_config) = config;

                // Load the Sinsemilla generator lookup table used by the whole circuit.
                SinsemillaChip::load(note_commit_config.sinsemilla_config.clone(), &mut layouter)?;

                // Construct a Sinsemilla chip
                let sinsemilla_chip =
                    SinsemillaChip::construct(note_commit_config.sinsemilla_config.clone());

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(ecc_config);

                // Witness g_d
                let g_d = {
                    let g_d = self.gd_x.map(|x| {
                        // Calculate y = (x^3 + 5).sqrt()
                        let y = (x.square() * x + pallas::Affine::b()).sqrt().unwrap();
                        pallas::Affine::from_xy(x, y).unwrap()
                    });

                    Point::new(ecc_chip.clone(), layouter.namespace(|| "witness g_d"), g_d)?
                };

                // Witness pk_d
                let pk_d = {
                    let pk_d = self.pkd_x.map(|x| {
                        // Calculate y = (x^3 + 5).sqrt()
                        let y = (x.square() * x + pallas::Affine::b()).sqrt().unwrap();
                        pallas::Affine::from_xy(x, y).unwrap()
                    });

                    Point::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "witness pk_d"),
                        pk_d,
                    )?
                };

                // Witness a random non-negative u64 note value
                // A note value cannot be negative.
                let value = {
                    let mut rng = OsRng;
                    let value = pallas::Base::from_u64(rng.next_u64());
                    self.load_private(
                        layouter.namespace(|| "witness value"),
                        note_commit_config.advices[0],
                        Some(value),
                    )?
                };

                // Witness rho
                let rho = self.load_private(
                    layouter.namespace(|| "witness rho"),
                    note_commit_config.advices[0],
                    self.rho,
                )?;

                // Witness psi
                let psi = self.load_private(
                    layouter.namespace(|| "witness psi"),
                    note_commit_config.advices[0],
                    self.psi,
                )?;

                let rcm = Some(pallas::Scalar::rand());

                let _cm = note_commit_config.assign_region(
                    layouter.namespace(|| "Hash NoteCommit pieces"),
                    sinsemilla_chip,
                    ecc_chip,
                    g_d.inner(),
                    pk_d.inner(),
                    value,
                    rho,
                    psi,
                    rcm,
                )?;

                Ok(())
            }
        }

        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        // Test different values of `ak`, `nk`
        let circuits = [
            // `gd_x` = -1, `pkd_x` = -1 (these have to be x-coordinates of curve points)
            // `rho` = 0, `psi` = 0
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(pallas::Base::zero()),
                psi: Some(pallas::Base::zero()),
            },
            // `rho` = T_Q - 1, `psi` = T_Q - 1
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(pallas::Base::from_u128(T_Q - 1)),
                psi: Some(pallas::Base::from_u128(T_Q - 1)),
            },
            // `rho` = T_Q, `psi` = T_Q
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(pallas::Base::from_u128(T_Q)),
                psi: Some(pallas::Base::from_u128(T_Q)),
            },
            // `rho` = 2^127 - 1, `psi` = 2^127 - 1
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(pallas::Base::from_u128((1 << 127) - 1)),
                psi: Some(pallas::Base::from_u128((1 << 127) - 1)),
            },
            // `rho` = 2^127, `psi` = 2^127
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(pallas::Base::from_u128(1 << 127)),
                psi: Some(pallas::Base::from_u128(1 << 127)),
            },
            // `rho` = 2^254 - 1, `psi` = 2^254 - 1
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(two_pow_254 - pallas::Base::one()),
                psi: Some(two_pow_254 - pallas::Base::one()),
            },
            // `rho` = 2^254, `psi` = 2^254
            MyCircuit {
                gd_x: Some(-pallas::Base::one()),
                pkd_x: Some(-pallas::Base::one()),
                rho: Some(two_pow_254),
                psi: Some(two_pow_254),
            },
        ];

        for circuit in circuits.iter() {
            let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
