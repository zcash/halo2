use super::super::{
    CellValue16, RoundWordA, RoundWordE, StateWord, Table16Assignment, ROUND_CONSTANTS,
};
use super::{compression_util::*, CompressionConfig, State};
use halo2::{arithmetic::FieldExt, circuit::Region, plonk::Error};

impl CompressionConfig {
    #[allow(clippy::many_single_char_names)]
    pub fn assign_round<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        idx: i32,
        state: State,
        schedule_word: (CellValue16, CellValue16),
    ) -> Result<State, Error> {
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];
        let a_7 = self.extras[3];

        let (a, b, c, d, e, f, g, h) = match_state(state);

        // s_upper_sigma_1(E)
        let sigma_1 = self.assign_upper_sigma_1(region, idx, e.pieces.unwrap())?;

        // Ch(E, F, G)
        let ch = self.assign_ch(region, idx, e.spread_halves.unwrap(), f.spread_halves)?;
        let ch_neg = self.assign_ch_neg(region, idx, e.spread_halves.unwrap(), g.spread_halves)?;

        // s_upper_sigma_0(A)
        let sigma_0 = self.assign_upper_sigma_0(region, idx, a.pieces.unwrap())?;

        // Maj(A, B, C)
        let maj = self.assign_maj(
            region,
            idx,
            a.spread_halves.unwrap(),
            b.spread_halves,
            c.spread_halves,
        )?;

        // H' = H + Ch(E, F, G) + s_upper_sigma_1(E) + K + W
        let h_prime = self.assign_h_prime(
            region,
            idx,
            h.dense_halves,
            ch,
            ch_neg,
            sigma_1,
            ROUND_CONSTANTS[idx as usize],
            schedule_word,
        )?;

        // E_new = H' + D
        let e_new_dense = self.assign_e_new(region, idx, d.dense_halves, h_prime)?;
        let e_new_val = val_from_dense_halves(e_new_dense);

        // A_new = H' + Maj(A, B, C) + sigma_0(A)
        let a_new_dense = self.assign_a_new(region, idx, maj, sigma_0, h_prime)?;
        let a_new_val = val_from_dense_halves(a_new_dense);

        if idx < 63 {
            // Assign and copy A_new
            let a_new_row = get_decompose_a_row(idx + 1);
            self.assign_and_constrain(region, || "a_new_lo", a_7, a_new_row, a_new_dense.0)?;
            self.assign_and_constrain(region, || "a_new_hi", a_7, a_new_row + 1, a_new_dense.1)?;

            // Assign and copy E_new
            let e_new_row = get_decompose_e_row(idx + 1);
            self.assign_and_constrain(region, || "e_new_lo", a_7, e_new_row, e_new_dense.0)?;
            self.assign_and_constrain(region, || "e_new_hi", a_7, e_new_row + 1, e_new_dense.1)?;

            // Decompose A into (2, 11, 9, 10)-bit chunks
            let a_new = self.decompose_a(region, idx + 1, a_new_val)?;

            // Decompose E into (6, 5, 14, 7)-bit chunks
            let e_new = self.decompose_e(region, idx + 1, e_new_val)?;

            Ok(State::new(
                StateWord::A(a_new),
                StateWord::B(a.into()),
                StateWord::C(b),
                StateWord::D(c.into()),
                StateWord::E(e_new),
                StateWord::F(e.into()),
                StateWord::G(f),
                StateWord::H(g.into()),
            ))
        } else {
            let abcd_row = get_digest_abcd_row();
            let efgh_row = get_digest_efgh_row();

            let a_final =
                self.assign_word_halves_dense(region, abcd_row, a_3, abcd_row, a_4, a_new_val)?;

            let e_final =
                self.assign_word_halves_dense(region, efgh_row, a_3, efgh_row, a_4, e_new_val)?;

            Ok(State::new(
                StateWord::A(RoundWordA::new_dense(a_final)),
                StateWord::B(a.into()),
                StateWord::C(b),
                StateWord::D(c.into()),
                StateWord::E(RoundWordE::new_dense(e_final)),
                StateWord::F(e.into()),
                StateWord::G(f),
                StateWord::H(g.into()),
            ))
        }
    }
}
