use crate::keys::Diversifier;

/// A shielded payment address.
#[derive(Debug)]
pub struct Address {
    d: Diversifier,
    pk_d: (),
}
