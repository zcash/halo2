use super::super::{util::*, SpreadWord, Table16Chip};
use super::{schedule_util::*, MessagePiece, MessageSchedule, MessageWord};
use crate::{arithmetic::FieldExt, gadget::Region, plonk::Error};

// A word in subregion 3
// (10, 7, 2, 13)-bit chunks
pub struct Subregion3Word {
    index: usize,
    a: MessagePiece,
    b: MessagePiece,
    c: MessagePiece,
    d: MessagePiece,
    spread_a: MessagePiece,
    spread_d: MessagePiece,
}

impl MessageSchedule {
    // W_[49..62]
    pub fn assign_subregion3<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        lower_sigma_0_v2_output: Vec<(MessagePiece, MessagePiece)>,
        w: &mut Vec<MessageWord>,
        w_halves: &mut Vec<(MessagePiece, MessagePiece)>,
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
                a_6,
                get_word_row(new_word_idx - 16),
                &lower_sigma_0_v2_output[idx - 49].0,
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                a_6,
                get_word_row(new_word_idx - 16) + 1,
                &lower_sigma_0_v2_output[idx - 49].1,
                &self.perm,
            )?;

            // Copy sigma_1(W_{i - 2})
            self.assign_and_constrain(
                region,
                a_7,
                get_word_row(new_word_idx - 16),
                &r_0_even,
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                a_7,
                get_word_row(new_word_idx - 16) + 1,
                &r_1_even,
                &self.perm,
            )?;

            // Copy W_{i - 7}
            self.assign_and_constrain(
                region,
                a_8,
                get_word_row(new_word_idx - 16),
                &w_halves[new_word_idx - 7].0,
                &self.perm,
            )?;
            self.assign_and_constrain(
                region,
                a_8,
                get_word_row(new_word_idx - 16) + 1,
                &w_halves[new_word_idx - 7].1,
                &self.perm,
            )?;

            // Calculate W_i, carry_i
            let word_lo = r_0_even.value.unwrap()
                + w_halves[new_word_idx - 7].0.value.unwrap()
                + lower_sigma_0_v2_output[idx - 49].0.value.unwrap()
                + w_halves[new_word_idx - 16].0.value.unwrap();
            let word_hi = r_1_even.value.unwrap()
                + w_halves[new_word_idx - 7].1.value.unwrap()
                + lower_sigma_0_v2_output[idx - 49].1.value.unwrap()
                + w_halves[new_word_idx - 16].1.value.unwrap();

            let word: u64 = word_lo as u64 + (1 << 16) * (word_hi as u64);
            let carry = word >> 32;
            let word = word as u32;

            // Assign W_i, carry_i
            region.assign_advice(a_5, get_word_row(new_word_idx - 16) + 1, || {
                Ok(F::from_u64(word as u64))
            })?;
            region.assign_advice(a_9, get_word_row(new_word_idx - 16) + 1, || {
                Ok(F::from_u64(carry as u64))
            })?;
            let (var, lo, hi) = self.assign_word_and_halves(region, word, new_word_idx)?;
            w.push(MessageWord {
                var,
                value: Some(word),
            });
            w_halves.push((lo, hi));

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
        let (a, spread_a_cell) = self.assign_lookup(region, &spread_a, row + 1)?;

        // Assign `b` (7-bit piece)
        let b = region.assign_advice(a_4, row + 1, || Ok(F::from_u64(pieces[1] as u64)))?;

        // Assign `c` (2-bit piece)
        let c = region.assign_advice(a_3, row + 1, || Ok(F::from_u64(pieces[2] as u64)))?;

        // Assign `d` (13-bit piece) lookup
        let spread_d = SpreadWord::new(pieces[3] as u16);
        let (d, spread_d_cell) = self.assign_lookup(region, &spread_d, row)?;

        Ok(Subregion3Word {
            index,
            a: MessagePiece::new(a, pieces[0].into()),
            b: MessagePiece::new(b, pieces[1].into()),
            c: MessagePiece::new(c, pieces[2].into()),
            d: MessagePiece::new(d, pieces[3].into()),
            spread_a: MessagePiece::new(spread_a_cell, spread_a.spread),
            spread_d: MessagePiece::new(spread_d_cell, spread_d.spread),
        })
    }

    fn lower_sigma_1<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: Subregion3Word,
    ) -> Result<(MessagePiece, MessagePiece), Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `spread_a` and copy constraint
        self.assign_and_constrain(region, a_4, row, &word.spread_a, &self.perm)?;

        // Split `b` (7-bit chunk) into (2,2,3)-bit `b_lo`, `b_mid` and `b_hi`
        let b = word.b.value.unwrap();
        let b_hi = (b & 0b1110000) >> 4;
        let (b_lo, b_mid) = bisect_four_bit(b & 0b1111);
        let spread_b_lo = interleave_u16_with_zeros(b_lo as u16);
        let spread_b_mid = interleave_u16_with_zeros(b_mid as u16);
        let spread_b_hi = interleave_u16_with_zeros(b_hi as u16);

        // Assign `b_lo`, `spread_b_lo`, `b_mid`, `spread_b_mid`, `b_hi`, `spread_b_hi`
        region.assign_advice(a_3, row - 1, || Ok(F::from_u64(b_lo as u64)))?;
        region.assign_advice(a_4, row - 1, || Ok(F::from_u64(spread_b_lo as u64)))?;
        region.assign_advice(a_5, row - 1, || Ok(F::from_u64(b_mid as u64)))?;
        region.assign_advice(a_6, row - 1, || Ok(F::from_u64(spread_b_mid as u64)))?;
        region.assign_advice(a_5, row + 1, || Ok(F::from_u64(b_hi as u64)))?;
        region.assign_advice(a_6, row + 1, || Ok(F::from_u64(spread_b_hi as u64)))?;

        // Assign `b` and copy constraint
        self.assign_and_constrain(region, a_6, row, &word.b, &self.perm)?;

        // Assign `c` and copy constraint
        self.assign_and_constrain(region, a_3, row + 1, &word.c, &self.perm)?;

        // Witness `spread_c`
        let spread_c = interleave_u16_with_zeros(word.c.value.unwrap() as u16);
        region.assign_advice(a_4, row + 1, || Ok(F::from_u64(spread_c as u64)))?;

        // Assign `spread_d` and copy constraint
        self.assign_and_constrain(region, a_5, row, &word.spread_d, &self.perm)?;

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

        // Lookup R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let (r_0_even_dense_cell, _) =
            self.assign_lookup(region, &SpreadWord::new(r_0_even), row - 1)?;
        self.assign_lookup(region, &SpreadWord::new(r_0_odd), row)?;
        let (r_1_even_dense_cell, _) =
            self.assign_lookup(region, &SpreadWord::new(r_1_even), row + 1)?;
        let (_, r_1_odd_spread_cell) =
            self.assign_lookup(region, &SpreadWord::new(r_1_odd), row + 2)?;

        // Assign and copy R_1^{odd}
        let r_1_odd_spread = region.assign_advice(a_3, row, || {
            Ok(F::from_u64(interleave_u16_with_zeros(r_1_odd).into()))
        })?;
        region.constrain_equal(&self.perm, r_1_odd_spread_cell, r_1_odd_spread)?;

        Ok((
            MessagePiece::new(r_0_even_dense_cell, r_0_even as u32),
            MessagePiece::new(r_1_even_dense_cell, r_1_even as u32),
        ))
    }
}
