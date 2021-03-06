use halo2::pasta::pallas;

use crate::keys::Diversifier;

/// A shielded payment address.
///
/// # Examples
///
/// ```
/// use orchard::keys::{SpendingKey, FullViewingKey};
///
/// let sk = SpendingKey::from_bytes([7; 32]).unwrap();
/// let address = FullViewingKey::from(&sk).default_address();
/// ```
#[derive(Debug)]
pub struct Address {
    d: Diversifier,
    pk_d: pallas::Point,
}

impl Address {
    pub(crate) fn from_parts(d: Diversifier, pk_d: pallas::Point) -> Self {
        Address { d, pk_d }
    }
}
