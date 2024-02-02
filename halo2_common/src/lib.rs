//! # halo2_proofs

#![cfg_attr(docsrs, feature(doc_cfg))]
// The actual lints we want to disable.
#![allow(clippy::op_ref, clippy::many_single_char_names)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]

pub mod arithmetic;
pub mod circuit;
pub use halo2curves;
pub mod multicore;
pub mod plonk;
// TODO: Try to move this to backend and use a lightweight Polynomial type in the frontend
// https://github.com/privacy-scaling-explorations/halo2/issues/257
pub mod poly;
pub mod transcript;

pub mod helpers;
pub use helpers::SerdeFormat;

// TODO: Everything that is moved from this crate to frontend or backend should recover the
// pub(crate) status whenever possible.
// https://github.com/privacy-scaling-explorations/halo2/issues/266
