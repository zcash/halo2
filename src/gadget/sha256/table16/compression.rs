use super::{
    super::DIGEST_SIZE, BlockWord, CellValue16, CellValue32, SpreadInputs, SpreadVar,
    Table16Assignment, Table16Chip, ROUNDS, STATE,
};
use crate::{
    arithmetic::FieldExt,
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation},
};

// mod compression_gates;
// mod compression_util;
// mod subregion_digest;
// mod subregion_initial;
// mod subregion_main;

// use compression_gates::CompressionGate;

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
#[derive(Copy, Clone, Debug)]
pub struct AbcdVar {
    idx: i32,
    val: u32,
    a: SpreadVar,
    b: SpreadVar,
    c_lo: SpreadVar,
    c_mid: SpreadVar,
    c_hi: SpreadVar,
    d: SpreadVar,
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
#[derive(Copy, Clone, Debug)]
pub struct EfghVar {
    idx: i32,
    val: u32,
    a_lo: SpreadVar,
    a_hi: SpreadVar,
    b_lo: SpreadVar,
    b_hi: SpreadVar,
    c: SpreadVar,
    d: SpreadVar,
}

#[derive(Clone, Debug)]
pub struct RoundWordDense {
    dense_halves: (CellValue16, CellValue16),
}

impl RoundWordDense {
    pub fn new(dense_halves: (CellValue16, CellValue16)) -> Self {
        RoundWordDense { dense_halves }
    }
}

#[derive(Clone, Debug)]
pub struct RoundWordSpread {
    dense_halves: (CellValue16, CellValue16),
    spread_halves: (CellValue32, CellValue32),
}

impl RoundWordSpread {
    pub fn new(
        dense_halves: (CellValue16, CellValue16),
        spread_halves: (CellValue32, CellValue32),
    ) -> Self {
        RoundWordSpread {
            dense_halves,
            spread_halves,
        }
    }
}

impl Into<RoundWordDense> for RoundWordSpread {
    fn into(self) -> RoundWordDense {
        RoundWordDense::new(self.dense_halves)
    }
}

#[derive(Clone, Debug)]
pub struct RoundWordA {
    pieces: Option<AbcdVar>,
    dense_halves: (CellValue16, CellValue16),
    spread_halves: Option<(CellValue32, CellValue32)>,
}

impl RoundWordA {
    pub fn new(
        pieces: AbcdVar,
        dense_halves: (CellValue16, CellValue16),
        spread_halves: (CellValue32, CellValue32),
    ) -> Self {
        RoundWordA {
            pieces: Some(pieces),
            dense_halves,
            spread_halves: Some(spread_halves),
        }
    }

    pub fn new_dense(dense_halves: (CellValue16, CellValue16)) -> Self {
        RoundWordA {
            pieces: None,
            dense_halves,
            spread_halves: None,
        }
    }
}

impl Into<RoundWordSpread> for RoundWordA {
    fn into(self) -> RoundWordSpread {
        RoundWordSpread::new(self.dense_halves, self.spread_halves.unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct RoundWordE {
    pieces: Option<EfghVar>,
    dense_halves: (CellValue16, CellValue16),
    spread_halves: Option<(CellValue32, CellValue32)>,
}

impl RoundWordE {
    pub fn new(
        pieces: EfghVar,
        dense_halves: (CellValue16, CellValue16),
        spread_halves: (CellValue32, CellValue32),
    ) -> Self {
        RoundWordE {
            pieces: Some(pieces),
            dense_halves,
            spread_halves: Some(spread_halves),
        }
    }

    pub fn new_dense(dense_halves: (CellValue16, CellValue16)) -> Self {
        RoundWordE {
            pieces: None,
            dense_halves,
            spread_halves: None,
        }
    }
}

impl Into<RoundWordSpread> for RoundWordE {
    fn into(self) -> RoundWordSpread {
        RoundWordSpread::new(self.dense_halves, self.spread_halves.unwrap())
    }
}

/// The internal state for SHA-256.
#[derive(Clone, Debug)]
pub struct State {
    a: Option<StateWord>,
    b: Option<StateWord>,
    c: Option<StateWord>,
    d: Option<StateWord>,
    e: Option<StateWord>,
    f: Option<StateWord>,
    g: Option<StateWord>,
    h: Option<StateWord>,
}

impl State {
    pub fn new(
        a: StateWord,
        b: StateWord,
        c: StateWord,
        d: StateWord,
        e: StateWord,
        f: StateWord,
        g: StateWord,
        h: StateWord,
    ) -> Self {
        State {
            a: Some(a),
            b: Some(b),
            c: Some(c),
            d: Some(d),
            e: Some(e),
            f: Some(f),
            g: Some(g),
            h: Some(h),
        }
    }

    pub fn empty_state() -> Self {
        State {
            a: None,
            b: None,
            c: None,
            d: None,
            e: None,
            f: None,
            g: None,
            h: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum StateWord {
    A(RoundWordA),
    B(RoundWordSpread),
    C(RoundWordSpread),
    D(RoundWordDense),
    E(RoundWordE),
    F(RoundWordSpread),
    G(RoundWordSpread),
    H(RoundWordDense),
}

#[derive(Clone, Debug)]
pub(super) struct Compression {
    lookup: SpreadInputs,
    message_schedule: Column<Advice>,
    extras: [Column<Advice>; 6],

    s_ch: Column<Fixed>,
    s_ch_neg: Column<Fixed>,
    s_maj: Column<Fixed>,
    s_h_prime: Column<Fixed>,
    s_a_new: Column<Fixed>,
    s_e_new: Column<Fixed>,

    s_upper_sigma_0: Column<Fixed>,
    s_upper_sigma_1: Column<Fixed>,

    // Decomposition gate for AbcdVar
    s_decompose_abcd: Column<Fixed>,
    // Decomposition gate for EfghVar
    s_decompose_efgh: Column<Fixed>,

    s_digest: Column<Fixed>,

    perm: Permutation,
}

impl<F: FieldExt> Table16Assignment<F> for Compression {}

impl Compression {
    pub(super) fn configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        lookup: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
        perm: Permutation,
    ) -> Self {
        let s_ch = meta.fixed_column();
        let s_ch_neg = meta.fixed_column();
        let s_maj = meta.fixed_column();
        let s_h_prime = meta.fixed_column();
        let s_a_new = meta.fixed_column();
        let s_e_new = meta.fixed_column();

        let s_upper_sigma_0 = meta.fixed_column();
        let s_upper_sigma_1 = meta.fixed_column();

        // Decomposition gate for AbcdVar
        let s_decompose_abcd = meta.fixed_column();
        // Decomposition gate for EfghVar
        let s_decompose_efgh = meta.fixed_column();

        let s_digest = meta.fixed_column();

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

        // TODO: Create gates.

        Compression {
            lookup,
            message_schedule,
            extras,
            s_ch,
            s_ch_neg,
            s_maj,
            s_h_prime,
            s_a_new,
            s_e_new,
            s_upper_sigma_0,
            s_upper_sigma_1,
            s_decompose_abcd,
            s_decompose_efgh,
            s_digest,
            perm,
        }
    }

    /// Initialize compression with a constant Initialization Vector of 32-byte words.
    /// Returns an initialized state.
    pub(super) fn initialize_with_iv<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        init_state: [u32; STATE],
    ) -> Result<State, Error> {
        let mut new_state = State::empty_state();
        todo!()
    }

    /// Initialize compression with some initialized state. This could be a state
    /// output from a previous compression round.
    pub(super) fn initialize_with_state<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        init_state: State,
    ) -> Result<State, Error> {
        let mut new_state = State::empty_state();
        todo!()
    }

    /// Given an initialized state and a message schedule, perform 64 compression rounds.
    pub(super) fn compress<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        initialized_state: State,
        w_halves: [(CellValue16, CellValue16); ROUNDS],
    ) -> Result<State, Error> {
        let mut state = State::empty_state();
        todo!()
    }

    /// After the final round, convert the state into the final digest.
    pub(super) fn digest<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        state: State,
    ) -> Result<[BlockWord; DIGEST_SIZE], Error> {
        let mut digest = [BlockWord::new(0); DIGEST_SIZE];
        todo!()
    }

    pub(super) fn empty_configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        lookup: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
        perm: Permutation,
    ) -> Self {
        let s_ch = meta.fixed_column();
        let s_ch_neg = meta.fixed_column();
        let s_maj = meta.fixed_column();
        let s_h_prime = meta.fixed_column();
        let s_a_new = meta.fixed_column();
        let s_e_new = meta.fixed_column();

        let s_upper_sigma_0 = meta.fixed_column();
        let s_upper_sigma_1 = meta.fixed_column();

        // Decomposition gate for AbcdVar
        let s_decompose_abcd = meta.fixed_column();
        // Decomposition gate for EfghVar
        let s_decompose_efgh = meta.fixed_column();

        let s_digest = meta.fixed_column();

        Compression {
            lookup,
            message_schedule,
            extras,
            s_ch,
            s_ch_neg,
            s_maj,
            s_h_prime,
            s_a_new,
            s_e_new,
            s_upper_sigma_0,
            s_upper_sigma_1,
            s_decompose_abcd,
            s_decompose_efgh,
            s_digest,
            perm,
        }
    }
}
