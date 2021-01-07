use std::convert::TryInto;

use super::{super::BLOCK_SIZE, BlockWord, Gate, SpreadInputs, SpreadWord, Table16Chip, ROUNDS};
use crate::{
    arithmetic::FieldExt,
    gadget::{Cell, Layouter, Permutation, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};

mod schedule_gates;
mod schedule_util;
mod subregion1;
mod subregion2;
mod subregion3;

use schedule_gates::ScheduleGate;
use schedule_util::*;

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

    // Assign a word and its hi and lo halves
    pub fn assign_word_and_halves<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        word: u32,
        word_idx: usize,
    ) -> Result<(Cell, MessagePiece, MessagePiece), Error> {
        // Rename these here for ease of matching the gates to the specification.
        let a_3 = self.extras[0];
        let a_4 = self.extras[1];

        let row = get_word_row(word_idx);

        let var =
            region.assign_advice(self.message_schedule, row, || Ok(F::from_u64(word as u64)))?;

        let w_lo = word as u16;
        let w_hi = (word >> 16) as u16;

        let w_lo_cell = region.assign_advice(a_3, row, || Ok(F::from_u64(w_lo as u64)))?;
        let w_hi_cell = region.assign_advice(a_4, row, || Ok(F::from_u64(w_hi as u64)))?;

        Ok((
            var,
            MessagePiece::new(w_lo_cell, w_lo as u32),
            MessagePiece::new(w_hi_cell, w_hi as u32),
        ))
    }

    // Assign a spread lookup
    pub fn assign_lookup<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        spread_word: &SpreadWord,
        row: usize,
    ) -> Result<(Cell, Cell), Error> {
        // Rename these here for ease of matching the gates to the specification.
        let a_0 = self.lookup.tag;
        let a_1 = self.lookup.dense;
        let a_2 = self.lookup.spread;

        region.assign_advice(a_0, row, || Ok(F::from_u64(spread_word.tag as u64)))?;
        let dense = region.assign_advice(a_1, row, || Ok(F::from_u64(spread_word.dense as u64)))?;
        let spread =
            region.assign_advice(a_2, row, || Ok(F::from_u64(spread_word.spread as u64)))?;
        Ok((dense, spread))
    }

    // Assign a cell the same value as another cell and set up a copy constraint between them
    pub fn assign_and_constrain<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        column: Column<Advice>,
        row: usize,
        copy: &MessagePiece,
        perm: &Permutation,
    ) -> Result<(), Error> {
        let cell =
            region.assign_advice(column, row, || Ok(F::from_u64(copy.value.unwrap() as u64)))?;
        region.constrain_equal(perm, cell, copy.var)
    }

    pub(super) fn process<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        input: [BlockWord; BLOCK_SIZE],
    ) -> Result<
        (
            [MessageWord; ROUNDS],
            [(MessagePiece, MessagePiece); ROUNDS],
        ),
        Error,
    > {
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

            // Assign W[0..16]
            for i in 0..16 {
                let (var, lo, hi) =
                    self.assign_word_and_halves(&mut region, input[i].value.unwrap(), i)?;
                w.push(MessageWord {
                    var,
                    value: input[i].value,
                });
                w_halves.push((lo, hi));
            }

            // Returns the output of sigma_0 on W_[1..14]
            let lower_sigma_0_output = self.assign_subregion1(&mut region, &input[1..14])?;

            // sigma_0_v2 and sigma_1_v2 on W_[14..49]
            // Returns the output of sigma_0_v2 on W_[36..49], to be used in subregion3
            let lower_sigma_0_v2_output =
                self.assign_subregion2(&mut region, lower_sigma_0_output, &mut w, &mut w_halves)?;

            // sigma_1 v1 on W[49..62]
            self.assign_subregion3(&mut region, lower_sigma_0_v2_output, &mut w, &mut w_halves)?;

            Ok(())
        })?;

        Ok((w.try_into().unwrap(), w_halves.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;
    use std::fmt;
    use std::marker::PhantomData;

    use super::super::{
        super::BLOCK_SIZE, BlockWord, Compression, SpreadInputs, SpreadTable, Table16Chip,
        Table16Config,
    };
    use super::{schedule_util::*, MessageSchedule};
    use crate::{
        arithmetic::FieldExt,
        dev::MockProver,
        gadget::{Cell, DynRegion, Layouter, Permutation, Region},
        pasta::Fp,
        plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed},
    };

    #[test]
    fn message_schedule() {
        /// This represents an advice column at a certain row in the ConstraintSystem
        #[derive(Copy, Clone, Debug)]
        pub struct Variable(Column<Advice>, usize);

        #[derive(Debug)]
        struct MyConfig {
            lookup_inputs: SpreadInputs,
            sha256: Table16Config,
        }

        struct MyCircuit {}

        struct MyLayouter<'a, F: FieldExt, CS: Assignment<F> + 'a> {
            cs: &'a mut CS,
            config: MyConfig,
            regions: Vec<usize>,
            current_gate: usize,
            _marker: PhantomData<F>,
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyLayouter<'a, F, CS> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("MyLayouter")
                    .field("config", &self.config)
                    .field("regions", &self.regions)
                    .field("current_gate", &self.current_gate)
                    .finish()
            }
        }

        impl<'a, FF: FieldExt, CS: Assignment<FF>> MyLayouter<'a, FF, CS> {
            fn new(cs: &'a mut CS, config: MyConfig) -> Result<Self, Error> {
                let mut res = MyLayouter {
                    cs,
                    config,
                    regions: vec![],
                    current_gate: 0,
                    _marker: PhantomData,
                };

                let table = res.config.sha256.lookup_table.clone();
                table.load(&mut res)?;

                Ok(res)
            }
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Layouter<Table16Chip<F>> for MyLayouter<'a, F, CS> {
            fn config(&self) -> &Table16Config {
                &self.config.sha256
            }

            fn assign_region(
                &mut self,
                assignment: impl FnOnce(Region<'_, Table16Chip<F>>) -> Result<(), Error>,
            ) -> Result<(), Error> {
                let region_index = self.regions.len();
                self.regions.push(self.current_gate);

                let mut region = MyRegion::new(self, region_index);
                assignment(Region {
                    region: &mut region,
                })?;
                self.current_gate += region.row_count;

                Ok(())
            }
        }

        struct MyRegion<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> {
            layouter: &'r mut MyLayouter<'a, F, CS>,
            region_index: usize,
            row_count: usize,
            _marker: PhantomData<F>,
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyRegion<'r, 'a, F, CS> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("MyRegion")
                    .field("layouter", &self.layouter)
                    .field("region_index", &self.region_index)
                    .field("row_count", &self.row_count)
                    .finish()
            }
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> MyRegion<'r, 'a, F, CS> {
            fn new(layouter: &'r mut MyLayouter<'a, F, CS>, region_index: usize) -> Self {
                MyRegion {
                    layouter,
                    region_index,
                    row_count: 0,
                    _marker: PhantomData::default(),
                }
            }
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> DynRegion<Table16Chip<F>>
            for MyRegion<'r, 'a, F, CS>
        {
            fn assign_advice<'v>(
                &'v mut self,
                column: Column<Advice>,
                offset: usize,
                to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
            ) -> Result<Cell, Error> {
                self.layouter.cs.assign_advice(
                    column,
                    self.layouter.regions[self.region_index] + offset,
                    to,
                )?;
                self.row_count = cmp::max(self.row_count, offset);

                Ok(Cell {
                    region_index: self.region_index,
                    row_offset: offset,
                    column: column.into(),
                })
            }

            fn assign_fixed<'v>(
                &'v mut self,
                column: Column<Fixed>,
                offset: usize,
                to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
            ) -> Result<Cell, Error> {
                self.layouter.cs.assign_fixed(
                    column,
                    self.layouter.regions[self.region_index] + offset,
                    to,
                )?;
                self.row_count = cmp::max(self.row_count, offset);
                Ok(Cell {
                    region_index: self.region_index,
                    row_offset: offset,
                    column: column.into(),
                })
            }

            fn constrain_equal(
                &mut self,
                permutation: &Permutation,
                left: Cell,
                right: Cell,
            ) -> Result<(), Error> {
                let left_column = permutation
                    .mapping
                    .iter()
                    .position(|c| c == &left.column)
                    .ok_or(Error::SynthesisError)?;
                let right_column = permutation
                    .mapping
                    .iter()
                    .position(|c| c == &right.column)
                    .ok_or(Error::SynthesisError)?;

                self.layouter.cs.copy(
                    permutation.index,
                    left_column,
                    self.layouter.regions[left.region_index] + left.row_offset,
                    right_column,
                    self.layouter.regions[right.region_index] + right.row_offset,
                )?;

                Ok(())
            }
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Config = MyConfig;

            fn configure(meta: &mut ConstraintSystem<F>) -> MyConfig {
                let a = meta.advice_column();
                let b = meta.advice_column();
                let c = meta.advice_column();

                let (lookup_inputs, lookup_table) = SpreadTable::configure(meta, a, b, c);

                let message_schedule = meta.advice_column();
                let extras = [
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                ];

                let compression =
                    Compression::configure(meta, lookup_inputs.clone(), message_schedule, extras);

                let message_schedule = MessageSchedule::configure(
                    meta,
                    lookup_inputs.clone(),
                    message_schedule,
                    extras,
                );

                MyConfig {
                    lookup_inputs,
                    sha256: Table16Config {
                        lookup_table,
                        message_schedule,
                        compression,
                    },
                }
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: MyConfig,
            ) -> Result<(), Error> {
                let lookup = config.lookup_inputs.clone();
                let mut layouter = MyLayouter::new(cs, config)?;

                // layouter.assign_region()

                // Provide input
                // Test vector: "abc"
                let inputs: [BlockWord; BLOCK_SIZE] = get_msg_schedule_test_input();

                // Run message_scheduler to get W_[0..64]
                let message_schedule = layouter.config.sha256.message_schedule.clone();
                let (w, w_halves) = message_schedule.process(&mut layouter, inputs)?;
                for (word, test_word) in w.iter().zip(MSG_SCHEDULE_TEST_OUTPUT.iter()) {
                    let word = word.value.unwrap();
                    assert_eq!(word, *test_word);
                }
                Ok(())
            }
        }

        let circuit: MyCircuit = MyCircuit {};

        // Use k = 17 because the current layouter doesn't place tables next to regions.
        let prover = match MockProver::<Fp>::run(17, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
