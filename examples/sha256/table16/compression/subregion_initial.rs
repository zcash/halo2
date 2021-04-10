use super::super::{RoundWordDense, RoundWordSpread, StateWord, STATE};
use super::{compression_util::*, CompressionConfig, State};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Config, Layouter, Region, RegionIndex},
    plonk::Error,
};

impl<F: FieldExt, L: Layouter<F>> CompressionConfig<'_, F, L> {
    #[allow(clippy::many_single_char_names)]
    pub fn initialize_iv(&mut self, iv: [u32; STATE]) -> Result<(State, RegionIndex), Error> {
        let configured = self.configured().clone();
        let a_7 = configured.extras[3];

        let idx = -1;
        let d_row = get_d_row(idx);

        // Dummy assignment to get region index
        let region_index = self.layouter().assign_new_region(
            &configured.columns(),
            || "Dummy assignment",
            |mut region: Region<'_, Self>| {
                let cell = region.assign_advice(|| "Dummy", a_7, d_row, || Ok(F::zero()))?;
                Ok(cell.region_index())
            },
        )?;

        // Assign D
        let d_dense =
            self.assign_word_halves_dense(region_index, d_row, a_7, d_row + 1, a_7, iv[3])?;
        let d = RoundWordDense::new(d_dense);

        // Assign H
        let h_row = get_h_row(idx);
        let h_dense =
            self.assign_word_halves_dense(region_index, h_row, a_7, h_row + 1, a_7, iv[7])?;
        let h = RoundWordDense::new(h_dense);

        // Decompose E into (6, 5, 14, 7)-bit chunks
        let e = self.decompose_e(region_index, idx, iv[4])?;

        // Decompose F, G
        let f = self.decompose_f(region_index, idx, iv[5])?;
        let g = self.decompose_g(region_index, idx, iv[6])?;

        // Decompose A into (2, 11, 9, 10)-bit chunks
        let a = self.decompose_a(region_index, idx, iv[0])?;

        // Decompose B, C
        let b = self.decompose_b(region_index, idx, iv[1])?;
        let c = self.decompose_c(region_index, idx, iv[2])?;

        Ok((
            State::new(
                StateWord::A(a),
                StateWord::B(b),
                StateWord::C(c),
                StateWord::D(d),
                StateWord::E(e),
                StateWord::F(f),
                StateWord::G(g),
                StateWord::H(h),
            ),
            region_index,
        ))
    }

    #[allow(clippy::many_single_char_names)]
    pub fn initialize_state(&mut self, state: State) -> Result<(State, RegionIndex), Error> {
        let configured = self.configured().clone();
        let a_7 = configured.extras[3];
        let (a, b, c, d, e, f, g, h) = match_state(state);

        let idx = -1;
        let d_row = get_d_row(idx);

        // Dummy assignment to get region index
        let region_index = self.layouter().assign_new_region(
            &configured.columns(),
            || "Dummy assignment",
            |mut region: Region<'_, Self>| {
                let cell = region.assign_advice(|| "Dummy", a_7, d_row, || Ok(F::zero()))?;
                Ok(cell.region_index())
            },
        )?;

        // Assign D
        let d = val_from_dense_halves(d.dense_halves);
        let d_dense = self.assign_word_halves_dense(region_index, d_row, a_7, d_row + 1, a_7, d)?;
        let d = RoundWordDense::new(d_dense);

        // Assign H
        let h = val_from_dense_halves(h.dense_halves);
        let h_row = get_h_row(idx);
        let h_dense = self.assign_word_halves_dense(region_index, h_row, a_7, h_row + 1, a_7, h)?;
        let h = RoundWordDense::new(h_dense);

        // Decompose E into (6, 5, 14, 7)-bit chunks
        let e = val_from_dense_halves(e.dense_halves);
        let e = self.decompose_e(region_index, idx, e)?;

        // Decompose F, G
        let f = val_from_dense_halves(f.dense_halves);
        let f = self.decompose_f(region_index, idx, f)?;
        let g = val_from_dense_halves(g.dense_halves);
        let g = self.decompose_g(region_index, idx, g)?;

        // Decompose A into (2, 11, 9, 10)-bit chunks
        let a = val_from_dense_halves(a.dense_halves);
        let a = self.decompose_a(region_index, idx, a)?;

        // Decompose B, C
        let b = val_from_dense_halves(b.dense_halves);
        let b = self.decompose_b(region_index, idx, b)?;
        let c = val_from_dense_halves(c.dense_halves);
        let c = self.decompose_c(region_index, idx, c)?;

        Ok((
            State::new(
                StateWord::A(a),
                StateWord::B(b),
                StateWord::C(c),
                StateWord::D(d),
                StateWord::E(e),
                StateWord::F(f),
                StateWord::G(g),
                StateWord::H(h),
            ),
            region_index,
        ))
    }

    fn decompose_b(
        &mut self,
        region_index: RegionIndex,
        idx: i32,
        b_val: u32,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_b_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region_index, row, b_val)?;
        self.decompose_abcd(region_index, row, b_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_c(
        &mut self,
        region_index: RegionIndex,
        idx: i32,
        c_val: u32,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_c_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region_index, row, c_val)?;
        self.decompose_abcd(region_index, row, c_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_f(
        &mut self,
        region_index: RegionIndex,
        idx: i32,
        f_val: u32,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_f_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region_index, row, f_val)?;
        self.decompose_efgh(region_index, row, f_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_g(
        &mut self,
        region_index: RegionIndex,
        idx: i32,
        g_val: u32,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_g_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region_index, row, g_val)?;
        self.decompose_efgh(region_index, row, g_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }
}
