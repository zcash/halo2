use rand_core::block::BlockRng;
use rand_core::block::BlockRngCore;
use rand_core::OsRng;
use tiny_keccak::Hasher;

/// One number generator, that can be used as a deterministic Rng, outputing fixed values.
pub struct OneNg {}

impl BlockRngCore for OneNg {
    type Item = u32;
    type Results = [u32; 16];

    fn generate(&mut self, results: &mut Self::Results) {
        for elem in results.iter_mut() {
            *elem = 1;
        }
    }
}

pub fn one_rng() -> BlockRng<OneNg> {
    BlockRng::<OneNg>::new(OneNg {})
}

/// Random number generator for testing
pub fn test_rng() -> OsRng {
    OsRng
}

/// Gets the hex representation of the keccak hash of the input data
pub fn keccak_hex<D: AsRef<[u8]>>(data: D) -> String {
    let mut hash = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(data.as_ref());
    hasher.finalize(&mut hash);
    hex::encode(hash)
}
