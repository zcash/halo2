use super::super::{super::DIGEST_SIZE, BlockWord, CellValue16, Table16Assignment};
use super::{compression_util::*, CompressionConfig, State};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Config, Layouter, Region, RegionIndex},
    plonk::{Advice, Column, Error},
};

impl<F: FieldExt, L: Layouter<F>> CompressionConfig<'_, F, L> {
    #[allow(clippy::many_single_char_names)]
    pub fn assign_digest(
        &mut self,
        region_index: RegionIndex,
        state: State,
    ) -> Result<[BlockWord; DIGEST_SIZE], Error> {
        let configured = self.configured().clone();

        let a_3 = configured.extras[0];
        let a_4 = configured.extras[1];
        let a_5 = configured.message_schedule;
        let a_6 = configured.extras[2];
        let a_7 = configured.extras[3];
        let a_8 = configured.extras[4];

        let (a, b, c, d, e, f, g, h) = match_state(state);

        let abcd_row = get_digest_abcd_row();
        let efgh_row = get_digest_efgh_row();

        let a_whole = a.dense_halves.0.value.unwrap() as u32
            + (1 << 16) * (a.dense_halves.1.value.unwrap() as u32);
        let e_whole = e.dense_halves.0.value.unwrap() as u32
            + (1 << 16) * (e.dense_halves.1.value.unwrap() as u32);

        let region_index = self.layouter().assign_existing_region(
            region_index,
            || "assign digest",
            |mut region: Region<'_, Self>| {
                region.assign_fixed(
                    || "s_digest",
                    configured.s_digest,
                    abcd_row,
                    || Ok(F::one()),
                )?;
                region.assign_fixed(
                    || "s_digest",
                    configured.s_digest,
                    efgh_row,
                    || Ok(F::one()),
                )?;

                region.assign_advice(|| "a", a_5, abcd_row, || Ok(F::from_u64(a_whole as u64)))?;

                let cell = region.assign_advice(
                    || "e",
                    a_5,
                    efgh_row,
                    || Ok(F::from_u64(e_whole as u64)),
                )?;

                Ok(cell.region_index())
            },
        )?;

        let b = self.assign_digest_word(region_index, abcd_row, a_6, a_7, a_8, b.dense_halves)?;
        let c =
            self.assign_digest_word(region_index, abcd_row + 1, a_3, a_4, a_5, c.dense_halves)?;
        let d =
            self.assign_digest_word(region_index, abcd_row + 1, a_6, a_7, a_8, d.dense_halves)?;

        let f = self.assign_digest_word(region_index, efgh_row, a_6, a_7, a_8, f.dense_halves)?;
        let g =
            self.assign_digest_word(region_index, efgh_row + 1, a_3, a_4, a_5, g.dense_halves)?;
        let h =
            self.assign_digest_word(region_index, efgh_row + 1, a_6, a_7, a_8, h.dense_halves)?;

        // Assign digest for A, B, C, D
        self.assign_and_constrain(
            region_index,
            || "a_lo",
            a_3,
            abcd_row,
            &a.dense_halves.0.into(),
            &configured.perm,
        )?;
        self.assign_and_constrain(
            region_index,
            || "a_hi",
            a_4,
            abcd_row,
            &a.dense_halves.1.into(),
            &configured.perm,
        )?;

        // Assign digest for E, F, G, H
        self.assign_and_constrain(
            region_index,
            || "e_lo",
            a_3,
            efgh_row,
            &e.dense_halves.0.into(),
            &configured.perm,
        )?;
        self.assign_and_constrain(
            region_index,
            || "e_hi",
            a_4,
            efgh_row,
            &e.dense_halves.1.into(),
            &configured.perm,
        )?;

        Ok([
            BlockWord::new(a_whole),
            BlockWord::new(b),
            BlockWord::new(c),
            BlockWord::new(d),
            BlockWord::new(e_whole),
            BlockWord::new(f),
            BlockWord::new(g),
            BlockWord::new(h),
        ])
    }

    fn assign_digest_word(
        &mut self,
        region_index: RegionIndex,
        row: usize,
        lo_col: Column<Advice>,
        hi_col: Column<Advice>,
        word_col: Column<Advice>,
        dense_halves: (CellValue16, CellValue16),
    ) -> Result<u32, Error> {
        let configured = self.configured().clone();

        self.assign_and_constrain(
            region_index,
            || "lo",
            lo_col,
            row,
            &dense_halves.0.into(),
            &configured.perm,
        )?;
        self.assign_and_constrain(
            region_index,
            || "hi",
            hi_col,
            row,
            &dense_halves.1.into(),
            &configured.perm,
        )?;
        let val = dense_halves.0.value.unwrap() as u32
            + (1 << 16) * (dense_halves.1.value.unwrap() as u32);

        self.layouter().assign_existing_region(
            region_index,
            || "assign digest word",
            |mut region: Region<'_, Self>| {
                region.assign_advice(|| "word", word_col, row, || Ok(F::from_u64(val as u64)))
            },
        )?;

        Ok(val)
    }
}
