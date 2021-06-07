use ecc::chip::EccChip;

pub(crate) mod ecc;
pub(crate) mod poseidon;
pub(crate) mod sinsemilla;
pub(crate) mod utilities;

impl super::Config {
    pub(super) fn ecc_chip(&self) -> EccChip {
        EccChip::construct(self.ecc_config.clone())
    }
}
