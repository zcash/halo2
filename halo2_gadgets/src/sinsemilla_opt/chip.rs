//! Chip implementations for the Sinsemilla gadgets.

use super::SinsemillaInstructionsOptimized;
use crate::utilities::lookup_range_check::DefaultLookupRangeCheck;
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
impl<Hash, Commit, F, LookupRangeCheckConfig>
    SinsemillaInstructionsOptimized<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip<Hash, Commit, F, LookupRangeCheckConfig>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
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

/// A chip that implements 10-bit Sinsemilla using a lookup table and 5 advice columns.
///
/// [Chip description](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html#plonk--halo-2-constraints).
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    inner: SinsemillaChip<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>,
}

// FIXME: is this needed?
impl<Hash, Commit, Fixed> Chip<pallas::Base> for SinsemillaChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    type Config = SinsemillaConfig<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        self.inner.config()
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F> SinsemillaChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Loads the lookup table required by this chip into the circuit.
    pub fn load(
        config: SinsemillaConfig<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        // Load the lookup table.
        generator_table::load_with_tag(
            &config.generator_table,
            // FIXME: consider to remove Option arount tag
            config.lookup_config.table_range_check_tag(),
            layouter,
        )
    }
}
