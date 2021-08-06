//! # orchard
//!
//! ## Nomenclature
//!
//! All types in the `orchard` crate, unless otherwise specified, are Orchard-specific
//! types. For example, [`Address`] is documented as being a shielded payment address; we
//! implicitly mean it is an Orchard payment address (as opposed to e.g. a Sapling payment
//! address, which is also shielded).

#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod address;
pub mod builder;
pub mod bundle;
pub mod circuit;
mod constants;
pub mod keys;
pub mod note;
pub mod note_encryption;
pub mod primitives;
mod spec;
pub mod tree;
pub mod value;

#[cfg(test)]
mod test_vectors;

pub use address::Address;
pub use bundle::Bundle;
pub use circuit::Proof;
pub use note::Note;
pub use tree::Anchor;
