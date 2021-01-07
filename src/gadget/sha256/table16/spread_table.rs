use super::{util::*, Table16Chip};
use crate::{
    arithmetic::FieldExt,
    gadget::{Cell, Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};

/// An input word into a lookup, containing (tag, dense, spread)
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
#[derive(Clone, Debug)]
pub(super) struct SpreadVar {
    tag: u8,
    dense_var: Cell,
    dense_val: Option<u16>,
    spread_var: Cell,
    spread_val: Option<u32>,
}

impl SpreadVar {
    pub(super) fn new<'r, C: Chip>(
        region: &mut Region<'r, C>,
        cols: &SpreadInputs,
        row: usize,
        word: SpreadWord,
    ) -> Result<Self, Error> {
        let tag = word.tag;
        let dense_val = Some(word.dense);
        let spread_val = Some(word.spread);
        region.assign_advice(cols.tag, row, || Ok(C::Field::from_u64(tag as u64)))?;
        let dense_var = region.assign_advice(cols.dense, row, || {
            dense_val
                .map(|v| C::Field::from_u64(v as u64))
                .ok_or(Error::SynthesisError)
        })?;
        let spread_var = region.assign_advice(cols.spread, row, || {
            spread_val
                .map(|v| C::Field::from_u64(v as u64))
                .ok_or(Error::SynthesisError)
        })?;

        Ok(SpreadVar {
            tag,
            dense_var,
            dense_val,
            spread_var,
            spread_val,
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
    table_tag: Column<Fixed>,
    table_dense: Column<Fixed>,
    table_spread: Column<Fixed>,
}

impl SpreadTable {
    pub(super) fn configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        tag: Column<Advice>,
        dense: Column<Advice>,
        spread: Column<Advice>,
    ) -> (SpreadInputs, Self) {
        let table_tag = meta.fixed_column();
        let table_dense = meta.fixed_column();
        let table_spread = meta.fixed_column();

        meta.lookup(
            &[tag.into(), dense.into(), spread.into()],
            &[table_tag.into(), table_dense.into(), table_spread.into()],
        );

        (
            SpreadInputs { tag, dense, spread },
            SpreadTable {
                table_tag,
                table_dense,
                table_spread,
            },
        )
    }

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

    pub(super) fn load<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
    ) -> Result<(), Error> {
        layouter.assign_region(|mut gate| {
            // We generate the row values lazily (we only need them during keygen).
            let mut rows = Self::generate::<F>();

            for index in 0..(1 << 16) {
                let mut row = None;
                gate.assign_fixed(self.table_tag, index, || {
                    row = rows.next();
                    row.map(|(tag, _, _)| tag).ok_or(Error::SynthesisError)
                })?;
                gate.assign_fixed(self.table_dense, index, || {
                    row.map(|(_, dense, _)| dense).ok_or(Error::SynthesisError)
                })?;
                gate.assign_fixed(self.table_spread, index, || {
                    row.map(|(_, _, spread)| spread)
                        .ok_or(Error::SynthesisError)
                })?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::cmp;
    use std::fmt;
    use std::marker::PhantomData;

    use super::{
        super::{util::*, Table16Chip, Table16Config},
        SpreadInputs, SpreadTable,
    };
    use crate::{
        arithmetic::FieldExt,
        dev::MockProver,
        gadget::{Cell, DynRegion, Layouter, Permutation, Region},
        pasta::Fp,
        plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed},
    };

    #[test]
    fn lookup_table() {
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

                // let message_schedule = MessageSchedule::configure(
                //     meta,
                //     lookup_inputs.clone(),
                //     message_schedule,
                //     extras,
                // );

                MyConfig {
                    lookup_inputs,
                    sha256: Table16Config {
                        lookup_table,
                        // message_schedule,
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

                layouter.assign_region(|mut gate| {
                    let mut row = 0;
                    let mut add_row = |tag, dense, spread| {
                        gate.assign_advice(lookup.tag, row, || Ok(tag))?;
                        gate.assign_advice(lookup.dense, row, || Ok(dense))?;
                        gate.assign_advice(lookup.spread, row, || Ok(spread))?;
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
                })
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
