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
use ff::{PrimeField, PrimeFieldBits};
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

fn bitrange_subset(field_elem: pallas::Base, bitrange: std::ops::Range<usize>) -> pallas::Base {
    let bits = &field_elem
        .to_le_bits()
        .iter()
        .by_val()
        .take(pallas::Base::NUM_BITS as usize)
        .collect::<Vec<_>>()[bitrange];
    let bits: Vec<bool> = bits
        .iter()
        .cloned()
        .chain(std::iter::repeat(false))
        .take(256)
        .collect();
    let bytearray: Vec<u8> = bits
        .chunks_exact(8)
        .map(|byte| byte.iter().rev().fold(0u8, |acc, bit| acc * 2 + *bit as u8))
        .collect();

    pallas::Base::from_bytes(&bytearray.try_into().unwrap()).unwrap()
}
