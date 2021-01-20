//! Key structures for Orchard.
//!
//! TODO: Should we have the concept of a Network here? Or make these standalone without
//! defined string encodings, and use newtypes in Zcash?
//! - The latter might get messy, but would maintain the crate abstraction.
//! - One approach might be to make all these types take a type parameter that provides
//!   encoding and decoding support, and then instantiate it in Zcash inside newtypes.
//! - We will need to encode some decisions here (like the size of the diversifier), which
//!   depend on the encoding, so another alternative is we just require Bech32 and then
//!   have the constrained type provide the HRP.

use std::marker::PhantomData;

use crate::{address::Address, Chain};

/// A spending key, from which all key material is derived.
///
/// TODO: In Sapling we never actually used this, instead deriving everything via ZIP 32,
/// so that we could maintain Bitcoin-like HD keys with properties like non-hardened
/// derivation. If we decide that we don't actually require non-hardened derivation, then
/// we could greatly simplify the HD structure and use this struct directly.
#[derive(Debug)]
pub struct SpendingKey<C: Chain>(PhantomData<C>);

#[derive(Debug)]
pub(crate) struct SpendAuthorizingKey<C: Chain>(PhantomData<C>);

impl<C: Chain> From<&SpendingKey<C>> for SpendAuthorizingKey<C> {
    fn from(_: &SpendingKey<C>) -> Self {
        todo!()
    }
}

/// TODO: This is its protocol spec name for Sapling, but I'd prefer a different name.
#[derive(Debug)]
pub(crate) struct AuthorizingKey<C: Chain>(PhantomData<C>);

impl<C: Chain> From<&SpendAuthorizingKey<C>> for AuthorizingKey<C> {
    fn from(_: &SpendAuthorizingKey<C>) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct NullifierDerivingKey<C: Chain>(PhantomData<C>);

impl<C: Chain> From<&SpendingKey<C>> for NullifierDerivingKey<C> {
    fn from(_: &SpendingKey<C>) -> Self {
        todo!()
    }
}

/// A key that provides the capability to recover outgoing transaction information from
/// the block chain.
#[derive(Debug)]
pub struct OutgoingViewingKey<C: Chain>(PhantomData<C>);

impl<C: Chain> From<&SpendingKey<C>> for OutgoingViewingKey<C> {
    fn from(_: &SpendingKey<C>) -> Self {
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
pub struct FullViewingKey<C: Chain> {
    ak: AuthorizingKey<C>,
    nk: NullifierDerivingKey<C>,
    ovk: OutgoingViewingKey<C>,
}

impl<C: Chain> From<&SpendingKey<C>> for FullViewingKey<C> {
    fn from(_: &SpendingKey<C>) -> Self {
        todo!()
    }
}

impl<C: Chain> FullViewingKey<C> {
    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, d: Diversifier<C>) -> Address<C> {
        IncomingViewingKey::from(self).address(d)
    }
}

/// A diversifier that can be used to derive a specific [`Address`] from a
/// [`FullViewingKey`] or [`IncomingViewingKey`].
//
// This is a newtype around a `u128` for simplicity, and enforces the diversifier size
// during all operations.
#[derive(Debug)]
pub struct Diversifier<C: Chain>(u128, PhantomData<C>);

/// A key that provides the capability to detect and decrypt incoming notes from the block
/// chain, without being able to spend the notes or detect when they are spent.
///
/// This key is useful in situations where you only need the capability to detect inbound
/// payments, such as merchant terminals.
///
/// This key is not suitable for use in a wallet, as it cannot maintain accurate balance.
/// You should use a [`FullViewingKey`] instead.
#[derive(Debug)]
pub struct IncomingViewingKey<C: Chain>(PhantomData<C>);

impl<C: Chain> From<&FullViewingKey<C>> for IncomingViewingKey<C> {
    fn from(_: &FullViewingKey<C>) -> Self {
        todo!()
    }
}

impl<C: Chain> IncomingViewingKey<C> {
    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, _: Diversifier<C>) -> Address<C> {
        todo!()
    }
}
