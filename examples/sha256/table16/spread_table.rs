use super::{util::*, CellValue16, CellValue32};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Config, Layouter, Region},
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
    pub(super) fn with_lookup<'r, C: Config>(
        region: &mut Region<'r, C>,
        cols: &SpreadInputs,
        row: usize,
        word: SpreadWord,
    ) -> Result<Self, Error> {
        let tag = word.tag;
        let dense_val = Some(word.dense);
        let spread_val = Some(word.spread);
        region.assign_advice(
            || "tag",
            cols.tag,
            row,
            || Ok(C::Field::from_u64(tag as u64)),
        )?;
        let dense_var = region.assign_advice(
            || "dense",
            cols.dense,
            row,
            || {
                dense_val
                    .map(|v| C::Field::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        let spread_var = region.assign_advice(
            || "spread",
            cols.spread,
            row,
            || {
                spread_val
                    .map(|v| C::Field::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        Ok(SpreadVar {
            tag,
            dense: CellValue16::new(dense_var, dense_val.unwrap()),
            spread: CellValue32::new(spread_var, spread_val.unwrap()),
        })
    }

    pub(super) fn without_lookup<C: Config>(
        region: &mut Region<'_, C>,
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
                    .map(|v| C::Field::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        let spread_var = region.assign_advice(
            || "spread",
            spread_col,
            spread_row,
            || {
                spread_val
                    .map(|v| C::Field::from_u64(v as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;

        Ok(SpreadVar {
            tag,
            dense: CellValue16::new(dense_var, dense_val.unwrap()),
            spread: CellValue32::new(spread_var, spread_val.unwrap()),
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
pub(super) struct SpreadTableConfigured {
    pub table: SpreadTable,
    pub inputs: SpreadInputs,
}

pub(super) struct SpreadTableConfig<'a, F: FieldExt, L: Layouter<F>> {
    pub configured: SpreadTableConfigured,
    pub layouter: &'a mut L,
    pub marker: std::marker::PhantomData<F>,
}

// ANCHOR: chip-impl
impl<F: FieldExt, L: Layouter<F>> Config for SpreadTableConfig<'_, F, L> {
    type Root = Self;
    type Configured = SpreadTableConfigured;
    type Loaded = ();
    type Field = F;
    type Layouter = L;

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn configured(&self) -> &Self::Configured {
        &self.configured
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

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // TODO
    }

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, _gadget_name: Option<String>) {
        // TODO
    }
}

impl<F: FieldExt, L: Layouter<F>> SpreadTableConfig<'_, F, L> {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<F>,
        tag: Column<Advice>,
        dense: Column<Advice>,
        spread: Column<Advice>,
    ) -> SpreadTableConfigured {
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

        SpreadTableConfigured {
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
        let configured = self.configured().clone();
        let tag = configured.table.tag;
        let dense = configured.table.dense;
        let spread = configured.table.spread;
        self.layouter().assign_new_region(
            &[tag.into(), dense.into(), spread.into()],
            || "spread table",
            |mut gate: Region<'_, Self>| {
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

    use super::super::util::*;
    use super::{SpreadTableConfig, SpreadTableConfigured};
    use halo2::{
        arithmetic::FieldExt,
        circuit::{layouter::SingleConfigLayouter, Config, Layouter, Region},
        dev::MockProver,
        pasta::Fp,
        plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error},
    };

    #[test]
    fn lookup_table() {
        /// This represents an advice column at a certain row in the ConstraintSystem
        #[derive(Copy, Clone, Debug)]
        pub struct Variable(Column<Advice>, usize);

        struct MyCircuit {}

        impl<F: FieldExt, L: Layouter<F>> SpreadTableConfig<'_, F, L> {
            fn assign_new_row(&mut self, tag: F, dense: F, spread: F) -> Result<(), Error> {
                let input_tag = self.configured.inputs.tag;
                let input_dense = self.configured.inputs.dense;
                let input_spread = self.configured.inputs.spread;

                self.layouter().assign_new_region(
                    &[input_tag.into(), input_dense.into(), input_spread.into()],
                    || "assign new row",
                    |mut region: Region<'_, Self>| {
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
                        let cell = region.assign_advice(
                            || "spread",
                            Column::<Advice>::try_from(input_spread).unwrap(),
                            0,
                            || Ok(spread),
                        )?;
                        Ok(())
                    },
                )
            }
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Configured = SpreadTableConfigured;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Configured {
                let tag = meta.advice_column();
                let dense = meta.advice_column();
                let spread = meta.advice_column();
                SpreadTableConfig::<F, ()>::configure(meta, tag, dense, spread)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                configured: Self::Configured,
            ) -> Result<(), Error> {
                let mut config = SpreadTableConfig {
                    configured,
                    layouter: &mut SingleConfigLayouter::new(cs),
                    marker: std::marker::PhantomData,
                };
                config.load()?;

                // Test the first few small values.
                config.assign_new_row(F::zero(), F::from_u64(0b000), F::from_u64(0b000000))?;
                config.assign_new_row(F::zero(), F::from_u64(0b001), F::from_u64(0b000001))?;
                config.assign_new_row(F::zero(), F::from_u64(0b010), F::from_u64(0b000100))?;

                config.assign_new_row(F::zero(), F::from_u64(0b011), F::from_u64(0b000101))?;
                config.assign_new_row(F::zero(), F::from_u64(0b100), F::from_u64(0b010000))?;
                config.assign_new_row(F::zero(), F::from_u64(0b101), F::from_u64(0b010001))?;

                // Test the tag boundaries:
                // 7-bit
                config.assign_new_row(
                    F::zero(),
                    F::from_u64(0b1111111),
                    F::from_u64(0b01010101010101),
                )?;
                config.assign_new_row(
                    F::one(),
                    F::from_u64(0b10000000),
                    F::from_u64(0b0100000000000000),
                )?;
                // - 10-bit
                config.assign_new_row(
                    F::one(),
                    F::from_u64(0b1111111111),
                    F::from_u64(0b01010101010101010101),
                )?;
                config.assign_new_row(
                    F::from_u64(2),
                    F::from_u64(0b10000000000),
                    F::from_u64(0b0100000000000000000000),
                )?;
                // - 11-bit
                config.assign_new_row(
                    F::from_u64(2),
                    F::from_u64(0b11111111111),
                    F::from_u64(0b0101010101010101010101),
                )?;
                config.assign_new_row(
                    F::from_u64(3),
                    F::from_u64(0b100000000000),
                    F::from_u64(0b010000000000000000000000),
                )?;
                // - 13-bit
                config.assign_new_row(
                    F::from_u64(3),
                    F::from_u64(0b1111111111111),
                    F::from_u64(0b01010101010101010101010101),
                )?;
                config.assign_new_row(
                    F::from_u64(4),
                    F::from_u64(0b10000000000000),
                    F::from_u64(0b0100000000000000000000000000),
                )?;
                // - 14-bit
                config.assign_new_row(
                    F::from_u64(4),
                    F::from_u64(0b11111111111111),
                    F::from_u64(0b0101010101010101010101010101),
                )?;
                config.assign_new_row(
                    F::from_u64(5),
                    F::from_u64(0b100000000000000),
                    F::from_u64(0b010000000000000000000000000000),
                )?;

                // Test random lookup values
                let mut rng = rand::thread_rng();

                for _ in 0..10 {
                    let word: u16 = rng.gen();
                    config.assign_new_row(
                        F::from_u64(get_tag(word).into()),
                        F::from_u64(word.into()),
                        F::from_u64(interleave_u16_with_zeros(word).into()),
                    )?;
                }

                Ok(())
            }
        }

        let circuit: MyCircuit = MyCircuit {};

        let prover = match MockProver::<Fp>::run(16, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
