//! # halo2_proofs

#![allow(dead_code)] // TODO: Remove
#![allow(unused_imports)] // TODO: Remove
#![cfg_attr(docsrs, feature(doc_cfg))]
// The actual lints we want to disable.
#![allow(clippy::op_ref, clippy::many_single_char_names)]
#![deny(rustdoc::broken_intra_doc_links)]
// #![deny(missing_debug_implementations)] // TODO: Uncomment
// #![deny(missing_docs)] // TODO: Uncomment
#![deny(unsafe_code)]

pub mod arithmetic;
pub mod circuit;
pub use halo2curves;
mod multicore;
pub mod plonk;
pub mod poly;
pub mod transcript;

// TODO: Move to backend for now.  The end goal is to have this in the frontend, but it requires
// many changes because the MockProver heavliy uses backend types.
// pub mod dev;
mod helpers;
pub use helpers::SerdeFormat;
