use super::{
    super::DIGEST_SIZE, BlockWord, CellValue16, CellValue32, SpreadInputs, SpreadVar,
    Table16Assignment, ROUNDS, STATE,
};
use halo2::{
    arithmetic::FieldExt,
    circuit::{Core, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation},
    poly::Rotation,
};

mod compression_gates;
mod compression_util;
mod subregion_digest;
mod subregion_initial;
mod subregion_main;

use compression_gates::CompressionGate;

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

impl From<RoundWordSpread> for RoundWordDense {
    fn from(spread_word: RoundWordSpread) -> Self {
        RoundWordDense::new(spread_word.dense_halves)
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

impl From<RoundWordA> for RoundWordSpread {
    fn from(round_word_a: RoundWordA) -> Self {
        RoundWordSpread::new(
            round_word_a.dense_halves,
            round_word_a.spread_halves.unwrap(),
        )
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

impl From<RoundWordE> for RoundWordSpread {
    fn from(round_word_e: RoundWordE) -> RoundWordSpread {
        RoundWordSpread::new(
            round_word_e.dense_halves,
            round_word_e.spread_halves.unwrap(),
        )
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
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
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

impl<F: FieldExt, C: Core<F>> Table16Assignment<F, C> for CompressionConfig {}

#[derive(Clone, Debug)]
pub(super) struct CompressionConfig {
    pub lookup: SpreadInputs,
    pub message_schedule: Column<Advice>,
    pub extras: [Column<Advice>; 6],

    pub s_ch: Column<Fixed>,
    pub s_ch_neg: Column<Fixed>,
    pub s_maj: Column<Fixed>,
    pub s_h_prime: Column<Fixed>,
    pub s_a_new: Column<Fixed>,
    pub s_e_new: Column<Fixed>,

    pub s_upper_sigma_0: Column<Fixed>,
    pub s_upper_sigma_1: Column<Fixed>,

    // Decomposition gate for AbcdVar
    pub s_decompose_abcd: Column<Fixed>,
    // Decomposition gate for EfghVar
    pub s_decompose_efgh: Column<Fixed>,

    pub s_digest: Column<Fixed>,

    pub perm: Permutation,
}

pub(super) struct CompressionCore<'a, F: FieldExt, L: Layouter<F>> {
    pub config: CompressionConfig,
    pub layouter: &'a mut L,
    pub marker: std::marker::PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Core<F> for CompressionCore<'_, F, L> {
    type Config = CompressionConfig;
    type Loaded = ();
    type Layouter = L;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        &mut self.layouter
    }
}

impl<'a, F: FieldExt, L: Layouter<F>> CompressionCore<'a, F, L> {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<F>,
        lookup: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
        perm: Permutation,
    ) -> CompressionConfig {
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

        // Decompose `A,B,C,D` words into (2, 11, 9, 10)-bit chunks.
        // `c` is split into (3, 3, 3)-bit c_lo, c_mid, c_hi.

        {
            let expressions = CompressionGate::s_decompose_abcd(
                meta.query_fixed(s_decompose_abcd, Rotation::cur()), // s_decompose_abcd
                meta.query_advice(a_3, Rotation::next()),            // a (2-bit chunk)
                meta.query_advice(a_4, Rotation::next()),            // spread_a
                meta.query_advice(a_1, Rotation::cur()),             // b (11-bit chunk)
                meta.query_advice(a_2, Rotation::cur()),             // spread_b
                meta.query_advice(a_0, Rotation::cur()),             // tag_b
                meta.query_advice(a_3, Rotation::cur()),             // c_lo (3-bit chunk)
                meta.query_advice(a_4, Rotation::cur()),             // spread_c_lo
                meta.query_advice(a_5, Rotation::cur()),             // c_mid (3-bit chunk)
                meta.query_advice(a_6, Rotation::cur()),             // spread_c_mid
                meta.query_advice(a_5, Rotation::next()),            // c_hi (3-bit chunk)
                meta.query_advice(a_6, Rotation::next()),            // spread_c_hi
                meta.query_advice(a_1, Rotation::next()),            // d (7-bit chunk)
                meta.query_advice(a_2, Rotation::next()),            // spread_d
                meta.query_advice(a_0, Rotation::next()),            // tag_d
                meta.query_advice(a_7, Rotation::cur()),             // word_lo
                meta.query_advice(a_8, Rotation::cur()),             // spread_word_lo
                meta.query_advice(a_7, Rotation::next()),            // word_hi
                meta.query_advice(a_8, Rotation::next()),            // spread_word_hi
            );

            for expr in expressions.into_iter() {
                meta.create_gate("decompose ABCD", |_| expr.0);
            }
        }

        // Decompose `E,F,G,H` words into (6, 5, 14, 7)-bit chunks.
        // `a` is split into (3, 3)-bit a_lo, a_hi
        // `b` is split into (2, 3)-bit b_lo, b_hi
        {
            let expressions = CompressionGate::s_decompose_efgh(
                meta.query_fixed(s_decompose_efgh, Rotation::cur()), // s_decompose_efgh
                meta.query_advice(a_3, Rotation::next()),            // a_lo (3-bit chunk)
                meta.query_advice(a_4, Rotation::next()),            // spread_a_lo
                meta.query_advice(a_5, Rotation::next()),            // a_hi (3-bit chunk)
                meta.query_advice(a_6, Rotation::next()),            // spread_a_hi
                meta.query_advice(a_3, Rotation::cur()),             // b_lo (2-bit chunk)
                meta.query_advice(a_4, Rotation::cur()),             // spread_b_lo
                meta.query_advice(a_5, Rotation::cur()),             // b_hi (3-bit chunk)
                meta.query_advice(a_6, Rotation::cur()),             // spread_b_hi
                meta.query_advice(a_1, Rotation::next()),            // c (14-bit chunk)
                meta.query_advice(a_2, Rotation::next()),            // spread_c
                meta.query_advice(a_0, Rotation::next()),            // tag_c
                meta.query_advice(a_1, Rotation::cur()),             // d (7-bit chunk)
                meta.query_advice(a_2, Rotation::cur()),             // spread_d
                meta.query_advice(a_0, Rotation::cur()),             // tag_d
                meta.query_advice(a_7, Rotation::cur()),             // word_lo
                meta.query_advice(a_8, Rotation::cur()),             // spread_word_lo
                meta.query_advice(a_7, Rotation::next()),            // word_hi
                meta.query_advice(a_8, Rotation::next()),            // spread_word_hi
            );

            for expr in expressions.into_iter() {
                meta.create_gate("decompose EFGH", |_| expr.0);
            }
        }

        // s_upper_sigma_0 on abcd words
        // (2, 11, 9, 10)-bit chunks
        meta.create_gate("s_upper_sigma_0", |meta| {
            CompressionGate::s_upper_sigma_0(
                meta.query_fixed(s_upper_sigma_0, Rotation::cur()), // s_upper_sigma_0
                meta.query_advice(a_2, Rotation::prev()),           // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),            // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),           // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),            // spread_r1_odd
                meta.query_advice(a_3, Rotation::next()),           // spread_a
                meta.query_advice(a_5, Rotation::cur()),            // spread_b
                meta.query_advice(a_3, Rotation::prev()),           // spread_c_lo
                meta.query_advice(a_4, Rotation::prev()),           // spread_c_mid
                meta.query_advice(a_4, Rotation::next()),           // spread_c_hi
                meta.query_advice(a_4, Rotation::cur()),            // spread_d
            )
            .0
        });

        // s_upper_sigma_1 on efgh words
        // (6, 5, 14, 7)-bit chunks
        meta.create_gate("s_upper_sigma_1", |meta| {
            CompressionGate::s_upper_sigma_1(
                meta.query_fixed(s_upper_sigma_1, Rotation::cur()), // s_upper_sigma_1
                meta.query_advice(a_2, Rotation::prev()),           // spread_r0_even
                meta.query_advice(a_2, Rotation::cur()),            // spread_r0_odd
                meta.query_advice(a_2, Rotation::next()),           // spread_r1_even
                meta.query_advice(a_3, Rotation::cur()),            // spread_r1_odd
                meta.query_advice(a_3, Rotation::next()),           // spread_a_lo
                meta.query_advice(a_4, Rotation::next()),           // spread_a_hi
                meta.query_advice(a_3, Rotation::prev()),           // spread_b_lo
                meta.query_advice(a_4, Rotation::prev()),           // spread_b_hi
                meta.query_advice(a_5, Rotation::cur()),            // spread_c
                meta.query_advice(a_4, Rotation::cur()),            // spread_d
            )
            .0
        });

        // s_ch on efgh words
        // First part of choice gate on (E, F, G), E ∧ F
        meta.create_gate("s_ch", |meta| {
            CompressionGate::s_ch(
                meta.query_fixed(s_ch, Rotation::cur()),  // s_ch
                meta.query_advice(a_2, Rotation::prev()), // spread_p0_even
                meta.query_advice(a_2, Rotation::cur()),  // spread_p0_odd
                meta.query_advice(a_2, Rotation::next()), // spread_p1_even
                meta.query_advice(a_3, Rotation::cur()),  // spread_p1_odd
                meta.query_advice(a_3, Rotation::prev()), // spread_e_lo
                meta.query_advice(a_4, Rotation::prev()), // spread_e_hi
                meta.query_advice(a_3, Rotation::next()), // spread_f_lo
                meta.query_advice(a_4, Rotation::next()), // spread_f_hi
            )
            .0
        });

        // s_ch_neg on efgh words
        // Second part of Choice gate on (E, F, G), ¬E ∧ G
        {
            let expressions = CompressionGate::s_ch_neg(
                meta.query_fixed(s_ch_neg, Rotation::cur()), // s_ch_neg
                meta.query_advice(a_2, Rotation::prev()),    // spread_q0_even
                meta.query_advice(a_2, Rotation::cur()),     // spread_q0_odd
                meta.query_advice(a_2, Rotation::next()),    // spread_q1_even
                meta.query_advice(a_3, Rotation::cur()),     // spread_q1_odd
                meta.query_advice(a_5, Rotation::prev()),    // spread_e_lo
                meta.query_advice(a_5, Rotation::cur()),     // spread_e_hi
                meta.query_advice(a_3, Rotation::prev()),    // spread_e_neg_lo
                meta.query_advice(a_4, Rotation::prev()),    // spread_e_neg_hi
                meta.query_advice(a_3, Rotation::next()),    // spread_g_lo
                meta.query_advice(a_4, Rotation::next()),    // spread_g_hi
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_ch_neg", |_| expr.0);
            }
        }

        // s_maj on abcd words
        meta.create_gate("s_maj", |meta| {
            CompressionGate::s_maj(
                meta.query_fixed(s_maj, Rotation::cur()), // s_maj
                meta.query_advice(a_2, Rotation::prev()), // spread_m0_even
                meta.query_advice(a_2, Rotation::cur()),  // spread_m0_odd
                meta.query_advice(a_2, Rotation::next()), // spread_m1_even
                meta.query_advice(a_3, Rotation::cur()),  // spread_m1_odd
                meta.query_advice(a_4, Rotation::prev()), // spread_a_lo
                meta.query_advice(a_5, Rotation::prev()), // spread_a_hi
                meta.query_advice(a_4, Rotation::cur()),  // spread_b_lo
                meta.query_advice(a_5, Rotation::cur()),  // spread_b_hi
                meta.query_advice(a_4, Rotation::next()), // spread_c_lo
                meta.query_advice(a_5, Rotation::next()), // spread_c_hi
            )
            .0
        });

        // s_h_prime to compute H' = H + Ch(E, F, G) + s_upper_sigma_1(E) + K + W
        meta.create_gate("s_h_prime", |meta| {
            CompressionGate::s_h_prime(
                meta.query_fixed(s_h_prime, Rotation::cur()), // s_h_prime
                meta.query_advice(a_7, Rotation::next()),     // h_prime_lo
                meta.query_advice(a_8, Rotation::next()),     // h_prime_hi
                meta.query_advice(a_9, Rotation::next()),     // h_prime_carry
                meta.query_advice(a_4, Rotation::cur()),      // sigma_e_lo
                meta.query_advice(a_5, Rotation::cur()),      // sigma_e_hi
                meta.query_advice(a_1, Rotation::cur()),      // ch_lo
                meta.query_advice(a_6, Rotation::next()),     // ch_hi
                meta.query_advice(a_5, Rotation::prev()),     // ch_neg_lo
                meta.query_advice(a_5, Rotation::next()),     // ch_neg_hi
                meta.query_advice(a_7, Rotation::prev()),     // h_lo
                meta.query_advice(a_7, Rotation::cur()),      // h_hi
                meta.query_advice(a_6, Rotation::prev()),     // k_lo
                meta.query_advice(a_6, Rotation::cur()),      // k_hi
                meta.query_advice(a_8, Rotation::prev()),     // w_lo
                meta.query_advice(a_8, Rotation::cur()),      // w_hi
            )
            .0
        });

        // s_a_new
        meta.create_gate("s_a_new", |meta| {
            CompressionGate::s_a_new(
                meta.query_fixed(s_a_new, Rotation::cur()), // s_a_new
                meta.query_advice(a_8, Rotation::cur()),    // a_new_lo
                meta.query_advice(a_8, Rotation::next()),   // a_new_hi
                meta.query_advice(a_9, Rotation::cur()),    // a_new_carry
                meta.query_advice(a_6, Rotation::cur()),    // sigma_a_lo
                meta.query_advice(a_6, Rotation::next()),   // sigma_a_hi
                meta.query_advice(a_1, Rotation::cur()),    // maj_abc_lo
                meta.query_advice(a_3, Rotation::prev()),   // maj_abc_hi
                meta.query_advice(a_7, Rotation::prev()),   // h_prime_lo
                meta.query_advice(a_8, Rotation::prev()),   // h_prime_hi
            )
            .0
        });

        // s_e_new
        meta.create_gate("s_e_new", |meta| {
            CompressionGate::s_e_new(
                meta.query_fixed(s_e_new, Rotation::cur()), // s_e_new
                meta.query_advice(a_8, Rotation::cur()),    // e_new_lo
                meta.query_advice(a_8, Rotation::next()),   // e_new_hi
                meta.query_advice(a_9, Rotation::next()),   // e_new_carry
                meta.query_advice(a_7, Rotation::cur()),    // d_lo
                meta.query_advice(a_7, Rotation::next()),   // d_hi
                meta.query_advice(a_7, Rotation::prev()),   // h_prime_lo
                meta.query_advice(a_8, Rotation::prev()),   // h_prime_hi
            )
            .0
        });

        // s_digest for final round
        {
            let expressions = CompressionGate::s_digest(
                meta.query_fixed(s_digest, Rotation::cur()), // s_digest
                meta.query_advice(a_3, Rotation::cur()),     // lo_0
                meta.query_advice(a_4, Rotation::cur()),     // hi_0
                meta.query_advice(a_5, Rotation::cur()),     // word_0
                meta.query_advice(a_6, Rotation::cur()),     // lo_1
                meta.query_advice(a_7, Rotation::cur()),     // hi_1
                meta.query_advice(a_8, Rotation::cur()),     // word_1
                meta.query_advice(a_3, Rotation::next()),    // lo_2
                meta.query_advice(a_4, Rotation::next()),    // hi_2
                meta.query_advice(a_5, Rotation::next()),    // word_2
                meta.query_advice(a_6, Rotation::next()),    // lo_3
                meta.query_advice(a_7, Rotation::next()),    // hi_3
                meta.query_advice(a_8, Rotation::next()),    // word_3
            );

            for expr in expressions.into_iter() {
                meta.create_gate("s_digest", |_| expr.0);
            }
        }

        CompressionConfig {
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
    pub(super) fn initialize_with_iv(&mut self, init_state: [u32; STATE]) -> Result<State, Error> {
        let config = self.config().clone();
        self.layouter().assign_region(
            || "initialize with iv",
            |mut region: Region<'_, F, Self>| config.initialize_iv(&mut region, init_state),
        )
    }

    /// Initialize compression with some initialized state. This could be a state
    /// output from a previous compression round.
    pub(super) fn initialize_with_state(&mut self, init_state: State) -> Result<State, Error> {
        let config = self.config().clone();
        self.layouter().assign_region(
            || "initialize with iv",
            |mut region: Region<'_, F, Self>| {
                config.initialize_state(&mut region, init_state.clone())
            },
        )
    }

    /// Given an initialized state and a message schedule, perform 64 compression rounds.
    pub(super) fn compress(
        &mut self,
        initialized_state: State,
        w_halves: [(CellValue16, CellValue16); ROUNDS],
    ) -> Result<State, Error> {
        let mut state = State::empty_state();
        let config = self.config().clone();
        self.layouter().assign_region(
            || "initialize with iv",
            |mut region: Region<'_, F, Self>| {
                state = initialized_state.clone();
                for idx in 0..64 {
                    state = config.assign_round(
                        &mut region,
                        idx,
                        state.clone(),
                        w_halves[idx as usize],
                    )?;
                }
                Ok(())
            },
        )?;
        Ok(state)
    }

    /// After the final round, convert the state into the final digest.
    pub(super) fn digest(&mut self, state: State) -> Result<[BlockWord; DIGEST_SIZE], Error> {
        let config = self.config().clone();
        self.layouter().assign_region(
            || "initialize with iv",
            |mut region: Region<'_, F, Self>| config.assign_digest(&mut region, state.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        super::BLOCK_SIZE, get_msg_schedule_test_input, BlockWord, MessageScheduleCore,
        Table16Chip, Table16Config, Table16Core, IV,
    };
    use super::CompressionCore;
    use halo2::{
        arithmetic::FieldExt,
        circuit::{layouter::SingleCoreLayouter, Core},
        dev::MockProver,
        pasta::Fp,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };
    use std::marker::PhantomData;

    #[test]
    fn compress() {
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
                core.load()?;

                // Test vector: "abc"
                let input: [BlockWord; BLOCK_SIZE] = get_msg_schedule_test_input();

                // Run message_scheduler to get W_[0..64]
                let message_schedule_config = core.config().message_schedule.clone();
                let mut message_schedule_core = MessageScheduleCore {
                    config: message_schedule_config,
                    layouter: core.layouter(),
                    marker: PhantomData,
                };
                let (_, w_halves) = message_schedule_core.process(input)?;

                // Compression rounds
                let compression_config = core.config().compression.clone();
                let mut compression_core = CompressionCore {
                    config: compression_config,
                    layouter: core.layouter(),
                    marker: PhantomData,
                };
                let initial_state = compression_core.initialize_with_iv(IV)?;

                let state = compression_core.compress(initial_state.clone(), w_halves)?;

                let digest = compression_core.digest(state)?;
                for (idx, digest_word) in digest.iter().enumerate() {
                    assert_eq!(
                        (digest_word.value.unwrap() as u64 + IV[idx] as u64) as u32,
                        super::compression_util::COMPRESSION_OUTPUT[idx]
                    );
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
