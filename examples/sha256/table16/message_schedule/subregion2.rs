use super::super::{util::*, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment};
use super::{schedule_util::*, MessageScheduleConfig, MessageWord};
use halo2::{arithmetic::FieldExt, circuit::Region, plonk::Error};

// A word in subregion 2
// (3, 4, 3, 7, 1, 1, 13)-bit chunks
#[derive(Clone, Debug)]
pub struct Subregion2Word {
    index: usize,
    a: CellValue32,
    b: CellValue16,
    c: CellValue32,
    d: CellValue16,
    e: CellValue32,
    f: CellValue32,
    g: CellValue16,
    spread_d: CellValue32,
    spread_g: CellValue32,
}

impl MessageScheduleConfig {
    // W_[14..49]
    pub fn assign_subregion2<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
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
                            sigma_0_output: (CellValue16, CellValue16)|
         -> Result<Vec<(CellValue16, CellValue16)>, Error> {
            // Decompose word into (3, 4, 3, 7, 1, 1, 13)-bit chunks
            let subregion2_word = self.decompose_subregion2_word(region, w[idx].value, idx)?;

            // sigma_0 v2 and sigma_1 v2 on subregion2_word
            lower_sigma_0_v2_results.push(self.lower_sigma_0_v2(region, subregion2_word.clone())?);
            lower_sigma_1_v2_results.push(self.lower_sigma_1_v2(region, subregion2_word)?);

            let new_word_idx = idx + 2;

            // Copy sigma_0(W_{i - 15}) output from Subregion 1
            self.assign_and_constrain(
                region,
                || format!("sigma_0(W_{})_lo", new_word_idx - 15),
                a_6,
                get_word_row(new_word_idx - 16),
                sigma_0_output.0,
            )?;
            self.assign_and_constrain(
                region,
                || format!("sigma_0(W_{})_hi", new_word_idx - 15),
                a_6,
                get_word_row(new_word_idx - 16) + 1,
                sigma_0_output.1,
            )?;

            // Copy sigma_1(W_{i - 2})
            self.assign_and_constrain(
                region,
                || format!("sigma_1(W_{})_lo", new_word_idx - 2),
                a_7,
                get_word_row(new_word_idx - 16),
                lower_sigma_1_v2_results[new_word_idx - 16].0,
            )?;
            self.assign_and_constrain(
                region,
                || format!("sigma_1(W_{})_hi", new_word_idx - 2),
                a_7,
                get_word_row(new_word_idx - 16) + 1,
                lower_sigma_1_v2_results[new_word_idx - 16].1,
            )?;

            // Copy W_{i - 7}
            self.assign_and_constrain(
                region,
                || format!("W_{}_lo", new_word_idx - 7),
                a_8,
                get_word_row(new_word_idx - 16),
                w_halves[new_word_idx - 7].0,
            )?;
            self.assign_and_constrain(
                region,
                || format!("W_{}_hi", new_word_idx - 7),
                a_8,
                get_word_row(new_word_idx - 16) + 1,
                w_halves[new_word_idx - 7].1,
            )?;

            // Calculate W_i, carry_i
            let (word, carry) = sum_with_carry(vec![
                (
                    lower_sigma_1_v2_results[new_word_idx - 16].0.value,
                    lower_sigma_1_v2_results[new_word_idx - 16].1.value,
                ),
                (
                    w_halves[new_word_idx - 7].0.value,
                    w_halves[new_word_idx - 7].1.value,
                ),
                (sigma_0_output.0.value, sigma_0_output.1.value),
                (
                    w_halves[new_word_idx - 16].0.value,
                    w_halves[new_word_idx - 16].1.value,
                ),
            ]);

            // Assign W_i, carry_i
            region.assign_advice(
                || format!("W_{}", new_word_idx),
                a_5,
                get_word_row(new_word_idx - 16) + 1,
                || {
                    word.map(|word| F::from_u64(word as u64))
                        .ok_or(Error::SynthesisError)
                },
            )?;
            region.assign_advice(
                || format!("carry_{}", new_word_idx),
                a_9,
                get_word_row(new_word_idx - 16) + 1,
                || {
                    carry
                        .map(|carry| F::from_u64(carry as u64))
                        .ok_or(Error::SynthesisError)
                },
            )?;
            let (var, halves) = self.assign_word_and_halves(region, word, new_word_idx)?;
            w.push(MessageWord { var, value: word });
            w_halves.push(halves);

            Ok(lower_sigma_0_v2_results.clone())
        };

        let mut tmp_lower_sigma_0_v2_results: Vec<(CellValue16, CellValue16)> =
            Vec::with_capacity(SUBREGION_2_LEN);

        // Use up all the output from Subregion 1 lower_sigma_0
        for i in 14..27 {
            tmp_lower_sigma_0_v2_results = new_word(i, lower_sigma_0_output[i - 14])?;
        }

        for i in 27..49 {
            tmp_lower_sigma_0_v2_results =
                new_word(i, tmp_lower_sigma_0_v2_results[i + 2 - 15 - 14])?;
        }

        // Return lower_sigma_0_v2 output for W_[36..49]
        Ok(lower_sigma_0_v2_results.split_off(36 - 14))
    }

    fn decompose_subregion2_word<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
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
        let a = region.assign_advice(
            || "a",
            a_3,
            row - 1,
            || {
                pieces[0]
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `b` (4-bit piece) lookup
        let spread_b = SpreadWord::opt_new(pieces[1].map(|value| value as u16));
        let spread_b = SpreadVar::with_lookup(region, &self.lookup, row + 1, spread_b)?;

        // Assign `c` (3-bit piece)
        let c = region.assign_advice(
            || "c",
            a_4,
            row - 1,
            || {
                pieces[2]
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `d` (7-bit piece) lookup
        let spread_d = SpreadWord::opt_new(pieces[3].map(|value| value as u16));
        let spread_d = SpreadVar::with_lookup(region, &self.lookup, row, spread_d)?;

        // Assign `e` (1-bit piece)
        let e = region.assign_advice(
            || "e",
            a_3,
            row + 1,
            || {
                pieces[4]
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `f` (1-bit piece)
        let f = region.assign_advice(
            || "f",
            a_4,
            row + 1,
            || {
                pieces[5]
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `g` (13-bit piece) lookup
        let spread_g = SpreadWord::opt_new(pieces[6].map(|value| value as u16));
        let spread_g = SpreadVar::with_lookup(region, &self.lookup, row - 1, spread_g)?;

        Ok(Subregion2Word {
            index,
            a: CellValue32::new(a, pieces[0]),
            b: CellValue16::new(spread_b.dense.var, spread_b.dense.value),
            c: CellValue32::new(c, pieces[2]),
            d: CellValue16::new(spread_d.dense.var, spread_d.dense.value),
            e: CellValue32::new(e, pieces[4]),
            f: CellValue32::new(f, pieces[5]),
            g: CellValue16::new(spread_g.dense.var, spread_g.dense.value),
            spread_d: CellValue32::new(spread_d.spread.var, spread_d.spread.value),
            spread_g: CellValue32::new(spread_g.spread.var, spread_g.spread.value),
        })
    }

    #[allow(clippy::type_complexity)]
    fn assign_lower_sigma_v2_pieces<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        row: usize,
        subregion2_word: Subregion2Word,
    ) -> Result<[Option<u32>; 8], Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];
        let a_7 = self.extras[3];

        // Assign `a` and copy constraint
        self.assign_and_constrain(region, || "a", a_3, row + 1, subregion2_word.a)?;

        // Witness `spread_a`
        let spread_a = subregion2_word
            .a
            .value
            .map(|value| interleave_u16_with_zeros(value as u16));
        region.assign_advice(
            || "spread_a",
            a_4,
            row + 1,
            || {
                spread_a
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Split `b` (2-bit chunk) into `b_hi` and `b_lo`
        let b = subregion2_word.b.value.map(bisect_four_bit);
        let spread_b_lo = b.map(|b| interleave_u16_with_zeros(b.0 as u16));
        let spread_b_hi = b.map(|b| interleave_u16_with_zeros(b.1 as u16));

        // Assign `b_hi`, `spread_b_hi`, `b_lo`, `spread_b_lo`
        region.assign_advice(
            || "b_lo",
            a_3,
            row - 1,
            || {
                b.map(|(b_lo, _)| F::from_u64(b_lo as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_lo",
            a_4,
            row - 1,
            || {
                spread_b_lo
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "b_hi",
            a_5,
            row - 1,
            || {
                b.map(|(_, b_hi)| F::from_u64(b_hi as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_hi",
            a_6,
            row - 1,
            || {
                spread_b_hi
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `b` and copy constraint
        self.assign_and_constrain(region, || "b", a_6, row, subregion2_word.b)?;

        // Assign `c` and copy constraint
        self.assign_and_constrain(region, || "c", a_5, row + 1, subregion2_word.c)?;

        // Witness `spread_c`
        let spread_c = subregion2_word.c.value.map(|value| {
            let spread = interleave_u16_with_zeros(value as u16);
            spread as u32
        });
        region.assign_advice(
            || "spread_c",
            a_6,
            row + 1,
            || {
                spread_c
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `spread_d` and copy constraint
        self.assign_and_constrain(region, || "spread_d", a_4, row, subregion2_word.spread_d)?;

        // Assign `e` and copy constraint
        self.assign_and_constrain(region, || "e", a_7, row, subregion2_word.e)?;

        // Assign `f` and copy constraint
        self.assign_and_constrain(region, || "f", a_7, row + 1, subregion2_word.f)?;

        // Assign `spread_g` and copy constraint
        self.assign_and_constrain(region, || "spread_g", a_5, row, subregion2_word.spread_g)?;

        Ok([
            spread_a,
            spread_b_lo,
            spread_b_hi,
            spread_c,
            subregion2_word.spread_d.value,
            subregion2_word.e.value,
            subregion2_word.f.value,
            subregion2_word.spread_g.value,
        ])
    }

    fn lower_sigma_0_v2<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
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

    fn lower_sigma_1_v2<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
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
