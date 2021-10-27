use super::super::{util::*, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment};
use super::{schedule_util::*, MessageScheduleConfig, MessageWord};
use halo2::{arithmetic::FieldExt, circuit::Region, pasta::pallas, plonk::Error};

// A word in subregion 2
// (3, 4, 3, 7, 1, 1, 13)-bit chunks
#[derive(Clone, Debug)]
pub struct Subregion2Word {
    index: usize,
    a: CellValue16,
    b: CellValue16,
    c: CellValue16,
    d: CellValue16,
    e: CellValue16,
    f: CellValue16,
    g: CellValue16,
    spread_d: CellValue32,
    spread_g: CellValue32,
}

impl MessageScheduleConfig {
    // W_[14..49]
    pub fn assign_subregion2(
        &self,
        region: &mut Region<'_, pallas::Base>,
        lower_sigma_0_output: Vec<(CellValue16, CellValue16)>,
        w: &mut Vec<MessageWord>,
        w_halves: &mut Vec<(CellValue16, CellValue16)>,
    ) -> Result<Vec<(CellValue16, CellValue16)>, Error> {
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];
        let a_7 = self.extras[3];
        let a_8 = self.extras[4];
        let a_9 = self.extras[5];

        let mut lower_sigma_0_v2_results =
            Vec::<(CellValue16, CellValue16)>::with_capacity(SUBREGION_2_LEN);
        let mut lower_sigma_1_v2_results =
            Vec::<(CellValue16, CellValue16)>::with_capacity(SUBREGION_2_LEN);

        // Closure to compose new word
        // W_i = sigma_1(W_{i - 2}) + W_{i - 7} + sigma_0(W_{i - 15}) + W_{i - 16}
        // e.g. W_16 = sigma_1(W_14) + W_9 + sigma_0(W_1) + W_0

        // sigma_0(W_[1..14]) will be used to get the new W_[16..29]
        // sigma_0_v2(W_[14..36]) will be used to get the new W_[29..51]
        // sigma_1_v2(W_[14..49]) will be used to get the W_[16..51]
        // The lowest-index words involved will be W_[0..13]
        let mut new_word = |idx: usize,
                            sigma_0_output: &(CellValue16, CellValue16)|
         -> Result<Vec<(CellValue16, CellValue16)>, Error> {
            // Decompose word into (3, 4, 3, 7, 1, 1, 13)-bit chunks
            let subregion2_word =
                self.decompose_subregion2_word(region, w[idx].value_u32(), idx)?;

            // sigma_0 v2 and sigma_1 v2 on subregion2_word
            lower_sigma_0_v2_results.push(self.lower_sigma_0_v2(region, subregion2_word.clone())?);
            lower_sigma_1_v2_results.push(self.lower_sigma_1_v2(region, subregion2_word)?);

            let new_word_idx = idx + 2;

            // Copy sigma_0(W_{i - 15}) output from Subregion 1
            sigma_0_output.0.copy_advice(
                || format!("sigma_0(W_{})_lo", new_word_idx - 15),
                region,
                a_6,
                get_word_row(new_word_idx - 16),
            )?;
            sigma_0_output.1.copy_advice(
                || format!("sigma_0(W_{})_hi", new_word_idx - 15),
                region,
                a_6,
                get_word_row(new_word_idx - 16) + 1,
            )?;

            // Copy sigma_1(W_{i - 2})
            lower_sigma_1_v2_results[new_word_idx - 16].0.copy_advice(
                || format!("sigma_1(W_{})_lo", new_word_idx - 2),
                region,
                a_7,
                get_word_row(new_word_idx - 16),
            )?;
            lower_sigma_1_v2_results[new_word_idx - 16].1.copy_advice(
                || format!("sigma_1(W_{})_hi", new_word_idx - 2),
                region,
                a_7,
                get_word_row(new_word_idx - 16) + 1,
            )?;

            // Copy W_{i - 7}
            w_halves[new_word_idx - 7].0.copy_advice(
                || format!("W_{}_lo", new_word_idx - 7),
                region,
                a_8,
                get_word_row(new_word_idx - 16),
            )?;
            w_halves[new_word_idx - 7].1.copy_advice(
                || format!("W_{}_hi", new_word_idx - 7),
                region,
                a_8,
                get_word_row(new_word_idx - 16) + 1,
            )?;

            // Calculate W_i, carry_i
            let (word, carry) = sum_with_carry(vec![
                (
                    lower_sigma_1_v2_results[new_word_idx - 16].0.value_u16(),
                    lower_sigma_1_v2_results[new_word_idx - 16].1.value_u16(),
                ),
                (
                    w_halves[new_word_idx - 7].0.value_u16(),
                    w_halves[new_word_idx - 7].1.value_u16(),
                ),
                (sigma_0_output.0.value_u16(), sigma_0_output.1.value_u16()),
                (
                    w_halves[new_word_idx - 16].0.value_u16(),
                    w_halves[new_word_idx - 16].1.value_u16(),
                ),
            ]);

            // Assign W_i, carry_i
            region.assign_advice(
                || format!("W_{}", new_word_idx),
                a_5,
                get_word_row(new_word_idx - 16) + 1,
                || {
                    word.map(|word| pallas::Base::from_u64(word as u64))
                        .ok_or(Error::SynthesisError)
                },
            )?;
            region.assign_advice(
                || format!("carry_{}", new_word_idx),
                a_9,
                get_word_row(new_word_idx - 16) + 1,
                || {
                    carry
                        .map(|carry| pallas::Base::from_u64(carry as u64))
                        .ok_or(Error::SynthesisError)
                },
            )?;
            let (word, halves) = self.assign_word_and_halves(region, word, new_word_idx)?;
            w.push(MessageWord(word));
            w_halves.push(halves);

            Ok(lower_sigma_0_v2_results.clone())
        };

        let mut tmp_lower_sigma_0_v2_results: Vec<(CellValue16, CellValue16)> =
            Vec::with_capacity(SUBREGION_2_LEN);

        // Use up all the output from Subregion 1 lower_sigma_0
        for i in 14..27 {
            tmp_lower_sigma_0_v2_results = new_word(i, &lower_sigma_0_output[i - 14])?;
        }

        for i in 27..49 {
            tmp_lower_sigma_0_v2_results =
                new_word(i, &tmp_lower_sigma_0_v2_results[i + 2 - 15 - 14])?;
        }

        // Return lower_sigma_0_v2 output for W_[36..49]
        Ok(lower_sigma_0_v2_results.split_off(36 - 14))
    }

    fn decompose_subregion2_word(
        &self,
        region: &mut Region<'_, pallas::Base>,
        word: Option<u32>,
        index: usize,
    ) -> Result<Subregion2Word, Error> {
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let pieces = word.map(|word| chop_u32(word, &[3, 4, 3, 7, 1, 1, 13]));
        let pieces = transpose_option_vec(pieces, 7);

        // Assign `a` (3-bit piece)
        let a = CellValue16::assign_unchecked(
            region,
            || "a",
            a_3,
            row - 1,
            pieces[0].map(|piece| piece as u16),
        )?;

        // Assign `b` (4-bit piece) lookup
        let spread_b = SpreadWord::opt_new(pieces[1].map(|value| value as u16));
        let spread_b = SpreadVar::with_lookup(region, &self.lookup, row + 1, spread_b)?;

        // Assign `c` (3-bit piece)
        let c = CellValue16::assign_unchecked(
            region,
            || "c",
            a_4,
            row - 1,
            pieces[2].map(|piece| piece as u16),
        )?;

        // Assign `d` (7-bit piece) lookup
        let spread_d = SpreadWord::opt_new(pieces[3].map(|value| value as u16));
        let spread_d = SpreadVar::with_lookup(region, &self.lookup, row, spread_d)?;

        // Assign `e` (1-bit piece)
        let e = CellValue16::assign_unchecked(
            region,
            || "e",
            a_3,
            row + 1,
            pieces[4].map(|piece| piece as u16),
        )?;

        // Assign `f` (1-bit piece)
        let f = CellValue16::assign_unchecked(
            region,
            || "f",
            a_4,
            row + 1,
            pieces[5].map(|piece| piece as u16),
        )?;

        // Assign `g` (13-bit piece) lookup
        let spread_g = SpreadWord::opt_new(pieces[6].map(|value| value as u16));
        let spread_g = SpreadVar::with_lookup(region, &self.lookup, row - 1, spread_g)?;

        Ok(Subregion2Word {
            index,
            a,
            b: spread_b.dense,
            c,
            d: spread_d.dense,
            e,
            f,
            g: spread_g.dense,
            spread_d: spread_d.spread,
            spread_g: spread_g.spread,
        })
    }

    #[allow(clippy::type_complexity)]
    fn assign_lower_sigma_v2_pieces(
        &self,
        region: &mut Region<'_, pallas::Base>,
        row: usize,
        subregion2_word: Subregion2Word,
    ) -> Result<[Option<u32>; 8], Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];
        let a_7 = self.extras[3];

        // Assign `a` and copy constraint
        subregion2_word
            .a
            .copy_advice(|| "a", region, a_3, row + 1)?;

        // Witness `spread_a`
        let spread_a = subregion2_word.a.value_u16().map(interleave_u16_with_zeros);
        region.assign_advice(
            || "spread_a",
            a_4,
            row + 1,
            || {
                spread_a
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Split `b` (2-bit chunk) into `b_hi` and `b_lo`
        let b = subregion2_word.b.value_u16().map(bisect_four_bit);
        let spread_b_lo = b.map(|b| interleave_u16_with_zeros(b.0 as u16));
        let spread_b_hi = b.map(|b| interleave_u16_with_zeros(b.1 as u16));

        // Assign `b_hi`, `spread_b_hi`, `b_lo`, `spread_b_lo`
        region.assign_advice(
            || "b_lo",
            a_3,
            row - 1,
            || {
                b.map(|(b_lo, _)| pallas::Base::from_u64(b_lo as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_lo",
            a_4,
            row - 1,
            || {
                spread_b_lo
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "b_hi",
            a_5,
            row - 1,
            || {
                b.map(|(_, b_hi)| pallas::Base::from_u64(b_hi as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_hi",
            a_6,
            row - 1,
            || {
                spread_b_hi
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `b` and copy constraint
        subregion2_word.b.copy_advice(|| "b", region, a_6, row)?;

        // Assign `c` and copy constraint
        subregion2_word
            .c
            .copy_advice(|| "c", region, a_5, row + 1)?;

        // Witness `spread_c`
        let spread_c = subregion2_word.c.value_u16().map(|value| {
            let spread = interleave_u16_with_zeros(value);
            spread as u32
        });
        region.assign_advice(
            || "spread_c",
            a_6,
            row + 1,
            || {
                spread_c
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `spread_d` and copy constraint
        subregion2_word
            .spread_d
            .copy_advice(|| "spread_d", region, a_4, row)?;

        // Assign `e` and copy constraint
        subregion2_word.e.copy_advice(|| "e", region, a_7, row)?;

        // Assign `f` and copy constraint
        subregion2_word
            .f
            .copy_advice(|| "f", region, a_7, row + 1)?;

        // Assign `spread_g` and copy constraint
        subregion2_word
            .spread_g
            .copy_advice(|| "spread_g", region, a_5, row)?;

        Ok([
            spread_a,
            spread_b_lo,
            spread_b_hi,
            spread_c,
            subregion2_word.spread_d.value_u32(),
            subregion2_word.e.value_u16().map(|v| v as u32),
            subregion2_word.f.value_u16().map(|v| v as u32),
            subregion2_word.spread_g.value_u32(),
        ])
    }

    fn lower_sigma_0_v2(
        &self,
        region: &mut Region<'_, pallas::Base>,
        subregion2_word: Subregion2Word,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let a_3 = self.extras[0];
        let row = get_word_row(subregion2_word.index) + 3;

        // Get spread pieces
        let pieces = self.assign_lower_sigma_v2_pieces(region, row, subregion2_word)?;

        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let (r_0_even, r_0_odd, r_1_even, r_1_odd) = if pieces[0].is_some() {
            let pieces = pieces
                .iter()
                .map(|piece| piece.unwrap() as u64)
                .collect::<Vec<_>>();
            let [spread_a, spread_b_lo, spread_b_hi, spread_c, spread_d, e, f, spread_g] = [
                pieces[0], pieces[1], pieces[2], pieces[3], pieces[4], pieces[5], pieces[6],
                pieces[7],
            ];
            let xor_0 = spread_b_lo
                + (1 << 4) * spread_b_hi
                + (1 << 8) * spread_c
                + (1 << 14) * spread_d
                + (1 << 28) * e
                + (1 << 30) * f
                + (1 << 32) * spread_g;
            let xor_1 = spread_c
                + (1 << 6) * spread_d
                + (1 << 20) * e
                + (1 << 22) * f
                + (1 << 24) * spread_g
                + (1 << 50) * spread_a
                + (1 << 56) * spread_b_lo
                + (1 << 60) * spread_b_hi;
            let xor_2 = f
                + (1 << 2) * spread_g
                + (1 << 28) * spread_a
                + (1 << 34) * spread_b_lo
                + (1 << 38) * spread_b_hi
                + (1 << 42) * spread_c
                + (1 << 48) * spread_d
                + (1 << 62) * e;

            let r = xor_0 + xor_1 + xor_2;
            let r_pieces = chop_u64(r, &[32, 32]); // r_0, r_1
            let (r_0_even, r_0_odd) = get_even_and_odd_bits_u32(r_pieces[0] as u32);
            let (r_1_even, r_1_odd) = get_even_and_odd_bits_u32(r_pieces[1] as u32);
            (Some(r_0_even), Some(r_0_odd), Some(r_1_even), Some(r_1_odd))
        } else {
            (None, None, None, None)
        };

        self.assign_sigma_outputs(
            region,
            &self.lookup,
            a_3,
            row,
            r_0_even,
            r_0_odd,
            r_1_even,
            r_1_odd,
        )
    }

    fn lower_sigma_1_v2(
        &self,
        region: &mut Region<'_, pallas::Base>,
        subregion2_word: Subregion2Word,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let a_3 = self.extras[0];
        let row = get_word_row(subregion2_word.index) + SIGMA_0_V2_ROWS + 3;

        let pieces = self.assign_lower_sigma_v2_pieces(region, row, subregion2_word)?;

        // (3, 4, 3, 7, 1, 1, 13)

        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let (r_0_even, r_0_odd, r_1_even, r_1_odd) = if pieces[0].is_some() {
            let pieces = pieces
                .iter()
                .map(|piece| piece.unwrap() as u64)
                .collect::<Vec<_>>();
            let [spread_a, spread_b_lo, spread_b_hi, spread_c, spread_d, e, f, spread_g] = [
                pieces[0], pieces[1], pieces[2], pieces[3], pieces[4], pieces[5], pieces[6],
                pieces[7],
            ];
            let xor_0 = spread_d + (1 << 14) * e + (1 << 16) * f + (1 << 18) * spread_g;
            let xor_1 = e
                + (1 << 2) * f
                + (1 << 4) * spread_g
                + (1 << 30) * spread_a
                + (1 << 36) * spread_b_lo
                + (1 << 40) * spread_b_hi
                + (1 << 44) * spread_c
                + (1 << 50) * spread_d;
            let xor_2 = spread_g
                + (1 << 26) * spread_a
                + (1 << 32) * spread_b_lo
                + (1 << 36) * spread_b_hi
                + (1 << 40) * spread_c
                + (1 << 46) * spread_d
                + (1 << 60) * e
                + (1 << 62) * f;

            let r = xor_0 + xor_1 + xor_2;
            let r_pieces = chop_u64(r, &[32, 32]); // r_0, r_1
            let (r_0_even, r_0_odd) = get_even_and_odd_bits_u32(r_pieces[0] as u32);
            let (r_1_even, r_1_odd) = get_even_and_odd_bits_u32(r_pieces[1] as u32);
            (Some(r_0_even), Some(r_0_odd), Some(r_1_even), Some(r_1_odd))
        } else {
            (None, None, None, None)
        };

        self.assign_sigma_outputs(
            region,
            &self.lookup,
            a_3,
            row,
            r_0_even,
            r_0_odd,
            r_1_even,
            r_1_odd,
        )
    }
}
