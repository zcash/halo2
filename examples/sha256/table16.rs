use std::marker::PhantomData;

use super::Sha256Instructions;
use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Config, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Permutation},
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

#[derive(Clone, Copy, Debug)]
/// A word in a `Table16` message block.
pub struct BlockWord {
    var: (),
    value: Option<u32>,
}

impl BlockWord {
    /// Create a new `BlockWord`.
    pub fn new(value: u32) -> Self {
        BlockWord {
            var: (),
            value: Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CellValue16 {
    var: Cell,
    value: Option<u16>,
}

impl CellValue16 {
    pub fn new(var: Cell, value: u16) -> Self {
        CellValue16 {
            var,
            value: Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CellValue32 {
    var: Cell,
    value: Option<u32>,
}

impl CellValue32 {
    pub fn new(var: Cell, value: u32) -> Self {
        CellValue32 {
            var,
            value: Some(value),
        }
    }
}

impl Into<CellValue32> for CellValue16 {
    fn into(self) -> CellValue32 {
        CellValue32::new(self.var, self.value.unwrap() as u32)
    }
}

/// Configuration for a [`Table16Config`].
#[derive(Clone, Debug)]
pub struct Table16Configured {
    lookup_table: SpreadTableConfigured,
    message_schedule: MessageScheduleConfigured,
    compression: CompressionConfigured,
}

/// A config that implements SHA-256 with a maximum lookup table size of $2^16$.
#[derive(Debug)]
pub struct Table16Config<'a, F: FieldExt, L: Layouter> {
    pub configured: Table16Configured,
    pub layouter: &'a mut L,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<Field = F>> Table16Config<'_, F, L> {
    /// Configures this config for use in a circuit.
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Table16Configured {
        // Columns required by this config:
        // - Three advice columns to interact with the lookup table.
        let tag = meta.advice_column();
        let dense = meta.advice_column();
        let spread = meta.advice_column();

        let message_schedule = meta.advice_column();
        let extras = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let (lookup_inputs, lookup_table) =
            SpreadTableConfig::<'_, F, L>::configure(meta, tag, dense, spread);

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

        let compression = CompressionConfig::<'_, F, L>::configure(
            meta,
            lookup_inputs.clone(),
            message_schedule,
            extras,
            perm.clone(),
        );

        let message_schedule = MessageScheduleConfig::<'_, F, L>::configure(
            meta,
            lookup_inputs,
            message_schedule,
            extras,
            perm,
        );

        Table16Configured {
            lookup_table,
            message_schedule,
            compression,
        }
    }
}

impl<F: FieldExt, L: Layouter<Field = F>> Config for Table16Config<'_, F, L> {
    type Root = Self;
    type Field = F;
    type Configured = Table16Configured;
    type Loaded = ();
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

    fn load(&mut self) -> Result<(), Error> {
        let configured = self.configured().lookup_table.clone();
        let mut spread_table = SpreadTableConfig::<'_, F, L> {
            configured,
            layouter: self.layouter(),
            marker: PhantomData,
        };
        spread_table.load()
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

impl<F: FieldExt, L: Layouter<Field = F>> Sha256Instructions for Table16Config<'_, F, L> {
    type State = State;
    type BlockWord = BlockWord;

    fn zero() -> Self::BlockWord {
        BlockWord::new(0)
    }

    fn initialization_vector(&mut self) -> Result<State, Error> {
        let configured = self.configured().compression.clone();

        let mut compression = CompressionConfig::<'_, F, L> {
            configured,
            layouter: self.layouter(),
            marker: PhantomData,
        };
        compression.initialize_with_iv(IV)
    }

    fn initialization(&mut self, init_state: &Self::State) -> Result<Self::State, Error> {
        let configured = self.configured().compression.clone();

        let mut compression = CompressionConfig::<'_, F, L> {
            configured,
            layouter: self.layouter(),
            marker: PhantomData,
        };
        compression.initialize_with_state(init_state.clone())
    }

    // Given an initialized state and an input message block, compress the
    // message block and return the final state.
    fn compress(
        &mut self,
        initialized_state: &Self::State,
        input: [Self::BlockWord; super::BLOCK_SIZE],
    ) -> Result<Self::State, Error> {
        let configured = self.configured().clone();

        let mut message_schedule = MessageScheduleConfig::<'_, F, L> {
            configured: configured.message_schedule.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };

        let (_, w_halves) = message_schedule.process(input)?;

        let mut compression = CompressionConfig::<'_, F, L> {
            configured: configured.compression,
            layouter: self.layouter(),
            marker: PhantomData,
        };
        compression.compress(initialized_state.clone(), w_halves)
    }

    fn digest(
        &mut self,
        state: &Self::State,
    ) -> Result<[Self::BlockWord; super::DIGEST_SIZE], Error> {
        // Copy the dense forms of the state variable chunks down to this gate.
        // Reconstruct the 32-bit dense words.
        let configured = self.configured().compression.clone();

        let mut compression = CompressionConfig::<'_, F, L> {
            configured,
            layouter: self.layouter(),
            marker: PhantomData,
        };
        compression.digest(state.clone())
    }
}

/// Common assignment patterns used by Table16 regions.
trait Table16Assignment<F: FieldExt, L: Layouter<Field = F>>: Config<Field = F, Layouter = L> {
    // Assign cells for general spread computation used in sigma, ch, ch_neg, maj gates
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn assign_spread_outputs(
        &mut self,
        lookup: &SpreadInputs,
        a_3: Column<Advice>,
        perm: &Permutation,
        row: usize,
        r_0_even: u16,
        r_0_odd: u16,
        r_1_even: u16,
        r_1_odd: u16,
    ) -> Result<((CellValue16, CellValue16), (CellValue16, CellValue16)), Error> {
        self.layouter().assign_region(
            || "assign spread outputs",
            |mut region: Region<'_, Self>| {
                // Lookup R_0^{even}, R_0^{odd}, R_1^{even}, R_1^{odd}
                let r_0_even = SpreadVar::with_lookup(
                    &mut region,
                    lookup,
                    row - 1,
                    SpreadWord::new(r_0_even),
                )?;
                let r_0_odd =
                    SpreadVar::with_lookup(&mut region, lookup, row, SpreadWord::new(r_0_odd))?;
                let r_1_even = SpreadVar::with_lookup(
                    &mut region,
                    lookup,
                    row + 1,
                    SpreadWord::new(r_1_even),
                )?;
                let r_1_odd =
                    SpreadVar::with_lookup(&mut region, lookup, row + 2, SpreadWord::new(r_1_odd))?;

                // Assign and copy R_1^{odd}
                let r_1_odd_spread = region.assign_advice(
                    || "Assign and copy R_1^{odd}",
                    a_3,
                    row,
                    || Ok(F::from_u64(r_1_odd.spread.value.unwrap().into())),
                )?;
                region.constrain_equal(perm, r_1_odd.spread.var, r_1_odd_spread)?;

                Ok((
                    (
                        CellValue16::new(r_0_even.dense.var, r_0_even.dense.value.unwrap()),
                        CellValue16::new(r_1_even.dense.var, r_1_even.dense.value.unwrap()),
                    ),
                    (
                        CellValue16::new(r_0_odd.dense.var, r_0_odd.dense.value.unwrap()),
                        CellValue16::new(r_1_odd.dense.var, r_1_odd.dense.value.unwrap()),
                    ),
                ))
            },
        )
    }

    // Assign outputs of sigma gates
    #[allow(clippy::too_many_arguments)]
    fn assign_sigma_outputs(
        &mut self,
        lookup: &SpreadInputs,
        a_3: Column<Advice>,
        perm: &Permutation,
        row: usize,
        r_0_even: u16,
        r_0_odd: u16,
        r_1_even: u16,
        r_1_odd: u16,
    ) -> Result<(CellValue16, CellValue16), Error> {
        let (even, _odd) = self
            .assign_spread_outputs(lookup, a_3, perm, row, r_0_even, r_0_odd, r_1_even, r_1_odd)?;

        Ok(even)
    }

    // Assign a cell the same value as another cell and set up a copy constraint between them
    fn assign_and_constrain<A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        copy: &CellValue32,
        perm: &Permutation,
    ) -> Result<Cell, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        self.layouter().assign_region(
            || "assign and constrain",
            |mut region: Region<'_, Self>| {
                let cell = region.assign_advice(&annotation, column, row, || {
                    Ok(F::from_u64(copy.value.unwrap() as u64))
                })?;
                region.constrain_equal(perm, cell, copy.var)?;
                Ok(cell)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "dev-graph")]
    use halo2::{
        arithmetic::FieldExt,
        circuit::Config,
        circuit::{layouter, Layouter},
        gadget::sha256::{BlockWord, Sha256, Table16Config, Table16Configured, BLOCK_SIZE},
        pasta::Fq,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_sha256_circuit() {
        struct MyCircuit {}

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Configured = Table16Configured;

            fn configure(meta: &mut ConstraintSystem<F>) -> Table16Configured {
                Table16Config::configure(meta)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                configured: Table16Configured,
            ) -> Result<(), Error> {
                let mut layouter =
                    layouter::SingleConfigLayouter::<Table16Config<F>, _>::new(cs, configured)?;
                Table16Config::load(&mut layouter)?;

                // Test vector: "abc"
                let test_input = [
                    BlockWord::new(0b01100001011000100110001110000000),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                ];

                // Create a message of length 31 blocks
                let mut input = Vec::with_capacity(31 * BLOCK_SIZE);
                for _ in 0..31 {
                    input.extend_from_slice(&test_input);
                }

                Sha256::digest(layouter.namespace(|| "'abc' * 31"), &input)?;

                Ok(())
            }
        }

        let circuit: MyCircuit = MyCircuit {};
        eprintln!("{}", crate::dev::circuit_dot_graph::<Fq, _>(&circuit));
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_table16_config() {
        use plotters::prelude::*;
        struct MyCircuit {}

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Configured = Table16Configured;

            fn configure(meta: &mut ConstraintSystem<F>) -> Table16Configured {
                Table16Config::configure(meta)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                configured: Table16Configured,
            ) -> Result<(), Error> {
                let mut layouter =
                    layouter::SingleConfigLayouter::<Table16Config<F>, _>::new(cs, configured)?;
                Table16Config::load(&mut layouter)?;

                // Test vector: "abc"
                let test_input = [
                    BlockWord::new(0b01100001011000100110001110000000),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                    BlockWord::new(0),
                ];

                // Create a message of length 2 blocks
                let mut input = Vec::with_capacity(2 * BLOCK_SIZE);
                for _ in 0..2 {
                    input.extend_from_slice(&test_input);
                }

                Sha256::digest(layouter.namespace(|| "'abc' * 2"), &input)?;

                Ok(())
            }
        }

        let root =
            SVGBackend::new("sha-256-table16-config-layout.svg", (1024, 20480)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("16-bit Table SHA-256 Config", ("sans-serif", 60))
            .unwrap();

        let circuit = MyCircuit {};
        crate::dev::circuit_layout::<Fq, _, _>(&circuit, &root).unwrap();
    }
}
