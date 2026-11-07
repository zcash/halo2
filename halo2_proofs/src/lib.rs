//! # halo2_proofs

#![cfg_attr(docsrs, feature(doc_cfg))]
// The actual lints we want to disable.
#![allow(clippy::op_ref, clippy::many_single_char_names)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod arithmetic;
pub mod circuit;
pub use pasta_curves as pasta;
mod multicore;
pub mod plonk;
pub mod poly;
pub mod transcript;

pub mod dev;
mod helpers;

/// Maximum contiguous point runs recorded or retained for one proof layout.
/// Orchard uses three runs regardless of its action count. Capping each layout
/// bounds the 256-slot cache's range data to 128 KiB on 64-bit targets.
#[cfg(feature = "batch")]
const MAX_CACHED_PROOF_POINT_RUNS: usize = 32;
