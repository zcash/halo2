use std::convert::TryInto;

use super::{super::BLOCK_SIZE, util::*, BlockWord, SpreadInputs, SpreadWord, Table16Chip, ROUNDS};
use crate::{
    arithmetic::FieldExt,
    gadget::{Cell, Layouter, Permutation, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};

// mod schedule_gates;
// mod subregion1;
// mod subregion2;
// mod subregion3;

// use schedule_gates::ScheduleGate;

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

        // TODO: Create gates

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

        layouter.assign_region(|region| {
            let region = std::cell::RefCell::new(region);

            // TODO: Assign cells

            Ok(())
        })?;

        Ok(w.try_into().unwrap())
    }
}
