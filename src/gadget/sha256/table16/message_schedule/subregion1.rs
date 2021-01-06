use super::super::{util::*, BlockWord, SpreadWord, Table16Chip};
use super::{MessagePiece, MessageSchedule};
use crate::{arithmetic::FieldExt, gadget::Region, plonk::Error};

// A word in subregion 1
// (3, 4, 11, 14)-bit chunks
#[derive(Debug)]
pub struct Subregion1Word {
    index: usize,
    a: MessagePiece,
    b: MessagePiece,
    c: MessagePiece,
    d: MessagePiece,
    spread_c: MessagePiece,
    spread_d: MessagePiece,
}

impl MessageSchedule {
    pub fn assign_subregion1<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        input: &[BlockWord],
    ) -> Result<Vec<(MessagePiece, MessagePiece)>, Error> {
        assert_eq!(input.len(), SUBREGION_1_LEN);
        Ok(input
            .iter()
            .enumerate()
            .map(|(idx, word)| {
                // s_decompose_1 on W_[1..14]
                let subregion1_word = self
                    .decompose_subregion1_word(region, word.value.unwrap(), idx + 1)
                    .unwrap();

                // lower_sigma_0 on W_[1..14]
                self.lower_sigma_0(region, subregion1_word).unwrap()
            })
            .collect::<Vec<_>>())
    }

    fn decompose_subregion1_word<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: u32,
        index: usize,
    ) -> Result<Subregion1Word, Error> {
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let pieces = chop_u32(word, &[3, 4, 11, 14]);

        // Assign `a` (3-bit piece)
        let a = region.assign_advice(a_3, row + 1, || Ok(F::from_u64(pieces[0] as u64)))?;
        // Assign `b` (4-bit piece)
        let b = region.assign_advice(a_4, row + 1, || Ok(F::from_u64(pieces[1] as u64)))?;

        // Assign `c` (11-bit piece) lookup
        let spread_c = SpreadWord::new(pieces[2] as u16);
        let (c, spread_c_cell) = self.assign_lookup(region, &spread_c, row + 1)?;

        // Assign `d` (14-bit piece) lookup
        let spread_d = SpreadWord::new(pieces[3] as u16);
        let (d, spread_d_cell) = self.assign_lookup(region, &spread_d, row)?;

        Ok(Subregion1Word {
            index,
            a: MessagePiece::new(a, pieces[0].into()),
            b: MessagePiece::new(b, pieces[1].into()),
            c: MessagePiece::new(c, pieces[2].into()),
            d: MessagePiece::new(d, pieces[3].into()),
            spread_c: MessagePiece::new(spread_c_cell, spread_c.spread),
            spread_d: MessagePiece::new(spread_d_cell, spread_d.spread),
        })
    }

    // sigma_0 v1 on a word in W_1 to W_13
    // (3, 4, 11, 14)-bit chunks
    fn lower_sigma_0<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: Subregion1Word,
    ) -> Result<(MessagePiece, MessagePiece), Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `a` and copy constraint
        self.assign_and_constrain(region, a_5, row + 1, &word.a, &self.perm)?;

        // Witness `spread_a`
        let spread_a = interleave_u16_with_zeros(word.a.value.unwrap() as u16);
        region.assign_advice(a_6, row + 1, || Ok(F::from_u64(spread_a as u64)))?;

        // Split `b` (2-bit chunk) into `b_hi` and `b_lo`
        let b = word.b.value.unwrap();
        let (b_lo, b_hi) = bisect_four_bit(b);
        let spread_b_lo = interleave_u16_with_zeros(b_lo as u16);
        let spread_b_hi = interleave_u16_with_zeros(b_hi as u16);

        // Assign `b_hi`, `spread_b_hi`, `b_lo`, `spread_b_lo`
        region.assign_advice(a_3, row - 1, || Ok(F::from_u64(b_lo as u64)))?;
        region.assign_advice(a_4, row - 1, || Ok(F::from_u64(spread_b_lo as u64)))?;
        region.assign_advice(a_5, row - 1, || Ok(F::from_u64(b_hi as u64)))?;
        region.assign_advice(a_6, row - 1, || Ok(F::from_u64(spread_b_hi as u64)))?;

        // Assign `b` and copy constraint
        self.assign_and_constrain(region, a_6, row, &word.b, &self.perm)?;

        // Assign `spread_c` and copy constraint
        self.assign_and_constrain(region, a_4, row, &word.spread_c, &self.perm)?;

        // Assign `spread_d` and copy constraint
        self.assign_and_constrain(region, a_5, row, &word.spread_d, &self.perm)?;

        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let spread_a = spread_a as u64;
        let spread_b_lo = spread_b_lo as u64;
        let spread_b_hi = spread_b_hi as u64;
        let spread_c = word.spread_c.value.unwrap() as u64;
        let spread_d = word.spread_d.value.unwrap() as u64;
        let xor_0: u64 =
            spread_b_lo + (1 << 4) * spread_b_hi + (1 << 8) * spread_c + (1 << 30) * spread_d;
        let xor_1: u64 = spread_c
            + (1 << 22) * spread_d
            + (1 << 50) * spread_a
            + (1 << 56) * spread_b_lo
            + (1 << 60) * spread_b_hi;
        let xor_2: u64 = spread_d
            + (1 << 28) * spread_a
            + (1 << 34) * spread_b_lo
            + (1 << 38) * spread_b_hi
            + (1 << 42) * spread_c;
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
