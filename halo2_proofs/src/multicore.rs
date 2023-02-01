//! An interface for dealing with the kinds of parallel computations involved in
//! `halo2`. It's currently just a (very!) thin wrapper around `rayon` but may
//! be extended in the future to allow for various parallelism strategies.

pub use maybe_rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    join, scope, Scope,
};

#[cfg(feature = "multicore")]
pub use maybe_rayon::{current_num_threads, iter::IndexedParallelIterator};

#[cfg(not(feature = "multicore"))]
pub fn current_num_threads() -> usize {
    1
}

#[cfg(not(feature = "multicore"))]
pub trait IndexedParallelIterator: std::iter::Iterator {}

pub trait TryFoldAndReduce<T, E> {
    /// Implements `iter.try_fold().try_reduce()` for `rayon::iter::ParallelIterator`,
    /// falling back on `Iterator::try_fold` when the `multicore` feature flag is
    /// disabled.
    fn try_fold_and_reduce(
        self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(Result<T, E>, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E>;
}

#[cfg(feature = "multicore")]
impl<T, E, I> TryFoldAndReduce<T, E> for I
where
    T: Send + Sync,
    E: Send + Sync,
    I: maybe_rayon::iter::ParallelIterator<Item = Result<T, E>>,
{
    fn try_fold_and_reduce(
        self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(Result<T, E>, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E> {
        self.try_fold(&identity, |msm, res| fold_op(Ok(msm), res))
            .try_reduce(&identity, |a, b| fold_op(Ok(a), Ok(b)))
    }
}

#[cfg(not(feature = "multicore"))]
impl<T, E, I> TryFoldAndReduce<T, E> for I
where
    I: std::iter::Iterator<Item = Result<T, E>>,
{
    fn try_fold_and_reduce(
        mut self,
        identity: impl Fn() -> T + Send + Sync,
        fold_op: impl Fn(Result<T, E>, Result<T, E>) -> Result<T, E> + Send + Sync,
    ) -> Result<T, E> {
        self.try_fold(identity(), |acc, res| fold_op(Ok(acc), res))
    }
}
