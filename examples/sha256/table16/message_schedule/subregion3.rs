use super::super::{
    util::*, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment, Table16Chip,
};
use super::{schedule_util::*, MessageSchedule, MessageWord};
use halo2::{
    arithmetic::FieldExt,
    plonk::{Error, Region},
};

// A word in subregion 3
// (10, 7, 2, 13)-bit chunks
pub struct Subregion3Word {
    index: usize,
    #[allow(dead_code)]
    a: CellValue32,
    b: CellValue32,
    c: CellValue32,
    #[allow(dead_code)]
    d: CellValue32,
    spread_a: CellValue32,
    spread_d: CellValue32,
}

impl MessageSchedule {
    // W_[49..62]
    pub fn assign_subregion3<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
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
                self.decompose_subregion3_word(region, w[idx].value.unwrap(), idx)?;

            // sigma_1 on subregion3_word
            let (r_0_even, r_1_even) = self.lower_sigma_1(region, subregion3_word)?;

            let new_word_idx = idx + 2;

            // Copy sigma_0_v2(W_{i - 15}) output from Subregion 2
            self.assign_and_constrain(
                region,
                || format!("sigma_0(W_{})_lo", new_word_idx - 15),
                a_6,
                get_word_row(new_word_idx - 16),
                &lower_sigma_0_v2_output[idx - 49].0.into(),
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                || format!("sigma_0(W_{})_hi", new_word_idx - 15),
                a_6,
                get_word_row(new_word_idx - 16) + 1,
                &lower_sigma_0_v2_output[idx - 49].1.into(),
                &self.perm,
            )?;

            // Copy sigma_1(W_{i - 2})
            self.assign_and_constrain(
                region,
                || format!("sigma_1(W_{})_lo", new_word_idx - 2),
                a_7,
                get_word_row(new_word_idx - 16),
                &r_0_even.into(),
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                || format!("sigma_1(W_{})_hi", new_word_idx - 2),
                a_7,
                get_word_row(new_word_idx - 16) + 1,
                &r_1_even.into(),
                &self.perm,
            )?;

            // Copy W_{i - 7}
            self.assign_and_constrain(
                region,
                || format!("W_{}_lo", new_word_idx - 7),
                a_8,
                get_word_row(new_word_idx - 16),
                &w_halves[new_word_idx - 7].0.into(),
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                || format!("W_{}_hi", new_word_idx - 7),
                a_8,
                get_word_row(new_word_idx - 16) + 1,
                &w_halves[new_word_idx - 7].1.into(),
                &self.perm,
            )?;

            // Calculate W_i, carry_i
            let word_lo: u32 = r_0_even.value.unwrap() as u32
                + w_halves[new_word_idx - 7].0.value.unwrap() as u32
                + lower_sigma_0_v2_output[idx - 49].0.value.unwrap() as u32
                + w_halves[new_word_idx - 16].0.value.unwrap() as u32;
            let word_hi: u32 = r_1_even.value.unwrap() as u32
                + w_halves[new_word_idx - 7].1.value.unwrap() as u32
                + lower_sigma_0_v2_output[idx - 49].1.value.unwrap() as u32
                + w_halves[new_word_idx - 16].1.value.unwrap() as u32;

            let word: u64 = word_lo as u64 + (1 << 16) * (word_hi as u64);
            let carry = word >> 32;
            let word = word as u32;

            // Assign W_i, carry_i
            region.assign_advice(
                || format!("W_{}", new_word_idx),
                a_5,
                get_word_row(new_word_idx - 16) + 1,
                || Ok(F::from_u64(word as u64)),
            )?;
            region.assign_advice(
                || format!("carry_{}", new_word_idx),
                a_9,
                get_word_row(new_word_idx - 16) + 1,
                || Ok(F::from_u64(carry as u64)),
            )?;
            let (var, halves) = self.assign_word_and_halves(region, word, new_word_idx)?;
            w.push(MessageWord {
                var,
                value: Some(word),
            });
            w_halves.push(halves);

            Ok(())
        };

        for i in 49..62 {
            new_word(i)?;
        }

        Ok(())
    }

    fn decompose_subregion3_word<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: u32,
        index: usize,
    ) -> Result<Subregion3Word, Error> {
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let pieces = chop_u32(word, &[10, 7, 2, 13]);

        // Assign `a` (10-bit piece)
        let spread_a = SpreadWord::new(pieces[0] as u16);
        let spread_a = SpreadVar::with_lookup(region, &self.lookup, row + 1, spread_a)?;

        // Assign `b` (7-bit piece)
        let b = region.assign_advice(|| "b", a_4, row + 1, || Ok(F::from_u64(pieces[1] as u64)))?;

        // Assign `c` (2-bit piece)
        let c = region.assign_advice(|| "c", a_3, row + 1, || Ok(F::from_u64(pieces[2] as u64)))?;

        // Assign `d` (13-bit piece) lookup
        let spread_d = SpreadWord::new(pieces[3] as u16);
        let spread_d = SpreadVar::with_lookup(region, &self.lookup, row, spread_d)?;

        Ok(Subregion3Word {
            index,
            a: CellValue32::new(spread_a.dense.var, spread_a.dense.value.unwrap().into()),
            b: CellValue32::new(b, pieces[1]),
            c: CellValue32::new(c, pieces[2]),
            d: CellValue32::new(spread_d.dense.var, spread_d.dense.value.unwrap().into()),
            spread_a: CellValue32::new(spread_a.spread.var, spread_a.spread.value.unwrap()),
            spread_d: CellValue32::new(spread_d.spread.var, spread_d.spread.value.unwrap()),
        })
    }

    fn lower_sigma_1<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: Subregion3Word,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `spread_a` and copy constraint
        self.assign_and_constrain(region, || "spread_a", a_4, row, &word.spread_a, &self.perm)?;

        // Split `b` (7-bit chunk) into (2,2,3)-bit `b_lo`, `b_mid` and `b_hi`
        let b = word.b.value.unwrap();
        let b_hi = (b & 0b1110000) >> 4;
        let (b_lo, b_mid) = bisect_four_bit(b & 0b1111);
        let spread_b_lo = interleave_u16_with_zeros(b_lo as u16);
        let spread_b_mid = interleave_u16_with_zeros(b_mid as u16);
        let spread_b_hi = interleave_u16_with_zeros(b_hi as u16);

        // Assign `b_lo`, `spread_b_lo`, `b_mid`, `spread_b_mid`, `b_hi`, `spread_b_hi`
        region.assign_advice(|| "b_lo", a_3, row - 1, || Ok(F::from_u64(b_lo as u64)))?;
        region.assign_advice(
            || "spread_b_lo",
            a_4,
            row - 1,
            || Ok(F::from_u64(spread_b_lo as u64)),
        )?;
        region.assign_advice(|| "b_mid", a_5, row - 1, || Ok(F::from_u64(b_mid as u64)))?;
        region.assign_advice(
            || "spread_b_mid",
            a_6,
            row - 1,
            || Ok(F::from_u64(spread_b_mid as u64)),
        )?;
        region.assign_advice(|| "b_hi", a_5, row + 1, || Ok(F::from_u64(b_hi as u64)))?;
        region.assign_advice(
            || "spread_b_hi",
            a_6,
            row + 1,
            || Ok(F::from_u64(spread_b_hi as u64)),
        )?;

        // Assign `b` and copy constraint
        self.assign_and_constrain(region, || "b", a_6, row, &word.b, &self.perm)?;

        // Assign `c` and copy constraint
        self.assign_and_constrain(region, || "c", a_3, row + 1, &word.c, &self.perm)?;

        // Witness `spread_c`
        let spread_c = interleave_u16_with_zeros(word.c.value.unwrap() as u16);
        region.assign_advice(
            || "spread_c",
            a_4,
            row + 1,
            || Ok(F::from_u64(spread_c as u64)),
        )?;

        // Assign `spread_d` and copy constraint
        self.assign_and_constrain(region, || "spread_d", a_5, row, &word.spread_d, &self.perm)?;

        // (10, 7, 2, 13)
        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let spread_a = word.spread_a.value.unwrap() as u64;
        let spread_b_lo = spread_b_lo as u64;
        let spread_b_mid = spread_b_mid as u64;
        let spread_b_hi = spread_b_hi as u64;
        let spread_c = spread_c as u64;
        let spread_d = word.spread_d.value.unwrap() as u64;
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

        self.assign_sigma_outputs(
            region,
            &self.lookup,
            a_3,
            &self.perm,
            row,
            r_0_even,
            r_0_odd,
            r_1_even,
            r_1_odd,
        )
    }
}
