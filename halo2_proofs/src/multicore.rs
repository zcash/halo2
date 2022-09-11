//! An interface for dealing with the kinds of parallel computations involved in
//! `halo2`. It's currently just a (very!) thin wrapper around [`rayon`] but may
//! be extended in the future to allow for various parallelism strategies.

pub use rayon::{current_num_threads, prelude, scope, Scope};

pub(crate) fn log_threads() -> u32 {
    let num_threads = current_num_threads();
    assert!(num_threads > 0);
    usize::BITS - 1 - num_threads.leading_zeros()
}

/// This simple utility function will parallelize an operation that is to be
/// performed over a mutable slice.
pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Sync + Clone>(v: &mut [T], f: F) {
    let n = v.len();
    let num_threads = current_num_threads();
    let mut chunk = (n as usize) / num_threads;
    if chunk < num_threads {
        chunk = n as usize;
    }

    scope(|scope| {
        for (chunk_num, v) in v.chunks_mut(chunk).enumerate() {
            let f = f.clone();
            scope.spawn(move |_| f(v, chunk_num * chunk));
        }
    });
}
