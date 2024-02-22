//! # halo2_proofs

#![cfg_attr(docsrs, feature(doc_cfg))]
// The actual lints we want to disable.
#![allow(clippy::op_ref, clippy::many_single_char_names)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]

pub mod circuit;
pub use halo2curves;
pub mod multicore;
pub mod plonk;
pub mod transcript;

pub mod helpers;
pub use helpers::SerdeFormat;

// TODO: Everything that is moved from this crate to frontend or backend should recover the
// pub(crate) status whenever possible.
// https://github.com/privacy-scaling-explorations/halo2/issues/266
