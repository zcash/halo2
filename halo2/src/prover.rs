use halo2_gadgets::endoscale::EndoscaleInstructions;
use halo2_proofs::{
    arithmetic::CurveAffine,
    pasta::group::ff::PrimeFieldBits,
    plonk::Circuit,
    transcript::{EncodedChallenge, TranscriptRead},
};

use crate::transcript::TranscriptInstructions;

use super::accumulator;

/// Application circuit
struct AppCircuit {}

/// Recursive verifier
struct Verifier<C, E, EndoscaleChip, TranscriptChip, TR>
where
    C: CurveAffine,
    C::Base: PrimeFieldBits,
    E: EncodedChallenge<C>,
    EndoscaleChip: EndoscaleInstructions<C>,
    TranscriptChip: TranscriptInstructions<C>,
    TR: TranscriptRead<C, E> + Clone,
{
    application: AppCircuit,
    acc_verifier: accumulator::Verifier<C, E, EndoscaleChip, TranscriptChip, TR>,
}
