use super::super::{
    util::*, BlockWord, CellValue16, CellValue32, SpreadVar, SpreadWord, Table16Assignment,
};
use super::{schedule_util::*, MessageScheduleConfig};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Config, Layouter, Region},
    plonk::Error,
};

// A word in subregion 1
// (3, 4, 11, 14)-bit chunks
#[derive(Debug)]
pub struct Subregion1Word {
    index: usize,
    a: CellValue32,
    b: CellValue32,
    c: CellValue32,
    d: CellValue32,
    spread_c: CellValue32,
    spread_d: CellValue32,
}

impl<F: FieldExt, L: Layouter<Field = F>> MessageScheduleConfig<'_, F, L> {
    pub fn assign_subregion1(
        &mut self,
        input: &[BlockWord],
    ) -> Result<Vec<(CellValue16, CellValue16)>, Error> {
        assert_eq!(input.len(), SUBREGION_1_LEN);
        Ok(input
            .iter()
            .enumerate()
            .map(|(idx, word)| {
                // s_decompose_1 on W_[1..14]
                let subregion1_word = self
                    .decompose_subregion1_word(word.value.unwrap(), idx + 1)
                    .unwrap();

                // lower_sigma_0 on W_[1..14]
                self.lower_sigma_0(subregion1_word).unwrap()
            })
            .collect::<Vec<_>>())
    }

    fn decompose_subregion1_word(
        &mut self,
        word: u32,
        index: usize,
    ) -> Result<Subregion1Word, Error> {
        let configured = self.configured().clone();
        let row = get_word_row(index);

        // Rename these here for ease of matching the gates to the specification.
        let a_3 = configured.extras[0];
        let a_4 = configured.extras[1];

        let pieces = chop_u32(word, &[3, 4, 11, 14]);
        self.layouter().assign_region(
            || "decompose subregion1 word",
            |mut region: Region<'_, Self>| {
                // Assign `a` (3-bit piece)
                let a = region.assign_advice(
                    || "word_a",
                    a_3,
                    row + 1,
                    || Ok(F::from_u64(pieces[0] as u64)),
                )?;
                // Assign `b` (4-bit piece)
                let b = region.assign_advice(
                    || "word_b",
                    a_4,
                    row + 1,
                    || Ok(F::from_u64(pieces[1] as u64)),
                )?;

                // Assign `c` (11-bit piece) lookup
                let spread_c = SpreadWord::new(pieces[2] as u16);
                let spread_c =
                    SpreadVar::with_lookup(&mut region, &configured.lookup, row + 1, spread_c)?;

                // Assign `d` (14-bit piece) lookup
                let spread_d = SpreadWord::new(pieces[3] as u16);
                let spread_d =
                    SpreadVar::with_lookup(&mut region, &configured.lookup, row, spread_d)?;

                Ok(Subregion1Word {
                    index,
                    a: CellValue32::new(a, pieces[0]),
                    b: CellValue32::new(b, pieces[1]),
                    c: CellValue32::new(spread_c.dense.var, spread_c.dense.value.unwrap().into()),
                    d: CellValue32::new(spread_d.dense.var, spread_d.dense.value.unwrap().into()),
                    spread_c: CellValue32::new(spread_c.spread.var, spread_c.spread.value.unwrap()),
                    spread_d: CellValue32::new(spread_d.spread.var, spread_d.spread.value.unwrap()),
                })
            },
        )
    }

    // sigma_0 v1 on a word in W_1 to W_13
    // (3, 4, 11, 14)-bit chunks
    fn lower_sigma_0(&mut self, word: Subregion1Word) -> Result<(CellValue16, CellValue16), Error> {
        let configured = self.configured().clone();

        let a_3 = configured.extras[0];
        let a_4 = configured.extras[1];
        let a_5 = configured.message_schedule;
        let a_6 = configured.extras[2];

        let row = get_word_row(word.index) + 3;

        // Assign `a` and copy constraint
        self.assign_and_constrain(|| "a", a_5, row + 1, &word.a, &configured.perm)?;

        let (spread_a, spread_b_lo, spread_b_hi) = self.layouter().assign_region(
            || "lower_sigma_0",
            |mut region: Region<'_, Self>| {
                // Witness `spread_a`
                let spread_a = interleave_u16_with_zeros(word.a.value.unwrap() as u16);
                region.assign_advice(
                    || "spread_a",
                    a_6,
                    row + 1,
                    || Ok(F::from_u64(spread_a as u64)),
                )?;

                // Split `b` (2-bit chunk) into `b_hi` and `b_lo`
                let b = word.b.value.unwrap();
                let (b_lo, b_hi) = bisect_four_bit(b);
                let spread_b_lo = interleave_u16_with_zeros(b_lo as u16);
                let spread_b_hi = interleave_u16_with_zeros(b_hi as u16);

                // Assign `b_hi`, `spread_b_hi`, `b_lo`, `spread_b_lo`
                region.assign_advice(|| "b_lo", a_3, row - 1, || Ok(F::from_u64(b_lo as u64)))?;
                region.assign_advice(
                    || "spread_b_lo",
                    a_4,
                    row - 1,
                    || Ok(F::from_u64(spread_b_lo as u64)),
                )?;
                region.assign_advice(|| "b_hi", a_5, row - 1, || Ok(F::from_u64(b_hi as u64)))?;
                region.assign_advice(
                    || "spread_b_hi",
                    a_6,
                    row - 1,
                    || Ok(F::from_u64(spread_b_hi as u64)),
                )?;
                Ok((spread_a, spread_b_lo, spread_b_hi))
            },
        )?;

        // Assign `b` and copy constraint
        self.assign_and_constrain(|| "b", a_6, row, &word.b, &configured.perm)?;

        // Assign `spread_c` and copy constraint
        self.assign_and_constrain(|| "spread_c", a_4, row, &word.spread_c, &configured.perm)?;

        // Assign `spread_d` and copy constraint
        self.assign_and_constrain(|| "spread_d", a_5, row, &word.spread_d, &configured.perm)?;

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

        self.assign_sigma_outputs(
            &configured.lookup,
            a_3,
            &configured.perm,
            row,
            r_0_even,
            r_0_odd,
            r_1_even,
            r_1_odd,
        )
    }
}
