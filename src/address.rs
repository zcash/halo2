use crate::keys::{DiversifiedTransmissionKey, Diversifier};

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
    pk_d: DiversifiedTransmissionKey,
}

impl Address {
    pub(crate) fn from_parts(d: Diversifier, pk_d: DiversifiedTransmissionKey) -> Self {
        Address { d, pk_d }
    }
}
