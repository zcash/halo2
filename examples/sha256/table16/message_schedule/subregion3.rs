use super::super::{util::*, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment};
use super::{schedule_util::*, MessageScheduleConfig, MessageWord};
use halo2::{arithmetic::FieldExt, circuit::Region, pasta::pallas, plonk::Error};

// A word in subregion 3
// (10, 7, 2, 13)-bit chunks
pub struct Subregion3Word {
    index: usize,
    #[allow(dead_code)]
    a: CellValue16,
    b: CellValue16,
    c: CellValue16,
    #[allow(dead_code)]
    d: CellValue16,
    spread_a: CellValue32,
    spread_d: CellValue32,
}

impl MessageScheduleConfig {
    // W_[49..62]
    pub fn assign_subregion3(
        &self,
        region: &mut Region<'_, pallas::Base>,
        lower_sigma_0_v2_output: Vec<(CellValue16, CellValue16)>,
        w: &mut Vec<MessageWord>,
        w_halves: &mut Vec<(CellValue16, CellValue16)>,
    ) -> Result<(), Error> {
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];
        let a_7 = self.extras[3];
        let a_8 = self.extras[4];
        let a_9 = self.extras[5];

        // Closure to compose new word
        // W_i = sigma_1(W_{i - 2}) + W_{i - 7} + sigma_0(W_{i - 15}) + W_{i - 16}
        // e.g. W_51 = sigma_1(W_49) + W_44 + sigma_0(W_36) + W_35

        // sigma_0_v2(W_[36..49]) will be used to get the new W_[51..64]
        // sigma_1(W_[49..62]) will also be used to get the W_[51..64]
        // The lowest-index words involved will be W_[35..58]
        let mut new_word = |idx: usize| -> Result<(), Error> {
            // Decompose word into (10, 7, 2, 13)-bit chunks
            let subregion3_word =
                self.decompose_subregion3_word(region, w[idx].value_u32(), idx)?;

            // sigma_1 on subregion3_word
            let (r_0_even, r_1_even) = self.lower_sigma_1(region, subregion3_word)?;

            let new_word_idx = idx + 2;

            // Copy sigma_0_v2(W_{i - 15}) output from Subregion 2
            lower_sigma_0_v2_output[idx - 49].0.copy_advice(
                || format!("sigma_0(W_{})_lo", new_word_idx - 15),
                region,
                a_6,
                get_word_row(new_word_idx - 16),
            )?;
            lower_sigma_0_v2_output[idx - 49].1.copy_advice(
                || format!("sigma_0(W_{})_hi", new_word_idx - 15),
                region,
                a_6,
                get_word_row(new_word_idx - 16) + 1,
            )?;

            // Copy sigma_1(W_{i - 2})
            r_0_even.copy_advice(
                || format!("sigma_1(W_{})_lo", new_word_idx - 2),
                region,
                a_7,
                get_word_row(new_word_idx - 16),
            )?;
            r_1_even.copy_advice(
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
                (r_0_even.value_u16(), r_1_even.value_u16()),
                (
                    w_halves[new_word_idx - 7].0.value_u16(),
                    w_halves[new_word_idx - 7].1.value_u16(),
                ),
                (
                    lower_sigma_0_v2_output[idx - 49].0.value_u16(),
                    lower_sigma_0_v2_output[idx - 49].1.value_u16(),
                ),
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

            Ok(())
        };

        for i in 49..62 {
            new_word(i)?;
        }

        Ok(())
    }

    fn decompose_subregion3_word(
        &self,
        region: &mut Region<'_, pallas::Base>,
        word: Option<u32>,
        index: usize,
    ) -> Result<Subregion3Word, Error> {
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let pieces = transpose_option_vec(
            word.map(|word| {
                chop_u32(word, &[10, 7, 2, 13])
                    .into_iter()
                    .map(|piece| piece as u16)
                    .collect()
            }),
            4,
        );

        // Assign `a` (10-bit piece)
        let spread_a = pieces[0].map(SpreadWord::new);
        let spread_a = SpreadVar::with_lookup(region, &self.lookup, row + 1, spread_a)?;

        // Assign `b` (7-bit piece)
        let b = CellValue16::assign_unchecked(region, || "b", a_4, row + 1, pieces[1])?;

        // Assign `c` (2-bit piece)
        let c = CellValue16::assign_unchecked(region, || "c", a_3, row + 1, pieces[2])?;

        // Assign `d` (13-bit piece) lookup
        let spread_d = pieces[3].map(SpreadWord::new);
        let spread_d = SpreadVar::with_lookup(region, &self.lookup, row, spread_d)?;

        Ok(Subregion3Word {
            index,
            a: spread_a.dense,
            b,
            c,
            d: spread_d.dense,
            spread_a: spread_a.spread,
            spread_d: spread_d.spread,
        })
    }

    fn lower_sigma_1(
        &self,
        region: &mut Region<'_, pallas::Base>,
        word: Subregion3Word,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `spread_a` and copy constraint
        word.spread_a.copy_advice(|| "spread_a", region, a_4, row)?;

        // Split `b` (7-bit chunk) into (2,2,3)-bit `b_lo`, `b_mid` and `b_hi`
        let (b_lo, b_mid, b_hi) = {
            let b_hi = word.b.value_u16().map(|b| (b & 0b1110000) >> 4);
            let (b_lo, b_mid) = {
                let b_lo_mid = word.b.value_u16().map(|b| bisect_four_bit(b & 0b1111));
                (b_lo_mid.map(|(lo, _)| lo), b_lo_mid.map(|(_, mid)| mid))
            };
            (b_lo, b_mid, b_hi)
        };
        let (spread_b_lo, spread_b_mid, spread_b_hi) = {
            let spread_b_lo = b_lo.map(|b_lo| interleave_u16_with_zeros(b_lo as u16));
            let spread_b_mid = b_mid.map(|b_mid| interleave_u16_with_zeros(b_mid as u16));
            let spread_b_hi = b_hi.map(|b_hi| interleave_u16_with_zeros(b_hi as u16));

            (spread_b_lo, spread_b_mid, spread_b_hi)
        };

        // Assign `b_lo`, `spread_b_lo`, `b_mid`, `spread_b_mid`, `b_hi`, `spread_b_hi`
        region.assign_advice(
            || "b_lo",
            a_3,
            row - 1,
            || {
                b_lo.map(|value| pallas::Base::from_u64(value as u64))
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
            || "b_mid",
            a_5,
            row - 1,
            || {
                b_mid
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_mid",
            a_6,
            row - 1,
            || {
                spread_b_mid
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "b_hi",
            a_5,
            row + 1,
            || {
                b_hi.map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.assign_advice(
            || "spread_b_hi",
            a_6,
            row + 1,
            || {
                spread_b_hi
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `b` and copy constraint
        word.b.copy_advice(|| "b", region, a_6, row)?;

        // Assign `c` and copy constraint
        word.c.copy_advice(|| "c", region, a_3, row + 1)?;

        // Witness `spread_c`
        let spread_c = word.c.value_u16().map(interleave_u16_with_zeros);
        region.assign_advice(
            || "spread_c",
            a_4,
            row + 1,
            || {
                spread_c
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Assign `spread_d` and copy constraint
        word.spread_d.copy_advice(|| "spread_d", region, a_5, row)?;

        // (10, 7, 2, 13)
        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let (r_0_even, r_0_odd, r_1_even, r_1_odd) = if word.spread_a.value().is_some() {
            let spread_a = word.spread_a.value().unwrap().0 as u64;
            let spread_b_lo = spread_b_lo.unwrap() as u64;
            let spread_b_mid = spread_b_mid.unwrap() as u64;
            let spread_b_hi = spread_b_hi.unwrap() as u64;
            let spread_c = spread_c.unwrap() as u64;
            let spread_d = word.spread_d.value().unwrap().0 as u64;
            let xor_0 = spread_b_lo
                + (1 << 4) * spread_b_mid
                + (1 << 8) * spread_b_hi
                + (1 << 14) * spread_c
                + (1 << 18) * spread_d;
            let xor_1 = spread_c
                + (1 << 4) * spread_d
                + (1 << 30) * spread_a
                + (1 << 50) * spread_b_lo
                + (1 << 54) * spread_b_mid
                + (1 << 58) * spread_b_hi;
            let xor_2 = spread_d
                + (1 << 26) * spread_a
                + (1 << 46) * spread_b_lo
                + (1 << 50) * spread_b_mid
                + (1 << 54) * spread_b_hi
                + (1 << 60) * spread_c;
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
