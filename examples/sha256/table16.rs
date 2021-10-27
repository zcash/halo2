use std::marker::PhantomData;

use super::Sha256Instructions;
use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error},
};

mod compression;
mod gates;
mod message_schedule;
mod spread_table;
mod util;

use compression::*;
use gates::*;
use message_schedule::*;
use spread_table::*;

const ROUNDS: usize = 64;
const STATE: usize = 8;

#[allow(clippy::unreadable_literal)]
pub(crate) const ROUND_CONSTANTS: [u32; ROUNDS] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const IV: [u32; STATE] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

#[derive(Clone, Copy, Debug, Default)]
/// A word in a `Table16` message block.
pub struct BlockWord(pub(crate) Option<u32>);

pub trait CellValue<T> {
    fn var(&self) -> Cell;
    fn value(&self) -> Option<T>;
}

#[derive(Clone, Copy, Debug)]
pub struct CellValue16 {
    var: Cell,
    value: Option<u16>,
}

impl<F: FieldExt> CellValue<F> for CellValue16 {
    fn var(&self) -> Cell {
        self.var
    }
    fn value(&self) -> Option<F> {
        self.value.map(|value| F::from_u64(value as u64))
    }
}

impl CellValue16 {
    pub fn new(var: Cell, value: Option<u16>) -> Self {
        CellValue16 { var, value }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CellValue32 {
    var: Cell,
    value: Option<u32>,
}

impl CellValue32 {
    pub fn new(var: Cell, value: Option<u32>) -> Self {
        CellValue32 { var, value }
    }
}

impl<F: FieldExt> CellValue<F> for CellValue32 {
    fn var(&self) -> Cell {
        self.var
    }
    fn value(&self) -> Option<F> {
        self.value.map(|value| F::from_u64(value as u64))
    }
}

#[allow(clippy::from_over_into)]
impl Into<CellValue32> for CellValue16 {
    fn into(self) -> CellValue32 {
        CellValue32::new(self.var, self.value.map(|value| value as u32))
    }
}

/// Configuration for a [`Table16Chip`].
#[derive(Clone, Debug)]
pub struct Table16Config {
    lookup: SpreadTableConfig,
    message_schedule: MessageScheduleConfig,
    compression: CompressionConfig,
}

/// A chip that implements SHA-256 with a maximum lookup table size of $2^16$.
#[derive(Clone, Debug)]
pub struct Table16Chip<F: FieldExt> {
    config: Table16Config,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for Table16Chip<F> {
    type Config = Table16Config;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> Table16Chip<F> {
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> <Self as Chip<F>>::Config {
        // Columns required by this chip:
        let message_schedule = meta.advice_column();
        let extras = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // - Three advice columns to interact with the lookup table.
        let input_tag = meta.advice_column();
        let input_dense = meta.advice_column();
        let input_spread = meta.advice_column();

        let lookup = SpreadTableChip::configure(meta, input_tag, input_dense, input_spread);
        let lookup_inputs = lookup.input.clone();

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

        // Add all advice columns to permutation
        for column in [a_1, a_2, a_3, a_4, a_5, a_6, a_7, a_8].iter() {
            meta.enable_equality((*column).into());
        }

        let compression =
            CompressionConfig::configure(meta, lookup_inputs.clone(), message_schedule, extras);

        let message_schedule =
            MessageScheduleConfig::configure(meta, lookup_inputs, message_schedule, extras);

        Table16Config {
            lookup,
            message_schedule,
            compression,
        }
    }

    pub fn load(config: Table16Config, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        SpreadTableChip::load(config.lookup, layouter)
    }
}

impl<F: FieldExt> Sha256Instructions<F> for Table16Chip<F> {
    type State = State;
    type BlockWord = BlockWord;

    fn initialization_vector(&self, layouter: &mut impl Layouter<F>) -> Result<State, Error> {
        self.config().compression.initialize_with_iv(layouter, IV)
    }

    fn initialization(
        &self,
        layouter: &mut impl Layouter<F>,
        init_state: &Self::State,
    ) -> Result<Self::State, Error> {
        self.config()
            .compression
            .initialize_with_state(layouter, init_state.clone())
    }

    // Given an initialized state and an input message block, compress the
    // message block and return the final state.
    fn compress(
        &self,
        layouter: &mut impl Layouter<F>,
        initialized_state: &Self::State,
        input: [Self::BlockWord; super::BLOCK_SIZE],
    ) -> Result<Self::State, Error> {
        let config = self.config();
        let (_, w_halves) = config.message_schedule.process(layouter, input)?;
        config
            .compression
            .compress(layouter, initialized_state.clone(), w_halves)
    }

    fn digest(
        &self,
        layouter: &mut impl Layouter<F>,
        state: &Self::State,
    ) -> Result<[Self::BlockWord; super::DIGEST_SIZE], Error> {
        // Copy the dense forms of the state variable chunks down to this gate.
        // Reconstruct the 32-bit dense words.
        self.config().compression.digest(layouter, state.clone())
    }
}

/// Common assignment patterns used by Table16 regions.
trait Table16Assignment<F: FieldExt> {
    // Assign cells for general spread computation used in sigma, ch, ch_neg, maj gates
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn assign_spread_outputs(
        &self,
        region: &mut Region<'_, F>,
        lookup: &SpreadInputs,
        a_3: Column<Advice>,
        row: usize,
        r_0_even: Option<u16>,
        r_0_odd: Option<u16>,
        r_1_even: Option<u16>,
        r_1_odd: Option<u16>,
    ) -> Result<((CellValue16, CellValue16), (CellValue16, CellValue16)), Error> {
        // Lookup R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
        let r_0_even =
            SpreadVar::with_lookup(region, lookup, row - 1, SpreadWord::opt_new(r_0_even))?;
        let r_0_odd = SpreadVar::with_lookup(region, lookup, row, SpreadWord::opt_new(r_0_odd))?;
        let r_1_even =
            SpreadVar::with_lookup(region, lookup, row + 1, SpreadWord::opt_new(r_1_even))?;
        let r_1_odd =
            SpreadVar::with_lookup(region, lookup, row + 2, SpreadWord::opt_new(r_1_odd))?;

        // Assign and copy R_1^{odd}
        let r_1_odd_spread = region.assign_advice(
            || "Assign and copy R_1^{odd}",
            a_3,
            row,
            || {
                r_1_odd
                    .spread
                    .value
                    .map(|value| F::from_u64(value as u64))
                    .ok_or(Error::SynthesisError)
            },
        )?;
        region.constrain_equal(r_1_odd.spread.var, r_1_odd_spread)?;

        Ok((
            (
                CellValue16::new(r_0_even.dense.var, r_0_even.dense.value),
                CellValue16::new(r_1_even.dense.var, r_1_even.dense.value),
            ),
            (
                CellValue16::new(r_0_odd.dense.var, r_0_odd.dense.value),
                CellValue16::new(r_1_odd.dense.var, r_1_odd.dense.value),
            ),
        ))
    }

    // Assign outputs of sigma gates
    #[allow(clippy::too_many_arguments)]
    fn assign_sigma_outputs(
        &self,
        region: &mut Region<'_, F>,
        lookup: &SpreadInputs,
        a_3: Column<Advice>,
        row: usize,
        r_0_even: Option<u16>,
        r_0_odd: Option<u16>,
        r_1_even: Option<u16>,
        r_1_odd: Option<u16>,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let (even, _odd) = self.assign_spread_outputs(
            region, lookup, a_3, row, r_0_even, r_0_odd, r_1_even, r_1_odd,
        )?;

        Ok(even)
    }

    // Assign a cell the same value as another cell and set up a copy constraint between them
    fn assign_and_constrain<A, AR>(
        &self,
        region: &mut Region<'_, F>,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        copy: impl CellValue<F>,
    ) -> Result<Cell, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let cell = region.assign_advice(annotation, column, row, || {
            copy.value().ok_or(Error::SynthesisError)
        })?;
        region.constrain_equal(cell, copy.var())?;
        Ok(cell)
    }
}

#[cfg(test)]
#[cfg(feature = "dev-graph")]
mod tests {
    use super::super::{Sha256, BLOCK_SIZE};
    use super::{message_schedule::msg_schedule_test_input, Table16Chip, Table16Config};
    use halo2::{
        arithmetic::FieldExt,
        circuit::{Layouter, SimpleFloorPlanner},
        pasta::Fq,
        plonk::{Circuit, ConstraintSystem, Error},
    };

    #[test]
    fn print_sha256_circuit() {
        use plotters::prelude::*;
        struct MyCircuit {}

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Config = Table16Config;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                MyCircuit {}
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                Table16Chip::configure(meta)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                let table16_chip = Table16Chip::<F>::construct(config.clone());
                Table16Chip::<F>::load(config, &mut layouter)?;

                // Test vector: "abc"
                let test_input = msg_schedule_test_input();

                // Create a message of length 31 blocks
                let mut input = Vec::with_capacity(31 * BLOCK_SIZE);
                for _ in 0..31 {
                    input.extend_from_slice(&test_input);
                }

                Sha256::digest(table16_chip, layouter.namespace(|| "'abc' * 31"), &input)?;

                Ok(())
            }
        }

        let root =
            BitMapBackend::new("sha-256-table16-chip-layout.png", (1024, 3480)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("16-bit Table SHA-256 Chip", ("sans-serif", 60))
            .unwrap();

        let circuit = MyCircuit {};
        halo2::dev::CircuitLayout::default()
            .render::<Fq, _, _>(17, &circuit, &root)
            .unwrap();
    }
}
