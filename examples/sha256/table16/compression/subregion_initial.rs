use super::super::{RoundWordDense, RoundWordSpread, StateWord, STATE};
use super::{compression_util::*, CompressionConfig, State};
use halo2::{arithmetic::FieldExt, circuit::Region, plonk::Error};

impl CompressionConfig {
    #[allow(clippy::many_single_char_names)]
    pub fn initialize_iv<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        iv: [u32; STATE],
    ) -> Result<State, Error> {
        let a_7 = self.extras[3];

        let idx = -1;

        // Decompose E into (6, 5, 14, 7)-bit chunks
        let e = self.decompose_e(region, idx, Some(iv[4]))?;

        // Decompose F, G
        let f = self.decompose_f(region, idx, Some(iv[5]))?;
        let g = self.decompose_g(region, idx, Some(iv[6]))?;

        // Assign H
        let h_row = get_h_row(idx);
        let h_dense =
            self.assign_word_halves_dense(region, h_row, a_7, h_row + 1, a_7, Some(iv[7]))?;
        let h = RoundWordDense::new(h_dense);

        // Decompose A into (2, 11, 9, 10)-bit chunks
        let a = self.decompose_a(region, idx, Some(iv[0]))?;

        // Decompose B, C
        let b = self.decompose_b(region, idx, Some(iv[1]))?;
        let c = self.decompose_c(region, idx, Some(iv[2]))?;

        // Assign D
        let d_row = get_d_row(idx);
        let d_dense =
            self.assign_word_halves_dense(region, d_row, a_7, d_row + 1, a_7, Some(iv[3]))?;
        let d = RoundWordDense::new(d_dense);

        Ok(State::new(
            StateWord::A(a),
            StateWord::B(b),
            StateWord::C(c),
            StateWord::D(d),
            StateWord::E(e),
            StateWord::F(f),
            StateWord::G(g),
            StateWord::H(h),
        ))
    }

    #[allow(clippy::many_single_char_names)]
    pub fn initialize_state<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        state: State,
    ) -> Result<State, Error> {
        let a_7 = self.extras[3];
        let (a, b, c, d, e, f, g, h) = match_state(state);

        let idx = -1;

        // Decompose E into (6, 5, 14, 7)-bit chunks
        let e = val_from_dense_halves(e.dense_halves);
        let e = self.decompose_e(region, idx, e)?;

        // Decompose F, G
        let f = val_from_dense_halves(f.dense_halves);
        let f = self.decompose_f(region, idx, f)?;
        let g = val_from_dense_halves(g.dense_halves);
        let g = self.decompose_g(region, idx, g)?;

        // Assign H
        let h = val_from_dense_halves(h.dense_halves);
        let h_row = get_h_row(idx);
        let h_dense = self.assign_word_halves_dense(region, h_row, a_7, h_row + 1, a_7, h)?;
        let h = RoundWordDense::new(h_dense);

        // Decompose A into (2, 11, 9, 10)-bit chunks
        let a = val_from_dense_halves(a.dense_halves);
        let a = self.decompose_a(region, idx, a)?;

        // Decompose B, C
        let b = val_from_dense_halves(b.dense_halves);
        let b = self.decompose_b(region, idx, b)?;
        let c = val_from_dense_halves(c.dense_halves);
        let c = self.decompose_c(region, idx, c)?;

        // Assign D
        let d = val_from_dense_halves(d.dense_halves);
        let d_row = get_d_row(idx);
        let d_dense = self.assign_word_halves_dense(region, d_row, a_7, d_row + 1, a_7, d)?;
        let d = RoundWordDense::new(d_dense);

        Ok(State::new(
            StateWord::A(a),
            StateWord::B(b),
            StateWord::C(c),
            StateWord::D(d),
            StateWord::E(e),
            StateWord::F(f),
            StateWord::G(g),
            StateWord::H(h),
        ))
    }

    fn decompose_b<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        idx: i32,
        b_val: Option<u32>,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_b_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region, row, b_val)?;
        self.decompose_abcd(region, row, b_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_c<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        idx: i32,
        c_val: Option<u32>,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_c_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region, row, c_val)?;
        self.decompose_abcd(region, row, c_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_f<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        idx: i32,
        f_val: Option<u32>,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_f_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region, row, f_val)?;
        self.decompose_efgh(region, row, f_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }

    fn decompose_g<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        idx: i32,
        g_val: Option<u32>,
    ) -> Result<RoundWordSpread, Error> {
        let row = get_decompose_g_row(idx);

        let (dense_halves, spread_halves) = self.assign_word_halves(region, row, g_val)?;
        self.decompose_efgh(region, row, g_val)?;
        Ok(RoundWordSpread::new(dense_halves, spread_halves))
    }
}
