use std::convert::TryInto;

use super::{
    super::BLOCK_SIZE, util::*, BlockWord, Gate, SpreadInputs, SpreadWord, Table16Chip, ROUNDS,
};
use crate::{
    arithmetic::FieldExt,
    gadget::{Cell, Layouter, Permutation, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};

mod schedule_gates;
// mod subregion1;
// mod subregion2;
// mod subregion3;

use schedule_gates::ScheduleGate;

#[derive(Clone, Debug)]
pub(super) struct MessageWord {
    var: Cell,
    value: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MessagePiece {
    pub var: Cell,
    pub value: Option<u32>,
}

impl MessagePiece {
    pub fn new(var: Cell, value: u32) -> Self {
        MessagePiece {
            var,
            value: Some(value),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct MessageSchedule {
    lookup: SpreadInputs,
    message_schedule: Column<Advice>,
    extras: [Column<Advice>; 6],

    /// Construct a word using reduce_4.
    s_word: Column<Fixed>,
    /// Decomposition gate for W_0, W_62, W_63.
    s_decompose_0: Column<Fixed>,
    /// Decomposition gate for W_[1..14]
    s_decompose_1: Column<Fixed>,
    /// Decomposition gate for W_[14..49]
    s_decompose_2: Column<Fixed>,
    /// Decomposition gate for W_[49..62]
    s_decompose_3: Column<Fixed>,
    /// sigma_0 gate for W_[1..14]
    s_lower_sigma_0: Column<Fixed>,
    /// sigma_1 gate for W_[49..62]
    s_lower_sigma_1: Column<Fixed>,
    /// sigma_0_v2 gate for W_[14..49]
    s_lower_sigma_0_v2: Column<Fixed>,
    /// sigma_1_v2 gate for W_[14..49]
    s_lower_sigma_1_v2: Column<Fixed>,
    perm: Permutation,
}

impl MessageSchedule {
    /// Configures the message schedule.
    ///
    /// `message_schedule` is the column into which the message schedule will be placed.
    /// The caller must create appropriate permutations in order to load schedule words
    /// into the compression rounds.
    ///
    /// `extras` contains columns that the message schedule will only use for internal
    /// gates, and will not place any constraints on (such as lookup constraints) outside
    /// itself.
    pub(super) fn configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        lookup: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
    ) -> Self {
        // Create fixed columns for the selectors we will require.
        let s_word = meta.fixed_column();
        let s_decompose_0 = meta.fixed_column();
        let s_decompose_1 = meta.fixed_column();
        let s_decompose_2 = meta.fixed_column();
        let s_decompose_3 = meta.fixed_column();
        let s_lower_sigma_0 = meta.fixed_column();
        let s_lower_sigma_1 = meta.fixed_column();
        let s_lower_sigma_0_v2 = meta.fixed_column();
        let s_lower_sigma_1_v2 = meta.fixed_column();

        // Rename these here for ease of matching the gates to the specification.
        let a_0 = lookup.tag;
        let a_1 = lookup.dense;
        let a_2 = lookup.spread;
        let a_3 = extras[0];
        let a_4 = extras[1];
        let a_5 = message_schedule;
        let a_6 = extras[2];
        let a_7 = extras[3];
        let a_8 = extras[4];
        let a_9 = extras[5];

        // Set up permutations
        let perm = Permutation::new(meta, &[a_1, a_2, a_3, a_4, a_5, a_6, a_7, a_8]);

        // s_word for W_[16..64]
        meta.create_gate(|meta| {
            let s_word = meta.query_fixed(s_word, 0);

            let sigma_0_lo = meta.query_advice(a_6, -1);
            let sigma_0_hi = meta.query_advice(a_6, 0);

            let sigma_1_lo = meta.query_advice(a_7, -1);
            let sigma_1_hi = meta.query_advice(a_7, 0);

            let w_minus_9_lo = meta.query_advice(a_8, -1);
            let w_minus_9_hi = meta.query_advice(a_8, 0);

            let w_minus_16_lo = meta.query_advice(a_3, -1);
            let w_minus_16_hi = meta.query_advice(a_4, -1);

            let word = meta.query_advice(a_5, 0);
            let carry = meta.query_advice(a_9, 0);

            ScheduleGate::s_word(
                s_word,
                sigma_0_lo,
                sigma_0_hi,
                sigma_1_lo,
                sigma_1_hi,
                w_minus_9_lo,
                w_minus_9_hi,
                w_minus_16_lo,
                w_minus_16_hi,
                word,
                carry,
            )
            .0
        });

        // s_decompose_0 for all words
        meta.create_gate(|meta| {
            let s_decompose_0 = meta.query_fixed(s_decompose_0, 0);
            let lo = meta.query_advice(a_3, 0);
            let hi = meta.query_advice(a_4, 0);
            let word = meta.query_advice(a_5, 0);

            ScheduleGate::s_decompose_0(s_decompose_0, lo, hi, word).0
        });

        // s_decompose_1 for W_[1..14]
        // (3, 4, 11, 14)-bit chunks
        meta.create_gate(|meta| {
            let s_decompose_1 = meta.query_fixed(s_decompose_1, 0);
            let a = meta.query_advice(a_3, 1); // 3-bit chunk
            let b = meta.query_advice(a_4, 1); // 4-bit chunk
            let c = meta.query_advice(a_1, 1); // 11-bit chunk
            let d = meta.query_advice(a_1, 0); // 14-bit chunk
            let word = meta.query_advice(a_5, 0);

            ScheduleGate::s_decompose_1(s_decompose_1, a, b, c, d, word).0
        });

        // s_decompose_2 for W_[14..49]
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        meta.create_gate(|meta| {
            let s_decompose_2 = meta.query_fixed(s_decompose_2, 0);
            let a = meta.query_advice(a_3, -1); // 3-bit chunk
            let b = meta.query_advice(a_1, 1); // 4-bit chunk
            let c = meta.query_advice(a_4, -1); // 3-bit chunk
            let d = meta.query_advice(a_1, 0); // 7-bit chunk
            let e = meta.query_advice(a_3, 1); // 1-bit chunk
            let f = meta.query_advice(a_4, 1); // 1-bit chunk
            let g = meta.query_advice(a_1, -1); // 13-bit chunk
            let word = meta.query_advice(a_5, 0);

            ScheduleGate::s_decompose_2(s_decompose_2, a, b, c, d, e, f, g, word).0
        });

        // s_decompose_3 for W_49 to W_61
        // (10, 7, 2, 13)-bit chunks
        meta.create_gate(|meta| {
            let s_decompose_3 = meta.query_fixed(s_decompose_3, 0);
            let a = meta.query_advice(a_1, 1); // 10-bit chunk
            let b = meta.query_advice(a_4, 1); // 7-bit chunk
            let c = meta.query_advice(a_3, 1); // 2-bit chunk
            let d = meta.query_advice(a_1, 0); // 13-bit chunk
            let word = meta.query_advice(a_5, 0);

            ScheduleGate::s_decompose_3(s_decompose_3, a, b, c, d, word).0
        });

        // sigma_0 v1 on W_[1..14]
        // (3, 4, 11, 14)-bit chunks
        meta.create_gate(|meta| {
            ScheduleGate::s_lower_sigma_0(
                meta.query_fixed(s_lower_sigma_0, 0), // s_lower_sigma_0
                meta.query_advice(a_2, -1),           // spread_r0_even
                meta.query_advice(a_2, 0),            // spread_r0_odd
                meta.query_advice(a_2, 1),            // spread_r1_even
                meta.query_advice(a_3, 0),            // spread_r1_odd
                meta.query_advice(a_5, 1),            // a
                meta.query_advice(a_6, 1),            // spread_a
                meta.query_advice(a_6, 0),            // b
                meta.query_advice(a_3, -1),           // b_lo
                meta.query_advice(a_4, -1),           // spread_b_lo
                meta.query_advice(a_5, -1),           // b_hi
                meta.query_advice(a_6, -1),           // spread_b_hi
                meta.query_advice(a_4, 0),            // spread_c
                meta.query_advice(a_5, 0),            // spread_d
            )
            .0
        });

        // sigma_0 v2 on W_[14..49]
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        meta.create_gate(|meta| {
            ScheduleGate::s_lower_sigma_0_v2(
                meta.query_fixed(s_lower_sigma_0_v2, 0), // s_lower_sigma_0_v2
                meta.query_advice(a_2, -1),              // spread_r0_even
                meta.query_advice(a_2, 0),               // spread_r0_odd
                meta.query_advice(a_2, 1),               // spread_r1_even
                meta.query_advice(a_3, 0),               // spread_r1_odd
                meta.query_advice(a_3, 1),               // a
                meta.query_advice(a_4, 1),               // spread_a
                meta.query_advice(a_6, 0),               // b
                meta.query_advice(a_3, -1),              // b_lo
                meta.query_advice(a_4, -1),              // spread_b_lo
                meta.query_advice(a_5, -1),              // b_hi
                meta.query_advice(a_6, -1),              // spread_b_hi
                meta.query_advice(a_5, 1),               // c
                meta.query_advice(a_6, 1),               // spread_c
                meta.query_advice(a_4, 0),               // spread_d
                meta.query_advice(a_7, 0),               // spread_e
                meta.query_advice(a_7, 1),               // spread_f
                meta.query_advice(a_5, 0),               // spread_g
            )
            .0
        });

        // sigma_1 v2 on W_14 to W_48
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        meta.create_gate(|meta| {
            ScheduleGate::s_lower_sigma_1_v2(
                meta.query_fixed(s_lower_sigma_1_v2, 0), // s_lower_sigma_1_v2
                meta.query_advice(a_2, -1),              // spread_r0_even
                meta.query_advice(a_2, 0),               // spread_r0_odd
                meta.query_advice(a_2, 1),               // spread_r1_even
                meta.query_advice(a_3, 0),               // spread_r1_odd
                meta.query_advice(a_3, 1),               // a
                meta.query_advice(a_4, 1),               // spread_a
                meta.query_advice(a_6, 0),               // b
                meta.query_advice(a_3, -1),              // b_lo
                meta.query_advice(a_4, -1),              // spread_b_lo
                meta.query_advice(a_5, -1),              // b_hi
                meta.query_advice(a_6, -1),              // spread_b_hi
                meta.query_advice(a_5, 1),               // c
                meta.query_advice(a_6, 1),               // spread_c
                meta.query_advice(a_4, 0),               // spread_d
                meta.query_advice(a_7, 0),               // spread_e
                meta.query_advice(a_7, 1),               // spread_f
                meta.query_advice(a_5, 0),               // spread_g
            )
            .0
        });

        // sigma_1 v1 on W_49 to W_61
        // (10, 7, 2, 13)-bit chunks
        meta.create_gate(|meta| {
            ScheduleGate::s_lower_sigma_1(
                meta.query_fixed(s_lower_sigma_1, 0), // s_lower_sigma_1
                meta.query_advice(a_2, -1),           // spread_r0_even
                meta.query_advice(a_2, 0),            // spread_r0_odd
                meta.query_advice(a_2, 1),            // spread_r1_even
                meta.query_advice(a_3, 0),            // spread_r1_odd
                meta.query_advice(a_4, 0),            // spread_a
                meta.query_advice(a_6, 0),            // b
                meta.query_advice(a_3, -1),           // b_lo
                meta.query_advice(a_4, -1),           // spread_b_lo
                meta.query_advice(a_5, -1),           // b_mid
                meta.query_advice(a_6, -1),           // spread_b_mid
                meta.query_advice(a_5, 1),            // b_hi
                meta.query_advice(a_6, 1),            // spread_b_hi
                meta.query_advice(a_3, 1),            // c
                meta.query_advice(a_4, 1),            // spread_c
                meta.query_advice(a_5, 0),            // spread_d
            )
            .0
        });

        MessageSchedule {
            lookup,
            message_schedule,
            extras,
            s_word,
            s_decompose_0,
            s_decompose_1,
            s_decompose_2,
            s_decompose_3,
            s_lower_sigma_0,
            s_lower_sigma_1,
            s_lower_sigma_0_v2,
            s_lower_sigma_1_v2,
            perm,
        }
    }

    pub(super) fn process<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        input: [BlockWord; BLOCK_SIZE],
    ) -> Result<[MessageWord; ROUNDS], Error> {
        let mut w = Vec::<MessageWord>::with_capacity(ROUNDS);
        let mut w_halves = Vec::<(MessagePiece, MessagePiece)>::with_capacity(ROUNDS);

        layouter.assign_region(|mut region| {
            // Assign all fixed columns
            for index in 1..14 {
                let row = get_word_row(index);
                region.assign_fixed(self.s_decompose_1, row, || Ok(F::one()))?;
                region.assign_fixed(self.s_lower_sigma_0, row + 3, || Ok(F::one()))?;
            }

            for index in 14..49 {
                let row = get_word_row(index);
                region.assign_fixed(self.s_decompose_2, row, || Ok(F::one()))?;
                region.assign_fixed(self.s_lower_sigma_0_v2, row + 3, || Ok(F::one()))?;
                region.assign_fixed(self.s_lower_sigma_1_v2, row + SIGMA_0_V2_ROWS + 3, || {
                    Ok(F::one())
                })?;

                let new_word_idx = index + 2;
                region.assign_fixed(self.s_word, get_word_row(new_word_idx - 16) + 1, || {
                    Ok(F::one())
                })?;
            }

            for index in 49..62 {
                let row = get_word_row(index);
                region.assign_fixed(self.s_decompose_3, row, || Ok(F::one()))?;
                region.assign_fixed(self.s_lower_sigma_1, row + 3, || Ok(F::one()))?;

                let new_word_idx = index + 2;
                region.assign_fixed(self.s_word, get_word_row(new_word_idx - 16) + 1, || {
                    Ok(F::one())
                })?;
            }

            for index in 0..64 {
                let row = get_word_row(index);
                region.assign_fixed(self.s_decompose_0, row, || Ok(F::one()))?;
            }

            // TODO: Assign advice columns

            Ok(())
        })?;

        Ok(w.try_into().unwrap())
    }
}
