//! Key structures for Orchard.

use crate::address::Address;

/// A spending key, from which all key material is derived.
///
/// TODO: In Sapling we never actually used this, instead deriving everything via ZIP 32,
/// so that we could maintain Bitcoin-like HD keys with properties like non-hardened
/// derivation. If we decide that we don't actually require non-hardened derivation, then
/// we could greatly simplify the HD structure and use this struct directly.
#[derive(Debug)]
pub struct SpendingKey;

#[derive(Debug)]
pub(crate) struct SpendAuthorizingKey;

impl From<&SpendingKey> for SpendAuthorizingKey {
    fn from(_: &SpendingKey) -> Self {
        todo!()
    }
}

/// TODO: This is its protocol spec name for Sapling, but I'd prefer a different name.
#[derive(Debug)]
pub(crate) struct AuthorizingKey;

impl From<&SpendAuthorizingKey> for AuthorizingKey {
    fn from(_: &SpendAuthorizingKey) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct NullifierDerivingKey;

impl From<&SpendingKey> for NullifierDerivingKey {
    fn from(_: &SpendingKey) -> Self {
        todo!()
    }
}

/// A key that provides the capability to recover outgoing transaction information from
/// the block chain.
#[derive(Debug)]
pub struct OutgoingViewingKey;

impl From<&SpendingKey> for OutgoingViewingKey {
    fn from(_: &SpendingKey) -> Self {
        todo!()
    }
}

/// A key that provides the capability to view incoming and outgoing transactions.
///
/// This key is useful in situations where you only need the capability to detect inbound
/// payments, such as merchant terminals.
///
/// This key is not suitable for use in a wallet, as it cannot maintain accurate balance.
/// You should use a [`FullViewingKey`] instead.
///
/// TODO: Should we just define the FVK to include extended stuff like the diversifier key?
#[derive(Debug)]
pub struct FullViewingKey {
    ak: AuthorizingKey,
    nk: NullifierDerivingKey,
    ovk: OutgoingViewingKey,
}

impl From<&SpendingKey> for FullViewingKey {
    fn from(_: &SpendingKey) -> Self {
        todo!()
    }
}

impl FullViewingKey {
    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, d: Diversifier) -> Address {
        IncomingViewingKey::from(self).address(d)
    }
}

/// A diversifier that can be used to derive a specific [`Address`] from a
/// [`FullViewingKey`] or [`IncomingViewingKey`].
#[derive(Debug)]
pub struct Diversifier([u8; 11]);

/// A key that provides the capability to detect and decrypt incoming notes from the block
/// chain, without being able to spend the notes or detect when they are spent.
///
/// This key is useful in situations where you only need the capability to detect inbound
/// payments, such as merchant terminals.
///
/// This key is not suitable for use in a wallet, as it cannot maintain accurate balance.
/// You should use a [`FullViewingKey`] instead.
#[derive(Debug)]
pub struct IncomingViewingKey;

impl From<&FullViewingKey> for IncomingViewingKey {
    fn from(_: &FullViewingKey) -> Self {
        todo!()
    }
}

impl IncomingViewingKey {
    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, _: Diversifier) -> Address {
        todo!()
    }
}
