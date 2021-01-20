//! Structs related to bundles of Orchard actions.

use std::marker::PhantomData;

use crate::{
    circuit::Proof,
    note::{EncryptedNote, NoteCommitment, Nullifier},
    tree::Anchor,
    value::{ValueCommitment, ValueSum},
    Chain,
};

/// An action applied to the global ledger.
///
/// Externally, this both creates a note (adding a commitment to the global ledger),
/// and consumes some note created prior to this action (adding a nullifier to the
/// global ledger).
///
/// Internally, this may both consume a note and create a note, or it may do only one of
/// the two. TODO: Determine which is more efficient (circuit size vs bundle size).
#[derive(Debug)]
pub struct Action {
    /// The nullifier of the note being spent.
    nf_old: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: (),
    /// A commitment to the new note being created.
    cm_new: NoteCommitment,
    /// The encrypted output note.
    encrypted_note: EncryptedNote,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
}

/// A bundle of actions to be applied to the ledger.
///
/// TODO: Will this ever exist independently of its signatures, outside of a builder?
#[derive(Debug)]
pub struct Bundle<C: Chain> {
    anchor: Anchor,
    actions: Vec<Action>,
    value_balance: ValueSum<C::Value>,
    proof: Proof,
    _chain: PhantomData<C>,
}

impl<C: Chain> Bundle<C> {
    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    pub fn commitment(&self) -> BundleCommitment {
        todo!()
    }
}

/// An authorized bundle of actions, ready to be committed to the ledger.
#[derive(Debug)]
pub struct SignedBundle<C: Chain> {
    bundle: Bundle<C>,
    action_signatures: Vec<()>,
    binding_signature: (),
}

impl<C: Chain> SignedBundle<C> {
    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    ///
    /// This is equivalent to [`Bundle::commitment`].
    pub fn commitment(&self) -> BundleCommitment {
        self.bundle.commitment()
    }

    /// Computes a commitment to the authorizing data within for this bundle.
    ///
    /// This together with `SignedBundle::commitment` bind the entire bundle.
    pub fn authorizing_commitment(&self) -> BundleAuthorizingCommitment {
        todo!()
    }
}

/// A commitment to a bundle of actions.
///
/// This commitment is non-malleable, in the sense that a bundle's commitment will only
/// change if the effects of the bundle are altered.
#[derive(Debug)]
pub struct BundleCommitment;

/// A commitment to the authorizing data within a bundle of actions.
#[derive(Debug)]
pub struct BundleAuthorizingCommitment;
