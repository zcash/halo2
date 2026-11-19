//! An interface for dealing with the kinds of parallel computations involved in
//! `halo2`. It's currently just a (very!) thin wrapper around `rayon` but may
//! be extended in the future to allow for various parallelism strategies.

#[cfg(all(
    feature = "multicore",
    target_arch = "wasm32",
    not(target_feature = "atomics")
))]
compile_error!(
    "The multicore feature flag is not supported on wasm32 architectures without atomics"
);

pub use maybe_rayon::{join, scope};

#[cfg(feature = "multicore")]
pub use maybe_rayon::current_num_threads;

#[cfg(not(feature = "multicore"))]
pub fn current_num_threads() -> usize {
    1
}

pub(crate) trait TheBestReduce {
    type Item;

    /// Combines the best of `std::iter` and `rayon` reductions.
    fn the_best_reduce(
        self,
        identity: impl Fn() -> Self::Item + Send + Sync,
        op: impl Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    ) -> Option<Self::Item>;
}

#[cfg(feature = "multicore")]
impl<I> TheBestReduce for I
where
    I: maybe_rayon::iter::ParallelIterator,
{
    type Item = <Self as maybe_rayon::iter::ParallelIterator>::Item;

    fn the_best_reduce(
        self,
        identity: impl Fn() -> Self::Item + Send + Sync,
        op: impl Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    ) -> Option<Self::Item> {
        Some(self.reduce(identity, op))
    }
}

#[cfg(not(feature = "multicore"))]
impl<I> TheBestReduce for I
where
    I: std::iter::Iterator,
{
    type Item = <Self as std::iter::Iterator>::Item;

    fn the_best_reduce(
        self,
        _: impl Fn() -> Self::Item + Send + Sync,
        f: impl Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    ) -> Option<Self::Item> {
        self.reduce(f)
    }
}
