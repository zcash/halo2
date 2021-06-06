use pasta_curves::pallas;

use ecc::chip::EccChip;
use sinsemilla::merkle::chip::MerkleChip;
use utilities::plonk::PLONKChip;

pub(crate) mod ecc;
pub(crate) mod poseidon;
pub(crate) mod sinsemilla;
pub(crate) mod utilities;

impl super::Config {
    pub(super) fn plonk_chip(&self) -> PLONKChip<pallas::Base> {
        PLONKChip::construct(self.plonk_config.clone())
    }

    pub(super) fn ecc_chip(&self) -> EccChip {
        EccChip::construct(self.ecc_config.clone())
    }

    pub(super) fn merkle_chip_1(&self) -> MerkleChip {
        MerkleChip::construct(self.merkle_config_1.clone())
    }

    pub(super) fn merkle_chip_2(&self) -> MerkleChip {
        MerkleChip::construct(self.merkle_config_2.clone())
    }
}
