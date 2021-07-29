use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use crate::{
    circuit::gadget::{
        ecc::{chip::EccChip, X},
        utilities::{bitrange_subset, bool_check, copy, CellValue, Var},
    },
    constants::T_P,
};

use super::{
    chip::{SinsemillaChip, SinsemillaCommitDomains, SinsemillaConfig},
    CommitDomain, Message, MessagePiece,
};

#[derive(Clone, Debug)]
pub struct CommitIvkConfig {
    q_commit_ivk: Selector,
    advices: [Column<Advice>; 10],
    sinsemilla_config: SinsemillaConfig,
}

impl CommitIvkConfig {
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        sinsemilla_config: SinsemillaConfig,
    ) -> Self {
        let q_commit_ivk = meta.selector();

        let config = Self {
            q_commit_ivk,
            advices,
            sinsemilla_config,
        };

        // <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
        // We need to hash `ak || nk` where each of `ak`, `nk` is a field element (255 bits).
        //
        // a = bits 0..=249 of `ak`
        // b = b_0||b_1||b_2`
        //   = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=4 of  `nk`)
        // c = bits 5..=244 of `nk`
        // d = d_0||d_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
        //
        // `a`, `b`, `c`, `d` have been constrained by the Sinsemilla hash to be:
        //   - a: 250 bits,
        //   - b: 10 bits,
        //   - c: 240 bits,
        //   - d: 10 bits
        //
        /*
            The pieces are laid out in this configuration:

            |  A_0  |  A_1  |  A_2  |  A_3  |  A_4  |  A_5  |  A_6  |    A_7    |       A_8      | q_commit_ivk |
            -----------------------------------------------------------------------------------------------------
            |   ak  |   a   |   b   |  b_0  |  b_1  |  b_2  | z13_a |  a_prime  |   z13_a_prime  |       1      |
            |   nk  |   c   |   d   |  d_0  |  d_1  |       | z13_c | b2_c_prime| z14_b2_c_prime |       0      |

        */
        meta.create_gate("CommitIvk canonicity check", |meta| {
            let q_commit_ivk = meta.query_selector(config.q_commit_ivk);

            // Useful constants
            let two_pow_4 = pallas::Base::from_u64(1 << 4);
            let two_pow_5 = pallas::Base::from_u64(1 << 5);
            let two_pow_9 = two_pow_4 * two_pow_5;
            let two_pow_250 = pallas::Base::from_u128(1 << 125).square();
            let two_pow_254 = two_pow_250 * two_pow_4;

            let ak = meta.query_advice(config.advices[0], Rotation::cur());
            let nk = meta.query_advice(config.advices[0], Rotation::next());

            // `a` is constrained by the Sinsemilla hash to be 250 bits.
            let a = meta.query_advice(config.advices[1], Rotation::cur());
            // `b` is constrained by the Sinsemilla hash to be 10 bits.
            let b_whole = meta.query_advice(config.advices[2], Rotation::cur());
            // `c` is constrained by the Sinsemilla hash to be 240 bits.
            let c = meta.query_advice(config.advices[1], Rotation::next());
            // `d` is constrained by the Sinsemilla hash to be 10 bits.
            let d_whole = meta.query_advice(config.advices[2], Rotation::next());

            // b = b_0||b_1||b_2`
            //   = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=4 of  `nk`)
            //
            // b_0 has been constrained outside this gate to be a four-bit value.
            let b_0 = meta.query_advice(config.advices[3], Rotation::cur());
            // This gate constrains b_1 to be a one-bit value.
            let b_1 = meta.query_advice(config.advices[4], Rotation::cur());
            // b_2 has been constrained outside this gate to be a five-bit value.
            let b_2 = meta.query_advice(config.advices[5], Rotation::cur());
            // Check that b_whole is consistent with the witnessed subpieces.
            let b_decomposition_check =
                b_whole - (b_0.clone() + b_1.clone() * two_pow_4 + b_2.clone() * two_pow_5);

            // d = d_0||d_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
            //
            // d_0 has been constrained outside this gate to be a nine-bit value.
            let d_0 = meta.query_advice(config.advices[3], Rotation::next());
            // This gate constrains d_1 to be a one-bit value.
            let d_1 = meta.query_advice(config.advices[4], Rotation::next());
            // Check that d_whole is consistent with the witnessed subpieces.
            let d_decomposition_check = d_whole - (d_0.clone() + d_1.clone() * two_pow_9);

            // Check `b_1` is a single-bit value
            let b1_bool_check = bool_check(b_1.clone());

            // Check `d_1` is a single-bit value
            let d1_bool_check = bool_check(d_1.clone());

            // Check that ak = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let ak_decomposition_check =
                a.clone() + b_0.clone() * two_pow_250 + b_1.clone() * two_pow_254 - ak;

            // Check that nk = b_2 (5 bits) || c (240 bits) || d_0 (9 bits) || d_1 (1 bit)
            let nk_decomposition_check = {
                let two_pow_245 = pallas::Base::from_u64(1 << 49).pow(&[5, 0, 0, 0]);

                b_2.clone()
                    + c.clone() * two_pow_5
                    + d_0.clone() * two_pow_245
                    + d_1.clone() * two_pow_254
                    - nk
            };

            // ak = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            // The `ak` canonicity checks are enforced if and only if `b_1` = 1.
            let ak_canonicity_checks = {
                // b_1 = 1 => b_0 = 0
                let b0_canon_check = b_1.clone() * b_0;

                // z13_a is the 13th running sum output by the 10-bit Sinsemilla decomposition of `a`.
                // b_1 = 1 => z13_a = 0
                let z13_a_check = {
                    let z13_a = meta.query_advice(config.advices[6], Rotation::cur());
                    b_1.clone() * z13_a
                };

                // Check that a_prime = a + 2^130 - t_P.
                // This is checked regardless of the value of b_1.
                let a_prime_check = {
                    let a_prime = meta.query_advice(config.advices[7], Rotation::cur());
                    let two_pow_130 =
                        Expression::Constant(pallas::Base::from_u128(1 << 65).square());
                    let t_p = Expression::Constant(pallas::Base::from_u128(T_P));
                    a + two_pow_130 - t_p - a_prime
                };

                // Check that the running sum output by the 130-bit little-endian decomposition of
                // `a_prime` is zero.
                let z13_a_prime = {
                    let z13_a_prime = meta.query_advice(config.advices[8], Rotation::cur());
                    b_1 * z13_a_prime
                };

                std::iter::empty()
                    .chain(Some(("b0_canon_check", b0_canon_check)))
                    .chain(Some(("z13_a_check", z13_a_check)))
                    .chain(Some(("a_prime_check", a_prime_check)))
                    .chain(Some(("z13_a_prime", z13_a_prime)))
            };

            // nk = b_2 (5 bits) || c (240 bits) || d_0 (9 bits) || d_1 (1 bit)
            // The `nk` canonicity checks are enforced if and only if `d_1` = 1.
            let nk_canonicity_checks = {
                // d_1 = 1 => d_0 = 0
                let c0_canon_check = d_1.clone() * d_0;

                // d_1 = 1 => z13_c = 0, where z13_c is the 13th running sum
                // output by the 10-bit Sinsemilla decomposition of `c`.
                let z13_c_check = {
                    let z13_c = meta.query_advice(config.advices[6], Rotation::next());
                    d_1.clone() * z13_c
                };

                // Check that b2_c_prime = b_2 + c * 2^5 + 2^140 - t_P.
                // This is checked regardless of the value of d_1.
                let b2_c_prime_check = {
                    let two_pow_5 = pallas::Base::from_u64(1 << 5);
                    let two_pow_140 =
                        Expression::Constant(pallas::Base::from_u128(1 << 70).square());
                    let t_p = Expression::Constant(pallas::Base::from_u128(T_P));
                    let b2_c_prime = meta.query_advice(config.advices[7], Rotation::next());
                    b_2 + c * two_pow_5 + two_pow_140 - t_p - b2_c_prime
                };

                // Check that the running sum output by the 140-bit little-
                // endian decomposition of b2_c_prime is zero.
                let z14_b2_c_prime = {
                    let z14_b2_c_prime = meta.query_advice(config.advices[8], Rotation::next());
                    d_1 * z14_b2_c_prime
                };

                std::iter::empty()
                    .chain(Some(("c0_canon_check", c0_canon_check)))
                    .chain(Some(("z13_c_check", z13_c_check)))
                    .chain(Some(("b2_c_prime_check", b2_c_prime_check)))
                    .chain(Some(("z14_b2_c_prime", z14_b2_c_prime)))
            };

            std::iter::empty()
                .chain(Some(("b1_bool_check", b1_bool_check)))
                .chain(Some(("d1_bool_check", d1_bool_check)))
                .chain(Some(("b_decomposition_check", b_decomposition_check)))
                .chain(Some(("d_decomposition_check", d_decomposition_check)))
                .chain(Some(("ak_decomposition_check", ak_decomposition_check)))
                .chain(Some(("nk_decomposition_check", nk_decomposition_check)))
                .chain(ak_canonicity_checks)
                .chain(nk_canonicity_checks)
                .map(move |(name, poly)| (name, q_commit_ivk.clone() * poly))
        });

        config
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    pub(in crate::circuit) fn assign_region(
        &self,
        sinsemilla_chip: SinsemillaChip,
        ecc_chip: EccChip,
        mut layouter: impl Layouter<pallas::Base>,
        ak: CellValue<pallas::Base>,
        nk: CellValue<pallas::Base>,
        rivk: Option<pallas::Scalar>,
    ) -> Result<X<pallas::Affine, EccChip>, Error> {
        // <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
        // We need to hash `ak || nk` where each of `ak`, `nk` is a field element (255 bits).
        //
        // a = bits 0..=249 of `ak`
        // b = b_0||b_1||b_2`
        //   = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=4 of  `nk`)
        // c = bits 5..=244 of `nk`
        // d = d_0||d_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)

        // `a` = bits 0..=249 of `ak`
        let a = {
            let a = ak.value().map(|value| bitrange_subset(value, 0..250));
            MessagePiece::from_field_elem(
                sinsemilla_chip.clone(),
                layouter.namespace(|| "a"),
                a,
                25,
            )?
        };

        // `b = b_0||b_1||b_2`
        //    = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=4 of  `nk`)
        let (b_0, b_1, b_2, b) = {
            let b_0 = ak.value().map(|value| bitrange_subset(value, 250..254));
            let b_1 = ak.value().map(|value| bitrange_subset(value, 254..255));
            let b_2 = nk.value().map(|value| bitrange_subset(value, 0..5));

            let b = b_0.zip(b_1).zip(b_2).map(|((b_0, b_1), b_2)| {
                let b1_shifted = b_1 * pallas::Base::from_u64(1 << 4);
                let b2_shifted = b_2 * pallas::Base::from_u64(1 << 5);
                b_0 + b1_shifted + b2_shifted
            });

            // Constrain b_0 to be 4 bits.
            let b_0 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "b_0 is 4 bits"),
                b_0,
                4,
            )?;
            // Constrain b_2 to be 5 bits.
            let b_2 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "b_2 is 5 bits"),
                b_2,
                5,
            )?;
            // b_1 will be boolean-constrained in the custom gate.

            let b = MessagePiece::from_field_elem(
                sinsemilla_chip.clone(),
                layouter.namespace(|| "b = b_0 || b_1 || b_2"),
                b,
                1,
            )?;

            (b_0, b_1, b_2, b)
        };

        // c = bits 5..=244 of `nk`
        let c = {
            let c = nk.value().map(|value| bitrange_subset(value, 5..245));
            MessagePiece::from_field_elem(
                sinsemilla_chip.clone(),
                layouter.namespace(|| "c"),
                c,
                24,
            )?
        };

        // `d = d_0||d_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
        let (d_0, d_1, d) = {
            let d_0 = nk.value().map(|value| bitrange_subset(value, 245..254));
            let d_1 = nk.value().map(|value| bitrange_subset(value, 254..255));

            let d = d_0
                .zip(d_1)
                .map(|(d_0, d_1)| d_0 + d_1 * pallas::Base::from_u64(1 << 9));

            // Constrain d_0 to be 9 bits.
            let d_0 = self.sinsemilla_config.lookup_config.witness_short_check(
                layouter.namespace(|| "d_0 is 9 bits"),
                d_0,
                9,
            )?;
            // d_1 will be boolean-constrained in the custom gate.

            let d = MessagePiece::from_field_elem(
                sinsemilla_chip.clone(),
                layouter.namespace(|| "d = d_0 || d_1"),
                d,
                1,
            )?;

            (d_0, d_1, d)
        };

        let (ivk, zs) = {
            let message = Message::from_pieces(
                sinsemilla_chip.clone(),
                vec![a.clone(), b.clone(), c.clone(), d.clone()],
            );
            let domain = CommitDomain::new(
                sinsemilla_chip,
                ecc_chip,
                &SinsemillaCommitDomains::CommitIvk,
            );
            domain.short_commit(layouter.namespace(|| "Hash ak||nk"), message, rivk)?
        };

        let z13_a = zs[0][13];
        let z13_c = zs[2][13];

        let (a_prime, z13_a_prime) = self.ak_canonicity(
            layouter.namespace(|| "ak canonicity"),
            a.inner().cell_value(),
        )?;

        let (b2_c_prime, z14_b2_c_prime) = self.nk_canonicity(
            layouter.namespace(|| "nk canonicity"),
            b_2,
            c.inner().cell_value(),
        )?;

        let gate_cells = GateCells {
            a: a.inner().cell_value(),
            b: b.inner().cell_value(),
            c: c.inner().cell_value(),
            d: d.inner().cell_value(),
            ak,
            nk,
            b_0,
            b_1,
            b_2,
            d_0,
            d_1,
            z13_a,
            a_prime,
            z13_a_prime,
            z13_c,
            b2_c_prime,
            z14_b2_c_prime,
        };

        self.assign_gate(
            layouter.namespace(|| "Assign cells used in canonicity gate"),
            gate_cells,
        )?;

        Ok(ivk)
    }

    #[allow(clippy::type_complexity)]
    // Check canonicity of `ak` encoding
    fn ak_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        a: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `ak` = `a (250 bits) || b_0 (4 bits) || b_1 (1 bit)`
        // - b_1 = 1 => b_0 = 0
        // - b_1 = 1 => a < t_P
        //     - (0 ≤ a < 2^130) => z13_a of SinsemillaHash(a) == 0
        //     - 0 ≤ a + 2^130 - t_P < 2^130 (thirteen 10-bit lookups)

        // Decompose the low 130 bits of a_prime = a + 2^130 - t_P, and output
        // the running sum at the end of it. If a_prime < 2^130, the running sum
        // will be 0.
        let a_prime = a.value().map(|a| {
            let two_pow_130 = pallas::Base::from_u128(1u128 << 65).square();
            let t_p = pallas::Base::from_u128(T_P);
            a + two_pow_130 - t_p
        });
        let zs = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 130 bits of (a + 2^130 - t_P)"),
            a_prime,
            13,
            false,
        )?;
        let a_prime = zs[0];
        assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z13_a]

        Ok((a_prime, zs[13]))
    }

    #[allow(clippy::type_complexity)]
    // Check canonicity of `nk` encoding
    fn nk_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        b_2: CellValue<pallas::Base>,
        c: CellValue<pallas::Base>,
    ) -> Result<(CellValue<pallas::Base>, CellValue<pallas::Base>), Error> {
        // `nk` = `b_2 (5 bits) || c (240 bits) || d_0 (9 bits) || d_1 (1 bit)
        // - d_1 = 1 => d_0 = 0
        // - d_1 = 1 => b_2 + c * 2^5 < t_P
        //      - 0 ≤ b_2 + c * 2^5 < 2^140
        //          - b_2 was constrained to be 5 bits.
        //          - z_13 of SinsemillaHash(c) constrains bits 5..=134 to 130 bits
        //          - so b_2 + c * 2^5 is constrained to be 135 bits < 2^140.
        //      - 0 ≤ b_2 + c * 2^5 + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Decompose the low 140 bits of b2_c_prime = b_2 + c * 2^5 + 2^140 - t_P, and output
        // the running sum at the end of it. If b2_c_prime < 2^140, the running sum will be 0.
        let b2_c_prime = b_2.value().zip(c.value()).map(|(b_2, c)| {
            let two_pow_5 = pallas::Base::from_u64(1 << 5);
            let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
            let t_p = pallas::Base::from_u128(T_P);
            b_2 + c * two_pow_5 + two_pow_140 - t_p
        });
        let zs = self.sinsemilla_config.lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (b_2 + c * 2^5 + 2^140 - t_P)"),
            b2_c_prime,
            14,
            false,
        )?;
        let b2_c_prime = zs[0];
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z14]

        Ok((b2_c_prime, zs[14]))
    }

    // Assign cells for the canonicity gate.
    /*
        The pieces are laid out in this configuration:

        |  A_0  |  A_1  |  A_2  |  A_3  |  A_4  |  A_5  |  A_6  |    A_7    |       A_8      | q_commit_ivk |
        -----------------------------------------------------------------------------------------------------
        |   ak  |   a   |   b   |  b_0  |  b_1  |  b_2  | z13_a |  a_prime  |   z13_a_prime  |       1      |
        |   nk  |   c   |   d   |  d_0  |  d_1  |       | z13_c | b2_c_prime| z14_b2_c_prime |       0      |

    */
    fn assign_gate(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        gate_cells: GateCells,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "Assign cells used in canonicity gate",
            |mut region| {
                // Enable selector on offset 0
                self.q_commit_ivk.enable(&mut region, 0)?;

                // Offset 0
                {
                    let offset = 0;
                    // Copy in `ak`
                    copy(
                        &mut region,
                        || "ak",
                        self.advices[0],
                        offset,
                        &gate_cells.ak,
                    )?;

                    // Copy in `a`
                    copy(&mut region, || "a", self.advices[1], offset, &gate_cells.a)?;

                    // Copy in `b`
                    copy(&mut region, || "b", self.advices[2], offset, &gate_cells.b)?;

                    // Copy in `b_0`
                    copy(
                        &mut region,
                        || "b_0",
                        self.advices[3],
                        offset,
                        &gate_cells.b_0,
                    )?;

                    // Witness `b_1`
                    region.assign_advice(
                        || "Witness b_1",
                        self.advices[4],
                        offset,
                        || gate_cells.b_1.ok_or(Error::SynthesisError),
                    )?;

                    // Copy in `b_2`
                    copy(
                        &mut region,
                        || "b_2",
                        self.advices[5],
                        offset,
                        &gate_cells.b_2,
                    )?;

                    // Copy in z13_a
                    copy(
                        &mut region,
                        || "z13_a",
                        self.advices[6],
                        offset,
                        &gate_cells.z13_a,
                    )?;

                    // Copy in a_prime
                    copy(
                        &mut region,
                        || "a_prime",
                        self.advices[7],
                        offset,
                        &gate_cells.a_prime,
                    )?;

                    // Copy in z13_a_prime
                    copy(
                        &mut region,
                        || "z13_a_prime",
                        self.advices[8],
                        offset,
                        &gate_cells.z13_a_prime,
                    )?;
                }

                // Offset 1
                {
                    let offset = 1;

                    // Copy in `nk`
                    copy(
                        &mut region,
                        || "nk",
                        self.advices[0],
                        offset,
                        &gate_cells.nk,
                    )?;

                    // Copy in `c`
                    copy(&mut region, || "c", self.advices[1], offset, &gate_cells.c)?;

                    // Copy in `d`
                    copy(&mut region, || "d", self.advices[2], offset, &gate_cells.d)?;

                    // Copy in `d_0`
                    copy(
                        &mut region,
                        || "d_0",
                        self.advices[3],
                        offset,
                        &gate_cells.d_0,
                    )?;

                    // Witness `d_1`
                    region.assign_advice(
                        || "Witness d_1",
                        self.advices[4],
                        offset,
                        || gate_cells.d_1.ok_or(Error::SynthesisError),
                    )?;

                    // Copy in z13_c
                    copy(
                        &mut region,
                        || "z13_c",
                        self.advices[6],
                        offset,
                        &gate_cells.z13_c,
                    )?;

                    // Copy in b2_c_prime
                    copy(
                        &mut region,
                        || "b2_c_prime",
                        self.advices[7],
                        offset,
                        &gate_cells.b2_c_prime,
                    )?;

                    // Copy in z14_b2_c_prime
                    copy(
                        &mut region,
                        || "z14_b2_c_prime",
                        self.advices[8],
                        offset,
                        &gate_cells.z14_b2_c_prime,
                    )?;
                }

                Ok(())
            },
        )
    }
}

// Cells used in the canonicity gate.
struct GateCells {
    a: CellValue<pallas::Base>,
    b: CellValue<pallas::Base>,
    c: CellValue<pallas::Base>,
    d: CellValue<pallas::Base>,
    ak: CellValue<pallas::Base>,
    nk: CellValue<pallas::Base>,
    b_0: CellValue<pallas::Base>,
    b_1: Option<pallas::Base>,
    b_2: CellValue<pallas::Base>,
    d_0: CellValue<pallas::Base>,
    d_1: Option<pallas::Base>,
    z13_a: CellValue<pallas::Base>,
    a_prime: CellValue<pallas::Base>,
    z13_a_prime: CellValue<pallas::Base>,
    z13_c: CellValue<pallas::Base>,
    b2_c_prime: CellValue<pallas::Base>,
    z14_b2_c_prime: CellValue<pallas::Base>,
}

#[cfg(test)]
mod tests {
    use super::CommitIvkConfig;
    use crate::{
        circuit::gadget::{
            ecc::chip::{EccChip, EccConfig},
            sinsemilla::chip::SinsemillaChip,
            utilities::{
                lookup_range_check::LookupRangeCheckConfig, CellValue, UtilitiesInstructions, Var,
            },
        },
        constants::{COMMIT_IVK_PERSONALIZATION, L_ORCHARD_BASE, T_Q},
        primitives::sinsemilla::CommitDomain,
    };
    use ff::PrimeFieldBits;
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use std::convert::TryInto;

    #[test]
    fn commit_ivk() {
        #[derive(Default)]
        struct MyCircuit {
            ak: Option<pallas::Base>,
            nk: Option<pallas::Base>,
        }

        impl UtilitiesInstructions<pallas::Base> for MyCircuit {
            type Var = CellValue<pallas::Base>;
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = (CommitIvkConfig, EccConfig);
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

                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                for advice in advices.iter() {
                    meta.enable_equality((*advice).into());
                }

                let table_idx = meta.lookup_table_column();
                let lookup = (
                    table_idx,
                    meta.lookup_table_column(),
                    meta.lookup_table_column(),
                );
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

                let commit_ivk_config =
                    CommitIvkConfig::configure(meta, advices, sinsemilla_config);

                let ecc_config = EccChip::configure(meta, advices, lagrange_coeffs, range_check);

                (commit_ivk_config, ecc_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let (commit_ivk_config, ecc_config) = config;

                // Load the Sinsemilla generator lookup table used by the whole circuit.
                SinsemillaChip::load(commit_ivk_config.sinsemilla_config.clone(), &mut layouter)?;

                // Construct a Sinsemilla chip
                let sinsemilla_chip =
                    SinsemillaChip::construct(commit_ivk_config.sinsemilla_config.clone());

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(ecc_config);

                // Witness ak
                let ak = self.load_private(
                    layouter.namespace(|| "load ak"),
                    commit_ivk_config.advices[0],
                    self.ak,
                )?;

                // Witness nk
                let nk = self.load_private(
                    layouter.namespace(|| "load nk"),
                    commit_ivk_config.advices[0],
                    self.nk,
                )?;

                // Use a random scalar for rivk
                let rivk = pallas::Scalar::rand();

                let ivk = commit_ivk_config.assign_region(
                    sinsemilla_chip,
                    ecc_chip,
                    layouter.namespace(|| "CommitIvk"),
                    ak,
                    nk,
                    Some(rivk),
                )?;

                let expected_ivk = {
                    let domain = CommitDomain::new(COMMIT_IVK_PERSONALIZATION);
                    // Hash ak || nk
                    domain
                        .short_commit(
                            std::iter::empty()
                                .chain(
                                    self.ak
                                        .unwrap()
                                        .to_le_bits()
                                        .iter()
                                        .by_val()
                                        .take(L_ORCHARD_BASE),
                                )
                                .chain(
                                    self.nk
                                        .unwrap()
                                        .to_le_bits()
                                        .iter()
                                        .by_val()
                                        .take(L_ORCHARD_BASE),
                                ),
                            &rivk,
                        )
                        .unwrap()
                };

                assert_eq!(expected_ivk, ivk.inner().value().unwrap());

                Ok(())
            }
        }

        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        // Test different values of `ak`, `nk`
        let circuits = [
            // `ak` = 0, `nk` = 0
            MyCircuit {
                ak: Some(pallas::Base::zero()),
                nk: Some(pallas::Base::zero()),
            },
            // `ak` = T_Q - 1, `nk` = T_Q - 1
            MyCircuit {
                ak: Some(pallas::Base::from_u128(T_Q - 1)),
                nk: Some(pallas::Base::from_u128(T_Q - 1)),
            },
            // `ak` = T_Q, `nk` = T_Q
            MyCircuit {
                ak: Some(pallas::Base::from_u128(T_Q)),
                nk: Some(pallas::Base::from_u128(T_Q)),
            },
            // `ak` = 2^127 - 1, `nk` = 2^127 - 1
            MyCircuit {
                ak: Some(pallas::Base::from_u128((1 << 127) - 1)),
                nk: Some(pallas::Base::from_u128((1 << 127) - 1)),
            },
            // `ak` = 2^127, `nk` = 2^127
            MyCircuit {
                ak: Some(pallas::Base::from_u128(1 << 127)),
                nk: Some(pallas::Base::from_u128(1 << 127)),
            },
            // `ak` = 2^254 - 1, `nk` = 2^254 - 1
            MyCircuit {
                ak: Some(two_pow_254 - pallas::Base::one()),
                nk: Some(two_pow_254 - pallas::Base::one()),
            },
            // `ak` = 2^254, `nk` = 2^254
            MyCircuit {
                ak: Some(two_pow_254),
                nk: Some(two_pow_254),
            },
        ];

        for circuit in circuits.iter() {
            let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
