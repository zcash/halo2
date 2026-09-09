//! This module contains implementations for the two finite fields of the Pallas
//! and Vesta curves.

mod fp;
mod fq;

pub use fp::*;
pub use fq::*;

/// Converts 64-bit little-endian limbs to 32-bit little endian limbs.
#[cfg(feature = "gpu")]
fn u64_to_u32(limbs: &[u64]) -> alloc::vec::Vec<u32> {
    limbs
        .iter()
        .flat_map(|limb| {
            Some((limb & 0xFFFF_FFFF) as u32)
                .into_iter()
                .chain(Some((limb >> 32) as u32))
        })
        .collect()
}
