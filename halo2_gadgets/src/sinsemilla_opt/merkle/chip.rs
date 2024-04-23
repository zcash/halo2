//! Chip implementing a Merkle hash using Sinsemilla as the hash function.

use halo2_proofs::{
    circuit::{Chip, Layouter},
    plonk::Error,
};
use pasta_curves::pallas;

use crate::sinsemilla::chip::generator_table::DefaultGeneratorTable;
use crate::sinsemilla::chip::SinsemillaChip;
use crate::utilities::lookup_range_check::DefaultLookupRangeCheck;
use crate::{
    sinsemilla::{merkle::chip::MerkleChip, primitives as sinsemilla},
    sinsemilla_opt::SinsemillaInstructionsOptimized,
    utilities_opt::lookup_range_check::DefaultLookupRangeCheckConfigOptimized,
    {
        ecc::FixedPoints,
        sinsemilla::{CommitDomains, HashDomains},
        utilities::cond_swap::CondSwapChip,
        utilities_opt::cond_swap::CondSwapInstructionsOptimized,
    },
};

impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    CondSwapInstructionsOptimized<pallas::Base>
    for MerkleChip<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    fn mux(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        choice: Self::Var,
        left: Self::Var,
        right: Self::Var,
    ) -> Result<Self::Var, Error> {
        let config = self.config().cond_swap_config.clone();
        let chip = CondSwapChip::<pallas::Base>::construct(config);
        chip.mux(layouter, choice, left, right)
    }
}

impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    SinsemillaInstructionsOptimized<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for MerkleChip<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
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
        layouter: impl Layouter<pallas::Base>,
        Q: &Self::NonIdentityPoint,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Vec<Self::CellValue>>), Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::<
            Hash,
            Commit,
            F,
            LookupRangeCheckConfig,
            GeneratorTableConfigType,
        >::construct(config);
        chip.hash_to_point_with_private_init(layouter, Q, message)
    }
}
