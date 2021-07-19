use super::{util::*, CellValue16, CellValue32};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};
use std::marker::PhantomData;

/// An input word into a lookup, containing (tag, dense, spread)
#[derive(Copy, Clone, Debug)]
pub(super) struct SpreadWord {
    pub tag: u8,
    pub dense: u16,
    pub spread: u32,
}

impl SpreadWord {
    pub(super) fn new(word: u16) -> Self {
        SpreadWord {
            tag: get_tag(word),
            dense: word,
            spread: interleave_u16_with_zeros(word),
        }
    }

    pub(super) fn opt_new(word: Option<u16>) -> Option<Self> {
        word.map(SpreadWord::new)
    }
}

/// A variable stored in advice columns corresponding to a row of [`SpreadTableConfig`].
#[derive(Copy, Clone, Debug)]
pub(super) struct SpreadVar {
    pub tag: Option<u8>,
    pub dense: CellValue16,
    pub spread: CellValue32,
}

impl SpreadVar {
    pub(super) fn with_lookup<F: FieldExt>(
        region: &mut Region<'_, F>,
        cols: &SpreadInputs,
        row: usize,
        word: Option<SpreadWord>,
    ) -> Result<Self, Error> {
        let tag = word.map(|word| word.tag);
        let dense_val = word.map(|word| word.dense);
        let spread_val = word.map(|word| word.spread);

        region.assign_advice(
            || "tag",
            cols.tag,
            row,
            || {
                tag.map(|tag| F::from_u64(tag as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        let dense_var = region.assign_advice(
            || "dense",
            cols.dense,
            row,
            || {
                dense_val
                    .map(|v| F::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        let spread_var = region.assign_advice(
            || "spread",
            cols.spread,
            row,
            || {
                spread_val
                    .map(|v| F::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        Ok(SpreadVar {
            tag,
            dense: CellValue16::new(dense_var, dense_val),
            spread: CellValue32::new(spread_var, spread_val),
        })
    }

    pub(super) fn without_lookup<F: FieldExt>(
        region: &mut Region<'_, F>,
        dense_col: Column<Advice>,
        dense_row: usize,
        spread_col: Column<Advice>,
        spread_row: usize,
        word: Option<SpreadWord>,
    ) -> Result<Self, Error> {
        let tag = word.map(|word| word.tag);
        let dense_val = word.map(|word| word.dense);
        let spread_val = word.map(|word| word.spread);
        let dense_var = region.assign_advice(
            || "dense",
            dense_col,
            dense_row,
            || {
                dense_val
                    .map(|v| F::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        let spread_var = region.assign_advice(
            || "spread",
            spread_col,
            spread_row,
            || {
                spread_val
                    .map(|v| F::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        Ok(SpreadVar {
            tag,
            dense: CellValue16::new(dense_var, dense_val),
            spread: CellValue32::new(spread_var, spread_val),
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct SpreadInputs {
    pub(super) tag: Column<Advice>,
    pub(super) dense: Column<Advice>,
    pub(super) spread: Column<Advice>,
}

#[derive(Clone, Debug)]
pub(super) struct SpreadTable {
    pub(super) tag: Column<Fixed>,
    pub(super) dense: Column<Fixed>,
    pub(super) spread: Column<Fixed>,
}

#[derive(Clone, Debug)]
pub(super) struct SpreadTableConfig {
    pub input: SpreadInputs,
    pub table: SpreadTable,
}

#[derive(Clone, Debug)]
pub(super) struct SpreadTableChip<F: FieldExt> {
    config: SpreadTableConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for SpreadTableChip<F> {
    type Config = SpreadTableConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> SpreadTableChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        input_tag: Column<Advice>,
        input_dense: Column<Advice>,
        input_spread: Column<Advice>,
    ) -> <Self as Chip<F>>::Config {
        let table_tag = meta.fixed_column();
        let table_dense = meta.fixed_column();
        let table_spread = meta.fixed_column();

        meta.lookup(|meta| {
            let tag_cur = meta.query_advice(input_tag, Rotation::cur());
            let dense_cur = meta.query_advice(input_dense, Rotation::cur());
            let spread_cur = meta.query_advice(input_spread, Rotation::cur());
            let table_tag_cur = meta.query_fixed(table_tag, Rotation::cur());
            let table_dense_cur = meta.query_fixed(table_dense, Rotation::cur());
            let table_spread_cur = meta.query_fixed(table_spread, Rotation::cur());

            vec![
                (tag_cur, table_tag_cur),
                (dense_cur, table_dense_cur),
                (spread_cur, table_spread_cur),
            ]
        });

        SpreadTableConfig {
            input: SpreadInputs {
                tag: input_tag,
                dense: input_dense,
                spread: input_spread,
            },
            table: SpreadTable {
                tag: table_tag,
                dense: table_dense,
                spread: table_spread,
            },
        }
    }

    pub fn load(
        config: SpreadTableConfig,
        layouter: &mut impl Layouter<F>,
    ) -> Result<<Self as Chip<F>>::Loaded, Error> {
        layouter.assign_region(
            || "spread table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = SpreadTableConfig::generate::<F>();

                for index in 0..(1 << 16) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "tag",
                        config.table.tag,
                        index,
                        || {
                            row = rows.next();
                            row.map(|(tag, _, _)| tag).ok_or(Error::SynthesisError)
                        },
                    )?;
                    gate.assign_fixed(
                        || "dense",
                        config.table.dense,
                        index,
                        || row.map(|(_, dense, _)| dense).ok_or(Error::SynthesisError),
                    )?;
                    gate.assign_fixed(
                        || "spread",
                        config.table.spread,
                        index,
                        || {
                            row.map(|(_, _, spread)| spread)
                                .ok_or(Error::SynthesisError)
                        },
                    )?;
                }
                Ok(())
            },
        )
    }
}

impl SpreadTableConfig {
    fn generate<F: FieldExt>() -> impl Iterator<Item = (F, F, F)> {
        (1..=(1 << 16)).scan(
            (F::zero(), F::zero(), F::zero()),
            |(tag, dense, spread), i| {
                // We computed this table row in the previous iteration.
                let res = (*tag, *dense, *spread);

                // i holds the zero-indexed row number for the next table row.
                match i {
                    BITS_7 | BITS_10 | BITS_11 | BITS_13 | BITS_14 => *tag += F::one(),
                    _ => (),
                }
                *dense += F::one();
                if i & 1 == 0 {
                    // On even-numbered rows we recompute the spread.
                    *spread = F::zero();
                    for b in 0..16 {
                        if (i >> b) & 1 != 0 {
                            *spread += F::from_u64(1 << (2 * b));
                        }
                    }
                } else {
                    // On odd-numbered rows we add one.
                    *spread += F::one();
                }

                Some(res)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{SpreadTableChip, SpreadTableConfig};
    use rand::Rng;

    use crate::table16::util::{get_tag, interleave_u16_with_zeros};
    use halo2::{
        arithmetic::FieldExt,
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        pasta::Fp,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
    };

    #[test]
    fn lookup_table() {
        /// This represents an advice column at a certain row in the ConstraintSystem
        #[derive(Copy, Clone, Debug)]
        pub struct Variable(Column<Advice>, usize);

        struct MyCircuit {}

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Config = SpreadTableConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                MyCircuit {}
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let input_tag = meta.advice_column();
                let input_dense = meta.advice_column();
                let input_spread = meta.advice_column();

                SpreadTableChip::configure(meta, input_tag, input_dense, input_spread)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                SpreadTableChip::load(config.clone(), &mut layouter)?;

                layouter.assign_region(
                    || "spread_test",
                    |mut gate| {
                        let mut row = 0;
                        let mut add_row = |tag, dense, spread| {
                            gate.assign_advice(|| "tag", config.input.tag, row, || Ok(tag))?;
                            gate.assign_advice(|| "dense", config.input.dense, row, || Ok(dense))?;
                            gate.assign_advice(
                                || "spread",
                                config.input.spread,
                                row,
                                || Ok(spread),
                            )?;
                            row += 1;
                            Ok(())
                        };

                        // Test the first few small values.
                        add_row(F::zero(), F::from_u64(0b000), F::from_u64(0b000000))?;
                        add_row(F::zero(), F::from_u64(0b001), F::from_u64(0b000001))?;
                        add_row(F::zero(), F::from_u64(0b010), F::from_u64(0b000100))?;
                        add_row(F::zero(), F::from_u64(0b011), F::from_u64(0b000101))?;
                        add_row(F::zero(), F::from_u64(0b100), F::from_u64(0b010000))?;
                        add_row(F::zero(), F::from_u64(0b101), F::from_u64(0b010001))?;

                        // Test the tag boundaries:
                        // 7-bit
                        add_row(
                            F::zero(),
                            F::from_u64(0b1111111),
                            F::from_u64(0b01010101010101),
                        )?;
                        add_row(
                            F::one(),
                            F::from_u64(0b10000000),
                            F::from_u64(0b0100000000000000),
                        )?;
                        // - 10-bit
                        add_row(
                            F::one(),
                            F::from_u64(0b1111111111),
                            F::from_u64(0b01010101010101010101),
                        )?;
                        add_row(
                            F::from_u64(2),
                            F::from_u64(0b10000000000),
                            F::from_u64(0b0100000000000000000000),
                        )?;
                        // - 11-bit
                        add_row(
                            F::from_u64(2),
                            F::from_u64(0b11111111111),
                            F::from_u64(0b0101010101010101010101),
                        )?;
                        add_row(
                            F::from_u64(3),
                            F::from_u64(0b100000000000),
                            F::from_u64(0b010000000000000000000000),
                        )?;
                        // - 13-bit
                        add_row(
                            F::from_u64(3),
                            F::from_u64(0b1111111111111),
                            F::from_u64(0b01010101010101010101010101),
                        )?;
                        add_row(
                            F::from_u64(4),
                            F::from_u64(0b10000000000000),
                            F::from_u64(0b0100000000000000000000000000),
                        )?;
                        // - 14-bit
                        add_row(
                            F::from_u64(4),
                            F::from_u64(0b11111111111111),
                            F::from_u64(0b0101010101010101010101010101),
                        )?;
                        add_row(
                            F::from_u64(5),
                            F::from_u64(0b100000000000000),
                            F::from_u64(0b010000000000000000000000000000),
                        )?;

                        // Test random lookup values
                        let mut rng = rand::thread_rng();

                        for _ in 0..10 {
                            let word: u16 = rng.gen();
                            add_row(
                                F::from_u64(get_tag(word).into()),
                                F::from_u64(word.into()),
                                F::from_u64(interleave_u16_with_zeros(word).into()),
                            )?;
                        }

                        Ok(())
                    },
                )
            }
        }

        let circuit: MyCircuit = MyCircuit {};

        let prover = match MockProver::<Fp>::run(17, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
