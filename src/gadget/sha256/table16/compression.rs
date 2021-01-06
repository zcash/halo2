use std::convert::TryInto;

use super::{
    super::BLOCK_SIZE, BlockWord, Gate, MessagePiece, SpreadInputs, SpreadWord, State, Table16Chip,
    ROUNDS,
};
use crate::{
    arithmetic::FieldExt,
    gadget::{Cell, Layouter, Permutation, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
};

mod compression_gates;
mod subregion_initial;
mod subregion_main;

use compression_gates::CompressionGate;

#[derive(Clone, Debug)]
pub(super) struct Compression {
    lookup: SpreadInputs,
    message_schedule: Column<Advice>,
    extras: [Column<Advice>; 6],

    s_ch: Column<Fixed>,
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

    // Negation used in s_maj
    s_neg: Column<Fixed>,
    perm: Permutation,
}

impl Compression {
    pub(super) fn configure<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        lookup: SpreadInputs,
        message_schedule: Column<Advice>,
        extras: [Column<Advice>; 6],
    ) -> Self {
        let s_ch = meta.fixed_column();
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

        // Helper gates
        let s_neg = meta.fixed_column();

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

        let perm = Permutation::new(meta, &[a_5, a_7]);

        // Decompose `A,B,C,D` words into (2, 11, 9, 10)-bit chunks.
        // `c` is split into (3, 3, 3)-bit c_lo, c_mid, c_hi.
        meta.create_gate(|meta| {
            let s_decompose_abcd = meta.query_fixed(s_decompose_abcd, 0);
            let a = meta.query_advice(a_3, 1); // 2-bit chunk
            let spread_a = meta.query_advice(a_4, 1);
            let b = meta.query_advice(a_1, 0); // 11-bit chunk
            let spread_b = meta.query_advice(a_2, 0);
            let c_lo = meta.query_advice(a_3, 0); // 3-bit chunk
            let spread_c_lo = meta.query_advice(a_4, 0);
            let c_mid = meta.query_advice(a_5, 0); // 3-bit chunk
            let spread_c_mid = meta.query_advice(a_6, 0);
            let c_hi = meta.query_advice(a_5, 1); // 3-bit chunk
            let spread_c_hi = meta.query_advice(a_6, 1);
            let d = meta.query_advice(a_1, 1); // 7-bit chunk
            let spread_d = meta.query_advice(a_2, 1);
            let word_lo = meta.query_advice(a_7, 0);
            let spread_word_lo = meta.query_advice(a_8, 0);
            let word_hi = meta.query_advice(a_7, 1);
            let spread_word_hi = meta.query_advice(a_8, 1);

            CompressionGate::s_decompose_abcd(
                s_decompose_abcd,
                a,
                spread_a,
                b,
                spread_b,
                c_lo,
                spread_c_lo,
                c_mid,
                spread_c_mid,
                c_hi,
                spread_c_hi,
                d,
                spread_d,
                word_lo,
                spread_word_lo,
                word_hi,
                spread_word_hi,
            )
            .0
        });

        // Decompose `E,F,G,H` words into (6, 5, 14, 7)-bit chunks.
        // `a` is split into (3, 3)-bit a_lo, a_hi
        // `b` is split into (2, 3)-bit b_lo, b_hi
        meta.create_gate(|meta| {
            let s_decompose_efgh = meta.query_fixed(s_decompose_efgh, 0);
            let a_lo = meta.query_advice(a_3, 1); // 3-bit chunk
            let spread_a_lo = meta.query_advice(a_4, 1);
            let a_hi = meta.query_advice(a_5, 1); // 3-bit chunk
            let spread_a_hi = meta.query_advice(a_6, 1);
            let b_lo = meta.query_advice(a_3, 0); // 2-bit chunk
            let spread_b_lo = meta.query_advice(a_4, 0);
            let b_hi = meta.query_advice(a_5, 0); // 3-bit chunk
            let spread_b_hi = meta.query_advice(a_6, 0);
            let c = meta.query_advice(a_1, 1); // 14-bit chunk
            let spread_c = meta.query_advice(a_2, 1);
            let d = meta.query_advice(a_1, 0); // 7-bit chunk
            let spread_d = meta.query_advice(a_2, 0);
            let word_lo = meta.query_advice(a_7, 0);
            let spread_word_lo = meta.query_advice(a_8, 0);
            let word_hi = meta.query_advice(a_7, 1);
            let spread_word_hi = meta.query_advice(a_8, 1);

            CompressionGate::s_decompose_efgh(
                s_decompose_efgh,
                a_lo,
                spread_a_lo,
                a_hi,
                spread_a_hi,
                b_lo,
                spread_b_lo,
                b_hi,
                spread_b_hi,
                c,
                spread_c,
                d,
                spread_d,
                word_lo,
                spread_word_lo,
                word_hi,
                spread_word_hi,
            )
            .0
        });

        // s_upper_sigma_0 on abcd words
        // (2, 11, 9, 10)-bit chunks
        meta.create_gate(|meta| {
            let s_upper_sigma_0 = meta.query_fixed(s_upper_sigma_0, 0);
            let spread_r0_even = meta.query_advice(a_4, 0);
            let spread_r0_odd = meta.query_advice(a_2, 0);
            let spread_r1_even = meta.query_advice(a_5, 0);
            let spread_r1_odd = meta.query_advice(a_3, 0);

            let a = meta.query_advice(a_3, 1);
            let spread_a = meta.query_advice(a_4, 1);
            let b = meta.query_advice(a_1, -1);
            let spread_b = meta.query_advice(a_2, -1);
            let c_lo = meta.query_advice(a_3, -1);
            let spread_c_lo = meta.query_advice(a_4, -1);
            let c_mid = meta.query_advice(a_5, -1);
            let spread_c_mid = meta.query_advice(a_6, -1);
            let c_hi = meta.query_advice(a_5, 1);
            let spread_c_hi = meta.query_advice(a_6, 1);
            let d = meta.query_advice(a_1, 1);
            let spread_d = meta.query_advice(a_2, 1);

            let word_lo = meta.query_advice(a_7, -1);
            let spread_word_lo = meta.query_advice(a_8, -1);
            let word_hi = meta.query_advice(a_7, 0);
            let spread_word_hi = meta.query_advice(a_8, 0);

            CompressionGate::s_upper_sigma_0(
                s_upper_sigma_0,
                spread_r0_even,
                spread_r0_odd,
                spread_r1_even,
                spread_r1_odd,
                a,
                spread_a,
                b,
                spread_b,
                c_lo,
                spread_c_lo,
                c_mid,
                spread_c_mid,
                c_hi,
                spread_c_hi,
                d,
                spread_d,
                word_lo,
                spread_word_lo,
                word_hi,
                spread_word_hi,
            )
            .0
        });

        // s_upper_sigma_1 on efgh words
        // (6, 5, 14, 7)-bit chunks
        meta.create_gate(|meta| {
            let s_upper_sigma_1 = meta.query_fixed(s_upper_sigma_1, 0);
            let spread_r0_even = meta.query_advice(a_4, 0);
            let spread_r0_odd = meta.query_advice(a_2, 0);
            let spread_r1_even = meta.query_advice(a_5, 0);
            let spread_r1_odd = meta.query_advice(a_3, 0);
            let a_lo = meta.query_advice(a_4, 1);
            let spread_a_lo = meta.query_advice(a_4, 1);
            let a_hi = meta.query_advice(a_6, 1);
            let spread_a_hi = meta.query_advice(a_6, 1);
            let b_lo = meta.query_advice(a_4, -1);
            let spread_b_lo = meta.query_advice(a_4, -1);
            let b_hi = meta.query_advice(a_6, -1);
            let spread_b_hi = meta.query_advice(a_6, -1);
            let c = meta.query_advice(a_2, 1);
            let spread_c = meta.query_advice(a_2, 1);
            let d = meta.query_advice(a_2, -1);
            let spread_d = meta.query_advice(a_2, -1);

            let word_lo = meta.query_advice(a_7, -1);
            let spread_word_lo = meta.query_advice(a_8, -1);
            let word_hi = meta.query_advice(a_7, 0);
            let spread_word_hi = meta.query_advice(a_8, 0);

            CompressionGate::s_upper_sigma_1(
                s_upper_sigma_1,
                spread_r0_even,
                spread_r0_odd,
                spread_r1_even,
                spread_r1_odd,
                a_lo,
                spread_a_lo,
                a_hi,
                spread_a_hi,
                b_lo,
                spread_b_lo,
                b_hi,
                spread_b_hi,
                c,
                spread_c,
                d,
                spread_d,
                word_lo,
                spread_word_lo,
                word_hi,
                spread_word_hi,
            )
            .0
        });

        Compression {
            lookup,
            message_schedule,
            extras,
            s_ch,
            s_maj,
            s_h_prime,
            s_a_new,
            s_e_new,
            s_upper_sigma_0,
            s_upper_sigma_1,
            s_decompose_abcd,
            s_decompose_efgh,
            s_neg,
            perm,
        }
    }

    pub(super) fn process<F: FieldExt>(
        &self,
        layouter: &mut impl Layouter<Table16Chip<F>>,
        initial_state: &State,
        w: [(MessagePiece, MessagePiece); ROUNDS],
    ) -> Result<State, Error> {
        // Initialise first 3 rounds [0..3]

        // Remaining 61 rounds [3..64]
        todo!()
    }
}
