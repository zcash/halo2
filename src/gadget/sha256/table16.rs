use std::marker::PhantomData;

use super::{Chip, Layouter, Sha256Instructions};
use crate::{
    arithmetic::FieldExt,
    gadget::ChipConfig,
    plonk::{ConstraintSystem, Error},
};

mod gates;
mod message_schedule;
mod spread_table;
mod util;

use gates::*;
use message_schedule::*;
use spread_table::*;

const ROUNDS: usize = 64;
const STATE: usize = 8;

#[allow(clippy::unreadable_literal)]
const ROUND_CONSTANTS: [u32; ROUNDS] = [
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
pub struct BlockWord {
    var: (),
    value: Option<u32>,
}

impl BlockWord {
    pub fn new(value: u32) -> Self {
        BlockWord {
            var: (),
            value: Some(value),
        }
    }
}

/// A variable that represents the `[A,B,C,D]` words of the SHA-256 internal state.
///
/// The structure of this variable is influenced by the following factors:
/// - In `Σ_0(A)` we need `A` to be split into pieces `(a,b,c,d)` of lengths `(2,11,9,10)`
///   bits respectively (counting from the little end), as well as their spread forms.
/// - `Maj(A,B,C)` requires having the bits of each input in spread form. For `A` we can
///   reuse the pieces from `Σ_0(A)`. Since `B` and `C` are assigned from `A` and `B`
///   respectively in each round, we therefore also have the same pieces in earlier rows.
///   We align the columns to make it efficient to copy-constrain these forms where they
///   are needed.
#[derive(Clone, Debug)]
struct AbcdVar {
    chunk_0: SpreadVar,
    chunk_1: SpreadVar,
    chunk_2: SpreadVar,
    chunk_3: SpreadVar,
}

/// A variable that represents the `[E,F,G,H]` words of the SHA-256 internal state.
///
/// The structure of this variable is influenced by the following factors:
/// - In `Σ_1(E)` we need `E` to be split into pieces `(a,b,c,d)` of lengths `(6,5,14,7)`
///   bits respectively (counting from the little end), as well as their spread forms.
/// - `Ch(E,F,G)` requires having the bits of each input in spread form. For `E` we can
///   reuse the pieces from `Σ_1(E)`. Since `F` and `G` are assigned from `E` and `F`
///   respectively in each round, we therefore also have the same pieces in earlier rows.
///   We align the columns to make it efficient to copy-constrain these forms where they
///   are needed.
#[derive(Clone, Debug)]
struct EfghVar {}

/// The internal state for SHA-256.
#[derive(Clone, Debug)]
pub struct State {
    h_0: AbcdVar,
    h_1: AbcdVar,
    h_2: AbcdVar,
    h_3: AbcdVar,
    h_4: EfghVar,
    h_5: EfghVar,
    h_6: EfghVar,
    h_7: EfghVar,
}

#[derive(Clone, Debug)]
struct HPrime {}

/// Configuration for a [`Table16Chip`].
#[derive(Clone, Debug)]
pub struct Table16Config {
    lookup_table: SpreadTable,
    message_schedule: MessageSchedule,
}

impl ChipConfig for Table16Config {}

/// A chip that implements SHA-256 with a maximum lookup table size of $2^16$.
#[derive(Clone, Debug)]
pub struct Table16Chip<F: FieldExt> {
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Table16Chip<F> {
    /// Configures this chip for use in a circuit.
    ///
    /// TODO: Figure out what the necessary shared columns are.
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Table16Config {
        // Columns required by this chip:
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

        // - N selector columns.
        let s_ch = meta.fixed_column();
        let s_maj = meta.fixed_column();
        let s_upper_sigma_0 = meta.fixed_column();
        let s_upper_sigma_1 = meta.fixed_column();
        let s_lower_sigma_0 = meta.fixed_column();
        let s_lower_sigma_1 = meta.fixed_column();
        let s_lower_sigma_0_v2 = meta.fixed_column();
        let s_lower_sigma_1_v2 = meta.fixed_column();

        let (lookup_inputs, lookup_table) = SpreadTable::configure(meta, tag, dense, spread);

        let message_schedule =
            MessageSchedule::configure(meta, lookup_inputs, message_schedule, extras);

        Table16Config {
            lookup_table,
            message_schedule,
        }
    }
}

impl<F: FieldExt> Chip for Table16Chip<F> {
    type Field = F;
    type Config = Table16Config;

    fn load(layouter: &mut impl Layouter<Self>) -> Result<(), Error> {
        let table = layouter.config().lookup_table.clone();
        table.load(layouter)
    }
}

impl<F: FieldExt> Sha256Instructions for Table16Chip<F> {
    type State = State;
    type BlockWord = BlockWord;

    fn initialization_vector(layouter: &mut impl Layouter<Self>) -> Result<State, Error> {
        todo!()
    }

    fn compress(
        layouter: &mut impl Layouter<Self>,
        initial_state: &Self::State,
        input: [Self::BlockWord; super::BLOCK_SIZE],
    ) -> Result<Self::State, Error> {
        let config = layouter.config().clone();
        let w = config.message_schedule.process(layouter, input)?;

        todo!()
    }

    fn digest(
        layouter: &mut impl Layouter<Self>,
        state: &Self::State,
    ) -> Result<[Self::BlockWord; super::DIGEST_SIZE], Error> {
        // Copy the dense forms of the state variable chunks down to this gate.
        // Reconstruct the 32-bit dense words.
        todo!()
    }
}
