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
