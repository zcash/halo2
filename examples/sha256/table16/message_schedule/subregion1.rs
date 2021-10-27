use super::super::{
    util::*, BlockWord, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment,
};
use super::{schedule_util::*, MessageScheduleConfig};
use halo2::{arithmetic::FieldExt, circuit::Region, pasta::pallas, plonk::Error};

// A word in subregion 1
// (3, 4, 11, 14)-bit chunks
#[derive(Debug)]
pub struct Subregion1Word {
    index: usize,
    a: CellValue16,
    b: CellValue16,
    c: CellValue16,
    d: CellValue16,
    spread_c: CellValue32,
    spread_d: CellValue32,
}

impl MessageScheduleConfig {
    pub fn assign_subregion1(
        &self,
        region: &mut Region<'_, pallas::Base>,
        input: &[BlockWord],
    ) -> Result<Vec<(CellValue16, CellValue16)>, Error> {
        assert_eq!(input.len(), SUBREGION_1_LEN);
        Ok(input
            .iter()
            .enumerate()
            .map(|(idx, word)| {
                // s_decompose_1 on W_[1..14]
                let subregion1_word = self
                    .decompose_subregion1_word(region, word.0, idx + 1)
                    .unwrap();

                // lower_sigma_0 on W_[1..14]
                self.lower_sigma_0(region, subregion1_word).unwrap()
            })
            .collect::<Vec<_>>())
    }

    fn decompose_subregion1_word(
        &self,
        region: &mut Region<'_, pallas::Base>,
        word: Option<u32>,
        index: usize,
    ) -> Result<Subregion1Word, Error> {
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let pieces = word.map(|word| chop_u32(word, &[3, 4, 11, 14]));
        let pieces = transpose_option_vec(pieces, 4);

        // Assign `a` (3-bit piece)
        let a = CellValue16::assign_unchecked(
            region,
            || "word_a",
            a_3,
            row + 1,
            pieces[0].map(|piece| piece as u16),
        )?;
        // Assign `b` (4-bit piece)
        let b = CellValue16::assign_unchecked(
            region,
            || "word_b",
            a_4,
            row + 1,
            pieces[1].map(|piece| piece as u16),
        )?;

        // Assign `c` (11-bit piece) lookup
        let spread_c = SpreadWord::opt_new(pieces[2].map(|value| value as u16));
        let spread_c = SpreadVar::with_lookup(region, &self.lookup, row + 1, spread_c)?;

        // Assign `d` (14-bit piece) lookup
        let spread_d = SpreadWord::opt_new(pieces[3].map(|value| value as u16));
        let spread_d = SpreadVar::with_lookup(region, &self.lookup, row, spread_d)?;

        Ok(Subregion1Word {
            index,
            a,
            b,
            c: spread_c.dense,
            d: spread_d.dense,
            spread_c: spread_c.spread,
            spread_d: spread_d.spread,
        })
    }

    // sigma_0 v1 on a word in W_1 to W_13
    // (3, 4, 11, 14)-bit chunks
    fn lower_sigma_0(
        &self,
        region: &mut Region<'_, pallas::Base>,
        word: Subregion1Word,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_5 = self.message_schedule;
        let a_6 = self.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `a` and copy constraint
        word.a.copy_advice(|| "a", region, a_5, row + 1)?;

        // Witness `spread_a`
        let spread_a = word.a.value_u16().map(interleave_u16_with_zeros);
        region.assign_advice(
            || "spread_a",
            a_6,
            row + 1,
            || {
                spread_a
                    .map(|value| pallas::Base::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        // Split `b` (2-bit chunk) into `b_hi` and `b_lo`
        let b = word.b.value_u16().map(bisect_four_bit);
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
        word.b.copy_advice(|| "b", region, a_6, row)?;

        // Assign `spread_c` and copy constraint
        word.spread_c.copy_advice(|| "spread_c", region, a_4, row)?;

        // Assign `spread_d` and copy constraint
        word.spread_d.copy_advice(|| "spread_d", region, a_5, row)?;

        // Calculate R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let (r_0_even, r_0_odd, r_1_even, r_1_odd) = if let Some(spread_a) = spread_a {
            let spread_a = spread_a as u64;
            let spread_b_lo = spread_b_lo.unwrap() as u64;
            let spread_b_hi = spread_b_hi.unwrap() as u64;
            let spread_c = word.spread_c.value().unwrap().0 as u64;
            let spread_d = word.spread_d.value().unwrap().0 as u64;

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
