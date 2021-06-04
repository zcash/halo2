use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use super::super::{
    chip::{SinsemillaChip, SinsemillaConfig},
    SinsemillaInstructions,
};
use super::MerkleInstructions;

use crate::{
    circuit::gadget::utilities::{
        cond_swap::{CondSwapChip, CondSwapConfig, CondSwapInstructions},
        copy,
        lookup_range_check::LookupRangeCheckConfig,
        CellValue, UtilitiesInstructions, Var,
    },
    constants::MERKLE_DEPTH_ORCHARD,
    primitives::sinsemilla,
};
use ff::PrimeFieldBits;
use std::{array, convert::TryInto};

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    advices: [Column<Advice>; 5],
    l_star_plus1: Column<Fixed>,
    perm: Permutation,
    lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    pub(super) cond_swap_config: CondSwapConfig,
    pub(super) sinsemilla_config: SinsemillaConfig,
}

#[derive(Clone, Debug)]
pub struct MerkleChip {
    config: MerkleConfig,
}

impl Chip<pallas::Base> for MerkleChip {
    type Config = MerkleConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl MerkleChip {
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig,
    ) -> MerkleConfig {
        let advices = sinsemilla_config.advices();
        let cond_swap_config =
            CondSwapChip::configure(meta, advices, sinsemilla_config.perm.clone());
        let lookup_config = LookupRangeCheckConfig::configure(
            meta,
            advices[0],
            sinsemilla_config.constants,
            sinsemilla_config.generator_table.table_idx,
            sinsemilla_config.perm.clone(),
        );

        // This fixed column serves two purposes:
        //  - Fixing the value of l* for rows in which a Merkle path layer
        //    is decomposed.
        //  - Disabling the entire decomposition gate (when set to zero)
        //    (i.e. replacing a Selector).

        let l_star_plus1 = meta.fixed_column();

        // Check that pieces have been decomposed correctly for Sinsemilla hash.
        // <https://zips.z.cash/protocol/nu5.pdf#orchardmerklecrh>
        //
        // a = a_0||a_1 = l_star || (bits 0..=239 of left)
        // b = b_0||b_1||b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        // c = bits 5..=254 of right
        //
        // The message pieces `a`, `b`, `c` are constrained by Sinsemilla to be
        // 250 bits, 20 bits, and 250 bits respectively.
        //
        meta.create_gate("Decomposition check", |meta| {
            let l_star_plus1_whole = meta.query_fixed(l_star_plus1, Rotation::cur());

            let two_pow_5 = pallas::Base::from_u64(1 << 5);
            let two_pow_10 = two_pow_5.square();

            // a_whole is constrained by Sinsemilla to be 250 bits.
            let a_whole = meta.query_advice(advices[0], Rotation::cur());
            // b_whole is constrained by Sinsemilla to be 20 bits.
            let b_whole = meta.query_advice(advices[1], Rotation::cur());
            // c_whole is constrained by Sinsemilla to be 250 bits.
            let c_whole = meta.query_advice(advices[2], Rotation::cur());
            let left_node = meta.query_advice(advices[3], Rotation::cur());
            let right_node = meta.query_advice(advices[4], Rotation::cur());

            // a = a_0||a_1 = l_star || (bits 0..=239 of left)
            // Check that a_0 = l_star
            //
            // z_1 of SinsemillaHash(a) = a_1
            let z1_a = meta.query_advice(advices[0], Rotation::next());
            let a_1 = z1_a;
            // a_0 = a - (a_1 * 2^10)
            let a_0 = a_whole - a_1.clone() * pallas::Base::from_u64(1 << 10);
            let l_star_check =
                a_0 - (l_star_plus1_whole.clone() - Expression::Constant(pallas::Base::one()));

            // b = b_0||b_1||b_2
            //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
            //
            //    z_1 of SinsemillaHash(b) = b_1 + 2^5 b_2
            // => b_0 = b - (z1_b * 2^10)
            let z1_b = meta.query_advice(advices[1], Rotation::next());
            // b_1 has been constrained to be 5 bits outside this gate.
            let b_1 = meta.query_advice(advices[2], Rotation::next());
            // b_2 has been constrained to be 5 bits outside this gate.
            let b_2 = meta.query_advice(advices[3], Rotation::next());
            // Derive b_0 (constrained by SinsemillaHash to be 10 bits)
            let b_0 = b_whole - (z1_b * two_pow_10);

            // Check that left = a_1 (240 bits) || b_0 (10 bits) || b_1 (5 bits)
            let left_check = {
                let reconstructed = {
                    let two_pow_240 = pallas::Base::from_u128(1 << 120).square();
                    let b0_shifted = b_0 * two_pow_240;
                    let b1_shifted = b_1 * two_pow_240 * two_pow_10;
                    a_1 + b0_shifted + b1_shifted
                };
                reconstructed - left_node
            };

            // Check that right = b_2 (5 bits) || c (250 bits)
            let right_check = b_2 + c_whole * two_pow_5 - right_node;

            array::IntoIter::new([l_star_check, left_check, right_check])
                .map(move |poly| l_star_plus1_whole.clone() * poly)
        });

        MerkleConfig {
            advices,
            l_star_plus1,
            perm: sinsemilla_config.perm.clone(),
            cond_swap_config,
            lookup_config,
            sinsemilla_config,
        }
    }

    pub fn construct(config: MerkleConfig) -> Self {
        MerkleChip { config }
    }
}

impl UtilitiesInstructions<pallas::Base> for MerkleChip {
    type Var = CellValue<pallas::Base>;
}

impl CondSwapInstructions<pallas::Base> for MerkleChip {
    #[allow(clippy::type_complexity)]
    fn swap(
        &self,
        layouter: impl Layouter<pallas::Base>,
        pair: (Self::Var, Option<pallas::Base>),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config().cond_swap_config.clone();
        let chip = CondSwapChip::<pallas::Base>::construct(config);
        chip.swap(layouter, pair, swap)
    }
}

impl SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }> for MerkleChip {
    type CellValue = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CellValue;

    type Message = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Message;
    type MessagePiece = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::MessagePiece;

    type X = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::X;
    type Point = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Point;

    type HashDomains = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::HashDomains;

    fn witness_message_piece(
        &self,
        layouter: impl Layouter<pallas::Base>,
        value: Option<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.witness_message_piece(layouter, value, num_words)
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Vec<Self::CellValue>>), Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.hash_to_point(layouter, Q, message)
    }

    fn extract(point: &Self::Point) -> Self::X {
        SinsemillaChip::extract(point)
    }
}
