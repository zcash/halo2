pub mod circuit;
pub mod lookup;
pub mod metadata;
pub mod permutation;
pub mod poly;
pub mod shuffle;

// TODO: Remove with permutation::Argument simplification
pub mod multicore {
    pub use rayon::{
        current_num_threads,
        iter::{IndexedParallelIterator, IntoParallelRefIterator},
        iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator},
        join, scope,
        slice::ParallelSliceMut,
        Scope,
    };

    pub trait TryFoldAndReduce<T, E> {
        /// Implements `iter.try_fold().try_reduce()` for `rayon::iter::ParallelIterator`,
        /// falling back on `Iterator::try_fold` when the `multicore` feature flag is
        /// disabled.
        /// The `try_fold_and_reduce` function can only be called by a iter with
        /// `Result<T, E>` item type because the `fold_op` must meet the trait
        /// bounds of both `try_fold` and `try_reduce` from rayon.   
        fn try_fold_and_reduce(
            self,
            identity: impl Fn() -> T + Send + Sync,
            fold_op: impl Fn(T, Result<T, E>) -> Result<T, E> + Send + Sync,
        ) -> Result<T, E>;
    }

    impl<T, E, I> TryFoldAndReduce<T, E> for I
    where
        T: Send + Sync,
        E: Send + Sync,
        I: rayon::iter::ParallelIterator<Item = Result<T, E>>,
    {
        fn try_fold_and_reduce(
            self,
            identity: impl Fn() -> T + Send + Sync,
            fold_op: impl Fn(T, Result<T, E>) -> Result<T, E> + Send + Sync,
        ) -> Result<T, E> {
            self.try_fold(&identity, &fold_op)
                .try_reduce(&identity, |a, b| fold_op(a, Ok(b)))
        }
    }
}

// TODO: Remove with permutation::Argument simplification
pub mod arithmetic {
    use super::multicore;

    /// This utility function will parallelize an operation that is to be
    /// performed over a mutable slice.
    pub fn parallelize<T: Send, F: Fn(&mut [T], usize) + Send + Sync + Clone>(v: &mut [T], f: F) {
        // Algorithm rationale:
        //
        // Using the stdlib `chunks_mut` will lead to severe load imbalance.
        // From https://github.com/rust-lang/rust/blob/e94bda3/library/core/src/slice/iter.rs#L1607-L1637
        // if the division is not exact, the last chunk will be the remainder.
        //
        // Dividing 40 items on 12 threads will lead to a chunk size of 40/12 = 3,
        // There will be a 13 chunks of size 3 and 1 of size 1 distributed on 12 threads.
        // This leads to 1 thread working on 6 iterations, 1 on 4 iterations and 10 on 3 iterations,
        // a load imbalance of 2x.
        //
        // Instead we can divide work into chunks of size
        // 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3 = 4*4 + 3*8 = 40
        //
        // This would lead to a 6/4 = 1.5x speedup compared to naive chunks_mut
        //
        // See also OpenMP spec (page 60)
        // http://www.openmp.org/mp-documents/openmp-4.5.pdf
        // "When no chunk_size is specified, the iteration space is divided into chunks
        // that are approximately equal in size, and at most one chunk is distributed to
        // each thread. The size of the chunks is unspecified in this case."
        // This implies chunks are the same size ±1

        let f = &f;
        let total_iters = v.len();
        let num_threads = multicore::current_num_threads();
        let base_chunk_size = total_iters / num_threads;
        let cutoff_chunk_id = total_iters % num_threads;
        let split_pos = cutoff_chunk_id * (base_chunk_size + 1);
        let (v_hi, v_lo) = v.split_at_mut(split_pos);

        multicore::scope(|scope| {
            // Skip special-case: number of iterations is cleanly divided by number of threads.
            if cutoff_chunk_id != 0 {
                for (chunk_id, chunk) in v_hi.chunks_exact_mut(base_chunk_size + 1).enumerate() {
                    let offset = chunk_id * (base_chunk_size + 1);
                    scope.spawn(move |_| f(chunk, offset));
                }
            }
            // Skip special-case: less iterations than number of threads.
            if base_chunk_size != 0 {
                for (chunk_id, chunk) in v_lo.chunks_exact_mut(base_chunk_size).enumerate() {
                    let offset = split_pos + (chunk_id * base_chunk_size);
                    scope.spawn(move |_| f(chunk, offset));
                }
            }
        });
    }
}
