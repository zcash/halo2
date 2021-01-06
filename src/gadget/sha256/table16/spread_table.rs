use super::{util::*, CellValue16, CellValue32, Table16Chip};
use crate::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
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
    pub(super) fn with_lookup<'r, C: Chip>(
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

    pub(super) fn without_lookup<'r, C: Chip>(
        region: &mut Region<'r, C>,
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
        layouter.assign_region(
            || "spread table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = Self::generate::<F>();

                for index in 0..(1 << 16) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "tag",
                        self.table_tag,
                        index,
                        || {
                            row = rows.next();
                            row.map(|(tag, _, _)| tag).ok_or(Error::SynthesisError)
                        },
                    )?;
                    gate.assign_fixed(
                        || "dense",
                        self.table_dense,
                        index,
                        || row.map(|(_, dense, _)| dense).ok_or(Error::SynthesisError),
                    )?;
                    gate.assign_fixed(
                        || "spread",
                        self.table_spread,
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
    use std::cmp;
    use std::collections::HashMap;
    use std::fmt;
    use std::marker::PhantomData;

    use super::{
        super::{util::*, MessageSchedule, Table16Chip, Table16Config},
        SpreadInputs, SpreadTable,
    };
    use crate::{
        arithmetic::FieldExt,
        circuit::{layouter, Cell, Layouter, Region, RegionIndex},
        dev::MockProver,
        pasta::Fp,
        plonk::{
            Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed, Permutation,
        },
    };

    #[test]
    fn lookup_table() {
        /// This represents an advice column at a certain row in the ConstraintSystem
        #[derive(Copy, Clone, Debug)]
        pub struct Variable(Column<Advice>, usize);

        #[derive(Clone, Debug)]
        struct MyConfig {
            lookup_inputs: SpreadInputs,
            sha256: Table16Config,
        }

        struct MyCircuit {}

        struct MyLayouter<'a, F: FieldExt, CS: Assignment<F> + 'a> {
            cs: &'a mut CS,
            config: MyConfig,
            regions: Vec<usize>,
            /// Stores the first empty row for each column.
            columns: HashMap<Column<Any>, usize>,
            _marker: PhantomData<F>,
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyLayouter<'a, F, CS> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("MyLayouter")
                    .field("config", &self.config)
                    .field("regions", &self.regions)
                    .field("columns", &self.columns)
                    .finish()
            }
        }

        impl<'a, FF: FieldExt, CS: Assignment<FF>> MyLayouter<'a, FF, CS> {
            fn new(cs: &'a mut CS, config: MyConfig) -> Result<Self, Error> {
                let mut res = MyLayouter {
                    cs,
                    config,
                    regions: vec![],
                    columns: HashMap::default(),
                    _marker: PhantomData,
                };

                let table = res.config.sha256.lookup_table.clone();
                table.load(&mut res)?;

                Ok(res)
            }
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Layouter<Table16Chip<F>> for MyLayouter<'a, F, CS> {
            type Root = Self;

            fn config(&self) -> &Table16Config {
                &self.config.sha256
            }

            fn loaded(&self) -> &() {
                &()
            }

            fn assign_region<A, AR, N, NR>(
                &mut self,
                name: N,
                mut assignment: A,
            ) -> Result<AR, Error>
            where
                A: FnMut(Region<'_, Table16Chip<F>>) -> Result<AR, Error>,
                N: Fn() -> NR,
                NR: Into<String>,
            {
                let region_index = self.regions.len();

                // Get shape of the region.
                let mut shape = layouter::RegionShape::new(region_index.into());
                {
                    let region: &mut dyn layouter::RegionLayouter<Table16Chip<F>> = &mut shape;
                    assignment(region.into())?;
                }

                // Lay out this region. We implement the simplest approach here: position the
                // region starting at the earliest row for which none of the columns are in use.
                let mut region_start = 0;
                for column in shape.columns() {
                    region_start =
                        cmp::max(region_start, self.columns.get(column).cloned().unwrap_or(0));
                }
                self.regions.push(region_start);

                // Update column usage information.
                for column in shape.columns() {
                    self.columns
                        .insert(*column, region_start + shape.row_count());
                }

                self.cs.enter_region(name);
                let mut region = MyRegion::new(self, region_index.into());
                let result = {
                    let region: &mut dyn layouter::RegionLayouter<Table16Chip<F>> = &mut region;
                    assignment(region.into())
                }?;
                self.cs.exit_region();

                Ok(result)
            }

            fn get_root(&mut self) -> &mut Self::Root {
                self
            }

            fn push_namespace<NR, N>(&mut self, name_fn: N)
            where
                NR: Into<String>,
                N: FnOnce() -> NR,
            {
                self.cs.push_namespace(name_fn)
            }

            fn pop_namespace(&mut self, gadget_name: Option<String>) {
                self.cs.pop_namespace(gadget_name)
            }
        }

        struct MyRegion<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> {
            layouter: &'r mut MyLayouter<'a, F, CS>,
            region_index: RegionIndex,
            _marker: PhantomData<F>,
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyRegion<'r, 'a, F, CS> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("MyRegion")
                    .field("layouter", &self.layouter)
                    .field("region_index", &self.region_index)
                    .finish()
            }
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> MyRegion<'r, 'a, F, CS> {
            fn new(layouter: &'r mut MyLayouter<'a, F, CS>, region_index: RegionIndex) -> Self {
                MyRegion {
                    layouter,
                    region_index,
                    _marker: PhantomData::default(),
                }
            }
        }

        impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> layouter::RegionLayouter<Table16Chip<F>>
            for MyRegion<'r, 'a, F, CS>
        {
            fn assign_advice<'v>(
                &'v mut self,
                annotation: &'v (dyn Fn() -> String + 'v),
                column: Column<Advice>,
                offset: usize,
                to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
            ) -> Result<Cell, Error> {
                self.layouter.cs.assign_advice(
                    annotation,
                    column,
                    self.layouter.regions[*self.region_index] + offset,
                    to,
                )?;

                Ok(Cell {
                    region_index: self.region_index,
                    row_offset: offset,
                    column: column.into(),
                })
            }

            fn assign_fixed<'v>(
                &'v mut self,
                annotation: &'v (dyn Fn() -> String + 'v),
                column: Column<Fixed>,
                offset: usize,
                to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
            ) -> Result<Cell, Error> {
                self.layouter.cs.assign_fixed(
                    annotation,
                    column,
                    self.layouter.regions[*self.region_index] + offset,
                    to,
                )?;
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
                self.layouter.cs.copy(
                    permutation,
                    left.column,
                    self.layouter.regions[*left.region_index] + left.row_offset,
                    right.column,
                    self.layouter.regions[*right.region_index] + right.row_offset,
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

                // Rename these here for ease of matching the gates to the specification.
                let _a_0 = lookup_inputs.tag;
                let a_1 = lookup_inputs.dense;
                let a_2 = lookup_inputs.spread;
                let a_3 = extras[0];
                let a_4 = extras[1];
                let a_5 = message_schedule;
                let a_6 = extras[2];
                let a_7 = extras[3];
                let a_8 = extras[4];
                let _a_9 = extras[5];

                let perm = Permutation::new(
                    meta,
                    &[
                        a_1.into(),
                        a_2.into(),
                        a_3.into(),
                        a_4.into(),
                        a_5.into(),
                        a_6.into(),
                        a_7.into(),
                        a_8.into(),
                    ],
                );

                let message_schedule = MessageSchedule::empty_configure(
                    meta,
                    lookup_inputs.clone(),
                    message_schedule,
                    extras,
                    perm.clone(),
                );

                MyConfig {
                    lookup_inputs,
                    sha256: Table16Config {
                        lookup_table,
                        message_schedule,
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

                layouter.assign_region(
                    || "spread_test",
                    |mut gate| {
                        let mut row = 0;
                        let mut add_row = |tag, dense, spread| {
                            gate.assign_advice(|| "tag", lookup.tag, row, || Ok(tag))?;
                            gate.assign_advice(|| "dense", lookup.dense, row, || Ok(dense))?;
                            gate.assign_advice(|| "spread", lookup.spread, row, || Ok(spread))?;
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

        let prover = match MockProver::<Fp>::run(16, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
