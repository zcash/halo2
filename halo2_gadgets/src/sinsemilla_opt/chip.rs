//! Chip implementations for the Sinsemilla gadgets.

use super::SinsemillaInstructionsOptimized;
use crate::sinsemilla::chip::generator_table::{DefaultGeneratorTable, GeneratorTableConfig};
use crate::sinsemilla_opt::chip::generator_table::GeneratorTableConfigOptimized;
use crate::utilities::lookup_range_check::{DefaultLookupRangeCheck, LookupRangeCheckConfig};
use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;
use crate::{
    ecc::{chip::NonIdentityEccPoint, FixedPoints},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        message::{Message, MessagePiece},
        primitives as sinsemilla, CommitDomains, HashDomains, SinsemillaInstructions,
    },
    utilities_opt::lookup_range_check::DefaultLookupRangeCheckConfigOptimized,
};
use halo2_proofs::plonk::Expression;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{
        Advice, Column, ConstraintSystem, Constraints, Error, Fixed, TableColumn, VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::pallas;
use pasta_curves::pallas::Base;

pub(crate) mod generator_table;

mod hash_to_point;

// Implement `SinsemillaInstructionsOptimized` for `SinsemillaChip`
impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    SinsemillaInstructionsOptimized<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point_with_private_init(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: &Self::NonIdentityPoint,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error> {
        layouter.assign_region(
            || "hash_to_point",
            |mut region| self.hash_message_with_private_init(&mut region, Q, &message),
        )
    }
}
