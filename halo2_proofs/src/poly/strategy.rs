use super::commitment::{CommitmentScheme, Verifier};
use crate::plonk::Error;

/// Guards is unfinished verification result. Implement this to construct various
/// verification strategies such as aggregation and recursion.
pub trait Guard<Scheme: CommitmentScheme> {
    /// Multi scalar engine which is not evaluated yet.
    type MSMAccumulator;
}

/// Trait representing a strategy for verifying Halo 2 proofs.
pub trait VerificationStrategy<'params, Scheme: CommitmentScheme, V: Verifier<'params, Scheme>> {
    /// The output type of this verification strategy after processing a proof.
    type Output;

    /// Creates new verification strategy instance
    fn new(params: &'params Scheme::ParamsVerifier) -> Self;

    /// Obtains an MSM from the verifier strategy and yields back the strategy's
    /// output.
    fn process(
        self,
        f: impl FnOnce(V::MSMAccumulator) -> Result<V::Guard, Error>,
    ) -> Result<Self::Output, Error>;

    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    fn finalize(self) -> bool;
}
