//! Decomposes an $n$-bit field element $\alpha$ into $W$ windows, each window
//! being a $K$-bit word, using a running sum $z$.
//! We constrain $K \leq 3$ for this helper.
//!     $$\alpha = k_0 + (2^K) k_1 + (2^{2K}) k_2 + ... + (2^{(W-1)K}) k_{W-1}$$

pub mod be;
pub mod le;

use ff::{PrimeField, PrimeFieldBits};
use halo2_proofs::plonk::{Advice, Assigned, Column, Selector};

use super::lebs2ip;
use pasta_curves::arithmetic::FieldExt;
use std::{convert::TryInto, marker::PhantomData};

/// Decomposes an element `alpha` into `WINDOW_NUM_BITS` little-endian bits
/// For a window size of `w`, this returns [k_0, ..., k_n] where each `k_i`
/// is a `w`-bit value, and `scalar = k_0 + k_1 * w + k_n * w^n`.
pub fn decompose_element_le<
    F: PrimeFieldBits,
    const ELEM_NUM_BITS: usize,
    const WINDOW_NUM_BITS: usize,
>(
    alpha: &F,
) -> Vec<Window<WINDOW_NUM_BITS>> {
    // Pad bits to multiple of WINDOW_NUM_BITS
    let padding = (WINDOW_NUM_BITS - (ELEM_NUM_BITS % WINDOW_NUM_BITS)) % WINDOW_NUM_BITS;
    let bits: Vec<bool> = alpha
        .to_le_bits()
        .into_iter()
        .take(ELEM_NUM_BITS)
        .chain(std::iter::repeat(false).take(padding))
        .collect();
    assert_eq!(bits.len(), ELEM_NUM_BITS + padding);

    bits.chunks_exact(WINDOW_NUM_BITS)
        .map(|window| {
            let window: [bool; WINDOW_NUM_BITS] = window.try_into().unwrap();
            Window(window)
        })
        .collect()
}

/// A window that is at most 8 bits.
#[derive(Clone, Copy, Debug)]
pub struct Window<const NUM_BITS: usize>([bool; NUM_BITS]);

impl<const NUM_BITS: usize> Window<NUM_BITS> {
    /// Returns the value of this window as a field element.
    pub fn value_field<F: PrimeField>(&self) -> F {
        F::from(lebs2ip(&self.0))
    }
}

impl<F: PrimeField, const NUM_BITS: usize> From<Window<NUM_BITS>> for Assigned<F> {
    fn from(window: Window<NUM_BITS>) -> Self {
        Assigned::Trivial(window.value_field())
    }
}

/// Configuration that provides methods for running sum decomposition.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RunningSumConfig<F, const WINDOW_NUM_BITS: usize>
where
    F: FieldExt + PrimeFieldBits,
{
    q_range_check: Selector,
    z: Column<Advice>,
    _marker: PhantomData<F>,
}
