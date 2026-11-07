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

// Signed width nine needs one more cached multiple than unsigned width eight.
#[cfg(feature = "batch")]
const INSTANCE_WINDOW_BITS: usize = 9;
#[cfg(feature = "batch")]
const INSTANCE_WINDOW_ENTRIES_PER_BASE: usize = 1 << (INSTANCE_WINDOW_BITS - 1);
// Each cached row retains 256 affine points, so this bounds the Pasta cache at
// one MiB while covering Orchard's ten-row instance columns.
#[cfg(feature = "batch")]
const MAX_CACHED_INSTANCE_ROWS: usize = 64;

// Orchard's public instance column has seven full-width rows and three flags.
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_DENSE_ROWS: usize = 7;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_BOOLEAN_ROWS: usize = 3;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_ROWS: usize = PREPARED_INSTANCE_DENSE_ROWS + PREPARED_INSTANCE_BOOLEAN_ROWS;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_OFFSETS: usize = 1 << PREPARED_INSTANCE_BOOLEAN_ROWS;

/// Doubling-free dense sums and default-blind/Boolean offsets, shared by Params clones.
#[cfg(feature = "batch")]
struct GlvInstanceTable<C: pasta_curves::arithmetic::CurveAffine> {
    // Immutable prepared tables cannot be left partially modified by a panic.
    dense: core::panic::AssertUnwindSafe<
        Box<dyn pasta_curves::arithmetic::PreparedFixedBaseSum<C::CurveExt>>,
    >,
    offsets: [C::Curve; PREPARED_INSTANCE_OFFSETS],
}

/// Returns the Boolean-row mask, declining all unrecognized shapes or flags.
#[cfg(feature = "batch")]
fn instance_flag_mask<F: ff::Field>(instance: &[F]) -> Option<usize> {
    if instance.len() != PREPARED_INSTANCE_ROWS {
        return None;
    }
    let mut mask = 0;
    for (flag, scalar) in instance[PREPARED_INSTANCE_DENSE_ROWS..].iter().enumerate() {
        if bool::from(scalar.is_zero()) {
            continue;
        }
        if *scalar == F::ONE {
            mask |= 1 << flag;
        } else {
            return None;
        }
    }
    Some(mask)
}
