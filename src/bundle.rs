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
    nf: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cmx: ExtractedNoteCommitment,
    /// The encrypted output note.
    encrypted_note: EncryptedNote,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
    /// The authorization for this action.
    authorization: T,
}

impl<T> Action<T> {
    /// Transitions this action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(T) -> U) -> Action<U> {
        Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization),
        }
    }

    /// Transitions this action from one authorization state to another.
    pub fn try_map<U, E>(self, step: impl FnOnce(T) -> Result<U, E>) -> Result<Action<U>, E> {
        Ok(Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization)?,
        })
    }
}

/// Orchard-specific flags.
#[derive(Clone, Copy, Debug)]
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
    flags: Flags,
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

    /// Transitions this bundle from one authorization state to another.
    pub fn map<U: Authorization>(
        self,
        mut spend_auth: impl FnMut(&T, T::SpendAuth) -> U::SpendAuth,
        step: impl FnOnce(T) -> U,
    ) -> Bundle<U> {
        let authorization = self.authorization;
        Bundle {
            actions: self
                .actions
                .map(|a| a.map(|a_auth| spend_auth(&authorization, a_auth))),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(authorization),
        }
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn try_map<U: Authorization, E>(
        self,
        mut spend_auth: impl FnMut(&T, T::SpendAuth) -> Result<U::SpendAuth, E>,
        step: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<Bundle<U>, E> {
        let authorization = self.authorization;
        let new_actions = self
            .actions
            .into_iter()
            .map(|a| a.try_map(|a_auth| spend_auth(&authorization, a_auth)))
            .collect::<Result<Vec<_>, E>>()?;

        Ok(Bundle {
            actions: NonEmpty::from_vec(new_actions).unwrap(),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(authorization)?,
        })
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
