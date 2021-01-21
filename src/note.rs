use crate::{keys::FullViewingKey, value::NoteValue, Address};

/// A discrete amount of funds received by an address.
#[derive(Debug)]
pub struct Note {
    /// The recipient of the funds.
    recipient: Address,
    /// The value of this note.
    value: NoteValue,
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
