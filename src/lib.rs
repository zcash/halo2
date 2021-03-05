//! # orchard

#![cfg_attr(docsrs, feature(doc_cfg))]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod address;
pub mod bundle;
mod circuit;
mod constants;
pub mod keys;
mod note;
pub mod primitives;
mod spec;
mod tree;
pub mod value;

pub use address::Address;
pub use note::{EncryptedNote, Note, NoteCommitment, Nullifier};
