use crate::{keys::FullViewingKey, value::NoteValue, Address};

/// The ZIP 212 seed randomness for a note.
#[derive(Debug)]
struct RandomSeed([u8; 32]);

impl RandomSeed {
    fn psi(&self) -> () {
        todo!()
    }

    fn rcm(&self) -> () {
        todo!()
    }

    fn esk(&self) -> () {
        todo!()
    }
}

/// A discrete amount of funds received by an address.
#[derive(Debug)]
pub struct Note {
    /// The recipient of the funds.
    recipient: Address,
    /// The value of this note.
    value: NoteValue,
    /// A unique creation ID for this note.
    ///
    /// This is set to the nullifier of the note that was spent in the [`Action`] that
    /// created this note.
    ///
    /// [`Action`]: crate::bundle::Action
    rho: Nullifier,
    /// The seed randomness for various note components.
    rseed: RandomSeed,
}

impl Note {
    /// Derives the commitment to this note.
    pub fn commitment(&self) -> NoteCommitment {
        todo!()
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, _: &FullViewingKey) -> Nullifier {
        todo!()
    }
}

/// An encrypted note.
#[derive(Debug)]
pub struct EncryptedNote;

/// A commitment to a note.
#[derive(Debug)]
pub struct NoteCommitment;

/// A unique nullifier for a note.
#[derive(Debug)]
pub struct Nullifier;
