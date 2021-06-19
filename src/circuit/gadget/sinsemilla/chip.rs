use super::{
    message::{Message, MessagePiece},
    HashDomains, SinsemillaInstructions,
};
use crate::{
    circuit::gadget::{
        ecc::chip::EccPoint,
        utilities::{CellValue, Var},
    },
    primitives::sinsemilla::{
        self, Q_COMMIT_IVK_M_GENERATOR, Q_MERKLE_CRH, Q_NOTE_COMMITMENT_M_GENERATOR,
    },
};

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::pallas;

use std::convert::TryInto;

// mod generator_table;
// pub use generator_table::get_s_by_idx;
// use generator_table::GeneratorTableConfig;

// mod hash_to_point;

/// Configuration for the Sinsemilla hash chip
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaConfig {
    // Selector used in the lookup argument as well as Sinsemilla custom gates.
    q_sinsemilla1: Selector,
    // Fixed column used in Sinsemilla custom gates.
    q_sinsemilla2: Column<Fixed>,
    // Fixed column used to constrain hash initialization to be consistent with
    // the y-coordinate of the domain Q.
    fixed_y_q: Column<Fixed>,
    // Advice column used to store the x-coordinate of the accumulator at each
    // iteration of the hash.
    x_a: Column<Advice>,
    // Advice column used to store the x-coordinate of the generator corresponding
    // to the message word at each iteration of the hash. This is looked up in the
    // generator table.
    x_p: Column<Advice>,
    // Advice column used to load the message.
    bits: Column<Advice>,
    // Advice column used to store the lambda_1 intermediate value at each
    // iteration.
    lambda_1: Column<Advice>,
    // Advice column used to store the lambda_2 intermediate value at each
    // iteration.
    lambda_2: Column<Advice>,
    // Fixed column shared by the whole circuit. This is used to load the
    // x-coordinate of the domain Q, which is then constrained to equal the
    // initial x_a.
    constants: Column<Fixed>,
    // Permutation over all advice columns and the `constants` fixed column.
    perm: Permutation,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChip {
    config: SinsemillaConfig,
}

impl Chip<pallas::Base> for SinsemillaChip {
    type Config = SinsemillaConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl SinsemillaChip {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    pub fn load(
        config: SinsemillaConfig,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 5],
        lookup: (Column<Fixed>, Column<Fixed>, Column<Fixed>),
        constants: Column<Fixed>,
        perm: Permutation,
    ) -> <Self as Chip<pallas::Base>>::Config {
        todo!()
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip
{
    type CellValue = CellValue<pallas::Base>;

    type Message = Message<pallas::Base, { sinsemilla::K }, { sinsemilla::C }>;
    type MessagePiece = MessagePiece<pallas::Base, { sinsemilla::K }>;

    type X = CellValue<pallas::Base>;
    type Point = EccPoint;

    type HashDomains = SinsemillaHashDomains;

    #[allow(non_snake_case)]
    fn witness_message(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        message: Vec<Option<bool>>,
    ) -> Result<Self::Message, Error> {
        todo!()
    }

    #[allow(non_snake_case)]
    fn witness_message_piece_bitstring(
        &self,
        layouter: impl Layouter<pallas::Base>,
        message_piece: &[Option<bool>],
    ) -> Result<Self::MessagePiece, Error> {
        todo!()
    }

    fn witness_message_piece_field(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        field_elem: Option<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        todo!()
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Vec<Self::CellValue>>), Error> {
        todo!()
    }

    fn extract(point: &Self::Point) -> Self::X {
        point.x()
    }
}

#[derive(Clone, Debug)]
pub enum SinsemillaHashDomains {
    NoteCommit,
    CommitIvk,
    MerkleCrh,
}

#[allow(non_snake_case)]
impl HashDomains<pallas::Affine> for SinsemillaHashDomains {
    fn Q(&self) -> pallas::Affine {
        match self {
            SinsemillaHashDomains::CommitIvk => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::NoteCommit => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::MerkleCrh => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_MERKLE_CRH.0).unwrap(),
                pallas::Base::from_bytes(&Q_MERKLE_CRH.1).unwrap(),
            )
            .unwrap(),
        }
    }
}
