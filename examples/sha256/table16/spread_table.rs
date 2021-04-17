use super::{util::*, CellValue16, CellValue32};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Core, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

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
}

/// A variable stored in advice columns corresponding to a row of [`SpreadTable`].
#[derive(Copy, Clone, Debug)]
pub(super) struct SpreadVar {
    pub tag: u8,
    pub dense: CellValue16,
    pub spread: CellValue32,
}

impl SpreadVar {
    pub(super) fn with_lookup<F: FieldExt, C: Core<F>>(
        region: &mut Region<'_, F, C>,
        cols: &SpreadInputs,
        row: usize,
        word: SpreadWord,
    ) -> Result<Self, Error> {
        let tag = word.tag;
        let dense_val = Some(word.dense);
        let spread_val = Some(word.spread);
        region.assign_advice(|| "tag", cols.tag, row, || Ok(F::from_u64(tag as u64)))?;
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

    pub(super) fn without_lookup<F: FieldExt, C: Core<F>>(
        region: &mut Region<'_, F, C>,
        dense_col: Column<Advice>,
        dense_row: usize,
        spread_col: Column<Advice>,
        spread_row: usize,
        word: SpreadWord,
    ) -> Result<Self, Error> {
        let tag = word.tag;
        let dense_val = Some(word.dense);
        let spread_val = Some(word.spread);
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
    tag: Column<Fixed>,
    dense: Column<Fixed>,
    spread: Column<Fixed>,
}

#[derive(Clone, Debug)]
pub(super) struct SpreadTableConfig {
    pub table: SpreadTable,
    pub inputs: SpreadInputs,
}

pub(super) struct SpreadTableCore<'a, F: FieldExt, L: Layouter<F>> {
    pub config: SpreadTableConfig,
    pub layouter: &'a mut L,
    pub marker: std::marker::PhantomData<F>,
}

// ANCHOR: chip-impl
impl<F: FieldExt, L: Layouter<F>> Core<F> for SpreadTableCore<'_, F, L> {
    type Config = SpreadTableConfig;
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

impl<F: FieldExt, L: Layouter<F>> SpreadTableCore<'_, F, L> {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<F>,
        tag: Column<Advice>,
        dense: Column<Advice>,
        spread: Column<Advice>,
    ) -> SpreadTableConfig {
        let table_tag = meta.fixed_column();
        let table_dense = meta.fixed_column();
        let table_spread = meta.fixed_column();

        let tag_ = meta.query_any(tag.into(), Rotation::cur());
        let dense_ = meta.query_any(dense.into(), Rotation::cur());
        let spread_ = meta.query_any(spread.into(), Rotation::cur());
        let table_tag_ = meta.query_any(table_tag.into(), Rotation::cur());
        let table_dense_ = meta.query_any(table_dense.into(), Rotation::cur());
        let table_spread_ = meta.query_any(table_spread.into(), Rotation::cur());
        meta.lookup(
            &[tag_, dense_, spread_],
            &[table_tag_, table_dense_, table_spread_],
        );

        SpreadTableConfig {
            table: SpreadTable {
                tag: table_tag,
                dense: table_dense,
                spread: table_spread,
            },
            inputs: SpreadInputs { tag, dense, spread },
        }
    }

    fn generate() -> impl Iterator<Item = (F, F, F)> {
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

    pub(super) fn load(&mut self) -> Result<(), Error> {
        let config = self.config().clone();
        let tag = config.table.tag;
        let dense = config.table.dense;
        let spread = config.table.spread;
        self.layouter().assign_region(
            || "spread table",
            |mut gate: Region<'_, F, Self>| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = Self::generate();

                for index in 0..(1 << 16) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "tag",
                        tag,
                        index,
                        || {
                            row = rows.next();
                            row.map(|(tag, _, _)| tag).ok_or(Error::SynthesisError)
                        },
                    )?;
                    gate.assign_fixed(
                        || "dense",
                        dense,
                        index,
                        || row.map(|(_, dense, _)| dense).ok_or(Error::SynthesisError),
                    )?;
                    gate.assign_fixed(
                        || "spread",
                        spread,
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

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::convert::TryFrom;
    use std::marker::PhantomData;

    use super::{
        super::{util::*, Table16Chip, Table16Config, Table16Core},
        SpreadTableConfig, SpreadTableCore,
    };
    use halo2::{
        arithmetic::FieldExt,
        circuit::{layouter::SingleCoreLayouter, Core, Layouter, Region},
        dev::MockProver,
        pasta::Fp,
        plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error},
    };

    #[test]
    fn lookup_table() {
        /// This represents an advice column at a certain row in the ConstraintSystem
        #[derive(Copy, Clone, Debug)]
        pub struct Variable(Column<Advice>, usize);

        struct MyCircuit<'a, F: FieldExt, CS: Assignment<F>> {
            marker: PhantomData<F>,
            marker_cs: PhantomData<&'a CS>,
        }

        impl SpreadTableConfig {
            fn add_row<F: FieldExt, C: Core<F>>(
                &self,
                region: &mut Region<'_, F, C>,
                row: usize,
                tag: F,
                dense: F,
                spread: F,
            ) -> Result<usize, Error> {
                let input_tag = self.inputs.tag;
                let input_dense = self.inputs.dense;
                let input_spread = self.inputs.spread;

                region.assign_advice(
                    || "tag",
                    Column::<Advice>::try_from(input_tag).unwrap(),
                    0,
                    || Ok(tag),
                )?;
                region.assign_advice(
                    || "dense",
                    Column::<Advice>::try_from(input_dense).unwrap(),
                    0,
                    || Ok(dense),
                )?;
                region.assign_advice(
                    || "spread",
                    Column::<Advice>::try_from(input_spread).unwrap(),
                    0,
                    || Ok(spread),
                )?;

                Ok(row + 1)
            }
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
                let config = core.config().lookup_table.clone();
                let mut lookup_table_core = SpreadTableCore {
                    config: config.clone(),
                    layouter: core.layouter(),
                    marker: PhantomData,
                };
                lookup_table_core.load()?;

                lookup_table_core.layouter().assign_region(
                    || "spread_test",
                    |mut region: Region<'_, F, Self::Core>| {
                        let row = 0;

                        // Test the first few small values.
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b000),
                            F::from_u64(0b000000),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b001),
                            F::from_u64(0b000001),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b010),
                            F::from_u64(0b000100),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b011),
                            F::from_u64(0b000101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b100),
                            F::from_u64(0b010000),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b101),
                            F::from_u64(0b010001),
                        )?;

                        // Test the tag boundaries:
                        // 7-bit
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::zero(),
                            F::from_u64(0b1111111),
                            F::from_u64(0b01010101010101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::one(),
                            F::from_u64(0b10000000),
                            F::from_u64(0b0100000000000000),
                        )?;
                        // - 10-bit
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::one(),
                            F::from_u64(0b1111111111),
                            F::from_u64(0b01010101010101010101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(2),
                            F::from_u64(0b10000000000),
                            F::from_u64(0b0100000000000000000000),
                        )?;
                        // - 11-bit
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(2),
                            F::from_u64(0b11111111111),
                            F::from_u64(0b0101010101010101010101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(3),
                            F::from_u64(0b100000000000),
                            F::from_u64(0b010000000000000000000000),
                        )?;
                        // - 13-bit
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(3),
                            F::from_u64(0b1111111111111),
                            F::from_u64(0b01010101010101010101010101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(4),
                            F::from_u64(0b10000000000000),
                            F::from_u64(0b0100000000000000000000000000),
                        )?;
                        // - 14-bit
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(4),
                            F::from_u64(0b11111111111111),
                            F::from_u64(0b0101010101010101010101010101),
                        )?;
                        let row = config.add_row(
                            &mut region,
                            row,
                            F::from_u64(5),
                            F::from_u64(0b100000000000000),
                            F::from_u64(0b010000000000000000000000000000),
                        )?;

                        // Test random lookup values
                        let mut rng = rand::thread_rng();

                        for _ in 0..10 {
                            let word: u16 = rng.gen();
                            let row = config.add_row(
                                &mut region,
                                row,
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
