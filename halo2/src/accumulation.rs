//! # Accumulation schemes
//!
//! The `halo2` crate achieves proof-carrying data (PCD) through a number of
//! techniques first described in the Halo paper [BGH19] and later formalized in
//! [BCMS20]. The primary cryptographic tool is an **accumulation scheme**.
//!
//! ## Recursive SNARKs from Polynomial Commitments
//!
//! It was shown in [MBKM19] that a SNARK could be realized with a protocol
//! (which we'll call a Polynomial Interactive Oracle Proof, or PIOP for short)
//! that needed only a polynomial commitment scheme such as [KZG10]. Later works
//! resulted in newer polynomial commitment schemes (such as [BFS19]) and PIOPs
//! (such as [GWC19]) with improved efficiency or other tradeoffs.
//!
//! Folklore polynomial commitment schemes for both univariate and multilinear
//! polynomials were known to be constructable using a tweaked version of the
//! inner product argument from [BCCGP16]. One of the first observations of
//! [BGH19] is that these tweaked protocols possess a "helped" verification mode
//! whereby multiple independently-generated arguments could be checked with the
//! same effort as checking a single proof so long as an untrusted "helper"
//! produced an efficiently checkable proof.
//!
//! This was combined with the main observation of [BGH19] which is that this
//! helped verification mode can be used to achieve recursive proofs even for
//! arguments that were not succinct. The idea is to continually defer full
//! verification of proofs by using the untrusted helper at each layer of proof
//! composition. The [BGH19] referred to this technique as "nested amortization"
//! and described a concrete protocol that realized recursive proof composition
//! without a trusted setup.
//!
//! ## Accumulation schemes
//!
//! Later, the techniques in [BGH19] were formalized by [BCMS20] into a more
//! general cryptographic tool called an "accumulation scheme". Protocols have
//! an attached "accumulation scheme" when their proofs can be added to an
//! "accumulator" object that changes but does not grow in size; this is most
//! often done with the assistance of an untrusted prover (playing the role of
//! the "helper" mentioned previously) whose proofs are efficient to check. At
//! any point, the accumulation scheme can be "decided" to establish whether it
//! is correct. If it is, then all the proofs added to it previously were also
//! correct.
//!
//! Proof-carrying data can be achieved using accumulation schemes because the
//! protocol can efficiently check updates to the accumulator without having to
//! decide it; this "decision" procedure happens at the end.
//!
//! ## Private vs. Public accumulation
//!
//! The first accumulation scheme described in [BGH19] (but formalized later in
//! [BCMS20]) is for a polynomial commitment scheme based on Pedersen
//! commitments that are queried through the use of a tweaked version of the
//! inner product argument. In this particular scheme, the information possessed
//! by the prover of the update procedure is the same as the information
//! possessed by the decider. We'll refer to these as "public" accumulation
//! schemes. It was later discovered in several works
//! ([1](https://eprint.iacr.org/2020/1618),
//! [2](https://eprint.iacr.org/2020/1536),
//! [3](https://eprint.iacr.org/2021/370)) that accumulation schemes exist with
//! more efficient verifiers for the updating procedure, but which require the
//! prover to maintain a potentially large "witness" value (which is not needed
//! by the verifier) that is used to perform updates and ultimately used by the
//! decision procedure. We'll refer to these as "private" accumulation schemes.
//!
//! [MBKM19]: https://eprint.iacr.org/2019/099
//! [BGH19]: https://eprint.iacr.org/2019/1021
//! [BCMS20]: https://eprint.iacr.org/2020/499
//! [GWC19]: https://eprint.iacr.org/2019/953
//! [KZG10]: https://www.iacr.org/archive/asiacrypt2010/6477178/6477178.pdf
//! [BFS19]: https://eprint.iacr.org/2019/1229
//! [BCCGP16]: https://eprint.iacr.org/2016/263

use std::io;

use halo2_proofs::transcript::{EncodedChallenge, TranscriptRead, TranscriptWrite};
use pasta_curves::arithmetic::CurveAffine;
use rand_core::RngCore;

mod private;
//mod bivariate;

pub use private::*;
//pub use bivariate::*;

/// This is an abstract representation of an accumulation scheme.
pub trait AccumulationScheme<C: CurveAffine> {
    /// An accumulator that is used as input to the update procedure
    type InputAccumulator;
    /// An accumulator that is output from the update procedure
    type OutputAccumulator;
    /// The witness data needed to update the accumulator
    type InputWitness;
    /// The witness data needed to decide an update to the accumulator
    type OutputWitness;

    /// Creates a new (and valid) accumulator and corresponding witness.
    fn blank(&self) -> (Self::OutputAccumulator, Self::OutputWitness);

    /// Check the proof of combination for several old accumulators into a new
    /// accumulator.
    fn verify_update<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[Self::InputAccumulator],
    ) -> io::Result<Self::OutputAccumulator>
    where
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>;

    /// Combine accumulators together, producing a new accumulator and witness.
    /// It is assumed that the accumulators (but not necessarily the witnesses)
    /// have already been entered into the transcript.
    fn prove_update<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[(Self::InputAccumulator, Self::InputWitness)],
        rng: impl RngCore,
    ) -> io::Result<(Self::OutputAccumulator, Self::OutputWitness)>
    where
        E: EncodedChallenge<C>,
        T: TranscriptWrite<C, E>;

    /// Determine if this accumulator is valid; if so, if this accumulator is
    /// the product of a merge of previous accumulators, then if that merging
    /// proof was correct than the previous accumulators were correct as well.
    fn decide(&self, accumulator: &Self::OutputAccumulator, witness: &Self::OutputWitness) -> bool;
}
