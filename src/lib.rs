//! # halo2

#![allow(
    clippy::op_ref,
    clippy::assign_op_pattern,
    clippy::too_many_arguments,
    clippy::suspicious_arithmetic_impl,
    clippy::many_single_char_names,
    clippy::same_item_push
)]
#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod arithmetic;
pub mod plonk;
pub mod polycommit;
pub mod transcript;
