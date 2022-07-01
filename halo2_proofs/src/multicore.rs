//! An interface for dealing with the kinds of parallel computations involved in
//! `halo2`. It's currently just a (very!) thin wrapper around [`rayon`] but may
//! be extended in the future to allow for various parallelism strategies.

pub use rayon::{
    current_num_threads,
    prelude::{self, *},
    scope, Scope,
};

pub(crate) fn log_threads() -> u32 {
    let num_threads = current_num_threads();
    assert!(num_threads > 0);
    usize::BITS - 1 - num_threads.leading_zeros()
}

/// This simple utility function will parallelize an operation that is to be
/// performed over a mutable slice.
pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Sync + Clone>(
    v: &mut [T],
    task_point: usize,
    f: F,
) {
    let n = v.len();
    let (is_parallel, mut chunk_size) = parallel_params(n, task_point);
    if chunk_size == 0 {
        chunk_size = 1;
    }

    if is_parallel {
        v.par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(i, v)| f(v, i * chunk_size))
    } else {
        f(v, 0)
    }
}

pub fn parallel_params(n: usize, task_point: usize) -> (bool, usize) {
    let k = get_log(n);
    let task_point_log = get_log(task_point);
    let turning_degree = 14 - task_point_log;

    if k < turning_degree {
        (false, n / (1 << k))
    } else {
        let thread_num = current_num_threads();
        let degree_diff = k - turning_degree;
        let task_size_divisor = (8 * task_point_log + 2 * degree_diff + thread_num) / 8;
        (true, n / (1 << task_size_divisor))
    }
}

fn get_log(n: usize) -> usize {
    (usize::BITS - 1 - n.leading_zeros()) as usize
}
