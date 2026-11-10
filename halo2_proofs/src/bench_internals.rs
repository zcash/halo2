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

/// See `arithmetic::parallel_batch_invert`.
pub fn parallel_batch_invert<F: pasta_curves::arithmetic::VartimeField>(v: &mut [F]) {
    arithmetic::parallel_batch_invert(v)
}

/// See `arithmetic::grand_product`.
pub fn grand_product<F: Field>(values: &[F], len: usize) -> Vec<F> {
    arithmetic::grand_product(values, len)
}

/// See `arithmetic::chained_grand_products`.
pub fn chained_grand_products<F: Field>(sets: Vec<Vec<F>>, len: usize) -> Vec<Vec<F>> {
    arithmetic::chained_grand_products(sets, len)
}

/// See `plonk::lookup::prover::permute_lookup_values`.
pub fn permute_lookup_values<F: ff::PrimeField>(
    input: &[F],
    table: &[F],
) -> Result<(Vec<F>, Vec<F>), crate::plonk::Error> {
    crate::plonk::lookup::prover::permute_lookup_values(input, table)
}
