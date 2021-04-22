//! Structs related to bundles of Orchard actions.

use nonempty::NonEmpty;

use crate::{
    circuit::Proof,
    note::{EncryptedNote, ExtractedNoteCommitment, Nullifier},
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::Anchor,
    value::{ValueCommitment, ValueSum},
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
pub struct Action<T> {
    /// The nullifier of the note being spent.
    nf_old: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cm_new: ExtractedNoteCommitment,
    /// The encrypted output note.
    encrypted_note: EncryptedNote,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
    /// The authorization for this action.
    authorization: T,
}

/// Orchard-specific flags.
#[derive(Debug)]
pub struct Flags {
    spends_enabled: bool,
    outputs_enabled: bool,
}

/// Defines the authorization type of an Orchard bundle.
pub trait Authorization {
    /// The authorization type of an Orchard action.
    type SpendAuth;
}

/// A bundle of actions to be applied to the ledger.
#[derive(Debug)]
pub struct Bundle<T: Authorization> {
    actions: NonEmpty<Action<T::SpendAuth>>,
    flag: Flags,
    value_balance: ValueSum,
    anchor: Anchor,
    authorization: T,
}

impl<T: Authorization> Bundle<T> {
    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    pub fn commitment(&self) -> BundleCommitment {
        todo!()
    }
}

/// Marker for an unauthorized bundle with no proofs or signatures.
#[derive(Debug)]
pub struct Unauthorized {}

impl Authorization for Unauthorized {
    type SpendAuth = ();
}

/// Authorizing data for a bundle of actions, ready to be committed to the ledger.
#[derive(Debug)]
pub struct Authorized {
    proof: Proof,
    binding_signature: redpallas::Signature<Binding>,
}

impl Authorization for Authorized {
    type SpendAuth = redpallas::Signature<SpendAuth>;
}

impl Bundle<Authorized> {
    /// Computes a commitment to the authorizing data within for this bundle.
    ///
    /// This together with `Bundle::commitment` bind the entire bundle.
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
