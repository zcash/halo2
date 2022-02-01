//! # halo2_gadgets

#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod ecc;
pub mod poseidon;
#[cfg(feature = "unstable")]
pub mod sha256;
pub mod sinsemilla;
pub mod utilities;

pub mod primitives;
