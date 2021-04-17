use std::convert::TryInto;

use super::{super::BLOCK_SIZE, BlockWord, CellValue16, SpreadInputs, Table16Assignment, ROUNDS};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Core, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation},
    poly::Rotation,
};

mod schedule_gates;
mod schedule_util;
mod subregion1;
mod subregion2;
mod subregion3;

use schedule_gates::ScheduleGate;
use schedule_util::*;

#[cfg(test)]
pub use schedule_util::get_msg_schedule_test_input;

#[derive(Clone, Debug)]
pub(super) struct MessageWord {
    var: Cell,
    value: Option<u32>,
}

impl<F: FieldExt, C: Core<F>> Table16Assignment<F, C> for MessageScheduleConfig {}

#[derive(Clone, Debug)]
pub(super) struct MessageScheduleConfig {
    pub lookup_inputs: SpreadInputs,
    pub message_schedule: Column<Advice>,
    pub extras: [Column<Advice>; 6],

    /// Construct a word using reduce_4.
    pub s_word: Column<Fixed>,
    /// Decomposition gate for W_0, W_62, W_63.
    pub s_decompose_0: Column<Fixed>,
    /// Decomposition gate for W_[1..14]
    pub s_decompose_1: Column<Fixed>,
    /// Decomposition gate for W_[14..49]
    pub s_decompose_2: Column<Fixed>,
    /// Decomposition gate for W_[49..62]
    pub s_decompose_3: Column<Fixed>,
    /// sigma_0 gate for W_[1..14]
    pub s_lower_sigma_0: Column<Fixed>,
    /// sigma_1 gate for W_[49..62]
    pub s_lower_sigma_1: Column<Fixed>,
    /// sigma_0_v2 gate for W_[14..49]
    pub s_lower_sigma_0_v2: Column<Fixed>,
    /// sigma_1_v2 gate for W_[14..49]
    pub s_lower_sigma_1_v2: Column<Fixed>,
    pub perm: Permutation,
}

pub(super) struct MessageScheduleCore<'a, F: FieldExt, L: Layouter<F>> {
    pub config: MessageScheduleConfig,
    pub layouter: &'a mut L,
    pub marker: std::marker::PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Core<F> for MessageScheduleCore<'_, F, L> {
    type Config = MessageScheduleConfig;
    type Loaded = ();
    type Layouter = L;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this chip have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        self.layouter
    }
}

impl<F: FieldExt, L: Layouter<F>> MessageScheduleCore<'_, F, L> {
    /// Configures the message schedule.
    ///
    /// `message_schedule` is the column into which the message schedule will be placed.
    /// The caller must create appropriate permutations in order to load schedule words
    /// into the compression rounds.
    ///
    /// `extras` contains columns that the message schedule will only use for internal
    /// gates, and will not place any constraints on (such as lookup constraints) outside
    /// itself.
    #[allow(clippy::many_single_char_names)]
    pub(super) fn configure(
        meta: &mut ConstraintSystem<F>,
        lookup_inputs: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
        perm: Permutation,
    ) -> MessageScheduleConfig {
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
        let a_0 = lookup_inputs.tag;
        let a_1 = lookup_inputs.dense;
        let a_2 = lookup_inputs.spread;
        let a_3 = extras[0];
        let a_4 = extras[1];
        let a_5 = message_schedule;
        let a_6 = extras[2];
        let a_7 = extras[3];
        let a_8 = extras[4];
        let a_9 = extras[5];

        // s_word for W_[16..64]
        meta.create_gate("s_word for W_[16..64]", |meta| {
            ScheduleGate::s_word(
                meta.query_fixed(s_word, Rotation::cur()), // s_word
                meta.query_advice(a_6, Rotation::prev()),  // sigma_0_lo
                meta.query_advice(a_6, Rotation::cur()),   // sigma_0_hi
                meta.query_advice(a_7, Rotation::prev()),  // sigma_1_lo
                meta.query_advice(a_7, Rotation::cur()),   // sigma_1_hi
                meta.query_advice(a_8, Rotation::prev()),  // w_minus_9_lo
                meta.query_advice(a_8, Rotation::cur()),   // w_minus_9_hi
                meta.query_advice(a_3, Rotation::prev()),  // w_minus_16_lo
                meta.query_advice(a_4, Rotation::prev()),  // w_minus_16_hi
                meta.query_advice(a_5, Rotation::cur()),   // word
                meta.query_advice(a_9, Rotation::cur()),   // carry
            )
            .0
        });

        // s_decompose_0 for all words
        meta.create_gate("s_decompose_0", |meta| {
            ScheduleGate::s_decompose_0(
                meta.query_fixed(s_decompose_0, Rotation::cur()), //s_decompose_0
                meta.query_advice(a_3, Rotation::cur()),          //lo
                meta.query_advice(a_4, Rotation::cur()),          //hi
                meta.query_advice(a_5, Rotation::cur()),          //word
            )
            .0
        });

        // s_decompose_1 for W_[1..14]
        // (3, 4, 11, 14)-bit chunks
        {
            let expressions = ScheduleGate::s_decompose_1(
                meta.query_fixed(s_decompose_1, Rotation::cur()), // s_decompose_1
                meta.query_advice(a_3, Rotation::next()),         // a (3-bit chunk)
                meta.query_advice(a_4, Rotation::next()),         // b (4-bit chunk)
                meta.query_advice(a_1, Rotation::next()),         // c (11-bit chunk)
                meta.query_advice(a_0, Rotation::next()),         // tag_c
                meta.query_advice(a_1, Rotation::cur()),          // d (14-bit chunk)
                meta.query_advice(a_0, Rotation::cur()),          // tag_d
                meta.query_advice(a_5, Rotation::cur()),          // word
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_decompose_1", |_| expr.0)
            }
        }

        // s_decompose_2 for W_[14..49]
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        {
            let expressions = ScheduleGate::s_decompose_2(
                meta.query_fixed(s_decompose_2, Rotation::cur()), // s_decompose_2
                meta.query_advice(a_3, Rotation::prev()),         // a
                meta.query_advice(a_1, Rotation::next()),         // b
                meta.query_advice(a_4, Rotation::prev()),         // c
                meta.query_advice(a_1, Rotation::cur()),          // d
                meta.query_advice(a_0, Rotation::cur()),          // tag_d
                meta.query_advice(a_3, Rotation::next()),         // e
                meta.query_advice(a_4, Rotation::next()),         // f
                meta.query_advice(a_1, Rotation::prev()),         // g
                meta.query_advice(a_0, Rotation::prev()),         // tag_g
                meta.query_advice(a_5, Rotation::cur()),          // word
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_decompose_2", |_| expr.0)
            }
        }

        // s_decompose_3 for W_49 to W_61
        // (10, 7, 2, 13)-bit chunks
        {
            let expressions = ScheduleGate::s_decompose_3(
                meta.query_fixed(s_decompose_3, Rotation::cur()), // s_decompose_3
                meta.query_advice(a_1, Rotation::next()),         // a
                meta.query_advice(a_0, Rotation::next()),         // tag_a
                meta.query_advice(a_4, Rotation::next()),         // b
                meta.query_advice(a_3, Rotation::next()),         // c
                meta.query_advice(a_1, Rotation::cur()),          // d
                meta.query_advice(a_0, Rotation::cur()),          // tag_d
                meta.query_advice(a_5, Rotation::cur()),          // word
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_decompose_3", |_| expr.0)
            }
        }

        // sigma_0 v1 on W_[1..14]
        // (3, 4, 11, 14)-bit chunks
        {
            let expressions = ScheduleGate::s_lower_sigma_0(
                meta.query_fixed(s_lower_sigma_0, Rotation::cur()), // s_lower_sigma_0
                meta.query_advice(a_2, Rotation::prev()),           // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),            // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),           // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),            // spread_r1_odd
                meta.query_advice(a_5, Rotation::next()),           // a
                meta.query_advice(a_6, Rotation::next()),           // spread_a
                meta.query_advice(a_6, Rotation::cur()),            // b
                meta.query_advice(a_3, Rotation::prev()),           // b_lo
                meta.query_advice(a_4, Rotation::prev()),           // spread_b_lo
                meta.query_advice(a_5, Rotation::prev()),           // b_hi
                meta.query_advice(a_6, Rotation::prev()),           // spread_b_hi
                meta.query_advice(a_4, Rotation::cur()),            // spread_c
                meta.query_advice(a_5, Rotation::cur()),            // spread_d
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_lower_sigma_0", |_| expr.0)
            }
        }

        // sigma_0 v2 on W_[14..49]
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        {
            let expressions = ScheduleGate::s_lower_sigma_0_v2(
                meta.query_fixed(s_lower_sigma_0_v2, Rotation::cur()), // s_lower_sigma_0_v2
                meta.query_advice(a_2, Rotation::prev()),              // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),               // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),              // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),               // spread_r1_odd
                meta.query_advice(a_3, Rotation::next()),              // a
                meta.query_advice(a_4, Rotation::next()),              // spread_a
                meta.query_advice(a_6, Rotation::cur()),               // b
                meta.query_advice(a_3, Rotation::prev()),              // b_lo
                meta.query_advice(a_4, Rotation::prev()),              // spread_b_lo
                meta.query_advice(a_5, Rotation::prev()),              // b_hi
                meta.query_advice(a_6, Rotation::prev()),              // spread_b_hi
                meta.query_advice(a_5, Rotation::next()),              // c
                meta.query_advice(a_6, Rotation::next()),              // spread_c
                meta.query_advice(a_4, Rotation::cur()),               // spread_d
                meta.query_advice(a_7, Rotation::cur()),               // spread_e
                meta.query_advice(a_7, Rotation::next()),              // spread_f
                meta.query_advice(a_5, Rotation::cur()),               // spread_g
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_lower_sigma_0_v2", |_| expr.0)
            }
        }

        // sigma_1 v2 on W_14 to W_48
        // (3, 4, 3, 7, 1, 1, 13)-bit chunks
        {
            let expressions = ScheduleGate::s_lower_sigma_1_v2(
                meta.query_fixed(s_lower_sigma_1_v2, Rotation::cur()), // s_lower_sigma_1_v2
                meta.query_advice(a_2, Rotation::prev()),              // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),               // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),              // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),               // spread_r1_odd
                meta.query_advice(a_3, Rotation::next()),              // a
                meta.query_advice(a_4, Rotation::next()),              // spread_a
                meta.query_advice(a_6, Rotation::cur()),               // b
                meta.query_advice(a_3, Rotation::prev()),              // b_lo
                meta.query_advice(a_4, Rotation::prev()),              // spread_b_lo
                meta.query_advice(a_5, Rotation::prev()),              // b_hi
                meta.query_advice(a_6, Rotation::prev()),              // spread_b_hi
                meta.query_advice(a_5, Rotation::next()),              // c
                meta.query_advice(a_6, Rotation::next()),              // spread_c
                meta.query_advice(a_4, Rotation::cur()),               // spread_d
                meta.query_advice(a_7, Rotation::cur()),               // spread_e
                meta.query_advice(a_7, Rotation::next()),              // spread_f
                meta.query_advice(a_5, Rotation::cur()),               // spread_g
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_lower_sigma_1_v2", |_| expr.0)
            }
        }

        // sigma_1 v1 on W_49 to W_61
        // (10, 7, 2, 13)-bit chunks
        {
            let expressions = ScheduleGate::s_lower_sigma_1(
                meta.query_fixed(s_lower_sigma_1, Rotation::cur()), // s_lower_sigma_1
                meta.query_advice(a_2, Rotation::prev()),           // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),            // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),           // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),            // spread_r1_odd
                meta.query_advice(a_4, Rotation::cur()),            // spread_a
                meta.query_advice(a_6, Rotation::cur()),            // b
                meta.query_advice(a_3, Rotation::prev()),           // b_lo
                meta.query_advice(a_4, Rotation::prev()),           // spread_b_lo
                meta.query_advice(a_5, Rotation::prev()),           // b_mid
                meta.query_advice(a_6, Rotation::prev()),           // spread_b_mid
                meta.query_advice(a_5, Rotation::next()),           // b_hi
                meta.query_advice(a_6, Rotation::next()),           // spread_b_hi
                meta.query_advice(a_3, Rotation::next()),           // c
                meta.query_advice(a_4, Rotation::next()),           // spread_c
                meta.query_advice(a_5, Rotation::cur()),            // spread_d
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_lower_sigma_1", |_| expr.0)
            }
        }

        MessageScheduleConfig {
            lookup_inputs,
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

    #[allow(clippy::type_complexity)]
    pub(super) fn process(
        &mut self,
        input: [BlockWord; BLOCK_SIZE],
    ) -> Result<([MessageWord; ROUNDS], [(CellValue16, CellValue16); ROUNDS]), Error> {
        let config = self.config().clone();
        let mut w = Vec::<MessageWord>::with_capacity(ROUNDS);
        let mut w_halves = Vec::<(CellValue16, CellValue16)>::with_capacity(ROUNDS);
        self.layouter().assign_region(
            || "process message block",
            |mut region: Region<'_, F, Self>| {
                w = Vec::<MessageWord>::with_capacity(ROUNDS);
                w_halves = Vec::<(CellValue16, CellValue16)>::with_capacity(ROUNDS);

                // Assign all fixed columns
                for index in 1..14 {
                    let row = get_word_row(index);
                    region.assign_fixed(
                        || "s_decompose_1",
                        config.s_decompose_1,
                        row,
                        || Ok(F::one()),
                    )?;
                    region.assign_fixed(
                        || "s_lower_sigma_0",
                        config.s_lower_sigma_0,
                        row + 3,
                        || Ok(F::one()),
                    )?;
                }

                for index in 14..49 {
                    let row = get_word_row(index);
                    region.assign_fixed(
                        || "s_decompose_2",
                        config.s_decompose_2,
                        row,
                        || Ok(F::one()),
                    )?;
                    region.assign_fixed(
                        || "s_lower_sigma_0_v2",
                        config.s_lower_sigma_0_v2,
                        row + 3,
                        || Ok(F::one()),
                    )?;
                    region.assign_fixed(
                        || "s_lower_sigma_1_v2",
                        config.s_lower_sigma_1_v2,
                        row + SIGMA_0_V2_ROWS + 3,
                        || Ok(F::one()),
                    )?;

                    let new_word_idx = index + 2;
                    region.assign_fixed(
                        || "s_word",
                        config.s_word,
                        get_word_row(new_word_idx - 16) + 1,
                        || Ok(F::one()),
                    )?;
                }

                for index in 49..62 {
                    let row = get_word_row(index);
                    region.assign_fixed(
                        || "s_decompose_3",
                        config.s_decompose_3,
                        row,
                        || Ok(F::one()),
                    )?;
                    region.assign_fixed(
                        || "s_lower_sigma_1",
                        config.s_lower_sigma_1,
                        row + 3,
                        || Ok(F::one()),
                    )?;

                    let new_word_idx = index + 2;
                    region.assign_fixed(
                        || "s_word",
                        config.s_word,
                        get_word_row(new_word_idx - 16) + 1,
                        || Ok(F::one()),
                    )?;
                }

                for index in 0..64 {
                    let row = get_word_row(index);
                    region.assign_fixed(
                        || "s_decompose_0",
                        config.s_decompose_0,
                        row,
                        || Ok(F::one()),
                    )?;
                }

                // Assign W[0..16]
                for (i, word) in input.iter().enumerate() {
                    let (var, halves) =
                        config.assign_word_and_halves(&mut region, word.value.unwrap(), i)?;
                    w.push(MessageWord {
                        var,
                        value: word.value,
                    });
                    w_halves.push(halves);
                }

                // Returns the output of sigma_0 on W_[1..14]
                let lower_sigma_0_output = config.assign_subregion1(&mut region, &input[1..14])?;

                // sigma_0_v2 and sigma_1_v2 on W_[14..49]
                // Returns the output of sigma_0_v2 on W_[36..49], to be used in subregion3
                let lower_sigma_0_v2_output = config.assign_subregion2(
                    &mut region,
                    lower_sigma_0_output,
                    &mut w,
                    &mut w_halves,
                )?;

                // sigma_1 v1 on W[49..62]
                config.assign_subregion3(
                    &mut region,
                    lower_sigma_0_v2_output,
                    &mut w,
                    &mut w_halves,
                )?;

                Ok(())
            },
        )?;

        Ok((w.try_into().unwrap(), w_halves.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        super::BLOCK_SIZE, BlockWord, SpreadTableCore, Table16Chip, Table16Config, Table16Core,
    };
    use super::{schedule_util::*, MessageScheduleCore};
    use halo2::{
        arithmetic::FieldExt,
        circuit::{layouter::SingleCoreLayouter, Core},
        dev::MockProver,
        pasta::Fp,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };
    use std::marker::PhantomData;

    #[test]
    fn message_schedule() {
        struct MyCircuit<'a, F: FieldExt, CS: Assignment<F>> {
            marker: PhantomData<F>,
            marker_cs: PhantomData<&'a CS>,
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Circuit<F> for MyCircuit<'a, F, CS> {
            type Chip = Table16Chip<F, Self::Layouter>;
            type Core = Table16Core<F, Self::Layouter>;
            type Layouter = SingleCoreLayouter<'a, F, CS>;
            type Config = Table16Config;
            type CS = CS;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                Table16Core::<F, ()>::configure(meta)
            }

            fn synthesize(&self, cs: &mut CS, config: Self::Config) -> Result<(), Error> {
                let mut core = Table16Core {
                    config,
                    layouter: SingleCoreLayouter::new(cs),
                    _marker: std::marker::PhantomData,
                };

                // Load lookup table
                let lookup_table_config = core.config().lookup_table.clone();
                let mut lookup_table_core = SpreadTableCore {
                    config: lookup_table_config,
                    layouter: core.layouter(),
                    marker: PhantomData,
                };
                lookup_table_core.load()?;

                // Provide input
                // Test vector: "abc"
                let inputs: [BlockWord; BLOCK_SIZE] = get_msg_schedule_test_input();

                // Run message_scheduler to get W_[0..64]
                let message_schedule_config = core.config().message_schedule.clone();
                let mut message_schedule_core = MessageScheduleCore {
                    config: message_schedule_config,
                    layouter: core.layouter(),
                    marker: PhantomData,
                };

                let (w, _) = message_schedule_core.process(inputs)?;
                for (word, test_word) in w.iter().zip(MSG_SCHEDULE_TEST_OUTPUT.iter()) {
                    let word = word.value.unwrap();
                    assert_eq!(word, *test_word);
                }
                Ok(())
            }
        }

        let circuit: MyCircuit<'_, Fp, MockProver<Fp>> = MyCircuit {
            marker: PhantomData,
            marker_cs: PhantomData,
        };

        let prover = match MockProver::<Fp>::run(16, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
