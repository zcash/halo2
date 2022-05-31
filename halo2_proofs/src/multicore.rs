//! An interface for dealing with the kinds of parallel computations involved in
//! `halo2`. It's currently just a (very!) thin wrapper around [`rayon`] but may
//! be extended in the future to allow for various parallelism strategies.

pub use rayon::{current_num_threads, scope, Scope};

pub(crate) fn log_threads() -> u32 {
    let num_threads = current_num_threads();
    assert!(num_threads > 0);
    usize::BITS - 1 - num_threads.leading_zeros()
}
