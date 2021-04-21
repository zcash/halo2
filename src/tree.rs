use rand::RngCore;

/// The root of an Orchard commitment tree.
#[derive(Clone, Debug)]
pub struct Anchor(pub [u8; 32]);

#[derive(Debug)]
pub struct MerklePath;

impl MerklePath {
    /// Generates a dummy Merkle path for use in dummy spent notes.
    pub(crate) fn dummy(_rng: &mut impl RngCore) -> Self {
        // TODO
        MerklePath
    }
}
