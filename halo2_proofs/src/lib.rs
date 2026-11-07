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

// Orchard's current public instance has seven full-width field elements
// followed by three Boolean flags. The fixed-base prover path is deliberately
// restricted to this exact shape.
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_DENSE_ROWS: usize = 7;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_BOOLEAN_ROWS: usize = 3;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_ROWS: usize = PREPARED_INSTANCE_DENSE_ROWS + PREPARED_INSTANCE_BOOLEAN_ROWS;
#[cfg(feature = "batch")]
const PREPARED_INSTANCE_OFFSETS: usize = 1 << PREPARED_INSTANCE_BOOLEAN_ROWS;

/// A doubling-free GLV table for the dense rows of the one-column Orchard
/// instance shape, built lazily for batch verification.
///
/// `dense` evaluates the full-width rows (see
/// [`pasta_curves::arithmetic::PreparedFixedBaseSum`]). The eight possible
/// Boolean-row contributions are pre-added to `w` in `offsets`, indexed by
/// [`instance_flag_mask`].
#[cfg(feature = "batch")]
struct GlvInstanceTable<C: pasta_curves::arithmetic::CurveAffine> {
    // The table is immutable once built and its methods take `&self`, so a
    // panic cannot leave it in a broken state; asserting unwind safety keeps
    // `Params` unwind safe, as the prepared zero-check caches do.
    dense: core::panic::AssertUnwindSafe<
        Box<dyn pasta_curves::arithmetic::PreparedFixedBaseSum<C::CurveExt>>,
    >,
    offsets: [C::Curve; PREPARED_INSTANCE_OFFSETS],
}

/// The bit mask of a one-column Orchard-shaped instance's Boolean rows, or
/// `None` if any of them is neither zero nor one.
///
/// # Panics
///
/// Panics if `instance` has fewer than [`PREPARED_INSTANCE_ROWS`] rows.
#[cfg(feature = "batch")]
fn instance_flag_mask<F: ff::Field>(instance: &[F]) -> Option<usize> {
    let mut mask = 0;
    let flags = &instance[PREPARED_INSTANCE_DENSE_ROWS
        ..PREPARED_INSTANCE_DENSE_ROWS + PREPARED_INSTANCE_BOOLEAN_ROWS];
    for (flag, scalar) in flags.iter().enumerate() {
        if bool::from(scalar.is_zero()) {
            continue;
        }
        if bool::from((*scalar - F::ONE).is_zero()) {
            mask |= 1 << flag;
        } else {
            return None;
        }
    }
    Some(mask)
}
