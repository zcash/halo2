//! # halo2

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(unknown_lints)]
#![allow(
    clippy::op_ref,
    clippy::assign_op_pattern,
    clippy::too_many_arguments,
    clippy::suspicious_arithmetic_impl,
    clippy::many_single_char_names,
    clippy::same_item_push,
    clippy::upper_case_acronyms,
    clippy::unknown_clippy_lints
)]
#![deny(broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod arithmetic;
pub mod circuit;
pub mod gadget;
pub mod pasta;
pub mod plonk;
pub mod poly;
pub mod transcript;

pub mod dev;
pub mod model;
