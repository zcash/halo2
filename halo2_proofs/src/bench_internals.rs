//! Crate-internal prover kernels, exposed only for the `prover_parallelism`
//! benchmark so that each can be measured against the sequential code it
//! replaced. This is not a stable API and may change in any release.

use crate::arithmetic::{self, FftGroup};
use ff::Field;

/// See `arithmetic::fft_twiddles`.
pub fn fft_twiddles<F: Field>(omega: F, len: usize) -> Vec<F> {
    arithmetic::fft_twiddles(omega, len)
}

/// See `arithmetic::bitreverse_permutation`.
pub fn bitreverse_permutation<T: Copy + Send + Sync>(a: &mut [T], log_n: u32) {
    arithmetic::bitreverse_permutation(a, log_n)
}

/// See `arithmetic::fft_with_twiddles`.
pub fn fft_with_twiddles<F: Field, G: FftGroup<F>>(a: &mut [G], twiddles: &[F], log_n: u32) {
    arithmetic::fft_with_twiddles(a, twiddles, log_n)
}
